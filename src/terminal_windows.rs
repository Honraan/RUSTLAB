//! A ConPTY session for the active Cargo project. I/O drains on independent
//! threads so console shutdown can complete even when a child is producing output.

use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::windows::{ffi::OsStrExt, io::AsRawHandle};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows_sys::Win32::System::Console::{
    COORD, ClosePseudoConsole, CreatePseudoConsole, HPCON, ResizePseudoConsole,
};
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::{
    CREATE_UNICODE_ENVIRONMENT, CreateProcessW, DeleteProcThreadAttributeList,
    EXTENDED_STARTUPINFO_PRESENT, GetExitCodeProcess, InitializeProcThreadAttributeList,
    LPPROC_THREAD_ATTRIBUTE_LIST, PROC_THREAD_ATTRIBUTE_JOB_LIST,
    PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, PROCESS_INFORMATION, STARTUPINFOEXW,
    UpdateProcThreadAttribute, WaitForSingleObject,
};

use super::{kill_on_close_job, owned};

const MAX_OUTPUT: usize = 512 * 1024;
const MAX_INPUT: usize = 8192;
const SESSION_LIMIT: Duration = Duration::from_secs(30 * 60);

pub struct Session {
    shared: Arc<Shared>,
    input: Option<mpsc::SyncSender<Vec<u8>>>,
}

#[derive(Default)]
struct Shared {
    display: Mutex<Display>,
    pending_size: Mutex<Option<(u16, u16)>>,
    stop: AtomicBool,
    overflow: AtomicBool,
}

#[derive(Default)]
struct Display {
    bytes: Vec<u8>,
    finished: Option<End>,
}

pub struct Output {
    pub bytes: Vec<u8>,
    pub finished: Option<End>,
}

pub enum End {
    Exited(u32),
    Cancelled,
    TimedOut,
    OutputLimit,
    Failed(String),
}

impl Session {
    pub fn cargo_run(
        root: PathBuf,
        allow_downloads: bool,
        columns: u16,
        rows: u16,
    ) -> io::Result<Self> {
        let cargo = crate::execution::find_cargo().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Cargo não encontrado. Consulte Configurações.",
            )
        })?;
        let shared = Arc::new(Shared::default());
        let (input, rx) = mpsc::sync_channel(32);
        let worker = Arc::clone(&shared);
        thread::Builder::new()
            .name("rustlab-conpty".into())
            .spawn(move || {
                let result = run(
                    &cargo,
                    &root,
                    allow_downloads,
                    size(columns, rows),
                    &worker,
                    rx,
                );
                let end = result.unwrap_or_else(|error| End::Failed(error.to_string()));
                worker
                    .display
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .finished = Some(end);
            })?;
        Ok(Self {
            shared,
            input: Some(input),
        })
    }

    pub fn write(&self, input: &[u8]) -> Result<(), &'static str> {
        if input.len() > MAX_INPUT {
            return Err("A entrada do terminal excede 8 KiB.");
        }
        self.input
            .as_ref()
            .ok_or("Terminal encerrado.")?
            .try_send(input.to_vec())
            .map_err(|_| "Terminal indisponível ou fila de entrada cheia.")
    }

    pub fn resize(&self, columns: u16, rows: u16) {
        *self
            .shared
            .pending_size
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some((columns, rows));
    }

    pub fn poll(&mut self) -> Output {
        let mut display = self
            .shared
            .display
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        Output {
            bytes: std::mem::take(&mut display.bytes),
            finished: display.finished.take(),
        }
    }

    pub fn cancel(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        self.input = None;
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.cancel();
    }
}

struct PseudoConsole(HPCON);

impl Drop for PseudoConsole {
    fn drop(&mut self) {
        // The output reader continues draining the pipe during shutdown.
        unsafe { ClosePseudoConsole(self.0) };
    }
}

