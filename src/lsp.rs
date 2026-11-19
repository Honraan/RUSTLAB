//! Inteligência de IDE para o Modo Criar: cliente LSP (`rust-analyzer`) opcional
//! e índice determinístico de símbolos do workspace sempre disponível.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::execution::{DiagLevel, Diagnostic};
use crate::json::JsonValue;
use crate::projects::Project;

// ---------------------------------------------------------------------------
// Índice determinístico de projeto (offline, zero processo externo)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Fn,
    Struct,
    Enum,
    Trait,
    Type,
    Const,
    Static,
    Mod,
    Macro,
}

impl SymbolKind {
    pub fn label(self) -> &'static str {
        match self {
            SymbolKind::Fn => "fn",
            SymbolKind::Struct => "struct",
            SymbolKind::Enum => "enum",
            SymbolKind::Trait => "trait",
            SymbolKind::Type => "type",
            SymbolKind::Const => "const",
            SymbolKind::Static => "static",
            SymbolKind::Mod => "mod",
            SymbolKind::Macro => "macro",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSymbol {
    pub name: String,
    pub kind: SymbolKind,
    /// Caminho completo do arquivo no projeto.
    pub file: PathBuf,
    /// Índice de linha (0-based).
    pub line: usize,
    /// Coluna de caractere do identificador (0-based).
    pub column: usize,
    /// Comentários `///` imediatos, quando presentes.
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub file: PathBuf,
    /// Índice de linha (0-based).
    pub line: usize,
    /// Coluna de caractere inicial da ocorrência (0-based).
    pub column: usize,
    pub preview: String,
}

#[derive(Debug, Default, Clone)]
pub struct WorkspaceIndex {
    symbols: Vec<WorkspaceSymbol>,
}

impl WorkspaceIndex {
    /// Reindexa todos os arquivos `.rs` listados do projeto.
    pub fn reindex(project: &Project, files: &[PathBuf]) -> Self {
        let mut symbols = Vec::new();
        for file in files {
            if file.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let Ok(source) = project.read(file) else {
                continue;
            };
            symbols.extend(index_rust_source(file, &source));
        }
        Self { symbols }
    }

    pub fn symbols(&self) -> &[WorkspaceSymbol] {
        &self.symbols
    }

    /// Nomes únicos de símbolos do projeto ordenados alfabeticamente para autocompletar.
    pub fn symbol_names(&self) -> Vec<String> {
        let set: BTreeSet<String> = self.symbols.iter().map(|s| s.name.clone()).collect();
        set.into_iter().collect()
    }

    /// Encontra a definição de `ident`, priorizando `preferred_file`.
    pub fn find_definition(
        &self,
        ident: &str,
        preferred_file: Option<&Path>,
    ) -> Option<&WorkspaceSymbol> {
        if let Some(pref) = preferred_file
            && let Some(sym) = self
                .symbols
                .iter()
                .find(|s| s.name == ident && s.file == pref)
        {
            return Some(sym);
        }
        self.symbols.iter().find(|s| s.name == ident)
    }

    /// Retorna um resumo legível de documentação do símbolo no projeto, se houver.
    pub fn find_doc(&self, ident: &str) -> Option<String> {
        let sym = self.find_definition(ident, None)?;
        let file_name = sym
            .file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("src");
        let loc = format!(
            "{} `{}` ({}:{})",
            sym.kind.label(),
            sym.name,
            file_name,
            sym.line + 1
        );
        match &sym.doc {
            Some(doc) if !doc.is_empty() => Some(format!("{loc} — {doc}")),
            _ => Some(loc),
        }
    }
}

/// Busca texto livre (insensível a maiúsculas) nos arquivos listados do projeto.
pub fn search_project_text(
    project: &Project,
    files: &[PathBuf],
    query: &str,
    max_hits: usize,
) -> Vec<SearchMatch> {
    let q = query.trim();
    if q.is_empty() || max_hits == 0 {
        return Vec::new();
    }
    let needle = q.to_lowercase();
    let mut hits = Vec::new();
    for file in files {
        let Ok(content) = project.read(file) else {
            continue;
        };
        for (line_idx, raw_line) in content.lines().enumerate() {
            let lowered = raw_line.to_lowercase();
            if let Some(byte_pos) = lowered.find(&needle) {
                let col = lowered[..byte_pos].chars().count();
                hits.push(SearchMatch {
                    file: file.clone(),
                    line: line_idx,
                    column: col,
                    preview: raw_line.trim().to_string(),
                });
                if hits.len() >= max_hits {
                    return hits;
                }
            }
        }
    }
    hits
}

/// Extrai o identificador Rust sob ou imediatamente adjacente ao cursor (0-based).
pub fn word_at_cursor(line: &str, char_col: usize) -> Option<String> {
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        return None;
    }
    let mut idx = char_col.min(chars.len());
    if idx == chars.len() || !is_ident_char(chars[idx]) {
        if idx > 0 && is_ident_char(chars[idx - 1]) {
            idx -= 1;
        } else {
            return None;
        }
    }
    let mut start = idx;
    while start > 0 && is_ident_char(chars[start - 1]) {
        start -= 1;
    }
    let mut end = idx + 1;
    while end < chars.len() && is_ident_char(chars[end]) {
        end += 1;
    }
    let word: String = chars[start..end].iter().collect();
    if word.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        None
    } else {
        Some(word)
    }
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn index_rust_source(file: &Path, source: &str) -> Vec<WorkspaceSymbol> {
    let mut symbols = Vec::new();
    let mut pending_docs: Vec<String> = Vec::new();
    let mut in_block_comment = false;

    for (line_idx, raw) in source.lines().enumerate() {
        let trimmed = raw.trim();
        if in_block_comment {
            if let Some(end) = trimmed.find("*/") {
                in_block_comment = false;
                let rest = trimmed[end + 2..].trim();
                if rest.is_empty() {
                    continue;
                }
            } else {
                continue;
            }
        }
        if trimmed.starts_with("/*") && !trimmed.contains("*/") {
            in_block_comment = true;
            pending_docs.clear();
            continue;
        }
        if let Some(doc_line) = trimmed.strip_prefix("///") {
            pending_docs.push(doc_line.trim().to_string());
            continue;
        }
        if trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("#[") || trimmed.starts_with("#![") {
            continue;
        }
        if trimmed.is_empty() {
            pending_docs.clear();
            continue;
        }

        if let Some((kind, name)) = parse_decl_line(trimmed) {
            let column = find_ident_column(raw, &name).unwrap_or(0);
            let doc = if pending_docs.is_empty() {
                None
            } else {
                Some(pending_docs.join(" "))
            };
            symbols.push(WorkspaceSymbol {
                name,
                kind,
                file: file.to_path_buf(),
                line: line_idx,
                column,
                doc,
            });
        }
        pending_docs.clear();
    }
    symbols
}

fn find_ident_column(raw_line: &str, ident: &str) -> Option<usize> {
    let bytes = raw_line.as_bytes();
    let mut pos = 0;
    while let Some(found) = raw_line[pos..].find(ident) {
        let abs = pos + found;
        let left_ok = abs == 0 || !is_ident_char(bytes[abs - 1] as char);
        let right_idx = abs + ident.len();
        let right_ok = right_idx >= bytes.len() || !is_ident_char(bytes[right_idx] as char);
        if left_ok && right_ok {
            return Some(raw_line[..abs].chars().count());
        }
        pos = abs + 1;
    }
    None
}

fn parse_decl_line(trimmed: &str) -> Option<(SymbolKind, String)> {
    if let Some(rest) = trimmed.strip_prefix("macro_rules!") {
        return take_ident(rest.trim_start()).map(|name| (SymbolKind::Macro, name));
    }

    let mut rest = trimmed;
    rest = strip_visibility(rest);
    loop {
        if let Some(next) = strip_keyword(rest, "async") {
            rest = next;
        } else if let Some(next) = strip_keyword(rest, "unsafe") {
            rest = next;
        } else if let Some(next) = rest.strip_prefix("const fn ") {
            return take_ident(next.trim_start()).map(|name| (SymbolKind::Fn, name));
        } else {
            break;
        }
    }

    let pairs = [
        ("fn", SymbolKind::Fn),
        ("struct", SymbolKind::Struct),
        ("enum", SymbolKind::Enum),
        ("trait", SymbolKind::Trait),
        ("type", SymbolKind::Type),
        ("const", SymbolKind::Const),
        ("mod", SymbolKind::Mod),
    ];
    for (kw, kind) in pairs {
        if let Some(after) = strip_keyword(rest, kw) {
            return take_ident(after).map(|name| (kind, name));
        }
    }
    if let Some(after_static) = strip_keyword(rest, "static") {
        let after = strip_keyword(after_static, "mut").unwrap_or(after_static);
        return take_ident(after).map(|name| (SymbolKind::Static, name));
    }
    None
}

fn strip_visibility(mut s: &str) -> &str {
    if let Some(after_pub) = s.strip_prefix("pub") {
        let rest = after_pub.trim_start();
        if rest.starts_with('(')
            && let Some(close) = rest.find(')')
        {
            return rest[close + 1..].trim_start();
        }
        if after_pub.starts_with(char::is_whitespace) {
            s = rest;
        }
    }
    s
}

fn strip_keyword<'a>(s: &'a str, kw: &str) -> Option<&'a str> {
    let rest = s.strip_prefix(kw)?;
    if rest.starts_with(char::is_whitespace) {
        Some(rest.trim_start())
    } else {
        None
    }
}

