//! Gerenciador de execução (Fase 3).
//!
//! Regras de segurança inegociáveis (requisitos da seção 10):
//! * NUNCA monta strings de shell. Toda execução é `Command::new("cargo")`
//!   com argv explícito — nenhum conteúdo de lição/IA/nome de arquivo é
//!   interpretado por shell.
//! * O processo filho roda em process group próprio; cancelamento/timeout
//!   matam o GRUPO (evita órfãos quando o programa do aluno faz spawn).
//! * Saída tem limite de bytes por stream (evita OOM com loop infinito
//!   imprimindo) — e o pipe continua sendo drenado após o limite.
//! * A UI nunca bloqueia: o executor vive em outra thread com runtime
//!   tokio; a comunicação é por canais com `try_recv`.
//!
//! AVISO EXIBIDO AO USUÁRIO (consentimento, tela própria): executar código
//! localmente NÃO é sandbox. O app só executa após consentimento explícito.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;

/// Comandos que o RustLab sabe executar. Enum fechado: nada fora desta
/// lista pode ser disparado pela UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolCommand {
    Check,
    Build,
    BuildLinuxRelease,
    BuildWindowsRelease,
    Run,
    /// Console por linhas com stdin; não é PTY/emulador de terminal.
    RunInteractive,
    AddDependency {
        name: String,
    },
    RemoveDependency {
        name: String,
    },
    Test,
    /// Validação de exercício: apenas a suíte fornecida, offline e sequencial.
    ExerciseTest,
    Fmt,
    Clippy,
    /// `cargo new <nome>` dentro do diretório escolhido.
    NewProject {
        name: String,
    },
}

impl ToolCommand {
    /// argv explícito (argumentos do cargo). Nunca uma string de shell.
    pub fn argv(&self) -> Vec<String> {
        let s = |x: &str| x.to_string();
        match self {
            ToolCommand::Check => vec![s("check"), s("--message-format=short")],
            ToolCommand::Build => vec![s("build"), s("--message-format=short")],
            ToolCommand::BuildLinuxRelease => {
                vec![s("build"), s("--release"), s("--message-format=short")]
            }
            ToolCommand::BuildWindowsRelease => vec![
                s("build"),
                s("--release"),
                s("--target"),
                s("x86_64-pc-windows-gnu"),
                s("--message-format=short"),
            ],
            ToolCommand::Run | ToolCommand::RunInteractive => vec![s("run")],
            ToolCommand::AddDependency { name } => vec![s("add"), s("--"), name.clone()],
            ToolCommand::RemoveDependency { name } => vec![s("remove"), s("--"), name.clone()],
            ToolCommand::Test => vec![s("test"), s("--message-format=short")],
            ToolCommand::ExerciseTest => vec![
                s("test"),
                s("--offline"),
                s("--message-format=short"),
                s("--test"),
                s("rustlab_validation"),
                s("--"),
                s("--test-threads=1"),
            ],
            ToolCommand::Fmt => vec![s("fmt")],
            ToolCommand::Clippy => vec![s("clippy"), s("--message-format=short")],
            ToolCommand::NewProject { name } => {
                vec![
                    s("new"),
                    sanitize_project_name(name).unwrap_or_else(|| s("projeto")),
                ]
            }
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ToolCommand::Check => "cargo check",
            ToolCommand::Build => "cargo build",
            ToolCommand::BuildLinuxRelease => "cargo build (Linux)",
            ToolCommand::BuildWindowsRelease => "cargo build (Windows)",
            ToolCommand::Run | ToolCommand::RunInteractive => "cargo run",
            ToolCommand::AddDependency { .. } => "cargo add",
            ToolCommand::RemoveDependency { .. } => "cargo remove",
            ToolCommand::Test | ToolCommand::ExerciseTest => "cargo test",
            ToolCommand::Fmt => "cargo fmt",
            ToolCommand::Clippy => "cargo clippy",
            ToolCommand::NewProject { .. } => "cargo new",
        }
    }

    /// Timeout padrão por tipo de comando.
    pub fn timeout(&self) -> Duration {
        match self {
            ToolCommand::Run => Duration::from_secs(30),
            ToolCommand::RunInteractive => Duration::from_secs(300),
            ToolCommand::Test | ToolCommand::ExerciseTest => Duration::from_secs(120),
            ToolCommand::BuildLinuxRelease | ToolCommand::BuildWindowsRelease => {
                Duration::from_secs(600)
            }
            _ => Duration::from_secs(180),
        }
    }
}

