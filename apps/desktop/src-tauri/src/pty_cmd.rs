use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use rustlab::config::ExecConsent;

use crate::error::{CmdResult, CommandError};
use crate::state::{AppState, PtySessionHandle};

static PTY_SEQ: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PtyDataEvent {
    pub session_id: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PtyExitEvent {
    pub session_id: String,
    pub code: Option<u32>,
}

pub fn pty_start(
    app: &AppHandle,
    state: &AppState,
    root_dir: &str,
    cols: u16,
    rows: u16,
) -> CmdResult<String> {
    let root = {
        let inner = state
            .inner
            .lock()
            .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
        if inner.config.execution.consent != ExecConsent::Granted {
            return Err(CommandError::consent(
                "Consentimento explícito necessário antes de iniciar terminal/Cargo.",
            ));
        }
        let proj = rustlab::projects::Project::detect(root_dir)
            .ok_or_else(|| CommandError::not_found("Projeto Cargo não encontrado na pasta."))?;
        let canon = proj.root.to_string_lossy().to_string();
        if !inner
            .config
            .desktop
            .trusted_projects
            .iter()
            .any(|p| p == &canon)
        {
            return Err(CommandError::consent(
                "Confie neste projeto antes de executar código no terminal ConPTY.",
            ));
        }
        proj.root
    };

    let cargo = rustlab::execution::resolve_cargo_path()
        .ok_or_else(|| CommandError::new("TOOLCHAIN_MISSING", "cargo.exe não encontrado.", None))?;

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: rows.max(8),
            cols: cols.max(20),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| CommandError::io("Falha ao abrir Windows ConPTY", e))?;

    let mut cmd = CommandBuilder::new(&cargo);
    cmd.arg("run");
    cmd.cwd(&root);
    if let Some(dir) = cargo.parent() {
        let mut paths = vec![dir.to_path_buf()];
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let mingw = PathBuf::from(local)
                .join("Programs")
                .join("llvm-mingw")
                .join("bin");
            if mingw.is_dir() {
                paths.push(mingw);
            }
        }
        if let Some(existing) = std::env::var_os("PATH") {
            paths.extend(std::env::split_paths(&existing));
        }
        if let Ok(joined) = std::env::join_paths(paths) {
            cmd.env("PATH", joined);
        }
    }

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| CommandError::io("Falha ao iniciar processo no ConPTY", e))?;
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| CommandError::io("Falha ao conectar leitor ConPTY", e))?;
    let mut writer = pair
        .master
        .take_writer()
        .map_err(|e| CommandError::io("Falha ao conectar escritor ConPTY", e))?;
    // Responde imediatamente ao DSR (\x1b[6n) exigido por PSEUDOCONSOLE_INHERIT_CURSOR no Windows ConPTY.
    let _ = writer.write_all(b"\x1b[1;1R");
    let _ = writer.flush();
    let killer = child.clone_killer();

    let session_id = format!("pty-{}", PTY_SEQ.fetch_add(1, Ordering::Relaxed));
    {
        let mut inner = state
            .inner
            .lock()
            .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
        inner.pty_sessions.insert(
            session_id.clone(),
            PtySessionHandle {
                writer,
                master: pair.master,
                killer,
            },
        );
        inner.append_log("pty", &format!("Sessão ConPTY iniciada: {session_id}"));
    }

    let app_handle = app.clone();
    let sid_read = session_id.clone();
    let sid_wait = session_id.clone();
    let shared_inner = state.inner.clone();
    let reader_handle = std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        let mut total_bytes: usize = 0;
        const MAX_PTY_BYTES: usize = 2 * 1024 * 1024;
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    total_bytes = total_bytes.saturating_add(n);
                    if total_bytes <= MAX_PTY_BYTES {
                        let data = String::from_utf8_lossy(&buf[..n]).into_owned();
                        let _ = app_handle.emit(
                            "pty://data",
                            PtyDataEvent {
                                session_id: sid_read.clone(),
                                data,
                            },
                        );
                    } else if total_bytes - n <= MAX_PTY_BYTES {
                        let _ = app_handle.emit(
                            "pty://data",
                            PtyDataEvent {
                                session_id: sid_read.clone(),
                                data: "\r\n[Saída ConPTY truncada por segurança em 2 MiB]\r\n"
                                    .into(),
                            },
                        );
                    }
                }
                Err(_) => break,
            }
        }
    });

    let app_exit = app.clone();
    std::thread::spawn(move || {
        let exit_code = child.wait().ok().map(|s| s.exit_code());
        // Remover a sessão antes do join libera MasterPty (ClosePseudoConsole) e fecha o pipe de saída.
        if let Ok(mut inner) = shared_inner.lock() {
            inner.pty_sessions.remove(&sid_wait);
        }
        let _ = reader_handle.join();
        let _ = app_exit.emit(
            "pty://exit",
            PtyExitEvent {
                session_id: sid_wait,
                code: exit_code,
            },
        );
    });

    Ok(session_id)
}

pub fn pty_write(state: &AppState, session_id: &str, data: &str) -> CmdResult<()> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    let sess = inner
        .pty_sessions
        .get_mut(session_id)
        .ok_or_else(|| CommandError::not_found("Sessão ConPTY não está ativa."))?;
    sess.writer
        .write_all(data.as_bytes())
        .and_then(|()| sess.writer.flush())
        .map_err(|e| CommandError::io("Erro ao escrever no ConPTY", e))?;
    Ok(())
}

pub fn pty_resize(state: &AppState, session_id: &str, cols: u16, rows: u16) -> CmdResult<()> {
    let inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    if let Some(sess) = inner.pty_sessions.get(session_id) {
        sess.master
            .resize(PtySize {
                rows: rows.max(4),
                cols: cols.max(10),
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| CommandError::io("Erro ao redimensionar ConPTY", e))?;
    }
    Ok(())
}

pub fn pty_kill(state: &AppState, session_id: &str) -> CmdResult<()> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    if let Some(mut sess) = inner.pty_sessions.remove(session_id) {
        let _ = sess.killer.kill();
    }
    Ok(())
}