fn take_ident(s: &str) -> Option<String> {
    let name: String = s.chars().take_while(|c| is_ident_char(*c)).collect();
    if name.is_empty() || name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        None
    } else {
        Some(name)
    }
}

// ---------------------------------------------------------------------------
// Cliente LSP (`rust-analyzer`) via JSON-RPC 2.0 sobre pipes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlayHint {
    pub line: usize,
    pub col: usize,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LspEvent {
    Initialized,
    Diagnostics {
        file: PathBuf,
        items: Vec<Diagnostic>,
    },
    Hover {
        request_id: u64,
        summary: Option<String>,
    },
    Definition {
        request_id: u64,
        location: Option<(PathBuf, usize, usize)>,
    },
    Completion {
        request_id: u64,
        labels: Vec<String>,
    },
    InlayHints {
        request_id: u64,
        hints: Vec<InlayHint>,
    },
    Closed {
        reason: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingKind {
    Initialize,
    Hover,
    Definition,
    Completion,
    InlayHints,
}

/// Inferência determinística offline de tipos para inlay hints (`let x = ...` sem `: Tipo`).
pub fn infer_local_inlay_hints(code: &str) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    let mut known_types: BTreeMap<String, String> = BTreeMap::new();
    for (idx, raw) in code.lines().enumerate() {
        let trimmed = raw.split("//").next().unwrap_or("").trim();
        let Some(rest) = trimmed.strip_prefix("let ") else {
            continue;
        };
        let Some((lhs, rhs)) = rest.split_once('=') else {
            continue;
        };
        let lhs_trim = lhs.trim();
        let var_part = lhs_trim.strip_prefix("mut ").unwrap_or(lhs_trim).trim();
        if let Some((vname, explicit_ty)) = var_part.split_once(':') {
            known_types.insert(vname.trim().to_string(), explicit_ty.trim().to_string());
            continue;
        }
        let ident = take_ident(var_part).unwrap_or_default();
        if ident.is_empty() || ident == "_" {
            continue;
        }
        let rhs_clean = rhs.trim().trim_end_matches(';').trim();
        let inferred = if rhs_clean == "true" || rhs_clean == "false" {
            Some("bool".to_string())
        } else if rhs_clean.starts_with('"') {
            Some("&str".to_string())
        } else if rhs_clean.starts_with("String::")
            || rhs_clean.ends_with(".to_string()")
            || rhs_clean.contains("format!(")
        {
            Some("String".to_string())
        } else if rhs_clean.starts_with("vec![") || rhs_clean.starts_with("Vec::") {
            Some("Vec<_>".to_string())
        } else if rhs_clean.starts_with("Some(") {
            Some("Option<_>".to_string())
        } else if rhs_clean.starts_with("Ok(") || rhs_clean.starts_with("Err(") {
            Some("Result<_, _>".to_string())
        } else if rhs_clean.ends_with(".len()") || rhs_clean.ends_with(".count()") {
            Some("usize".to_string())
        } else if rhs_clean.parse::<i64>().is_ok() {
            Some("i32".to_string())
        } else if rhs_clean.parse::<f64>().is_ok() {
            Some("f64".to_string())
        } else if let Some(ty) = rhs_clean.split("::").next().filter(|t| {
            t.chars().next().is_some_and(|c| c.is_ascii_uppercase()) && rhs_clean.contains("::new(")
        }) {
            Some(ty.to_string())
        } else if let Some(ty) = rhs_clean.split('{').next().map(str::trim).filter(|t| {
            rhs_clean.contains('{')
                && t.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                && t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        }) {
            Some(ty.to_string())
        } else if let Some(target) = rhs_clean.strip_prefix("&mut ").and_then(take_ident) {
            known_types.get(&target).map(|t| format!("&mut {t}"))
        } else if let Some(target) = rhs_clean.strip_prefix('&').and_then(take_ident) {
            known_types.get(&target).map(|t| format!("&{t}"))
        } else {
            take_ident(rhs_clean).and_then(|s| {
                (s == rhs_clean)
                    .then(|| known_types.get(&s).cloned())
                    .flatten()
            })
        };
        if let Some(ty) = inferred {
            known_types.insert(ident.clone(), ty.clone());
            let col = raw.find(&ident).map(|c| c + ident.len()).unwrap_or(0);
            hints.push(InlayHint {
                line: idx,
                col,
                label: format!(": {ty}"),
            });
        }
    }
    hints
}

#[derive(Debug, Clone)]
enum OutboundMsg {
    Frame(Vec<u8>),
    Shutdown,
}

/// Procura um binário funcional de `rust-analyzer`, ignorando proxies do `rustup`
/// quando o componente não estiver instalado.
pub fn find_rust_analyzer() -> Option<PathBuf> {
    if let Some(explicit) = std::env::var_os("RUSTLAB_RUST_ANALYZER") {
        let candidate = PathBuf::from(explicit);
        if probe_rust_analyzer(&candidate) {
            return Some(candidate);
        }
    }

    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path).map(|dir| dir.join("rust-analyzer")));
    }
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        candidates.push(home.join(".cargo/bin/rust-analyzer"));
        if let Ok(entries) = std::fs::read_dir(home.join(".rustup/toolchains")) {
            let mut dirs: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
            dirs.sort();
            candidates.extend(dirs.into_iter().map(|d| d.join("bin/rust-analyzer")));
        }
    }

    let mut seen = BTreeSet::new();
    candidates
        .into_iter()
        .filter(|p| seen.insert(p.clone()))
        .find(|p| probe_rust_analyzer(p))
}