/// Sanitização de nome de projeto: recusa qualquer coisa fora do
/// alfabeto de nomes de crate (impede `--flags`, espaços, caminhos).
pub fn sanitize_project_name(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return None;
    }
    for c in trimmed.chars() {
        if !(c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return None; // recusa em vez de "limpar" silenciosamente
        }
    }
    if !trimmed
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
    {
        return None;
    }
    if trimmed == "test" {
        return None;
    }
    Some(trimmed.to_string())
}

/// Resolve `cargo` mesmo quando a aplicação é iniciada por um atalho gráfico
/// cujo PATH é menor que o PATH de um shell de login.
fn cargo_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path).map(|dir| dir.join("cargo")));
    }

    let home = std::env::var_os("HOME").map(PathBuf::from);
    if let Some(home) = home.as_ref() {
        candidates.push(home.join(".cargo/bin/cargo"));
        if let Ok(read) = std::fs::read_dir(home.join(".rustup/toolchains")) {
            let mut entries: Vec<_> = read.filter_map(Result::ok).map(|e| e.path()).collect();
            entries.sort();
            candidates.extend(
                entries
                    .into_iter()
                    .map(|toolchain| toolchain.join("bin/cargo")),
            );
        }
    }
    candidates.extend([
        PathBuf::from("/usr/local/cargo/bin/cargo"),
        PathBuf::from("/usr/local/bin/cargo"),
        PathBuf::from("/usr/bin/cargo"),
        PathBuf::from("/opt/cargo/bin/cargo"),
    ]);
    candidates
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    metadata.is_file() && (metadata.permissions().mode() & 0o111) != 0
}