fn run(
    cargo: &Path,
    root: &Path,
    online: bool,
    dimensions: COORD,
    shared: &Arc<Shared>,
    input: mpsc::Receiver<Vec<u8>>,
) -> io::Result<End> {
    let (stdin_read, mut stdin_write) = pipe()?;
    let (mut stdout_read, stdout_write) = pipe()?;
    let mut handle = 0;
    check_hresult(unsafe {
        CreatePseudoConsole(
            dimensions,
            stdin_read.as_raw_handle(),
            stdout_write.as_raw_handle(),
            0,
            &mut handle,
        )
    })?;
    let console = PseudoConsole(handle);
    drop(stdin_read);
    drop(stdout_write);

    let read_state = Arc::clone(shared);
    let reader = thread::spawn(move || {
        let mut temp = [0u8; 8192];
        while let Ok(length @ 1..) = stdout_read.read(&mut temp) {
            if read_state.overflow.load(Ordering::Acquire) {
                continue;
            }
            let mut display = read_state.display.lock().unwrap_or_else(|e| e.into_inner());
            if display.bytes.len() + length > MAX_OUTPUT {
                read_state.overflow.store(true, Ordering::Release);
            } else {
                display.bytes.extend_from_slice(&temp[..length]);
            }
        }
    });
    // A blocked write cannot prevent the supervisor from killing the job.
    let _writer = thread::spawn(move || {
        for bytes in input {
            if stdin_write.write_all(&bytes).is_err() {
                break;
            }
        }
    });

    let result = run_child(cargo, root, online, &console, shared);
    drop(console);
    let _ = reader.join();
    result
}