fn probe_rust_analyzer(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    Command::new(path)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

pub struct LspClient {
    root: PathBuf,
    tx_out: mpsc::Sender<OutboundMsg>,
    rx_events: Receiver<LspEvent>,
    pending: Arc<Mutex<BTreeMap<u64, PendingKind>>>,
    next_id: u64,
    ready: bool,
    child: Option<Child>,
    reader_handle: Option<thread::JoinHandle<()>>,
    writer_handle: Option<thread::JoinHandle<()>>,
}

impl std::fmt::Debug for LspClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LspClient")
            .field("root", &self.root)
            .field("ready", &self.ready)
            .finish()
    }
}

impl LspClient {
    pub fn start(root: &Path) -> Result<Self, String> {
        let bin = find_rust_analyzer().ok_or_else(|| {
            "rust-analyzer não encontrado (usando índice local do workspace)".to_string()
        })?;
        Self::start_with_binary(&bin, root)
    }

    pub fn start_with_binary(bin: &Path, root: &Path) -> Result<Self, String> {
        let mut child = Command::new(bin)
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| format!("falha ao iniciar rust-analyzer: {err}"))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "stdin do rust-analyzer indisponível".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "stdout do rust-analyzer indisponível".to_string())?;

        let (tx_out, rx_out) = mpsc::channel::<OutboundMsg>();
        let (tx_events, rx_events) = mpsc::channel::<LspEvent>();
        let pending = Arc::new(Mutex::new(BTreeMap::new()));

        let writer_handle = thread::spawn(move || writer_loop(stdin, rx_out));
        let reader_pending = Arc::clone(&pending);
        let reader_handle = thread::spawn(move || reader_loop(stdout, tx_events, reader_pending));

        let mut client = Self {
            root: root.to_path_buf(),
            tx_out,
            rx_events,
            pending,
            next_id: 1,
            ready: false,
            child: Some(child),
            reader_handle: Some(reader_handle),
            writer_handle: Some(writer_handle),
        };
        client.send_initialize();
        Ok(client)
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn poll_events(&mut self) -> Vec<LspEvent> {
        let mut out = Vec::new();
        while let Ok(ev) = self.rx_events.try_recv() {
            match &ev {
                LspEvent::Initialized => {
                    self.ready = true;
                    self.send_initialized();
                }
                LspEvent::Closed { .. } => {
                    self.ready = false;
                }
                _ => {}
            }
            out.push(ev);
        }
        out
    }

    pub fn did_open(&mut self, file: &Path, text: &str) {
        let params = JsonValue::object(vec![(
            "textDocument",
            JsonValue::object(vec![
                ("uri", JsonValue::string(path_to_uri(file))),
                ("languageId", JsonValue::string("rust")),
                ("version", JsonValue::int(1)),
                ("text", JsonValue::string(text)),
            ]),
        )]);
        self.send_notification("textDocument/didOpen", params);
    }

    pub fn did_change(&mut self, file: &Path, version: i64, text: &str) {
        let params = JsonValue::object(vec![
            (
                "textDocument",
                JsonValue::object(vec![
                    ("uri", JsonValue::string(path_to_uri(file))),
                    ("version", JsonValue::int(version)),
                ]),
            ),
            (
                "contentChanges",
                JsonValue::array(vec![JsonValue::object(vec![(
                    "text",
                    JsonValue::string(text),
                )])]),
            ),
        ]);
        self.send_notification("textDocument/didChange", params);
    }

    pub fn request_hover(&mut self, file: &Path, line0: usize, col0: usize) -> Option<u64> {
        if !self.ready {
            return None;
        }
        let params = text_doc_position(file, line0, col0);
        Some(self.send_request("textDocument/hover", params, PendingKind::Hover))
    }

    pub fn request_definition(&mut self, file: &Path, line0: usize, col0: usize) -> Option<u64> {
        if !self.ready {
            return None;
        }
        let params = text_doc_position(file, line0, col0);
        Some(self.send_request("textDocument/definition", params, PendingKind::Definition))
    }

    pub fn request_completion(&mut self, file: &Path, line0: usize, col0: usize) -> Option<u64> {
        if !self.ready {
            return None;
        }
        let params = text_doc_position(file, line0, col0);
        Some(self.send_request("textDocument/completion", params, PendingKind::Completion))
    }

    pub fn request_inlay_hints(&mut self, file: &Path, line_count: usize) -> Option<u64> {
        if !self.ready {
            return None;
        }
        let end_line = line_count.max(1) as u64;
        let params = JsonValue::object(vec![
            (
                "textDocument",
                JsonValue::object(vec![("uri", JsonValue::string(path_to_uri(file)))]),
            ),
            (
                "range",
                JsonValue::object(vec![
                    (
                        "start",
                        JsonValue::object(vec![
                            ("line", JsonValue::uint(0)),
                            ("character", JsonValue::uint(0)),
                        ]),
                    ),
                    (
                        "end",
                        JsonValue::object(vec![
                            ("line", JsonValue::uint(end_line)),
                            ("character", JsonValue::uint(0)),
                        ]),
                    ),
                ]),
            ),
        ]);
        Some(self.send_request("textDocument/inlayHint", params, PendingKind::InlayHints))
    }

    pub fn shutdown(&mut self) {
        if self.child.is_none() {
            return;
        }
        let req = JsonValue::object(vec![
            ("jsonrpc", JsonValue::string("2.0")),
            ("id", JsonValue::uint(self.alloc_id())),
            ("method", JsonValue::string("shutdown")),
            ("params", JsonValue::null()),
        ]);
        let _ = self.tx_out.send(OutboundMsg::Frame(encode_lsp_frame(&req)));
        let exit = JsonValue::object(vec![
            ("jsonrpc", JsonValue::string("2.0")),
            ("method", JsonValue::string("exit")),
            ("params", JsonValue::null()),
        ]);
        let _ = self
            .tx_out
            .send(OutboundMsg::Frame(encode_lsp_frame(&exit)));
        let _ = self.tx_out.send(OutboundMsg::Shutdown);

        if let Some(mut child) = self.child.take() {
            for _ in 0..10 {
                if let Ok(Some(_)) = child.try_wait() {
                    break;
                }
                thread::sleep(std::time::Duration::from_millis(15));
            }
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(handle) = self.writer_handle.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.join();
        }
        self.ready = false;
    }

    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn send_initialize(&mut self) {
        let root_uri = path_to_uri(&self.root);
        let params = JsonValue::object(vec![
            ("processId", JsonValue::uint(u64::from(std::process::id()))),
            ("rootUri", JsonValue::string(root_uri)),
            (
                "capabilities",
                JsonValue::object(vec![(
                    "textDocument",
                    JsonValue::object(vec![(
                        "publishDiagnostics",
                        JsonValue::object(vec![("relatedInformation", JsonValue::bool(false))]),
                    )]),
                )]),
            ),
        ]);
        self.send_request("initialize", params, PendingKind::Initialize);
    }

    fn send_initialized(&mut self) {
        self.send_notification(
            "initialized",
            JsonValue::object(Vec::<(&str, JsonValue)>::new()),
        );
    }

    fn send_request(&mut self, method: &str, params: JsonValue, kind: PendingKind) -> u64 {
        let id = self.alloc_id();
        if let Ok(mut guard) = self.pending.lock() {
            guard.insert(id, kind);
        }
        let msg = JsonValue::object(vec![
            ("jsonrpc", JsonValue::string("2.0")),
            ("id", JsonValue::uint(id)),
            ("method", JsonValue::string(method)),
            ("params", params),
        ]);
        let _ = self.tx_out.send(OutboundMsg::Frame(encode_lsp_frame(&msg)));
        id
    }

    fn send_notification(&mut self, method: &str, params: JsonValue) {
        let msg = JsonValue::object(vec![
            ("jsonrpc", JsonValue::string("2.0")),
            ("method", JsonValue::string(method)),
            ("params", params),
        ]);
        let _ = self.tx_out.send(OutboundMsg::Frame(encode_lsp_frame(&msg)));
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn text_doc_position(file: &Path, line0: usize, col0: usize) -> JsonValue {
    JsonValue::object(vec![
        (
            "textDocument",
            JsonValue::object(vec![("uri", JsonValue::string(path_to_uri(file)))]),
        ),
        (
            "position",
            JsonValue::object(vec![
                ("line", JsonValue::uint(line0 as u64)),
                ("character", JsonValue::uint(col0 as u64)),
            ]),
        ),
    ])
}

fn writer_loop(mut stdin: ChildStdin, rx: mpsc::Receiver<OutboundMsg>) {
    while let Ok(msg) = rx.recv() {
        match msg {
            OutboundMsg::Frame(bytes) => {
                if stdin.write_all(&bytes).is_err() || stdin.flush().is_err() {
                    break;
                }
            }
            OutboundMsg::Shutdown => break,
        }
    }
}

fn reader_loop(
    stdout: std::process::ChildStdout,
    tx_events: mpsc::Sender<LspEvent>,
    pending: Arc<Mutex<BTreeMap<u64, PendingKind>>>,
) {
    let mut reader = BufReader::new(stdout);
    loop {
        match decode_lsp_frame(&mut reader) {
            Ok(Some(payload)) => {
                if let Ok(json) = JsonValue::parse(&payload)
                    && let Some(event) = handle_lsp_message(&json, &pending)
                {
                    let _ = tx_events.send(event);
                }
            }
            Ok(None) => {
                let _ = tx_events.send(LspEvent::Closed {
                    reason: "rust-analyzer encerrou a conexão".to_string(),
                });
                break;
            }
            Err(err) => {
                let _ = tx_events.send(LspEvent::Closed {
                    reason: format!("erro de leitura LSP: {err}"),
                });
                break;
            }
        }
    }
}

pub fn encode_lsp_frame(value: &JsonValue) -> Vec<u8> {
    let body = value.to_json_string();
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    let mut out = header.into_bytes();
    out.extend_from_slice(body.as_bytes());
    out
}

pub fn decode_lsp_frame<R: BufRead>(reader: &mut R) -> io::Result<Option<String>> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, val)) = trimmed.split_once(':')
            && name.eq_ignore_ascii_case("Content-Length")
        {
            content_length = val.trim().parse::<usize>().ok();
        }
    }
    let Some(len) = content_length else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cabeçalho LSP sem Content-Length",
        ));
    };
    if len > 16 * 1024 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "mensagem LSP acima do limite de 16 MiB",
        ));
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    String::from_utf8(buf)
        .map(Some)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