fn find_cargo() -> Option<PathBuf> {
    cargo_candidates()
        .into_iter()
        .find(|path| is_executable(path))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamKind {
    Stdout,
    Stderr,
    /// Mensagens do próprio RustLab no console (início/fim/erros).
    System,
}

#[derive(Debug, Clone)]
pub enum ExecEvent {
    Started {
        label: String,
    },
    Chunk {
        kind: StreamKind,
        text: String,
    },
    Finished {
        label: String,
        code: Option<i32>,
        success: bool,
    },
    TimedOut {
        label: String,
    },
    Cancelled {
        label: String,
    },
    /// Falha ao iniciar o processo (cargo ausente, cwd inválido etc.)
    SpawnFailed {
        label: String,
        reason: String,
    },
}

/// Limite de bytes por stream antes de truncar (256 KiB).
pub const MAX_STREAM_BYTES: usize = 256 * 1024;

enum Command_ {
    Run {
        cmd: ToolCommand,
        cwd: PathBuf,
        target_dir: Option<PathBuf>,
        input: Option<mpsc::Receiver<Option<Vec<u8>>>>,
    },
}

/// Estado compartilhado entre a fachada (thread da UI) e o executor.
#[derive(Default)]
struct ExecState {
    /// PID (== PGID, pois usamos process_group(0)) do filho atual.
    pgid: Mutex<Option<u32>>,
    cancelled: AtomicBool,
}

/// Fachada usada pela UI. `poll_events()` nunca bloqueia.
pub struct ExecutionManager {
    cmd_tx: mpsc::Sender<Command_>,
    event_rx: std::sync::mpsc::Receiver<ExecEvent>,
    state: Arc<ExecState>,
    pub running: bool,
    pub running_label: Option<String>,
    input: Option<mpsc::Sender<Option<Vec<u8>>>>,
    pub shared_target_dir: Option<PathBuf>,
}

impl ExecutionManager {
    pub fn new() -> Self {
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<Command_>(8);
        let (event_tx, event_rx) = std::sync::mpsc::channel::<ExecEvent>();
        let state = Arc::new(ExecState::default());
        let state_clone = state.clone();

        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = event_tx.send(ExecEvent::SpawnFailed {
                        label: "runtime".into(),
                        reason: format!("falha ao iniciar runtime de execução: {e}"),
                    });
                    return;
                }
            };
            rt.block_on(async move {
                while let Some(Command_::Run {
                    cmd,
                    cwd,
                    target_dir,
                    input,
                }) = cmd_rx.recv().await
                {
                    run_one(
                        &cmd,
                        &cwd,
                        target_dir.as_deref(),
                        &state_clone,
                        event_tx.clone(),
                        input,
                    )
                    .await;
                }
            });
        });

        Self {
            cmd_tx,
            event_rx,
            state,
            running: false,
            running_label: None,
            input: None,
            shared_target_dir: None,
        }
    }

    pub fn set_shared_target_dir(&mut self, dir: Option<PathBuf>) {
        self.shared_target_dir = dir;
    }

    pub fn shared_target_dir(&self) -> Option<&Path> {
        self.shared_target_dir.as_deref()
    }

    /// Enfileira um comando. Se algo estiver rodando, o pedido é recusado
    /// (a UI deve cancelar antes) — evita dois cargos simultâneos.
    pub fn submit(&mut self, cmd: ToolCommand, cwd: PathBuf) -> Result<(), &'static str> {
        if let ToolCommand::AddDependency { name } | ToolCommand::RemoveDependency { name } = &cmd {
            if !valid_dependency(name) {
                return Err("dependência inválida; use nome ou nome@versão");
            }
            if matches!(&cmd, ToolCommand::RemoveDependency { .. }) && name.contains('@') {
                return Err("remoção aceita apenas o nome da dependência");
            }
        }
        if let ToolCommand::NewProject { name } = &cmd
            && sanitize_project_name(name).is_none()
        {
            return Err("nome de projeto inválido");
        }
        if self.running {
            return Err("já existe um comando em execução — cancele antes (Ctrl+K)");
        }
        if !cwd.is_dir() {
            return Err("diretório do projeto não existe");
        }
        self.state.cancelled.store(false, Ordering::SeqCst);
        let label = cmd.label().to_string();
        let (sender, input) = if cmd == ToolCommand::RunInteractive {
            let (tx, rx) = mpsc::channel(16);
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };
        let target_dir = if matches!(cmd, ToolCommand::NewProject { .. }) {
            None
        } else {
            self.shared_target_dir.clone()
        };
        self.cmd_tx
            .try_send(Command_::Run {
                cmd,
                cwd,
                target_dir,
                input,
            })
            .map_err(|_| "fila de execução cheia")?;
        self.running = true;
        self.running_label = Some(label);
        self.input = sender;
        Ok(())
    }

    pub fn accepts_input(&self) -> bool {
        self.running && self.input.is_some()
    }

    pub fn send_line(&self, text: &str) -> Result<(), &'static str> {
        if text.len() > 4096 || text.contains(['\n', '\r', '\0']) {
            return Err("entrada deve ter uma linha de até 4096 bytes");
        }
        let sender = self
            .input
            .as_ref()
            .filter(|_| self.running)
            .ok_or("stdin não está aberto")?;
        sender
            .try_send(Some(format!("{text}\n").into_bytes()))
            .map_err(|_| "entrada indisponível ou fila cheia")
    }

    pub fn close_input(&mut self) -> Result<(), &'static str> {
        let sender = self.input.as_ref().ok_or("stdin não está aberto")?;
        sender
            .try_send(None)
            .map_err(|_| "não consegui enviar EOF; tente novamente")?;
        self.input = None;
        Ok(())
    }

    /// Cancela o comando atual matando o PROCESS GROUP inteiro (cargo +
    /// binário do aluno + filhos). Seguro chamar sem nada rodando.
    pub fn cancel(&mut self) {
        self.state.cancelled.store(true, Ordering::SeqCst);
        let pgid = *self.state.pgid.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(pgid) = pgid {
            kill_group(pgid);
        }
    }

    /// Drena eventos disponíveis sem bloquear a UI.
    pub fn poll_events(&mut self, out: &mut Vec<ExecEvent>) {
        while let Ok(ev) = self.event_rx.try_recv() {
            if matches!(
                ev,
                ExecEvent::Finished { .. }
                    | ExecEvent::TimedOut { .. }
                    | ExecEvent::Cancelled { .. }
                    | ExecEvent::SpawnFailed { .. }
            ) {
                self.running = false;
                self.running_label = None;
                self.input = None;
            }
            out.push(ev);
        }
    }
}

impl Default for ExecutionManager {
    fn default() -> Self {
        Self::new()
    }
}