fn run_child(
    cargo: &Path,
    root: &Path,
    online: bool,
    console: &PseudoConsole,
    shared: &Shared,
) -> io::Result<End> {
    let job = kill_on_close_job()?;
    let mut attributes = Attributes::new()?;
    // Win32 expects an HPCON passed by value, and an array of handles for the job list.
    let jobs = [job.as_raw_handle()];
    attributes.set(
        PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE,
        console.0 as *const _,
        size_of::<HPCON>(),
    )?;
    attributes.set(
        PROC_THREAD_ATTRIBUTE_JOB_LIST,
        jobs.as_ptr().cast(),
        size_of::<HANDLE>(),
    )?;
    let mut startup = STARTUPINFOEXW::default();
    startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    // Request default console handles even when the host starts with redirected stdio.
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdInput = INVALID_HANDLE_VALUE;
    startup.StartupInfo.hStdOutput = INVALID_HANDLE_VALUE;
    startup.StartupInfo.hStdError = INVALID_HANDLE_VALUE;
    startup.lpAttributeList = attributes.handle();
    let mut information = PROCESS_INFORMATION::default();
    let program = wide(cargo.as_os_str());
    let cwd = wide(root.as_os_str());
    let mut arguments = vec![u16::from(b'"')];
    arguments.extend_from_slice(&program[..program.len() - 1]);
    arguments.extend(if online { "\" run" } else { "\" run --offline" }.encode_utf16());
    arguments.push(0);
    let environment = environment_block(cargo, online)?;
    if unsafe {
        CreateProcessW(
            program.as_ptr(),
            arguments.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            0,
            EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,
            environment.as_ptr().cast(),
            cwd.as_ptr(),
            &startup.StartupInfo,
            &mut information,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let process = owned(information.hProcess)?;
    let _thread = owned(information.hThread)?;
    let start = Instant::now();
    loop {
        if shared.overflow.load(Ordering::Acquire) {
            return Ok(End::OutputLimit);
        }
        if shared.stop.load(Ordering::Acquire) {
            return Ok(End::Cancelled);
        }
        if start.elapsed() >= SESSION_LIMIT {
            return Ok(End::TimedOut);
        }
        let size_request = shared
            .pending_size
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        if let Some((columns, rows)) = size_request {
            check_hresult(unsafe { ResizePseudoConsole(console.0, size(columns, rows)) })?;
        }
        match unsafe { WaitForSingleObject(process.as_raw_handle(), 40) } {
            WAIT_TIMEOUT => {}
            WAIT_OBJECT_0 => {
                let mut code = 0;
                if unsafe { GetExitCodeProcess(process.as_raw_handle(), &mut code) } == 0 {
                    return Err(io::Error::last_os_error());
                }
                return Ok(End::Exited(code));
            }
            _ => return Err(io::Error::last_os_error()),
        }
    }
}

struct Attributes(Vec<usize>);

impl Attributes {
    fn new() -> io::Result<Self> {
        let mut bytes = 0usize;
        // Initial call obtains the size of the variable-length list.
        unsafe { InitializeProcThreadAttributeList(ptr::null_mut(), 2, 0, &mut bytes) };
        let mut raw = vec![0usize; bytes.div_ceil(size_of::<usize>())];
        if unsafe { InitializeProcThreadAttributeList(raw.as_mut_ptr().cast(), 2, 0, &mut bytes) }
            == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(raw))
    }

    fn handle(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.0.as_mut_ptr().cast()
    }

    fn set(&mut self, key: u32, value: *const std::ffi::c_void, bytes: usize) -> io::Result<()> {
        if unsafe {
            UpdateProcThreadAttribute(
                self.handle(),
                0,
                key as usize,
                value,
                bytes,
                ptr::null_mut(),
                ptr::null(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

impl Drop for Attributes {
    fn drop(&mut self) {
        unsafe { DeleteProcThreadAttributeList(self.handle()) }
    }
}

fn environment_block(cargo: &Path, online: bool) -> io::Result<Vec<u16>> {
    let mut values: Vec<_> = std::env::vars_os()
        .filter(|(name, _)| {
            let name = name.to_string_lossy();
            !name.eq_ignore_ascii_case("PATH")
                && !name.eq_ignore_ascii_case("RUSTUP_AUTO_INSTALL")
                && (online || !name.eq_ignore_ascii_case("CARGO_NET_OFFLINE"))
        })
        .collect();
    let paths = std::env::var_os("PATH").unwrap_or_default();
    let search = cargo
        .parent()
        .into_iter()
        .map(Path::to_owned)
        .chain(std::env::split_paths(&paths).filter(|path| path.is_absolute()));
    let search = std::env::join_paths(search).map_err(io::Error::other)?;
    values.push(("PATH".into(), search));
    values.push(("RUSTUP_AUTO_INSTALL".into(), "0".into()));
    if !online {
        values.push(("CARGO_NET_OFFLINE".into(), "true".into()));
    }
    values.sort_by_key(|(name, _)| name.to_string_lossy().to_uppercase());
    let mut block = Vec::new();
    for (key, value) in values {
        block.extend(key.encode_wide());
        block.push(u16::from(b'='));
        block.extend(value.encode_wide());
        block.push(0);
    }
    block.push(0);
    Ok(block)
}

fn wide(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain([0]).collect()
}

fn pipe() -> io::Result<(File, File)> {
    let (mut read, mut write) = (ptr::null_mut(), ptr::null_mut());
    if unsafe { CreatePipe(&mut read, &mut write, ptr::null(), 0) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((File::from(owned(read)?), File::from(owned(write)?)))
}

fn size(columns: u16, rows: u16) -> COORD {
    COORD {
        X: columns.clamp(10, 400) as i16,
        Y: rows.clamp(5, 200) as i16,
    }
}

fn check_hresult(result: i32) -> io::Result<()> {
    if result >= 0 {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "ConPTY: HRESULT 0x{:08x}",
            result as u32
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conpty_cargo_uses_a_console_and_delivers_keyboard_input() {
        let root =
            std::env::temp_dir().join(format!("RustLab terminal ação {}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='rustlab_conpty_test'\nversion='0.1.0'\nedition='2024'\n[workspace]",
        )
        .unwrap();
        std::fs::write(
            root.join("src/main.rs"),
            r#"use std::io::{self, IsTerminal, Write};
fn main() {
    assert!(io::stdin().is_terminal());
    assert!(io::stdout().is_terminal());
    println!("RUSTLAB_TERMINAL_READY");
    io::stdout().flush().unwrap();
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    println!("RESPOSTA={}", line.trim());
}"#,
        )
        .unwrap();
        let mut session = Session::cargo_run(root.clone(), false, 90, 22).unwrap();
        session.resize(106, 29);
        let mut output = Vec::new();
        let mut sent = false;
        let mut finished = None;
        for _ in 0..1000 {
            let update = session.poll();
            output.extend(update.bytes);
            if !sent && String::from_utf8_lossy(&output).contains("RUSTLAB_TERMINAL_READY") {
                session.write(b"Ferris\r").unwrap();
                sent = true;
            }
            if let Some(end) = update.finished {
                finished = Some(end);
                break;
            }
            thread::sleep(Duration::from_millis(40));
        }
        drop(session);
        let output = String::from_utf8_lossy(&output);
        if let Some(End::Failed(error)) = &finished {
            panic!("{error}: {output}");
        }
        assert!(matches!(finished, Some(End::Exited(0))), "{output}");
        assert!(output.contains("RESPOSTA=Ferris"), "{output}");
        assert!(
            output.contains('\u{001b}'),
            "ConPTY did not emit ANSI output: {output}"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