fn handle_lsp_message(
    msg: &JsonValue,
    pending: &Arc<Mutex<BTreeMap<u64, PendingKind>>>,
) -> Option<LspEvent> {
    if let Some(id) = msg.get("id").and_then(JsonValue::as_u64) {
        let kind = pending.lock().ok()?.remove(&id)?;
        let result = msg.get("result");
        return match kind {
            PendingKind::Initialize => Some(LspEvent::Initialized),
            PendingKind::Hover => Some(LspEvent::Hover {
                request_id: id,
                summary: result.and_then(extract_hover_text),
            }),
            PendingKind::Definition => Some(LspEvent::Definition {
                request_id: id,
                location: result.and_then(extract_definition_location),
            }),
            PendingKind::Completion => Some(LspEvent::Completion {
                request_id: id,
                labels: result.map(extract_completion_labels).unwrap_or_default(),
            }),
            PendingKind::InlayHints => Some(LspEvent::InlayHints {
                request_id: id,
                hints: result.map(extract_inlay_hints).unwrap_or_default(),
            }),
        };
    }

    let method = msg.get("method").and_then(JsonValue::as_str)?;
    if method == "textDocument/publishDiagnostics" {
        let params = msg.get("params")?;
        let uri = params.get("uri").and_then(JsonValue::as_str)?;
        let file = uri_to_path(uri)?;
        let diags_arr = params.get("diagnostics").and_then(JsonValue::as_array)?;
        let file_str = file.to_string_lossy().to_string();
        let mut items = Vec::new();
        for item in diags_arr {
            let Some(message) = item.get("message").and_then(JsonValue::as_str) else {
                continue;
            };
            let sev = item
                .get("severity")
                .and_then(JsonValue::as_u64)
                .unwrap_or(1);
            let level = match sev {
                1 => DiagLevel::Error,
                2 => DiagLevel::Warning,
                _ => DiagLevel::Note,
            };
            let start = item.get_path(&["range", "start"]);
            let line = start
                .and_then(|s| s.get("line"))
                .and_then(JsonValue::as_u64)
                .map(|n| (n + 1) as u32);
            let col = start
                .and_then(|s| s.get("character"))
                .and_then(JsonValue::as_u64)
                .map(|n| (n + 1) as u32);
            items.push(Diagnostic {
                level,
                message: message.lines().next().unwrap_or(message).to_string(),
                file: Some(file_str.clone()),
                line,
                col,
            });
        }
        return Some(LspEvent::Diagnostics { file, items });
    }
    None
}