async fn run_one(
    cmd: &ToolCommand,
    cwd: &Path,
    target_dir: Option<&Path>,
    state: &Arc<ExecState>,
    tx: std::sync::mpsc::Sender<ExecEvent>,
    input: Option<mpsc::Receiver<Option<Vec<u8>>>>,
) {
    use std::os::unix::process::CommandExt;
    use tokio::process::Command as AsyncCommand;

    let label = cmd.label().to_string();
    let Some(cargo) = find_cargo() else {
        let _ = tx.send(ExecEvent::SpawnFailed {
            label,
            reason: "não encontrei o executável do Cargo; instale o Rust ou adicione ~/.cargo/bin ao PATH"
                .to_string(),
        });
        return;
    };
    let mut ac = AsyncCommand::new(&cargo);
    ac.env("CARGO_TERM_COLOR", "never");
    if let Some(dir) = target_dir {
        let _ = std::fs::create_dir_all(dir);
        ac.env("CARGO_TARGET_DIR", dir);
    }
    if let Some(dir) = cargo.parent() {
        let mut search_paths = vec![dir.to_path_buf()];
        if let Some(path) = std::env::var_os("PATH") {
            search_paths.extend(std::env::split_paths(&path));
        }
        if let Ok(path) = std::env::join_paths(search_paths) {
            ac.env("PATH", path);
        }
    }
    for arg in cmd.argv() {
        ac.arg(arg);
    }
    ac.current_dir(cwd)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    // Process group próprio: permite matar a árvore inteira no cancel.
    ac.as_std_mut().process_group(0);

    let mut child = match ac.spawn() {
        Ok(c) => c,
        Err(e) => {
            let reason = if e.kind() == std::io::ErrorKind::NotFound {
                "cargo não encontrado no PATH — instale o Rust em rustup.rs".to_string()
            } else {
                format!("{e}")
            };
            let _ = tx.send(ExecEvent::SpawnFailed { label, reason });
            return;
        }
    };
    let pid = child.id().unwrap_or(0);
    {
        let mut g = state.pgid.lock().unwrap_or_else(|e| e.into_inner());
        *g = Some(pid);
    }
    if state.cancelled.load(Ordering::SeqCst) {
        kill_group(pid);
    }
    let _ = tx.send(ExecEvent::Started {
        label: label.clone(),
    });

    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");
    let h_out = tokio::spawn(pipe_reader(stdout, StreamKind::Stdout, tx.clone()));
    let h_err = tokio::spawn(pipe_reader(stderr, StreamKind::Stderr, tx.clone()));
    let h_input = input.map(|mut rx| {
        let mut stdin = child.stdin.take().expect("stdin piped");
        let tx = tx.clone();
        tokio::spawn(async move {
            while let Some(Some(bytes)) = rx.recv().await {
                if let Err(err) = stdin.write_all(&bytes).await {
                    let _ = tx.send(ExecEvent::Chunk {
                        kind: StreamKind::System,
                        text: format!("stdin fechado pelo programa: {err}"),
                    });
                    break;
                }
            }
        })
    });

    let status = tokio::select! {
        res = child.wait() => res,
        _ = tokio::time::sleep(cmd.timeout()) => {
            if let Some(handle) = &h_input { handle.abort(); }
            kill_group(pid);
            let _ = child.wait().await;
            let _ = h_out.await;
            let _ = h_err.await;
            clear_pgid(state);
            let _ = tx.send(ExecEvent::TimedOut { label });
            return;
        }
    };
    if let Some(handle) = h_input {
        handle.abort();
    }

    // Programas que deixam descendentes com os pipes abertos não podem
    // prender a UI após o processo principal terminar.
    kill_group(pid);

    // aguarda os leitores chegarem ao EOF para não perder o fim da saída
    let _ = h_out.await;
    let _ = h_err.await;
    clear_pgid(state);

    if state.cancelled.load(Ordering::SeqCst) {
        let _ = tx.send(ExecEvent::Cancelled { label });
        return;
    }
    match status {
        Ok(st) => {
            let code = st.code();
            let success = st.success();
            let _ = tx.send(ExecEvent::Finished {
                label,
                code,
                success,
            });
        }
        Err(e) => {
            let _ = tx.send(ExecEvent::SpawnFailed {
                label,
                reason: format!("wait: {e}"),
            });
        }
    }
}

fn clear_pgid(state: &Arc<ExecState>) {
    let mut g = state.pgid.lock().unwrap_or_else(|e| e.into_inner());
    *g = None;
}

fn kill_group(pid: u32) {
    if pid != 0 {
        unsafe {
            libc::killpg(pid as libc::pid_t, libc::SIGKILL);
        }
    }
}

/// Lê um pipe até EOF, enviando linhas e respeitando o limite de bytes.
/// IMPORTANTE: continua drenando mesmo após estourar o limite (descarta),
/// senão o pipe enche e o processo do aluno bloqueia para sempre.
async fn pipe_reader<R: tokio::io::AsyncRead + Unpin>(
    reader: R,
    kind: StreamKind,
    tx: std::sync::mpsc::Sender<ExecEvent>,
) {
    let mut reader = reader;
    let mut buffer = [0u8; 4096];
    let mut pending = Vec::new();
    let mut bytes: usize = 0;
    let mut truncated_notified = false;
    while let Ok(size) = reader.read(&mut buffer).await {
        if size == 0 {
            break;
        }
        let allowed = size.min(MAX_STREAM_BYTES.saturating_sub(bytes));
        bytes = bytes.saturating_add(size);
        pending.extend_from_slice(&buffer[..allowed]);
        while allowed > 0
            && let Some(end) = pending.iter().position(|&b| b == b'\n')
        {
            let line: Vec<_> = pending.drain(..=end).collect();
            let text = String::from_utf8_lossy(&line[..end])
                .trim_end_matches('\r')
                .to_string();
            let _ = tx.send(ExecEvent::Chunk { kind, text });
        }
        // Prompts sem newline precisam aparecer antes de o aluno responder.
        // Retém somente uma possível sequência UTF-8 incompleta.
        if kind == StreamKind::Stdout && !pending.is_empty() {
            let end = match std::str::from_utf8(&pending) {
                Ok(_) => pending.len(),
                Err(err) if err.error_len().is_none() => err.valid_up_to(),
                Err(_) => pending.len(),
            };
            if end > 0 {
                let text = String::from_utf8_lossy(&pending[..end]).into_owned();
                pending.drain(..end);
                let _ = tx.send(ExecEvent::Chunk { kind, text });
            }
        }
        if bytes > MAX_STREAM_BYTES && !truncated_notified {
            truncated_notified = true;
            let nome = match kind {
                StreamKind::Stdout => "stdout",
                StreamKind::Stderr => "stderr",
                StreamKind::System => "sistema",
            };
            let _ = tx.send(ExecEvent::Chunk {
                kind: StreamKind::System,
                text: format!("[saída {nome} truncada em {MAX_STREAM_BYTES} bytes — o processo continua sendo drenado]"),
            });
        }
    }
    if !pending.is_empty() {
        let _ = tx.send(ExecEvent::Chunk {
            kind,
            text: String::from_utf8_lossy(&pending).into_owned(),
        });
    }
}

pub fn valid_dependency(text: &str) -> bool {
    let (name, version) = text
        .split_once('@')
        .map_or((text, None), |(n, v)| (n, Some(v)));
    !name.is_empty()
        && name.len() <= 64
        && name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        && version.is_none_or(|v| {
            !v.is_empty()
                && v.len() <= 64
                && v.chars()
                    .all(|c| c.is_ascii_alphanumeric() || ".-*^~=+".contains(c))
        })
}

// ---------------------------------------------------------------------------
// Diagnósticos do compilador (determinístico, sem IA)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagLevel {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub level: DiagLevel,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub col: Option<u32>,
}