fn extract_hover_text(result: &JsonValue) -> Option<String> {
    let contents = result.get("contents")?;
    if let Some(s) = contents.as_str() {
        return clean_hover(s);
    }
    if let Some(v) = contents.get("value").and_then(JsonValue::as_str) {
        return clean_hover(v);
    }
    if let Some(arr) = contents.as_array() {
        let joined: Vec<&str> = arr
            .iter()
            .filter_map(|item| {
                item.as_str()
                    .or_else(|| item.get("value").and_then(JsonValue::as_str))
            })
            .collect();
        return clean_hover(&joined.join(" "));
    }
    None
}

fn clean_hover(raw: &str) -> Option<String> {
    let text: Vec<&str> = raw
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("```"))
        .collect();
    if text.is_empty() {
        None
    } else {
        Some(text.join(" — "))
    }
}

fn extract_definition_location(result: &JsonValue) -> Option<(PathBuf, usize, usize)> {
    let loc = if let Some(arr) = result.as_array() {
        arr.first()?
    } else {
        result
    };
    let uri = loc
        .get("uri")
        .or_else(|| loc.get("targetUri"))
        .and_then(JsonValue::as_str)?;
    let start = loc
        .get_path(&["range", "start"])
        .or_else(|| loc.get_path(&["targetSelectionRange", "start"]))
        .or_else(|| loc.get_path(&["targetRange", "start"]))?;
    let line = start.get("line").and_then(JsonValue::as_usize)?;
    let col = start.get("character").and_then(JsonValue::as_usize)?;
    let path = uri_to_path(uri)?;
    Some((path, line, col))
}