/// Interpreta a saída do cargo/rustc nos dois formatos:
/// curto (`path:line:col: error[E0308]: msg`) e legível
/// (`error[E0308]: msg` seguido de ` --> path:line:col`).
pub fn parse_diagnostics(output: &str) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut pending_header: Option<(DiagLevel, String)> = None;

    for raw in output.lines() {
        let line = raw.trim_end();
        if let Some(d) = parse_short(line) {
            out.push(d);
            pending_header = None;
            continue;
        }
        if line.starts_with("error:") && !line.contains("-->") {
            out.push(Diagnostic {
                level: DiagLevel::Error,
                message: line.trim_start_matches("error:").trim().to_string(),
                file: None,
                line: None,
                col: None,
            });
            pending_header = None;
        } else if let Some(rest) = line.strip_prefix("error") {
            pending_header = Some((DiagLevel::Error, header_message(rest)));
        } else if let Some(rest) = line.strip_prefix("warning:") {
            pending_header = Some((DiagLevel::Warning, rest.trim().to_string()));
        } else if let Some((file, l, c)) = parse_arrow(line)
            && let Some((level, message)) = pending_header.take()
        {
            out.push(Diagnostic {
                level,
                message,
                file: Some(file),
                line: Some(l),
                col: Some(c),
            });
        }
    }
    out
}

fn header_message(rest: &str) -> String {
    // rest começa com "[E0308]: msg" ou ": msg"
    if let Some(pos) = rest.find(": ") {
        rest[pos + 2..].trim().to_string()
    } else {
        rest.trim_start_matches(':').trim().to_string()
    }
}

fn parse_short(line: &str) -> Option<Diagnostic> {
    // path:line:col: level[...]: message
    let mut parts = line.splitn(4, ':');
    let path = parts.next()?;
    if !path.ends_with(".rs") {
        return None;
    }
    let l: u32 = parts.next()?.trim().parse().ok()?;
    let c: u32 = parts.next()?.trim().parse().ok()?;
    let rest = parts.next()?.trim();
    let (level, message) = if let Some(r) = rest.strip_prefix("error") {
        (DiagLevel::Error, header_message(r))
    } else if let Some(r) = rest.strip_prefix("warning") {
        (DiagLevel::Warning, header_message(r))
    } else {
        (DiagLevel::Note, rest.to_string())
    };
    if message.is_empty() {
        return None;
    }
    Some(Diagnostic {
        level,
        message,
        file: Some(path.to_string()),
        line: Some(l),
        col: Some(c),
    })
}