fn extract_completion_labels(result: &JsonValue) -> Vec<String> {
    let items = result
        .as_array()
        .or_else(|| result.get("items").and_then(JsonValue::as_array));
    let Some(items) = items else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in items {
        if let Some(label) = item.get("label").and_then(JsonValue::as_str) {
            let clean = label.split('(').next().unwrap_or(label).trim().to_string();
            if !clean.is_empty() && !out.contains(&clean) {
                out.push(clean);
            }
        }
        if out.len() >= 32 {
            break;
        }
    }
    out
}

fn extract_inlay_hints(result: &JsonValue) -> Vec<InlayHint> {
    let Some(items) = result.as_array() else {
        return Vec::new();
    };
    let mut hints = Vec::new();
    for item in items {
        let Some(pos) = item.get("position") else {
            continue;
        };
        let Some(line) = pos.get("line").and_then(JsonValue::as_usize) else {
            continue;
        };
        let Some(col) = pos.get("character").and_then(JsonValue::as_usize) else {
            continue;
        };
        let label = if let Some(s) = item.get("label").and_then(JsonValue::as_str) {
            s.trim().to_string()
        } else if let Some(parts) = item.get("label").and_then(JsonValue::as_array) {
            parts
                .iter()
                .filter_map(|p| p.get("value").and_then(JsonValue::as_str))
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_string()
        } else {
            continue;
        };
        if !label.is_empty() {
            hints.push(InlayHint { line, col, label });
        }
    }
    hints
}

pub fn path_to_uri(path: &Path) -> String {
    let raw = path.to_string_lossy();
    let mut out = String::from("file://");
    for byte in raw.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'.' | b'_' | b'~' => {
                out.push(*byte as char)
            }
            _ => {
                use std::fmt::Write as _;
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}

pub fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let raw = uri.strip_prefix("file://")?;
    let mut bytes = Vec::with_capacity(raw.len());
    let raw_bytes = raw.as_bytes();
    let mut i = 0;
    while i < raw_bytes.len() {
        if raw_bytes[i] == b'%'
            && i + 2 < raw_bytes.len()
            && let Ok(hex) = std::str::from_utf8(&raw_bytes[i + 1..i + 3])
            && let Ok(val) = u8::from_str_radix(hex, 16)
        {
            bytes.push(val);
            i += 3;
            continue;
        }
        bytes.push(raw_bytes[i]);
        i += 1;
    }
    String::from_utf8(bytes).ok().map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn indice_indexa_simbolos_docs_e_busca_definicao() {
        let dir = tempfile_dir("idx");
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(dir.join("src")).unwrap();
        let main_rs = dir.join("src/main.rs");
        let utils_rs = dir.join("src/utils.rs");
        std::fs::write(
            &main_rs,
            "mod utils;\nfn main() {\n    let _ = utils::somar(1, 2);\n}\n",
        )
        .unwrap();
        std::fs::write(
            &utils_rs,
            "/// Soma dois inteiros positivos.\npub fn somar(a: i32, b: i32) -> i32 {\n    a + b\n}\n\npub struct Ponto { pub x: i32 }\n",
        )
        .unwrap();

        let project = Project::detect(&dir).unwrap();
        let files = project.source_files();
        let index = WorkspaceIndex::reindex(&project, &files);

        let sym = index.find_definition("somar", Some(&main_rs)).unwrap();
        assert_eq!(sym.kind, SymbolKind::Fn);
        assert_eq!(sym.file, utils_rs);
        assert_eq!(sym.line, 1);
        assert_eq!(
            index.find_doc("somar").as_deref(),
            Some("fn `somar` (utils.rs:2) — Soma dois inteiros positivos.")
        );
        assert!(index.symbol_names().contains(&"Ponto".to_string()));

        let hits = search_project_text(&project, &files, "SOMAR", 10);
        assert_eq!(hits.len(), 2);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn extrai_palavra_no_cursor() {
        assert_eq!(
            word_at_cursor("  let total = somar(a);", 15).as_deref(),
            Some("somar")
        );
        assert_eq!(word_at_cursor("somar(a)", 5).as_deref(), Some("somar"));
        assert_eq!(word_at_cursor("123 + 4", 1), None);
    }

    #[test]
    fn framing_lsp_codifica_e_decodifica_e_trata_mensagens() {
        let req = JsonValue::object(vec![
            ("jsonrpc", JsonValue::string("2.0")),
            ("id", JsonValue::int(1)),
            ("method", JsonValue::string("initialize")),
        ]);
        let framed = encode_lsp_frame(&req);
        let mut cursor = Cursor::new(framed);
        let decoded = decode_lsp_frame(&mut cursor).unwrap().unwrap();
        let reparsed = JsonValue::parse(&decoded).unwrap();
        assert_eq!(
            reparsed.get("method").and_then(JsonValue::as_str),
            Some("initialize")
        );

        let pending = Arc::new(Mutex::new(BTreeMap::from([(2, PendingKind::Hover)])));
        let hover_msg = JsonValue::parse(
            r#"{"jsonrpc":"2.0","id":2,"result":{"contents":{"kind":"markdown","value":"```rust\nfn foo()\n```\nCalcula foo"}}}"#,
        )
        .unwrap();
        let ev = handle_lsp_message(&hover_msg, &pending).unwrap();
        assert_eq!(
            ev,
            LspEvent::Hover {
                request_id: 2,
                summary: Some("fn foo() — Calcula foo".to_string())
            }
        );

        let diag_msg = JsonValue::parse(
            r#"{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":"file:///tmp/rust%20teacher/src/main.rs","diagnostics":[{"severity":1,"message":"tipo esperado","range":{"start":{"line":4,"character":8}}}]}}"#,
        )
        .unwrap();
        let diag_ev = handle_lsp_message(&diag_msg, &pending).unwrap();
        match diag_ev {
            LspEvent::Diagnostics { file, items } => {
                assert_eq!(file, PathBuf::from("/tmp/rust teacher/src/main.rs"));
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].line, Some(5));
                assert_eq!(items[0].col, Some(9));
            }
            other => panic!("evento inesperado: {other:?}"),
        }
    }

    fn tempfile_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "rustlab-lsp-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn inlay_hints_locais_e_resposta_lsp_funcionam() {
        let code = "let count = 42;\nlet name = String::from(\"rust\");\nlet borrow = &name;\n";
        let local = infer_local_inlay_hints(code);
        assert!(local.iter().any(|h| h.line == 0 && h.label == ": i32"));
        assert!(local.iter().any(|h| h.line == 1 && h.label == ": String"));
        assert!(local.iter().any(|h| h.line == 2 && h.label == ": &String"));

        let pending = Arc::new(Mutex::new(BTreeMap::from([(7, PendingKind::InlayHints)])));
        let msg = JsonValue::parse(
            r#"{"jsonrpc":"2.0","id":7,"result":[{"position":{"line":0,"character":9},"label":": usize"}]}"#,
        )
        .unwrap();
        let ev = handle_lsp_message(&msg, &pending).unwrap();
        assert_eq!(
            ev,
            LspEvent::InlayHints {
                request_id: 7,
                hints: vec![InlayHint {
                    line: 0,
                    col: 9,
                    label: ": usize".to_string()
                }],
            }
        );
    }
}