fn parse_arrow(line: &str) -> Option<(String, u32, u32)> {
    let t = line.trim();
    let rest = t.strip_prefix("-->")?;
    let mut it = rest.trim().split(':');
    let file = it.next()?.to_string();
    let l: u32 = it.next()?.parse().ok()?;
    let c: u32 = it.next()?.parse().ok()?;
    Some((file, l, c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argv_nunca_monta_shell() {
        // Garantia estrutural: ToolCommand produz Vec<String> de args;
        // não existe API que gere uma string única para `sh -c`.
        let v = ToolCommand::NewProject {
            name: "meu_jogo".into(),
        }
        .argv();
        assert_eq!(v, vec!["new".to_string(), "meu_jogo".to_string()]);
        assert_eq!(ToolCommand::Run.argv(), vec!["run".to_string()]);
        assert_eq!(
            ToolCommand::Check.argv(),
            vec!["check".to_string(), "--message-format=short".to_string()]
        );
        assert_eq!(
            ToolCommand::Clippy.argv(),
            vec!["clippy".to_string(), "--message-format=short".to_string()]
        );
    }

    #[test]
    fn dependencias_tem_argumentos_fechados_e_validacao() {
        for valid in ["serde", "serde@1", "rand@0.8.5", "foo-bar@^1.0"] {
            assert!(valid_dependency(valid));
        }
        for invalid in [
            "--path",
            "../evil",
            "foo;touch",
            "foo bar",
            "foo@",
            "foo@1 --git",
            "foo\nbar",
        ] {
            assert!(!valid_dependency(invalid));
        }
        assert_eq!(
            ToolCommand::AddDependency {
                name: "serde@1".into()
            }
            .argv(),
            vec!["add", "--", "serde@1"]
        );
        assert_eq!(ToolCommand::RunInteractive.argv(), vec!["run"]);
        let manager = ExecutionManager::new();
        assert!(manager.send_line("oi").is_err());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn leitor_limita_saida_sem_newline_e_continua_drenando() {
        let data = vec![b'x'; MAX_STREAM_BYTES * 3];
        let (tx, rx) = std::sync::mpsc::channel();
        pipe_reader(data.as_slice(), StreamKind::Stderr, tx).await;
        let events: Vec<_> = rx.try_iter().collect();
        let output_bytes: usize = events
            .iter()
            .filter_map(|event| match event {
                ExecEvent::Chunk {
                    kind: StreamKind::Stderr,
                    text,
                } => Some(text.len()),
                _ => None,
            })
            .sum();
        assert_eq!(output_bytes, MAX_STREAM_BYTES);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    ExecEvent::Chunk {
                        kind: StreamKind::System,
                        ..
                    }
                ))
                .count(),
            1
        );
    }

    #[test]
    fn nome_de_projeto_e_sanitizado() {
        assert_eq!(
            sanitize_project_name("ola-mundo").as_deref(),
            Some("ola-mundo")
        );
        assert_eq!(sanitize_project_name("_ok9").as_deref(), Some("_ok9"));
        assert!(sanitize_project_name("").is_none());
        assert!(sanitize_project_name("9comeca-com-numero").is_none());
        assert!(sanitize_project_name("tem espaço").is_none());
        assert!(sanitize_project_name("--flag").is_none());
        assert!(sanitize_project_name("ponto.evil").is_none());
        assert!(sanitize_project_name(&"x".repeat(65)).is_none());
        assert_eq!(sanitize_project_name("test"), None);
        // flag de cargo jamais passa como nome
        assert!(sanitize_project_name("-rf").is_none());
    }

    #[test]
    fn diagnostico_formato_curto() {
        let out = "src/main.rs:3:5: error[E0308]: mismatched types\n\
                   src/main.rs:7:9: warning: unused variable: `x`";
        let d = parse_diagnostics(out);
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].level, DiagLevel::Error);
        assert_eq!(d[0].message, "mismatched types");
        assert_eq!(d[0].file.as_deref(), Some("src/main.rs"));
        assert_eq!((d[0].line, d[0].col), (Some(3), Some(5)));
        assert_eq!(d[1].level, DiagLevel::Warning);
    }

    #[test]
    fn diagnostico_formato_legivel() {
        let out = "\
error[E0308]: mismatched types
 --> src/main.rs:4:20
  |
4 |     let x: u32 = \"texto\";
  |                    ^^^^^^ expected `u32`, found `&str`

warning: unused variable: `y`
 --> src/main.rs:2:9
";
        let d = parse_diagnostics(out);
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].message, "mismatched types");
        assert_eq!(d[0].line, Some(4));
        assert_eq!(d[1].level, DiagLevel::Warning);
        assert_eq!(d[1].file.as_deref(), Some("src/main.rs"));
    }

    #[test]
    fn erro_sem_localizacao() {
        let d = parse_diagnostics(
            "error: could not compile `demo` (bin \"demo\") due to 1 previous error",
        );
        assert_eq!(d.len(), 1);
        assert!(d[0].file.is_none());
        assert!(d[0].message.contains("could not compile"));
    }

    #[test]
    fn saida_limpa_nao_gera_diagnostico() {
        let out = "   Compiling demo v0.1.0\n    Finished `dev` profile\n     Running `target/debug/demo`\nOlá!";
        assert!(parse_diagnostics(out).is_empty());
    }

    #[test]
    fn timeout_por_comando() {
        assert_eq!(ToolCommand::Run.timeout(), Duration::from_secs(30));
        assert!(ToolCommand::Build.timeout() > ToolCommand::Run.timeout());
    }

    #[test]
    fn target_builds_tem_argv_linux_e_windows() {
        assert_eq!(
            ToolCommand::BuildLinuxRelease.argv(),
            vec![
                "build".to_string(),
                "--release".to_string(),
                "--message-format=short".to_string()
            ]
        );
        assert_eq!(
            ToolCommand::BuildWindowsRelease.argv(),
            vec![
                "build".to_string(),
                "--release".to_string(),
                "--target".to_string(),
                "x86_64-pc-windows-gnu".to_string(),
                "--message-format=short".to_string()
            ]
        );
    }

    #[test]
    fn shared_target_dir_configura_e_limpa() {
        let mut mgr = ExecutionManager::new();
        assert!(mgr.shared_target_dir().is_none());
        let p = PathBuf::from("/tmp/rustlab-shared-target-test");
        mgr.set_shared_target_dir(Some(p.clone()));
        assert_eq!(mgr.shared_target_dir(), Some(p.as_path()));
        mgr.set_shared_target_dir(None);
        assert!(mgr.shared_target_dir().is_none());
    }
}
