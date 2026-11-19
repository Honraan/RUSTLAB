//! Estado e lógica do Modo Criar (Fases 3 e 6): tela inicial, prompts,
//! consentimento de execução, workspace com editor/console/diagnósticos.
//!
//! Toda a lógica é pura e testável; o desenho vive em `ui/create.rs`.
//! Fluxo de execução (requisito 10):
//!
//! ```text
//! tecla F5 ──> consentimento? ──não──> overlay explicando riscos
//!                   │sim
//!                   v
//!        salva arquivos sujos ──> ExecutionManager.submit(cargo run)
//!                   │
//!                   v
//!        eventos -> console + diagnóstico + status
//! ```

use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Screen, StatusKind};
use crate::config::{AiProvider, ExecConsent, RemoteAiConsent};
use crate::editor::Editor;
use crate::execution::{
    DiagLevel, Diagnostic, ExecEvent, StreamKind, ToolCommand, parse_diagnostics,
};
use crate::lsp::{
    InlayHint, LspClient, LspEvent, SearchMatch, WorkspaceIndex, infer_local_inlay_hints,
    search_project_text, word_at_cursor,
};
use crate::mentor::{MentorEvent, MentorManager, ModelPickerState};
use crate::projects::{Project, expand_home, resolve_new_project_target};

/// Limite de linhas no console (ring buffer).
pub const MAX_CONSOLE_LINES: usize = 4_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateView {
    Start,
    Workspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Files,
    Editor,
    Console,
    Agent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRole {
    User,
    Agent,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentChatMessage {
    pub role: AgentRole,
    pub sender: String,
    pub text: String,
}

#[derive(Debug)]
pub struct CreateAgentState {
    pub open: bool,
    pub messages: Vec<AgentChatMessage>,
    pub input: String,
    pub scroll: u16,
    pub proposed_code: Option<String>,
    pub manager: MentorManager,
    pub model_picker: ModelPickerState,
    pub consent_pending: Option<String>,
    pub ai_authored_lines: HashSet<String>,
    pub student_modified_since_review: bool,
    pub last_auto_review: Instant,
    pub next_auto_review_interval: Duration,
}

impl CreateAgentState {
    pub fn welcome_text(english: bool) -> String {
        crate::i18n::bi(
            "Chat lateral pronto! Peça funções, correções ou testes.\nAtalhos: /modelo (Alt+O) · /corrigir · /testes · /revisar · /aplicar (Alt+Enter) · /limpar.",
            "Lateral chat ready! Ask for functions, fixes, or tests.\nShortcuts: /model (Alt+O) · /fix · /tests · /review · /apply (Alt+Enter) · /clear.",
            english,
        )
        .to_string()
    }

    pub fn new_for(english: bool) -> Self {
        Self {
            open: false,
            messages: vec![AgentChatMessage {
                role: AgentRole::System,
                sender: "Antigravity".to_string(),
                text: Self::welcome_text(english),
            }],
            input: String::new(),
            scroll: 0,
            proposed_code: None,
            manager: MentorManager::new(),
            model_picker: ModelPickerState::default(),
            consent_pending: None,
            ai_authored_lines: HashSet::new(),
            student_modified_since_review: false,
            last_auto_review: Instant::now(),
            next_auto_review_interval: next_random_review_interval(),
        }
    }
}

impl Default for CreateAgentState {
    fn default() -> Self {
        Self::new_for(crate::i18n::is_english())
    }
}

fn next_random_review_interval() -> Duration {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let mix = nanos ^ u64::from(std::process::id()).wrapping_mul(2_654_435_761);
    Duration::from_secs(900 + (mix % 901))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PromptKind {
    #[default]
    NewProject,
    OpenPath,
    NewFile,
    AddDependency,
    RemoveDependency,
    ProjectSearch,
    SetProjectsDir,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocPopup {
    pub title: String,
    pub body: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PromptState {
    pub kind: PromptKind,
    pub text: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConsoleLine {
    pub kind: StreamKind,
    pub text: String,
}

#[derive(Debug, Default)]
pub struct Console {
    pub lines: VecDeque<ConsoleLine>,
    /// quantas linhas do fim o usuário rolou para cima (0 = colado no fim)
    pub scroll_back: usize,
    pub width: u16,
    pub page_rows: usize,
}

impl Console {
    pub fn push(&mut self, kind: StreamKind, text: impl Into<String>) {
        let text: String = text
            .into()
            .chars()
            .map(|ch| if ch.is_control() { ' ' } else { ch })
            .collect();
        if self.scroll_back > 0 {
            self.scroll_back = self.scroll_back.saturating_add(
                ratatui::widgets::Paragraph::new(text.as_str())
                    .wrap(ratatui::widgets::Wrap { trim: false })
                    .line_count(self.width.max(1)),
            );
        }
        self.lines.push_back(ConsoleLine { kind, text });
        while self.lines.len() > MAX_CONSOLE_LINES {
            self.lines.pop_front();
        }
    }
    pub fn system(&mut self, text: impl Into<String>) {
        self.push(StreamKind::System, text);
    }
    pub fn max_scroll(&self) -> usize {
        ratatui::widgets::Paragraph::new(
            self.lines
                .iter()
                .map(|line| line.text.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .wrap(ratatui::widgets::Wrap { trim: false })
        .line_count(self.width.max(1))
        .saturating_sub(self.page_rows.max(1))
    }
}

/// O que fazer quando um comando assíncrono terminar.
#[derive(Debug, Clone, PartialEq, Eq)]
enum AfterExec {
    None,
    /// `cargo new` terminou: abrir o projeto criado.
    OpenNewProject(PathBuf),
    /// Compilou e deve lançar o binário em janela/processo externo.
    LaunchExternalWindow,
}

#[derive(Debug)]
pub struct CreateState {
    pub view: CreateView,
    pub start_sel: usize,
    pub project: Option<Project>,
    pub files: Vec<PathBuf>,
    pub open_idx: usize,
    pub editor: Editor,
    pub focus: Focus,
    pub console: Console,
    pub diags: Vec<Diagnostic>,
    /// stderr acumulado do comando atual (para parsear diagnósticos)
    stderr_buf: String,
    after_exec: AfterExec,
    pub prompt: Option<PromptState>,
    pub consent_request: Option<ToolCommand>,
    pub spinner_tick: u64,
    pub last_exit_success: Option<bool>,
    pub last_failure: Option<String>,
    pub after_exec_reload: bool,
    pub editor_text_area: ratatui::layout::Rect,
    pub hovered_doc: Option<(String, u16, u16)>,
    pub completion: crate::completion::Completion,
    pub console_input: String,
    pub selected_file: usize,
    pub diagnostic_index: usize,
    pub workspace_index: WorkspaceIndex,
    pub lsp: Option<LspClient>,
    pub lsp_status: String,
    editor_doc_version: i64,
    lsp_pending_hover: Option<u64>,
    lsp_pending_def: Option<u64>,
    lsp_pending_comp: Option<u64>,
    lsp_pending_inlay: Option<u64>,
    pub inlay_hints: Vec<InlayHint>,
    pub project_matches: Vec<SearchMatch>,
    pub project_match_idx: usize,
    pub doc_popup: Option<DocPopup>,
    pub pending_new_project_dir: Option<PathBuf>,
    pub agent: CreateAgentState,
    pub lib_menu_open: bool,
    pub lib_menu_sel: usize,
    pub build_menu_open: bool,
    pub build_menu_sel: usize,
    pub top_lib_btn_area: ratatui::layout::Rect,
    pub top_build_btn_area: ratatui::layout::Rect,
    pub lib_modal_area: ratatui::layout::Rect,
    pub build_modal_area: ratatui::layout::Rect,
    pub lab_modal_area: ratatui::layout::Rect,
    pub lab: crate::lab_tools::LabState,
}

impl Default for CreateState {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateState {
    pub fn new() -> Self {
        Self::new_for(crate::i18n::is_english())
    }

    pub fn new_for(english: bool) -> Self {
        Self {
            view: CreateView::Start,
            start_sel: 0,
            project: None,
            files: Vec::new(),
            open_idx: 0,
            editor: Editor::new("", None),
            focus: Focus::Editor,
            console: Console::default(),
            diags: Vec::new(),
            stderr_buf: String::new(),
            after_exec: AfterExec::None,
            prompt: None,
            consent_request: None,
            spinner_tick: 0,
            last_exit_success: None,
            last_failure: None,
            after_exec_reload: false,
            editor_text_area: ratatui::layout::Rect::default(),
            hovered_doc: None,
            completion: crate::completion::Completion::default(),
            console_input: String::new(),
            selected_file: 0,
            diagnostic_index: 0,
            workspace_index: WorkspaceIndex::default(),
            lsp: None,
            lsp_status: crate::i18n::bi("índice local", "local index", english).to_string(),
            editor_doc_version: 1,
            lsp_pending_hover: None,
            lsp_pending_def: None,
            lsp_pending_comp: None,
            lsp_pending_inlay: None,
            inlay_hints: Vec::new(),
            project_matches: Vec::new(),
            project_match_idx: 0,
            doc_popup: None,
            pending_new_project_dir: None,
            agent: CreateAgentState::new_for(english),
            lib_menu_open: false,
            lib_menu_sel: 0,
            build_menu_open: false,
            build_menu_sel: 0,
            top_lib_btn_area: ratatui::layout::Rect::default(),
            top_build_btn_area: ratatui::layout::Rect::default(),
            lib_modal_area: ratatui::layout::Rect::default(),
            build_modal_area: ratatui::layout::Rect::default(),
            lab_modal_area: ratatui::layout::Rect::default(),
            lab: crate::lab_tools::LabState::default(),
        }
    }

    fn refresh_language_for(&mut self, en: bool) {
        self.lsp_status = match self.lsp_status.as_str() {
            "índice local" | "local index" => {
                crate::i18n::bi("índice local", "local index", en).to_string()
            }
            "iniciando rust-analyzer" | "starting rust-analyzer" => {
                crate::i18n::bi("iniciando rust-analyzer", "starting rust-analyzer", en).to_string()
            }
            "rust-analyzer pronto" | "rust-analyzer ready" => {
                crate::i18n::bi("rust-analyzer pronto", "rust-analyzer ready", en).to_string()
            }
            _ => std::mem::take(&mut self.lsp_status),
        };
        if self.agent.messages.len() == 1
            && self.agent.messages[0].role == AgentRole::System
            && self.agent.messages[0].sender == "Antigravity"
        {
            self.agent.messages[0].text = CreateAgentState::welcome_text(en);
        }
    }

    // ------------------------------------------------------------------
    // Projetos
    // ------------------------------------------------------------------

    /// Fecha o projeto aberto (salvando alterações pendentes) e retorna à
    /// tela inicial do Modo Criar.
    pub fn close_project(&mut self, app: &mut App) {
        let en = app.is_english();
        self.refresh_language_for(en);
        if app.exec.running {
            app.set_status(
                crate::i18n::bi(
                    "aguarde ou cancele antes de fechar o projeto",
                    "wait or cancel before closing the project",
                    en,
                ),
                StatusKind::Warning,
            );
            return;
        }
        if !self.save_ok(app) {
            return;
        }
        if let Some(mut lsp) = self.lsp.take() {
            lsp.shutdown();
        }
        self.agent.manager.cancel();
        self.agent.ai_authored_lines.clear();
        self.agent.student_modified_since_review = false;
        self.agent.last_auto_review = Instant::now();
        self.project = None;
        self.files.clear();
        self.open_idx = 0;
        self.selected_file = 0;
        self.editor = Editor::new("", None);
        self.console.lines.clear();
        self.console_input.clear();
        self.diags.clear();
        self.diagnostic_index = 0;
        self.workspace_index = WorkspaceIndex::default();
        self.lsp_status = crate::i18n::bi("índice local", "local index", en).to_string();
        self.lsp_pending_hover = None;
        self.lsp_pending_def = None;
        self.lsp_pending_comp = None;
        self.lsp_pending_inlay = None;
        self.inlay_hints.clear();
        self.project_matches.clear();
        self.project_match_idx = 0;
        self.doc_popup = None;
        self.hovered_doc = None;
        self.completion = Default::default();
        self.lib_menu_open = false;
        self.build_menu_open = false;
        self.lab.open = false;
        self.lab.debugger.stop();
        self.lab.debugger.breakpoints.clear();
        self.lab.active_quest_id = None;
        self.last_exit_success = None;
        self.last_failure = None;
        self.focus = Focus::Editor;
        self.view = CreateView::Start;
        self.start_sel = 0;
        app.set_status(
            crate::i18n::bi("projeto fechado", "project closed", en),
            StatusKind::Info,
        );
    }

    pub fn open_project(&mut self, root: &Path, app: &mut App) {
        let en = app.is_english();
        self.refresh_language_for(en);
        if app.exec.running {
            app.set_status(
                crate::i18n::bi(
                    "aguarde ou cancele o processo antes de trocar projeto",
                    "wait or cancel the running process before switching projects",
                    en,
                ),
                StatusKind::Warning,
            );
            return;
        }
        if !self.save_ok(app) {
            return;
        }
        self.completion = Default::default();
        let Some(project) = Project::detect(root) else {
            app.set_status(
                crate::i18n::bi(
                    "não achei Cargo.toml nesse caminho",
                    "Cargo.toml not found at that path",
                    en,
                ),
                StatusKind::Warning,
            );
            return;
        };
        let files = project.source_files();
        // prefere src/main.rs; senão o primeiro .rs; senão Cargo.toml
        let open_idx = files
            .iter()
            .position(|f| f.ends_with("src/main.rs"))
            .or_else(|| {
                files
                    .iter()
                    .position(|f| f.extension().is_some_and(|e| e == "rs"))
            })
            .unwrap_or(0);
        let text = match files.get(open_idx).map(|f| project.read(f)) {
            Some(Ok(text)) => text,
            Some(Err(err)) => {
                app.set_status(
                    if en {
                        format!("could not read file: {err}")
                    } else {
                        format!("não consegui ler arquivo: {err}")
                    },
                    StatusKind::Warning,
                );
                return;
            }
            None => {
                app.set_status(
                    crate::i18n::bi(
                        "projeto sem arquivo legível",
                        "project has no readable files",
                        en,
                    ),
                    StatusKind::Warning,
                );
                return;
            }
        };
        let path = files.get(open_idx).cloned();
        if let Some(old_lsp) = self.lsp.as_mut() {
            old_lsp.shutdown();
        }
        self.lsp = None;
        self.workspace_index = WorkspaceIndex::reindex(&project, &files);
        self.project_matches.clear();
        self.project_match_idx = 0;
        self.doc_popup = None;
        self.editor_doc_version = 1;
        self.lsp_pending_hover = None;
        self.lsp_pending_def = None;
        self.lsp_pending_comp = None;
        self.lsp_pending_inlay = None;
        self.inlay_hints.clear();
        match LspClient::start(&project.root) {
            Ok(client) => {
                self.lsp = Some(client);
                self.lsp_status =
                    crate::i18n::bi("iniciando rust-analyzer", "starting rust-analyzer", en)
                        .to_string();
            }
            Err(_) => {
                self.lsp_status = crate::i18n::bi("índice local", "local index", en).to_string();
            }
        }
        self.project = Some(project);
        self.last_exit_success = None;
        self.console.lines.clear();
        self.files = files;
        self.open_idx = open_idx;
        self.selected_file = open_idx;
        self.console_input.clear();
        self.editor = Editor::new(&text, path);
        self.refresh_inlay_hints();
        self.view = CreateView::Workspace;
        self.focus = Focus::Editor;
        self.lib_menu_open = false;
        self.build_menu_open = false;
        self.lab.open = false;
        self.lab.debugger.stop();
        self.lab.debugger.breakpoints.clear();
        self.lab.active_quest_id = None;
        self.agent.ai_authored_lines.clear();
        self.agent.student_modified_since_review = false;
        self.agent.last_auto_review = Instant::now();
        self.diags.clear();
        let proj_name = self.project.as_ref().map(|p| p.name()).unwrap_or_default();
        self.console.system(if en {
            format!(
                "project '{proj_name}' opened — {} file(s)",
                self.files.len()
            )
        } else {
            format!(
                "projeto '{proj_name}' aberto — {} arquivo(s)",
                self.files.len()
            )
        });
        app.config.workspace.last_project = Some(
            self.project
                .as_ref()
                .unwrap()
                .root
                .to_string_lossy()
                .to_string(),
        );
        app.set_status(
            crate::i18n::bi("projeto aberto", "project opened", en),
            StatusKind::Success,
        );
        let recent = app.config.workspace.last_project.clone().unwrap();
        app.config
            .workspace
            .recent_projects
            .retain(|p| p != &recent);
        app.config.workspace.recent_projects.insert(0, recent);
        app.config.workspace.recent_projects.truncate(8);
        if let Err(err) = app.save_config() {
            app.set_status(
                if en {
                    format!("project opened, but recent list not saved: {err}")
                } else {
                    format!("projeto aberto, mas recentes não salvos: {err}")
                },
                StatusKind::Warning,
            );
        }
    }

    pub fn switch_file(&mut self, idx: usize, app: &mut App) {
        let en = app.is_english();
        self.completion = Default::default();
        if idx >= self.files.len() {
            return;
        }
        if !self.save_ok(app) {
            return;
        }
        let path = self.files[idx].clone();
        let text = self.project.as_ref().map(|p| p.read(&path));
        let text = match text {
            Some(Ok(text)) => text,
            _ => {
                app.set_status(
                    crate::i18n::bi(
                        "não consegui ler arquivo; editor preservado",
                        "could not read file; editor preserved",
                        en,
                    ),
                    StatusKind::Warning,
                );
                return;
            }
        };
        self.open_idx = idx;
        self.selected_file = idx;
        self.doc_popup = None;
        self.editor_doc_version = 1;
        self.lab.debugger.stop();
        self.editor = Editor::new(&text, Some(path.clone()));
        self.refresh_inlay_hints();
        if let Some(lsp) = self.lsp.as_mut()
            && lsp.is_ready()
            && path.extension().is_some_and(|e| e == "rs")
        {
            lsp.did_open(&path, &text);
            self.lsp_pending_inlay = lsp.request_inlay_hints(&path, self.editor.buf.line_count());
        }
    }

    pub fn save_current(&mut self, app: &mut App) {
        if !self.editor.dirty {
            return;
        }
        let en = app.is_english();
        let (Some(project), Some(path)) = (&self.project, self.editor.path.clone()) else {
            return;
        };
        let content = self.editor.buf.to_string_lossy();
        match project.save(&path, &content) {
            Ok(()) => {
                self.editor.mark_saved();
                self.workspace_index = WorkspaceIndex::reindex(project, &self.files);
                self.console.system(format!(
                    "{}: {}",
                    crate::i18n::bi("salvo", "saved", en),
                    path.display()
                ));
            }
            Err(e) => {
                app.set_status(
                    format!(
                        "{}: {e}",
                        crate::i18n::bi("erro ao salvar", "save error", en)
                    ),
                    StatusKind::Warning,
                );
            }
        }
    }

    fn save_ok(&mut self, app: &mut App) -> bool {
        self.save_current(app);
        !self.editor.dirty
    }

    pub fn save_before_exit(&mut self, app: &mut App) -> bool {
        self.save_ok(app)
    }

    fn begin_prompt(&mut self, kind: PromptKind, app: &mut App) {
        let en = app.is_english();
        if app.exec.running {
            app.set_status(
                crate::i18n::bi(
                    "aguarde ou cancele o comando antes desta ação",
                    "wait or cancel the running command before this action",
                    en,
                ),
                StatusKind::Warning,
            );
            return;
        }
        if !self.save_ok(app) {
            return;
        }
        self.completion = Default::default();
        self.prompt = Some(PromptState {
            kind,
            ..Default::default()
        });
    }

    pub fn refresh_files(&mut self, app: &mut App) {
        if !self.save_ok(app) {
            return;
        }
        let Some(project) = &self.project else {
            return;
        };
        let en = app.is_english();
        self.files = project.source_files();
        self.workspace_index = WorkspaceIndex::reindex(project, &self.files);
        self.open_idx = self
            .editor
            .path
            .as_ref()
            .and_then(|path| self.files.iter().position(|p| p == path))
            .unwrap_or(0);
        self.selected_file = self.open_idx;
        app.set_status(
            crate::i18n::bi("lista de arquivos atualizada", "file list updated", en),
            StatusKind::Info,
        );
    }

    fn jump_diagnostic(&mut self, app: &mut App) {
        if self.diags.is_empty() {
            app.set_status(
                crate::i18n::bi(
                    "sem diagnósticos para navegar",
                    "no diagnostics to navigate",
                    app.is_english(),
                ),
                StatusKind::Info,
            );
            return;
        }
        let diag = self.diags[self.diagnostic_index % self.diags.len()].clone();
        self.diagnostic_index = (self.diagnostic_index + 1) % self.diags.len();
        let (Some(project), Some(file)) = (&self.project, diag.file) else {
            return;
        };
        let candidate = PathBuf::from(&file);
        let path = if candidate.is_absolute() {
            candidate
        } else {
            project.root.join(file)
        };
        let line0 = diag.line.unwrap_or(1).saturating_sub(1) as usize;
        let col0 = diag.col.unwrap_or(1).saturating_sub(1) as usize;
        self.jump_to_location(&path, line0, col0, app);
    }

    pub fn jump_to_location(&mut self, path: &Path, line0: usize, col0: usize, app: &mut App) {
        if let Some(index) = self.files.iter().position(|p| p == path) {
            if self.editor.path.as_deref() != Some(path) {
                self.switch_file(index, app);
            }
            if self.editor.path.as_deref() != Some(path) {
                return;
            }
            self.editor.cursor.line = line0.min(self.editor.buf.line_count().saturating_sub(1));
            self.editor.cursor.col = col0.min(
                self.editor
                    .buf
                    .line_str(self.editor.cursor.line)
                    .chars()
                    .count(),
            );
            self.editor.scroll_row = self.editor.cursor.line;
            self.focus = Focus::Editor;
        } else {
            app.set_status(
                crate::i18n::bi(
                    "arquivo fora da lista indexada; Ctrl+R atualiza",
                    "file outside indexed list; Ctrl+R refreshes",
                    app.is_english(),
                ),
                StatusKind::Info,
            );
        }
    }

    pub fn cursor_symbol(&self) -> Option<String> {
        let line = self.editor.buf.line_str(self.editor.cursor.line);
        word_at_cursor(&line, self.editor.cursor.col)
    }

    pub fn go_to_definition(&mut self, app: &mut App) {
        let en = app.is_english();
        let Some(symbol) = self.cursor_symbol() else {
            app.set_status(
                crate::i18n::bi(
                    "posicione o cursor sobre um identificador",
                    "place the cursor on an identifier",
                    en,
                ),
                StatusKind::Info,
            );
            return;
        };
        if let Some(project) = &self.project
            && self.editor.dirty
        {
            self.workspace_index = WorkspaceIndex::reindex(project, &self.files);
        }
        let current_file = self.editor.path.as_deref();
        if let Some(sym) = self
            .workspace_index
            .find_definition(&symbol, current_file)
            .cloned()
        {
            self.jump_to_location(&sym.file, sym.line, sym.column, app);
            app.set_status(
                format!(
                    "{} `{}` ({}:{})",
                    crate::i18n::bi("definição de", "definition of", en),
                    sym.name,
                    sym.file
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("src"),
                    sym.line + 1
                ),
                StatusKind::Info,
            );
            return;
        }
        if let (Some(lsp), Some(path)) = (self.lsp.as_mut(), self.editor.path.clone())
            && let Some(id) =
                lsp.request_definition(&path, self.editor.cursor.line, self.editor.cursor.col)
        {
            self.lsp_pending_def = Some(id);
            app.set_status(
                if en {
                    format!("querying `{symbol}` definition in rust-analyzer…")
                } else {
                    format!("consultando definição de `{symbol}` no rust-analyzer…")
                },
                StatusKind::Info,
            );
            return;
        }
        app.set_status(
            if en {
                format!("local definition of `{symbol}` not found")
            } else {
                format!("definição local de `{symbol}` não encontrada")
            },
            StatusKind::Info,
        );
    }

    pub fn show_doc_at_cursor(&mut self, app: &mut App) {
        let en = app.is_english();
        let Some(symbol) = self.cursor_symbol() else {
            app.set_status(
                crate::i18n::bi(
                    "posicione o cursor sobre um símbolo Rust",
                    "place the cursor on a Rust symbol",
                    en,
                ),
                StatusKind::Info,
            );
            return;
        };
        if let Some(project) = &self.project
            && self.editor.dirty
        {
            self.workspace_index = WorkspaceIndex::reindex(project, &self.files);
        }
        let local_doc = self.workspace_index.find_doc(&symbol);
        let resolved = crate::doc_hint::resolve_symbol_doc_for(
            &symbol,
            &self.editor.buf.to_string_lossy(),
            en,
        );
        if let Some(doc) = local_doc {
            self.doc_popup = Some(DocPopup {
                title: symbol.clone(),
                body: doc,
                url: resolved.map(|r| r.best_url_for(en).to_string()),
            });
        } else if let Some(r) = resolved {
            self.doc_popup = Some(DocPopup {
                title: r.title.clone(),
                body: r.summary.clone(),
                url: Some(r.best_url_for(en).to_string()),
            });
        }
        if let (Some(lsp), Some(path)) = (self.lsp.as_mut(), self.editor.path.clone())
            && let Some(id) =
                lsp.request_hover(&path, self.editor.cursor.line, self.editor.cursor.col)
        {
            self.lsp_pending_hover = Some(id);
            if self.doc_popup.is_none() {
                app.set_status(
                    if en {
                        format!("querying `{symbol}` in rust-analyzer…")
                    } else {
                        format!("consultando `{symbol}` no rust-analyzer…")
                    },
                    StatusKind::Info,
                );
            }
            return;
        }
        if self.doc_popup.is_none() {
            app.set_status(
                if en {
                    format!("no local documentation for `{symbol}`")
                } else {
                    format!("sem documentação local para `{symbol}`")
                },
                StatusKind::Info,
            );
        }
    }

    pub fn jump_next_project_match(&mut self, app: &mut App) {
        let en = app.is_english();
        if self.project_matches.is_empty() {
            app.set_status(
                crate::i18n::bi(
                    "nenhuma busca de projeto ativa (Ctrl+G)",
                    "no active project search (Ctrl+G)",
                    en,
                ),
                StatusKind::Info,
            );
            return;
        }
        let idx = self.project_match_idx % self.project_matches.len();
        self.project_match_idx = (idx + 1) % self.project_matches.len();
        let hit = self.project_matches[idx].clone();
        self.jump_to_location(&hit.file, hit.line, hit.column, app);
        app.set_status(
            format!(
                "{} ({}/{}) — {}:{}",
                crate::i18n::bi("busca no projeto", "project search", en),
                idx + 1,
                self.project_matches.len(),
                hit.file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(crate::i18n::bi("arquivo", "file", en)),
                hit.line + 1
            ),
            StatusKind::Info,
        );
    }

    pub fn lsp_pending(&self) -> bool {
        self.lsp_pending_hover.is_some()
            || self.lsp_pending_def.is_some()
            || self.lsp_pending_comp.is_some()
            || self.lsp_pending_inlay.is_some()
    }

    pub fn refresh_inlay_hints(&mut self) {
        let is_rs = self
            .editor
            .path
            .as_ref()
            .is_none_or(|p| p.extension().is_some_and(|e| e == "rs"));
        if !is_rs {
            self.inlay_hints.clear();
            return;
        }
        let code = self.editor.buf.to_string_lossy();
        self.inlay_hints = infer_local_inlay_hints(&code);
    }

    pub fn diagnostic_level_for_open_line(&self, line_1based: u32) -> Option<DiagLevel> {
        let open_path = self.editor.path.as_ref()?;
        let root = self.project.as_ref().map(|p| p.root.as_path());
        let mut best: Option<DiagLevel> = None;
        for diag in &self.diags {
            if diag.line != Some(line_1based) {
                continue;
            }
            let Some(file_str) = diag.file.as_deref() else {
                continue;
            };
            let diag_path = Path::new(file_str);
            let matches_open =
                diag_path == open_path || root.is_some_and(|r| r.join(diag_path) == *open_path);
            if !matches_open {
                continue;
            }
            best = match (best, diag.level) {
                (_, DiagLevel::Error) | (Some(DiagLevel::Error), _) => Some(DiagLevel::Error),
                (_, DiagLevel::Warning) | (Some(DiagLevel::Warning), _) => Some(DiagLevel::Warning),
                _ => Some(DiagLevel::Note),
            };
        }
        best
    }

    pub fn poll_lsp(&mut self, app: &mut App) -> bool {
        let en = app.is_english();
        let Some(events) = self.lsp.as_mut().map(LspClient::poll_events) else {
            return false;
        };
        if events.is_empty() {
            return false;
        }
        for ev in events {
            match ev {
                LspEvent::Initialized => {
                    self.lsp_status =
                        crate::i18n::bi("rust-analyzer pronto", "rust-analyzer ready", en)
                            .to_string();
                    if let (Some(lsp), Some(path)) = (self.lsp.as_mut(), self.editor.path.as_ref())
                        && path.extension().is_some_and(|e| e == "rs")
                    {
                        let text = self.editor.buf.to_string_lossy();
                        lsp.did_open(path, &text);
                        self.lsp_pending_inlay =
                            lsp.request_inlay_hints(path, self.editor.buf.line_count());
                    }
                }
                LspEvent::Diagnostics { file, mut items } => {
                    let root = self.project.as_ref().map(|p| p.root.clone());
                    self.diags.retain(|d| {
                        let Some(df) = d.file.as_deref() else {
                            return true;
                        };
                        let p = Path::new(df);
                        p != file && root.as_ref().is_none_or(|r| r.join(p) != file)
                    });
                    self.diags.append(&mut items);
                }
                LspEvent::Hover {
                    request_id,
                    summary,
                } => {
                    if self.lsp_pending_hover == Some(request_id) {
                        self.lsp_pending_hover = None;
                        if let Some(body) = summary {
                            let title = self
                                .cursor_symbol()
                                .unwrap_or_else(|| "rust-analyzer".to_string());
                            let url =
                                crate::doc_hint::lookup_for(&title, en).map(|h| h.url.to_string());
                            self.doc_popup = Some(DocPopup { title, body, url });
                        }
                    }
                }
                LspEvent::Definition {
                    request_id,
                    location,
                } => {
                    if self.lsp_pending_def == Some(request_id) {
                        self.lsp_pending_def = None;
                        if let Some((path, line, col)) = location {
                            self.jump_to_location(&path, line, col, app);
                        } else {
                            app.set_status(
                                crate::i18n::bi(
                                    "definição não encontrada",
                                    "definition not found",
                                    en,
                                ),
                                StatusKind::Info,
                            );
                        }
                    }
                }
                LspEvent::Completion { request_id, labels } => {
                    if self.lsp_pending_comp == Some(request_id) {
                        self.lsp_pending_comp = None;
                        if !labels.is_empty() && self.focus == Focus::Editor {
                            let mut extras = self.workspace_index.symbol_names();
                            extras.extend(labels);
                            self.completion
                                .refresh_with_extras(&self.editor, true, &extras);
                        }
                    }
                }
                LspEvent::InlayHints { request_id, hints } => {
                    if self.lsp_pending_inlay == Some(request_id) {
                        self.lsp_pending_inlay = None;
                        if !hints.is_empty() {
                            self.inlay_hints = hints;
                        }
                    }
                }
                LspEvent::Closed { reason } => {
                    self.lsp_status = reason;
                    self.lsp = None;
                    self.lsp_pending_hover = None;
                    self.lsp_pending_def = None;
                    self.lsp_pending_comp = None;
                    self.lsp_pending_inlay = None;
                }
            }
        }
        true
    }

    fn notify_lsp_change(&mut self) {
        self.refresh_inlay_hints();
        if let (Some(lsp), Some(path)) = (self.lsp.as_mut(), self.editor.path.as_ref())
            && lsp.is_ready()
            && path.extension().is_some_and(|e| e == "rs")
        {
            self.editor_doc_version += 1;
            let text = self.editor.buf.to_string_lossy();
            lsp.did_change(path, self.editor_doc_version, &text);
            self.lsp_pending_inlay = lsp.request_inlay_hints(path, self.editor.buf.line_count());
        }
    }

    // ------------------------------------------------------------------
    // Execução
    // ------------------------------------------------------------------

    /// Ponto de entrada de qualquer comando: aplica o gate de
    /// consentimento e depois delega.
    pub fn request_exec(&mut self, cmd: ToolCommand, app: &mut App) {
        if app.config.execution.consent != ExecConsent::Granted {
            self.consent_request = Some(cmd);
            return;
        }
        self.submit_exec(cmd, app);
    }

    fn submit_exec(&mut self, cmd: ToolCommand, app: &mut App) {
        let en = app.is_english();
        if app.exec.running {
            app.set_status(
                crate::i18n::bi(
                    "já há um comando Cargo rodando; aguarde ou use Ctrl+K",
                    "another Cargo command is already running; wait or press Ctrl+K",
                    en,
                ),
                StatusKind::Warning,
            );
            return;
        }
        self.stderr_buf.clear();
        self.diags.clear();
        self.diagnostic_index = 0;
        self.last_exit_success = None;
        self.last_failure = None;
        if let ToolCommand::NewProject { ref name } = cmd {
            if !self.save_ok(app) {
                return;
            }
            let dir = self
                .pending_new_project_dir
                .take()
                .unwrap_or_else(|| app.projects_base_dir());
            let project_path = dir.join(name);
            if let Err(e) = std::fs::create_dir_all(&dir) {
                app.set_status(
                    if en {
                        format!("could not create {}: {e}", dir.display())
                    } else {
                        format!("não consegui criar {}: {e}", dir.display())
                    },
                    StatusKind::Warning,
                );
                return;
            }
            match app.exec.submit(cmd, dir) {
                Ok(()) => {
                    self.after_exec = AfterExec::OpenNewProject(project_path);
                    self.console.system(crate::i18n::bi(
                        "▶ cargo new (criando projeto…)",
                        "▶ cargo new (creating project…)",
                        en,
                    ));
                }
                Err(e) => {
                    self.after_exec = AfterExec::None;
                    app.set_status(e, StatusKind::Warning);
                }
            }
            return;
        }
        let Some(project) = self.project.clone() else {
            app.set_status(
                crate::i18n::bi("nenhum projeto aberto", "no project open", en),
                StatusKind::Warning,
            );
            return;
        };
        // salvar antes de executar: rodar código velho confunde qualquer um
        if !self.save_ok(app) {
            app.set_status(
                crate::i18n::bi(
                    "execução cancelada: não consegui salvar o arquivo",
                    "execution canceled: could not save the file",
                    en,
                ),
                StatusKind::Warning,
            );
            return;
        }
        self.stderr_buf.clear();
        self.last_exit_success = None;
        self.last_failure = None;
        let label = cmd.label().to_string();
        let interactive = cmd == ToolCommand::RunInteractive;
        self.after_exec_reload = matches!(
            cmd,
            ToolCommand::Fmt
                | ToolCommand::AddDependency { .. }
                | ToolCommand::RemoveDependency { .. }
        );
        match app.exec.submit(cmd, project.root.clone()) {
            Ok(()) => {
                let timeout = secs_of(&label);
                let stdin_mode = if interactive {
                    crate::i18n::bi(
                        "entrada por linhas · Enter envia · Ctrl+D EOF",
                        "line input · Enter sends · Ctrl+D EOF",
                        en,
                    )
                } else {
                    crate::i18n::bi("stdin fechado", "stdin closed", en)
                };
                self.console.system(if en {
                    format!(
                        "▶ {label} — timeout {}s · Ctrl+K cancels · {stdin_mode}",
                        if interactive { 300 } else { timeout }
                    )
                } else {
                    format!(
                        "▶ {label} — timeout {}s · Ctrl+K cancela · {stdin_mode}",
                        if interactive { 300 } else { timeout }
                    )
                });
                self.console_input.clear();
                self.focus = Focus::Console;
            }
            Err(e) => app.set_status(e, StatusKind::Warning),
        }
    }

    /// Consome eventos do executor (chamado pelo loop principal).
    pub fn handle_exec_event(&mut self, ev: ExecEvent, app: &mut App) {
        let en = app.is_english();
        match ev {
            ExecEvent::Started { label } => {
                self.console.system(format!(
                    "… {label} {}",
                    crate::i18n::bi("iniciado", "started", en)
                ));
            }
            ExecEvent::Chunk { kind, text } => {
                if kind == StreamKind::Stderr {
                    // captura para diagnóstico (com teto próprio, além do
                    // limite do executor)
                    if self.stderr_buf.len() < 128 * 1024 {
                        self.stderr_buf.push_str(&text);
                        self.stderr_buf.push('\n');
                    }
                }
                self.console.push(kind, text);
            }
            ExecEvent::Finished {
                label,
                code,
                success,
            } => {
                self.last_exit_success = Some(success);
                let after = std::mem::replace(&mut self.after_exec, AfterExec::None);
                if success {
                    self.console.system(if en {
                        format!("✔ {label} finished (exit code {})", code.unwrap_or(0))
                    } else {
                        format!("✔ {label} terminou (código {})", code.unwrap_or(0))
                    });
                    if matches!(after, AfterExec::None | AfterExec::LaunchExternalWindow) {
                        self.record_project_milestones(&label, app);
                    }
                    if let Some(project) = &self.project {
                        let bin_name = package_binary_name(project);
                        if label == "cargo build (Linux)" {
                            let path = project.root.join("target/release").join(&bin_name);
                            self.console.system(format!(
                                "{}: {}",
                                crate::i18n::bi(
                                    "✔ Binário Linux pronto",
                                    "✔ Linux binary ready",
                                    en
                                ),
                                path.display()
                            ));
                        } else if label == "cargo build (Windows)" {
                            let path = project
                                .root
                                .join("target/x86_64-pc-windows-gnu/release")
                                .join(format!("{bin_name}.exe"));
                            self.console.system(format!(
                                "{}: {}",
                                crate::i18n::bi(
                                    "✔ Executável Windows pronto",
                                    "✔ Windows executable ready",
                                    en
                                ),
                                path.display()
                            ));
                        } else if after == AfterExec::LaunchExternalWindow {
                            let path = project.root.join("target/debug").join(&bin_name);
                            match spawn_external_app(&path, &project.root, en) {
                                Ok(msg) => {
                                    self.console.system(msg.clone());
                                    app.set_status(msg, StatusKind::Success);
                                }
                                Err(err) => {
                                    self.console.system(format!(
                                        "{}: {err}",
                                        crate::i18n::bi(
                                            "Aviso janela externa",
                                            "External window notice",
                                            en
                                        )
                                    ));
                                    app.set_status(err, StatusKind::Warning);
                                }
                            }
                        }
                    }
                    if after != AfterExec::LaunchExternalWindow {
                        app.set_status(
                            format!("{label}: {}", crate::i18n::bi("sucesso", "succeeded", en)),
                            StatusKind::Success,
                        );
                    }
                } else {
                    let exit = code.map(|c| c.to_string()).unwrap_or_else(|| "?".into());
                    let generic_detail = self
                        .stderr_buf
                        .lines()
                        .map(str::trim)
                        .rfind(|line| !line.is_empty() && !line.starts_with("note:"))
                        .map(|line| {
                            if en {
                                format!("{label} failed (code {exit}): {line}")
                            } else {
                                format!("{label} falhou (código {exit}): {line}")
                            }
                        })
                        .unwrap_or_else(|| {
                            if en {
                                format!("{label} failed (code {exit})")
                            } else {
                                format!("{label} falhou (código {exit})")
                            }
                        });
                    self.console.system(if en {
                        format!("✘ {label} exited with error (code {exit})")
                    } else {
                        format!("✘ {label} terminou com erro (código {exit})")
                    });
                    if label == "cargo build (Windows)" {
                        self.console.system(crate::i18n::bi(
                            "Dica Windows: instale `rustup target add x86_64-pc-windows-gnu` e `mingw-w64` no Linux para gerar .exe.",
                            "Windows hint: install `rustup target add x86_64-pc-windows-gnu` and `mingw-w64` on Linux to build .exe.",
                            en,
                        ));
                    } else if [
                        "alsa",
                        "asound",
                        "libudev",
                        "wayland",
                        "xkbcommon",
                        "pkg-config",
                    ]
                    .iter()
                    .any(|needle| self.stderr_buf.contains(needle))
                    {
                        self.console.system(crate::i18n::bi(
                            "Dica Bevy/GUI Linux (dependências do sistema): Ubuntu/Debian: `sudo apt install pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-0 libwayland-dev` | openSUSE: `sudo zypper install pkg-config libudev-devel alsa-devel libX11-devel libXi-devel libXcursor-devel libxkbcommon-devel wayland-devel` | Fedora: `sudo dnf install pkgconf-pkg-config systemd-devel alsa-lib-devel libX11-devel libxkbcommon-devel wayland-devel` | Arch: `sudo pacman -S pkgconf systemd-libs alsa-lib libx11 libxkbcommon wayland`.",
                            "Bevy/GUI Linux hint (system dependencies): Ubuntu/Debian: `sudo apt install pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-0 libwayland-dev` | openSUSE: `sudo zypper install pkg-config libudev-devel alsa-devel libX11-devel libXi-devel libXcursor-devel libxkbcommon-devel wayland-devel` | Fedora: `sudo dnf install pkgconf-pkg-config systemd-devel alsa-lib-devel libX11-devel libxkbcommon-devel wayland-devel` | Arch: `sudo pacman -S pkgconf systemd-libs alsa-lib libx11 libxkbcommon wayland`.",
                            en,
                        ));
                    }
                    self.last_failure = Some(generic_detail.clone());
                    app.progress.record_compiler_errors(&self.stderr_buf);
                    let _ = app.persist_progress();
                    app.set_status(generic_detail, StatusKind::Warning);
                }
                // diagnósticos só fazem sentido para check/build/test
                if matches!(after, AfterExec::None | AfterExec::LaunchExternalWindow)
                    && !self.stderr_buf.is_empty()
                {
                    self.diags = parse_diagnostics(&self.stderr_buf);
                    if !self.diags.is_empty() {
                        let (e, w) = count_diags(&self.diags);
                        self.console.system(if en {
                            format!("diagnostics: {e} error(s), {w} warning(s)")
                        } else {
                            format!("diagnóstico: {e} erro(s), {w} aviso(s)")
                        });
                    }
                }
                if let AfterExec::OpenNewProject(path) = after {
                    if success {
                        self.open_project(&path, app);
                    } else {
                        let exit = code.map(|c| c.to_string()).unwrap_or_else(|| "?".into());
                        let detail = self
                            .stderr_buf
                            .lines()
                            .map(str::trim)
                            .rfind(|line| !line.is_empty() && !line.starts_with("note:"))
                            .map(|line| {
                                if en {
                                    format!("cargo new failed (code {exit}): {line}")
                                } else {
                                    format!("cargo new falhou (código {exit}): {line}")
                                }
                            })
                            .unwrap_or_else(|| {
                                if en {
                                    format!(
                                        "cargo new failed (code {exit}); Cargo produced no error message"
                                    )
                                } else {
                                    format!(
                                        "cargo new falhou (código {exit}); o Cargo não produziu uma mensagem de erro"
                                    )
                                }
                            });
                        self.last_failure = Some(detail.clone());
                        app.set_status(detail, StatusKind::Warning);
                    }
                }
                if self.after_exec_reload {
                    self.after_exec_reload = false;
                    if success
                        && !self.editor.dirty
                        && let (Some(project), Some(path)) =
                            (&self.project, self.editor.path.clone())
                        && let Ok(text) = project.read(&path)
                    {
                        self.editor = Editor::new(&text, Some(path));
                    }
                }
            }
            ExecEvent::TimedOut { label } => {
                self.console.system(if en {
                    format!("⏱ {label} timed out and was terminated (infinite loop?)")
                } else {
                    format!("⏱ {label} estourou o tempo limite e foi finalizado (loop infinito?)")
                });
                let reason = if en {
                    format!("{label} timed out")
                } else {
                    format!("{label} atingiu o tempo limite")
                };
                self.last_failure = Some(reason.clone());
                app.set_status(reason, StatusKind::Warning);
                self.last_exit_success = Some(false);
                self.after_exec = AfterExec::None;
                self.after_exec_reload = false;
            }
            ExecEvent::Cancelled { label } => {
                self.console.system(format!(
                    "■ {label} {}",
                    crate::i18n::bi("cancelado", "canceled", en)
                ));
                app.set_status(
                    crate::i18n::bi("comando cancelado", "command canceled", en),
                    StatusKind::Info,
                );
                self.after_exec = AfterExec::None;
                self.after_exec_reload = false;
            }
            ExecEvent::SpawnFailed { label, reason } => {
                let msg = if en {
                    format!("{label} failed to start: {reason}")
                } else {
                    format!("{label} não pôde iniciar: {reason}")
                };
                self.console.system(format!("✘ {msg}"));
                self.last_failure = Some(msg.clone());
                app.set_status(msg, StatusKind::Warning);
                self.last_exit_success = Some(false);
                self.after_exec = AfterExec::None;
                self.after_exec_reload = false;
            }
        }
    }

    pub fn cancel_exec(&mut self, app: &mut App) {
        if app.exec.running {
            app.exec.cancel();
        }
    }

    // ------------------------------------------------------------------
    // Teclado
    // ------------------------------------------------------------------

    pub fn on_key(&mut self, app: &mut App, key: KeyEvent) {
        self.refresh_language_for(app.is_english());
        // overlays primeiro: consentimento > prompt > resto
        if self.consent_request.is_some() {
            self.on_consent_key(app, key);
            return;
        }
        if self.prompt.is_some() {
            self.on_prompt_key(app, key);
            return;
        }
        match self.view {
            CreateView::Start => self.on_start_key(app, key),
            CreateView::Workspace => self.on_workspace_key(app, key),
        }
    }

    fn on_consent_key(&mut self, app: &mut App, key: KeyEvent) {
        let en = app.is_english();
        match key.code {
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('s') => {
                app.config.execution.consent = ExecConsent::Granted;
                if let Err(err) = app.save_config() {
                    app.set_status(
                        if en {
                            format!("could not save consent: {err}")
                        } else {
                            format!("não consegui salvar consentimento: {err}")
                        },
                        StatusKind::Warning,
                    );
                }
                let cmd = self.consent_request.take();
                self.consent_request = None;
                if let Some(cmd) = cmd {
                    self.submit_exec(cmd, app);
                }
            }
            KeyCode::Esc | KeyCode::Char('n') => {
                self.consent_request = None;
                self.pending_new_project_dir = None;
                self.after_exec_reload = false;
                app.set_status(
                    crate::i18n::bi(
                        "execução não autorizada — nada foi rodado",
                        "execution not authorized — nothing was run",
                        en,
                    ),
                    StatusKind::Info,
                );
            }
            _ => {}
        }
    }

    fn on_prompt_key(&mut self, app: &mut App, key: KeyEvent) {
        let en = app.is_english();
        let Some(prompt) = self.prompt.as_mut() else {
            return;
        };
        match key.code {
            KeyCode::Esc => {
                self.prompt = None;
            }
            KeyCode::Tab if prompt.kind == PromptKind::NewProject && prompt.text.is_empty() => {
                prompt.text = format!("{}/", app.projects_base_dir().display());
                prompt.error = None;
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                if prompt.text.chars().count() < 200 {
                    prompt.text.push(c);
                }
                prompt.error = None;
            }
            KeyCode::Backspace => {
                prompt.text.pop();
                prompt.error = None;
            }
            KeyCode::Enter => {
                let kind = prompt.kind;
                let text = prompt.text.clone();
                match kind {
                    PromptKind::NewProject => {
                        match resolve_new_project_target(&text, &app.projects_base_dir()) {
                            Ok((parent_dir, name)) => {
                                let target = parent_dir.join(&name);
                                if target.exists() {
                                    if let Some(p) = self.prompt.as_mut() {
                                        p.error = Some(if en {
                                            format!(
                                                "{} already exists — choose another name",
                                                target.display()
                                            )
                                        } else {
                                            format!(
                                                "{} já existe — escolha outro nome",
                                                target.display()
                                            )
                                        });
                                    }
                                    return;
                                }
                                self.pending_new_project_dir = Some(parent_dir);
                                self.prompt = None;
                                self.request_exec(ToolCommand::NewProject { name }, app);
                            }
                            Err(msg) => {
                                if let Some(p) = self.prompt.as_mut() {
                                    p.error = Some(msg);
                                }
                            }
                        }
                    }
                    PromptKind::SetProjectsDir => {
                        let trimmed = text.trim();
                        if trimmed.is_empty() {
                            app.config.workspace.projects_dir = None;
                            let _ = app.save_config();
                            self.prompt = None;
                            app.set_status(
                                format!(
                                    "{}: {}",
                                    crate::i18n::bi(
                                        "Pasta padrão restaurada",
                                        "Default folder restored",
                                        en
                                    ),
                                    app.projects_base_dir().display()
                                ),
                                StatusKind::Info,
                            );
                            return;
                        }
                        let dir = expand_home(Path::new(trimmed));
                        if dir
                            .components()
                            .any(|c| matches!(c, std::path::Component::ParentDir))
                        {
                            if let Some(p) = self.prompt.as_mut() {
                                p.error = Some(
                                    crate::i18n::bi(
                                        "caminho inválido: não use '..'",
                                        "invalid path: do not use '..'",
                                        en,
                                    )
                                    .into(),
                                );
                            }
                            return;
                        }
                        if let Err(err) = std::fs::create_dir_all(&dir) {
                            if let Some(p) = self.prompt.as_mut() {
                                p.error = Some(if en {
                                    format!("could not create/open folder: {err}")
                                } else {
                                    format!("não consegui criar/abrir a pasta: {err}")
                                });
                            }
                            return;
                        }
                        app.config.workspace.projects_dir = Some(dir.to_string_lossy().to_string());
                        if let Err(err) = app.save_config() {
                            app.set_status(
                                if en {
                                    format!("folder set, but config not saved: {err}")
                                } else {
                                    format!("pasta definida, mas config não salva: {err}")
                                },
                                StatusKind::Warning,
                            );
                        } else {
                            app.set_status(
                                format!(
                                    "{}: {}",
                                    crate::i18n::bi(
                                        "Pasta para novos projetos",
                                        "Folder for new projects",
                                        en
                                    ),
                                    dir.display()
                                ),
                                StatusKind::Success,
                            );
                        }
                        self.prompt = None;
                    }
                    PromptKind::OpenPath => {
                        let path = expand_home(Path::new(text.trim()));
                        if Project::detect(&path).is_some() {
                            self.prompt = None;
                            let p = path.clone();
                            self.open_project(&p, app);
                        } else if let Some(p) = self.prompt.as_mut() {
                            p.error = Some(if en {
                                format!("Cargo.toml not found in {}", path.display())
                            } else {
                                format!("não achei Cargo.toml em {}", path.display())
                            });
                        }
                    }
                    PromptKind::NewFile => {
                        if !self.save_ok(app) {
                            return;
                        }
                        let result = self
                            .project
                            .as_ref()
                            .ok_or_else(|| {
                                std::io::Error::other(crate::i18n::bi(
                                    "nenhum projeto aberto",
                                    "no project open",
                                    en,
                                ))
                            })
                            .and_then(|p| p.create_file(text.trim()));
                        match result {
                            Ok(path) => {
                                self.prompt = None;
                                self.refresh_files(app);
                                if let Some(idx) = self.files.iter().position(|p| p == &path) {
                                    self.switch_file(idx, app);
                                    self.focus = Focus::Editor;
                                }
                                app.set_status(
                                    crate::i18n::bi(
                                        "arquivo criado; declare mod no pai quando necessário",
                                        "file created; declare mod in parent if needed",
                                        en,
                                    ),
                                    StatusKind::Success,
                                );
                            }
                            Err(err) => {
                                if let Some(prompt) = &mut self.prompt {
                                    prompt.error = Some(err.to_string());
                                }
                            }
                        }
                    }
                    PromptKind::AddDependency | PromptKind::RemoveDependency => {
                        let name = text.trim().to_string();
                        if !crate::execution::valid_dependency(&name)
                            || kind == PromptKind::RemoveDependency && name.contains('@')
                        {
                            if let Some(prompt) = &mut self.prompt {
                                prompt.error = Some(
                                    crate::i18n::bi(
                                        "use nome ou nome@versão; remoção aceita só nome",
                                        "use name or name@version; removal takes name only",
                                        en,
                                    )
                                    .into(),
                                );
                            }
                            return;
                        }
                        self.prompt = None;
                        let cmd = if kind == PromptKind::AddDependency {
                            ToolCommand::AddDependency { name }
                        } else {
                            ToolCommand::RemoveDependency { name }
                        };
                        self.request_exec(cmd, app);
                    }
                    PromptKind::ProjectSearch => {
                        let query = text.trim().to_string();
                        if query.is_empty() {
                            if let Some(prompt) = &mut self.prompt {
                                prompt.error = Some(
                                    crate::i18n::bi(
                                        "digite um símbolo ou trecho para buscar",
                                        "enter a symbol or snippet to search",
                                        en,
                                    )
                                    .into(),
                                );
                            }
                            return;
                        }
                        self.prompt = None;
                        self.run_project_search(&query, app);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn run_project_search(&mut self, query: &str, app: &mut App) {
        let Some(project) = &self.project else {
            return;
        };
        let hits = search_project_text(project, &self.files, query, 100);
        self.project_matches = hits;
        self.project_match_idx = 0;
        if self.project_matches.is_empty() {
            app.set_status(
                if app.is_english() {
                    format!("no matches for \"{query}\" in project")
                } else {
                    format!("nenhuma ocorrência de \"{query}\" no projeto")
                },
                StatusKind::Info,
            );
        } else {
            self.jump_next_project_match(app);
        }
    }

    fn on_start_key(&mut self, app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.screen = Screen::Menu;
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.open_projects_dir_prompt(app);
            }
            KeyCode::Up | KeyCode::Char('k') => self.start_sel = self.start_sel.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => {
                self.start_sel =
                    (self.start_sel + 1).min(2 + app.config.workspace.recent_projects.len().min(8))
            }
            KeyCode::Enter => {
                // 0 = novo; 1 = abrir; 2 = reabrir último (se existir)
                match self.start_sel {
                    0 => {
                        self.prompt = Some(PromptState {
                            kind: PromptKind::NewProject,
                            text: String::new(),
                            error: None,
                        });
                    }
                    1 => {
                        self.prompt = Some(PromptState {
                            kind: PromptKind::OpenPath,
                            text: String::new(),
                            error: None,
                        });
                    }
                    2 => {
                        if let Some(path) = app.config.workspace.last_project.clone() {
                            self.open_project(Path::new(&path), app);
                        } else {
                            app.set_status(
                                crate::i18n::bi(
                                    "nenhum projeto anterior salvo",
                                    "no previous project saved",
                                    app.is_english(),
                                ),
                                StatusKind::Info,
                            );
                        }
                    }
                    index => {
                        if let Some(path) =
                            app.config.workspace.recent_projects.get(index - 3).cloned()
                        {
                            self.open_project(Path::new(&path), app);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn on_workspace_key(&mut self, app: &mut App, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        self.hovered_doc = None;
        if self.agent.model_picker.handle_key(app, key) {
            return;
        }
        if self.agent.consent_pending.is_some() {
            match key.code {
                KeyCode::Enter | KeyCode::Char('s') | KeyCode::Char('y') => {
                    app.config.ai.remote_consent = RemoteAiConsent::Granted;
                    let _ = app.save_config();
                    if let Some(q) = self.agent.consent_pending.take() {
                        self.dispatch_agent_query(app, q);
                    }
                }
                KeyCode::Esc | KeyCode::Char('n') => {
                    app.config.ai.remote_consent = RemoteAiConsent::Denied;
                    let _ = app.save_config();
                    self.agent.consent_pending = None;
                    app.set_status(
                        crate::i18n::bi(
                            "envio remoto recusado para o agente; usando guia local",
                            "remote sending declined for agent; using local guide",
                            app.is_english(),
                        ),
                        StatusKind::Info,
                    );
                }
                _ => {}
            }
            return;
        }
        if self.lab.open {
            self.on_lab_modal_key(app, key);
            return;
        }
        if self.lib_menu_open {
            self.on_lib_menu_key(app, key);
            return;
        }
        if self.build_menu_open {
            self.on_build_menu_key(app, key);
            return;
        }
        if self.doc_popup.is_some() {
            let dismiss_only = matches!(key.code, KeyCode::Esc | KeyCode::F(1));
            self.doc_popup = None;
            if dismiss_only {
                return;
            }
        }
        if self.focus == Focus::Editor && self.editor.search.is_none() {
            if ctrl && matches!(key.code, KeyCode::Char(' ') | KeyCode::Char('\0')) {
                let extras = self.workspace_index.symbol_names();
                self.completion
                    .refresh_with_extras(&self.editor, true, &extras);
                if let (Some(lsp), Some(path)) = (self.lsp.as_mut(), self.editor.path.as_ref()) {
                    self.lsp_pending_comp = lsp.request_completion(
                        path,
                        self.editor.cursor.line,
                        self.editor.cursor.col,
                    );
                }
                return;
            }
            if !self.completion.items.is_empty() {
                match key.code {
                    KeyCode::Down => {
                        self.completion.selected =
                            (self.completion.selected + 1) % self.completion.items.len();
                        return;
                    }
                    KeyCode::Up => {
                        self.completion.selected =
                            (self.completion.selected + self.completion.items.len() - 1)
                                % self.completion.items.len();
                        return;
                    }
                    KeyCode::Enter | KeyCode::Tab => {
                        self.completion.accept(&mut self.editor);
                        self.notify_lsp_change();
                        return;
                    }
                    KeyCode::Esc => {
                        self.completion = Default::default();
                        return;
                    }
                    _ => self.completion = Default::default(),
                }
            }
        }

        // busca ativa captura o teclado do editor
        if let Some(search) = self.editor.search.as_ref()
            && search.editing
            && self.focus == Focus::Editor
        {
            match key.code {
                KeyCode::Esc => self.editor.search_cancel(),
                KeyCode::Enter => self.editor.search_commit(),
                KeyCode::Backspace => self.editor.search_backspace(),
                KeyCode::Char(c) if !ctrl && !alt => self.editor.search_input(c),
                _ => {}
            }
            return;
        }

        // atalhos globais do workspace
        match key.code {
            KeyCode::BackTab => {
                self.cycle_focus(false);
                return;
            }
            KeyCode::Char('a') if alt => {
                self.toggle_agent_panel(app);
                return;
            }
            KeyCode::Char('x') if alt => {
                self.open_lab_tab(crate::lab_tools::LabToolTab::RaioXMemoria, app);
                return;
            }
            KeyCode::Char('e') if alt => {
                self.diagnose_error_with_agent(app);
                return;
            }
            KeyCode::Char('b') if alt => {
                self.debugger_toggle_breakpoint_at_cursor(app);
                return;
            }
            KeyCode::Char('j') if alt => {
                self.open_lab_tab(crate::lab_tools::LabToolTab::DebuggerVisual, app);
                return;
            }
            KeyCode::Char('p') if alt => {
                self.open_lab_tab(crate::lab_tools::LabToolTab::PlaygroundComunidade, app);
                return;
            }
            KeyCode::Char('l') if alt => {
                self.lab.lifetime_gutter_enabled = true;
                self.open_lab_tab(crate::lab_tools::LabToolTab::MapaLifetimes, app);
                return;
            }
            KeyCode::Char('y') if alt => {
                self.open_lab_tab(crate::lab_tools::LabToolTab::LayoutMemoria, app);
                return;
            }
            KeyCode::Char('u') if alt => {
                self.open_lab_tab(crate::lab_tools::LabToolTab::DesacucararMacros, app);
                return;
            }
            KeyCode::Char('i') if alt => {
                self.lab.polyglot_mode_flashcards = true;
                self.open_lab_tab(crate::lab_tools::LabToolTab::PoliglotaIntuicao, app);
                return;
            }
            KeyCode::Char('y') if ctrl && self.focus != Focus::Editor => {
                self.toggle_agent_panel(app);
                return;
            }
            KeyCode::Char('o') if alt => {
                self.agent.open = true;
                self.focus = Focus::Agent;
                self.agent.model_picker.open_picker(app);
                return;
            }
            KeyCode::Char('d') if alt => {
                self.open_projects_dir_prompt(app);
                return;
            }
            KeyCode::Char('p') if ctrl => {
                self.close_project(app);
                return;
            }
            KeyCode::Char('o') if ctrl => {
                if self.focus == Focus::Agent {
                    self.agent.model_picker.open_picker(app);
                } else {
                    self.begin_prompt(PromptKind::OpenPath, app);
                }
                return;
            }
            KeyCode::Char('n') if ctrl => {
                self.begin_prompt(PromptKind::NewProject, app);
                return;
            }
            KeyCode::Char('g') if ctrl => {
                self.begin_prompt(PromptKind::ProjectSearch, app);
                return;
            }
            KeyCode::Char('g') if alt => {
                self.jump_next_project_match(app);
                return;
            }
            KeyCode::Char('c') if alt => {
                self.request_exec(ToolCommand::Clippy, app);
                return;
            }
            KeyCode::Char('m') if alt => {
                self.create_project_plan(app);
                return;
            }
            KeyCode::Char('l') if ctrl => {
                self.request_exec(ToolCommand::Clippy, app);
                return;
            }
            KeyCode::Char('r') if ctrl => {
                self.refresh_files(app);
                return;
            }
            KeyCode::F(1) => {
                self.show_doc_at_cursor(app);
                return;
            }
            KeyCode::F(2) => {
                self.begin_prompt(PromptKind::NewFile, app);
                return;
            }
            KeyCode::F(3) if self.editor.search.is_none() && !self.project_matches.is_empty() => {
                self.jump_next_project_match(app);
                return;
            }
            KeyCode::F(4) => {
                self.build_menu_open = !self.build_menu_open;
                self.lib_menu_open = false;
                return;
            }
            KeyCode::F(9) => {
                self.lib_menu_open = !self.lib_menu_open;
                self.build_menu_open = false;
                return;
            }
            KeyCode::F(10) => {
                if self.lab.debugger.active || !self.lab.debugger.breakpoints.is_empty() {
                    self.debugger_step_over(app);
                } else {
                    self.begin_prompt(PromptKind::RemoveDependency, app);
                }
                return;
            }
            KeyCode::F(11) => {
                self.go_to_definition(app);
                return;
            }
            KeyCode::F(12) => {
                self.jump_diagnostic(app);
                return;
            }
            KeyCode::Esc => {
                self.close_project(app);
                return;
            }
            KeyCode::Char('k') if ctrl && self.agent.manager.running => {
                self.agent.manager.cancel();
                app.set_status(
                    crate::i18n::bi(
                        "consulta ao agente cancelada",
                        "agent query canceled",
                        app.is_english(),
                    ),
                    StatusKind::Info,
                );
                return;
            }
            KeyCode::Char('k') if ctrl && app.exec.running => {
                self.cancel_exec(app);
                return;
            }
            KeyCode::Tab if self.focus != Focus::Editor => {
                self.cycle_focus(true);
                return;
            }
            KeyCode::Char('s') if ctrl => {
                self.save_current(app);
                if !self.editor.dirty {
                    app.set_status(
                        crate::i18n::bi("arquivo salvo", "file saved", app.is_english()),
                        StatusKind::Success,
                    );
                }
                return;
            }
            KeyCode::Tab if self.focus == Focus::Editor => {
                self.editor.insert_text("    ");
                self.notify_lsp_change();
                return;
            }
            KeyCode::Tab => {
                self.cycle_focus(true);
                return;
            }
            KeyCode::Char('k') if ctrl => {
                self.cancel_exec(app);
                return;
            }
            KeyCode::F(5) => {
                self.request_exec(ToolCommand::RunInteractive, app);
                return;
            }
            KeyCode::F(6) => {
                self.request_exec(ToolCommand::Test, app);
                return;
            }
            KeyCode::F(7) => {
                self.request_exec(ToolCommand::Check, app);
                return;
            }
            KeyCode::F(8) => {
                self.request_exec(ToolCommand::Fmt, app);
                return;
            }
            _ => {}
        }

        match self.focus {
            Focus::Files => self.on_files_key(app, key),
            Focus::Editor => {
                self.on_editor_key(app, key, ctrl, alt, shift);
                if matches!(
                    key.code,
                    KeyCode::Char(_) | KeyCode::Enter | KeyCode::Backspace | KeyCode::Delete
                ) {
                    self.notify_lsp_change();
                }
            }
            Focus::Console => self.on_console_key(app, key),
            Focus::Agent => self.on_agent_key(app, key),
        }
        if self.focus == Focus::Editor
            && self.editor.search.is_none()
            && !ctrl
            && !alt
            && matches!(key.code, KeyCode::Char(c) if c.is_alphanumeric() || c == '_')
        {
            let extras = self.workspace_index.symbol_names();
            self.completion
                .refresh_with_extras(&self.editor, false, &extras);
        }
    }

    fn on_files_key(&mut self, app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_file = self.selected_file.saturating_sub(1)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected_file =
                    (self.selected_file + 1).min(self.files.len().saturating_sub(1))
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                let idx = self.selected_file;
                self.switch_file(idx, app);
                self.focus = Focus::Editor;
            }
            _ => {}
        }
    }

    fn on_editor_key(&mut self, _app: &mut App, key: KeyEvent, ctrl: bool, alt: bool, shift: bool) {
        let ed = &mut self.editor;
        let mut modified = false;
        match key.code {
            KeyCode::Char(c) if !ctrl && !alt => {
                if c == ' ' && shift {
                    ed.insert_char(' ');
                } else {
                    ed.insert_char(c);
                }
                modified = true;
            }
            KeyCode::Enter => {
                ed.newline();
                modified = true;
            }
            KeyCode::Backspace => {
                ed.backspace();
                modified = true;
            }
            KeyCode::Delete => {
                ed.delete();
                modified = true;
            }
            KeyCode::Tab => {
                ed.insert_text("    ");
                modified = true;
            }
            KeyCode::Left => ed.move_cursor(-1, 0, shift, ctrl),
            KeyCode::Right => ed.move_cursor(1, 0, shift, ctrl),
            KeyCode::Up => ed.move_cursor(0, -1, shift, false),
            KeyCode::Down => ed.move_cursor(0, 1, shift, false),
            KeyCode::Home if ctrl => ed.move_doc(false, shift),
            KeyCode::End if ctrl => ed.move_doc(true, shift),
            KeyCode::Home => ed.move_home(shift),
            KeyCode::End => ed.move_end(shift),
            KeyCode::PageUp => ed.move_cursor(0, -10, shift, false),
            KeyCode::PageDown => ed.move_cursor(0, 10, shift, false),
            KeyCode::Char('a') if ctrl => ed.select_all(),
            KeyCode::Char('c') if ctrl => {
                ed.copy();
            }
            KeyCode::Char('x') if ctrl => {
                ed.cut();
                modified = true;
            }
            KeyCode::Char('v') if ctrl => {
                ed.paste();
                modified = true;
            }
            KeyCode::Char('z') if ctrl && shift => ed.redo(),
            KeyCode::Char('y') if ctrl => ed.redo(),
            KeyCode::Char('z') if ctrl => ed.undo(),
            KeyCode::Char('f') if ctrl => ed.begin_search(),
            KeyCode::F(3) => ed.search_next(),
            _ => {}
        }
        if modified {
            self.agent.student_modified_since_review = true;
        }
    }

    fn on_console_key(&mut self, app: &mut App, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        if alt && matches!(key.code, KeyCode::Char('e')) {
            self.diagnose_error_with_agent(app);
            return;
        }
        if app.exec.accepts_input() {
            match key.code {
                KeyCode::Char('d') if ctrl => {
                    match app.exec.close_input() {
                        Ok(()) => self.console.system(crate::i18n::bi(
                            "stdin: EOF enviado",
                            "stdin: EOF sent",
                            app.is_english(),
                        )),
                        Err(err) => app.set_status(err, StatusKind::Warning),
                    };
                    return;
                }
                KeyCode::Enter => {
                    match app.exec.send_line(&self.console_input) {
                        Ok(()) => {
                            self.console.system(format!("> {}", self.console_input));
                            self.console_input.clear();
                            self.console.scroll_back = 0;
                        }
                        Err(err) => app.set_status(err, StatusKind::Warning),
                    }
                    return;
                }
                KeyCode::Backspace => {
                    self.console_input.pop();
                    return;
                }
                KeyCode::Char(c) if !ctrl => {
                    if self.console_input.len() + c.len_utf8() <= 4096 {
                        self.console_input.push(c);
                    }
                    return;
                }
                _ => {}
            }
        }
        let max = self.console.max_scroll();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::PageUp => {
                self.console.scroll_back = (self.console.scroll_back + 5).min(max)
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::PageDown => {
                self.console.scroll_back = self.console.scroll_back.saturating_sub(5)
            }
            KeyCode::Home => self.console.scroll_back = max,
            KeyCode::End => self.console.scroll_back = 0,
            _ => {}
        }
    }

    /// Colar multilinha do terminal (bracketed paste).
    pub fn on_paste(&mut self, text: &str) {
        self.completion = Default::default();
        if self.consent_request.is_some() {
            return;
        }
        if let Some(prompt) = &mut self.prompt {
            for ch in text.chars().filter(|c| !c.is_control()) {
                if prompt.text.len() + ch.len_utf8() <= 4096 {
                    prompt.text.push(ch);
                }
            }
            return;
        }
        if self.view == CreateView::Workspace && self.focus == Focus::Console {
            for ch in text.chars().filter(|c| !c.is_control()) {
                if self.console_input.len() + ch.len_utf8() <= 4096 {
                    self.console_input.push(ch);
                }
            }
            return;
        }
        if self.view == CreateView::Workspace && self.focus == Focus::Agent {
            for ch in text.chars().filter(|c| !c.is_control()) {
                if self.agent.input.chars().count() < 400 {
                    self.agent.input.push(ch);
                }
            }
            return;
        }
        if self.view == CreateView::Workspace && self.focus == Focus::Editor {
            self.editor.insert_text(text);
            self.agent.student_modified_since_review = true;
            self.notify_lsp_change();
        }
    }

    pub fn file_label(&self, idx: usize) -> String {
        self.files
            .get(idx)
            .map(|p| {
                p.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "?".into())
            })
            .unwrap_or_default()
    }

    pub fn cursor_line_col(&self) -> (usize, usize) {
        (self.editor.cursor.line + 1, self.editor.cursor.col + 1)
    }

    fn symbol_at_screen(&self, x: u16, y: u16) -> Option<String> {
        let area = self.editor_text_area;
        if !rect_contains(area, x, y) {
            return None;
        }
        let line = self.editor.scroll_row + usize::from(y - area.y);
        let gutter = self.editor.buf.line_count().to_string().len().max(2) + 1;
        let column = self.editor.scroll_col + usize::from(x - area.x).saturating_sub(gutter);
        let chars = self.editor.buf.line(line)?;
        if column >= chars.len() || !(chars[column].is_ascii_alphanumeric() || chars[column] == '_')
        {
            return None;
        }
        let mut start = column;
        let mut end = column + 1;
        while start > 0 && (chars[start - 1].is_ascii_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }
        while end < chars.len() && (chars[end].is_ascii_alphanumeric() || chars[end] == '_') {
            end += 1;
        }
        Some(chars[start..end].iter().collect())
    }

    pub fn on_mouse_move(&mut self, x: u16, y: u16) {
        if self.view != CreateView::Workspace
            || self.lib_menu_open
            || self.build_menu_open
            || self.lab.open
            || self.agent.model_picker.open
            || self.prompt.is_some()
            || self.consent_request.is_some()
        {
            self.hovered_doc = None;
            return;
        }
        let Some(symbol) = self.symbol_at_screen(x, y) else {
            self.hovered_doc = None;
            return;
        };
        let code = self.editor.buf.to_string_lossy();
        if crate::doc_hint::resolve_symbol_doc(&symbol, &code).is_some()
            || self.workspace_index.find_doc(&symbol).is_some()
        {
            self.hovered_doc = Some((symbol, x, y));
        } else {
            self.hovered_doc = None;
        }
    }

    /// Trata clique esquerdo na barra superior (▼ Bibliotecas / Compilar),
    /// nos modais centrais ou em símbolos do editor (abre documentação PT/EN).
    pub fn on_mouse_down(&mut self, x: u16, y: u16, app: &mut App) {
        let en = app.is_english();
        self.refresh_language_for(en);
        if self.view != CreateView::Workspace
            || self.consent_request.is_some()
            || self.prompt.is_some()
            || self.agent.model_picker.open
        {
            return;
        }
        if self.lab.open {
            if !rect_contains(self.lab_modal_area, x, y) {
                self.lab.open = false;
            }
            return;
        }
        if self.lib_menu_open {
            if !rect_contains(self.lib_modal_area, x, y) {
                self.lib_menu_open = false;
                return;
            }
            let libs = crate::doc_hint::extra_libraries(en);
            let list_top = self.lib_modal_area.y.saturating_add(2);
            if y >= list_top && x < self.lib_modal_area.x.saturating_add(30) {
                let idx = usize::from(y - list_top);
                if idx < libs.len() {
                    self.lib_menu_sel = idx;
                    self.install_library_into_code(idx, app);
                }
                return;
            }
            let footer_y = self
                .lib_modal_area
                .y
                .saturating_add(self.lib_modal_area.height.saturating_sub(2));
            if y == footer_y
                && let Some(lib) = libs.get(self.lib_menu_sel)
            {
                let msg = open_documentation_url(lib.doc_url, en);
                app.set_status(msg, StatusKind::Info);
            }
            return;
        }
        if self.build_menu_open {
            if !rect_contains(self.build_modal_area, x, y) {
                self.build_menu_open = false;
                return;
            }
            let list_top = self.build_modal_area.y.saturating_add(2);
            if y >= list_top {
                let idx = usize::from(y - list_top);
                if idx < 5 {
                    self.build_menu_sel = idx;
                    self.execute_build_menu_option(idx, app);
                }
            }
            return;
        }
        if rect_contains(self.top_lib_btn_area, x, y) {
            self.lib_menu_open = true;
            self.build_menu_open = false;
            self.doc_popup = None;
            self.hovered_doc = None;
            return;
        }
        if rect_contains(self.top_build_btn_area, x, y) {
            self.build_menu_open = true;
            self.lib_menu_open = false;
            self.doc_popup = None;
            self.hovered_doc = None;
            return;
        }
        if rect_contains(self.editor_text_area, x, y) {
            self.focus = Focus::Editor;
            let Some(symbol) = self.symbol_at_screen(x, y) else {
                return;
            };
            let code = self.editor.buf.to_string_lossy();
            let resolved = crate::doc_hint::resolve_symbol_doc_for(&symbol, &code, en);
            let local_doc = self.workspace_index.find_doc(&symbol);
            let doc_url = resolved
                .as_ref()
                .map(|r| r.best_url_for(en).to_string())
                .or_else(|| {
                    local_doc.as_ref().map(|_| {
                        if en {
                            "https://doc.rust-lang.org/book/".to_string()
                        } else {
                            "https://rust-br.github.io/rust-book-pt-br/".to_string()
                        }
                    })
                });
            if let Some(url) = doc_url.clone() {
                if let Some(doc) = local_doc {
                    self.doc_popup = Some(DocPopup {
                        title: symbol,
                        body: doc,
                        url: Some(url.clone()),
                    });
                } else if let Some(r) = resolved {
                    self.doc_popup = Some(DocPopup {
                        title: r.title,
                        body: r.summary,
                        url: Some(url.clone()),
                    });
                }
                let msg = open_documentation_url(&url, en);
                app.set_status(msg, StatusKind::Info);
            }
        }
    }

    fn on_lib_menu_key(&mut self, app: &mut App, key: KeyEvent) {
        let en = app.is_english();
        let libs = crate::doc_hint::extra_libraries(en);
        let len = libs.len();
        match key.code {
            KeyCode::Esc | KeyCode::F(9) => self.lib_menu_open = false,
            KeyCode::Up | KeyCode::Char('k') => {
                self.lib_menu_sel = self.lib_menu_sel.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.lib_menu_sel + 1 < len {
                    self.lib_menu_sel += 1;
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if let Some(lib) = libs.get(self.lib_menu_sel) {
                    let msg = open_documentation_url(lib.doc_url, en);
                    app.set_status(msg, StatusKind::Info);
                }
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.install_library_template(self.lib_menu_sel, app);
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                self.lib_menu_open = false;
                self.open_lab_tab(crate::lab_tools::LabToolTab::EcossistemasModernos, app);
            }
            KeyCode::Enter => self.install_library_into_code(self.lib_menu_sel, app),
            _ => {}
        }
    }

    /// Adiciona a dependência escolhida ao `Cargo.toml` e insere `use ...;`
    /// no topo do arquivo `.rs` ativo (ou em `src/main.rs`).
    pub fn install_library_into_code(&mut self, idx: usize, app: &mut App) {
        let en = app.is_english();
        let Some(lib) = crate::doc_hint::extra_libraries(en).get(idx) else {
            return;
        };
        let Some(project) = self.project.as_ref().cloned() else {
            return;
        };
        let cargo_path = project.root.join("Cargo.toml");
        let viewing_cargo = self.editor.path.as_deref() == Some(cargo_path.as_path());
        let cargo_raw = if viewing_cargo {
            self.editor.buf.to_string_lossy()
        } else {
            match project.read(&cargo_path) {
                Ok(text) => text,
                Err(e) => {
                    app.set_status(
                        if en {
                            format!("error reading Cargo.toml: {e}")
                        } else {
                            format!("erro ao ler Cargo.toml: {e}")
                        },
                        StatusKind::Warning,
                    );
                    return;
                }
            }
        };
        let mut updated_cargo =
            crate::doc_hint::ensure_dependency_in_cargo_toml(&cargo_raw, lib.name, lib.version);
        if lib.name == "bevy" {
            updated_cargo =
                crate::doc_hint::ensure_bevy_fast_dev_profile_in_cargo_toml(&updated_cargo);
        }
        if updated_cargo != cargo_raw {
            if let Err(e) = project.save(&cargo_path, &updated_cargo) {
                app.set_status(
                    if en {
                        format!("error saving Cargo.toml: {e}")
                    } else {
                        format!("erro ao salvar Cargo.toml: {e}")
                    },
                    StatusKind::Warning,
                );
                return;
            }
            if viewing_cargo {
                self.editor.select_all();
                self.editor.insert_text(&updated_cargo);
                self.editor.mark_saved();
            }
        }

        let viewing_rs = self
            .editor
            .path
            .as_ref()
            .is_some_and(|p| p.extension().is_some_and(|ext| ext == "rs"));
        let target_name = if viewing_rs {
            let cur = self.editor.buf.to_string_lossy();
            let updated_rs = crate::doc_hint::insert_use_line_in_rust_code(&cur, lib.use_line);
            if updated_rs != cur {
                self.editor.select_all();
                self.editor.insert_text(&updated_rs);
                self.notify_lsp_change();
            }
            self.editor
                .path
                .as_ref()
                .and_then(|p| p.strip_prefix(&project.root).ok().or(Some(p.as_path())))
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "editor".into())
        } else {
            let main_path = project.root.join("src/main.rs");
            if let Ok(main_raw) = project.read(&main_path) {
                let updated_rs =
                    crate::doc_hint::insert_use_line_in_rust_code(&main_raw, lib.use_line);
                if updated_rs != main_raw {
                    let _ = project.save(&main_path, &updated_rs);
                }
            }
            "src/main.rs".to_string()
        };

        self.workspace_index = WorkspaceIndex::reindex(&project, &self.files);
        self.lib_menu_open = false;
        self.focus = Focus::Editor;
        self.console.system(if en {
            format!(
                "✔ Library `{}` ({}) ready in Cargo.toml + `{}` in {}",
                lib.name, lib.category, lib.use_line, target_name
            )
        } else {
            format!(
                "✔ Biblioteca `{}` ({}) pronta em Cargo.toml + `{}` em {}",
                lib.name, lib.category, lib.use_line, target_name
            )
        });
        if lib.name == "bevy" {
            self.console.system(crate::i18n::bi(
                "Dica Bevy: pressione `T` no menu F9 (ou `/bevy` no agente) para inserir template 2D e use `F4 → Rodar em outro terminal` se preferir rodar fora da IDE.",
                "Bevy tip: press `T` in the F9 menu (or `/bevy` in the agent) to insert the 2D template, and use `F4 → Run in another terminal` to run outside the IDE.",
                en,
            ));
        }
        app.set_status(
            if en {
                format!(
                    "Library `{}` added to project and imported in {}",
                    lib.name, target_name
                )
            } else {
                format!(
                    "Biblioteca `{}` adicionada ao projeto e importada em {}",
                    lib.name, target_name
                )
            },
            StatusKind::Success,
        );
    }

    /// Instala a biblioteca escolhida e insere um template inicial funcional no editor.
    pub fn install_library_template(&mut self, idx: usize, app: &mut App) {
        let en = app.is_english();
        let Some(lib) = crate::doc_hint::extra_libraries(en).get(idx) else {
            return;
        };
        self.install_library_into_code(idx, app);
        let template = if lib.name == "bevy" {
            crate::doc_hint::bevy_starter_template_for(en).to_string()
        } else if lib.example.contains("fn main(") {
            format!("{}\n\n{}\n", lib.use_line, lib.example)
        } else {
            let indented = lib
                .example
                .lines()
                .map(|l| format!("    {l}"))
                .collect::<Vec<_>>()
                .join("\n");
            format!("{}\n\nfn main() {{\n{indented}\n}}\n", lib.use_line)
        };
        let viewing_rs = self
            .editor
            .path
            .as_ref()
            .is_some_and(|p| p.extension().is_some_and(|ext| ext == "rs"));
        if viewing_rs {
            self.editor.select_all();
            self.editor.insert_text(&template);
            self.notify_lsp_change();
        } else if let Some(project) = &self.project {
            let main_path = project.root.join("src/main.rs");
            let _ = project.save(&main_path, &template);
        }
        self.console.system(if en {
            format!(
                "★ Starter template for `{}` applied (Ctrl+Z undoes · F4 opens/runs in another terminal).",
                lib.name
            )
        } else {
            format!(
                "★ Template inicial de `{}` aplicado (Ctrl+Z desfaz · F4 abre/roda em outro terminal).",
                lib.name
            )
        });
        app.set_status(
            if en {
                format!(
                    "Starter template for `{}` inserted (Ctrl+Z undoes)",
                    lib.name
                )
            } else {
                format!(
                    "Template inicial de `{}` inserido (Ctrl+Z desfaz)",
                    lib.name
                )
            },
            StatusKind::Success,
        );
    }

    fn on_build_menu_key(&mut self, app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::F(4) => self.build_menu_open = false,
            KeyCode::Up | KeyCode::Char('k') => {
                self.build_menu_sel = self.build_menu_sel.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.build_menu_sel < 4 {
                    self.build_menu_sel += 1;
                }
            }
            KeyCode::Enter => self.execute_build_menu_option(self.build_menu_sel, app),
            _ => {}
        }
    }

    pub fn execute_build_menu_option(&mut self, idx: usize, app: &mut App) {
        let en = app.is_english();
        self.build_menu_open = false;
        match idx {
            0 => {
                if !app.exec.running {
                    self.after_exec = AfterExec::LaunchExternalWindow;
                }
                self.request_exec(ToolCommand::Build, app);
            }
            1 => {
                let Some(project) = self.project.as_ref().cloned() else {
                    app.set_status(
                        crate::i18n::bi("nenhum projeto aberto", "no project open", en),
                        StatusKind::Warning,
                    );
                    return;
                };
                self.save_current(app);
                match spawn_external_cargo_terminal(&project.root, en) {
                    Ok(msg) => {
                        self.console.system(format!("✔ {msg}"));
                        self.console.system(if en {
                            format!(
                                "Equivalent command to open another terminal manually: cd \"{}\" && cargo run",
                                project.root.display()
                            )
                        } else {
                            format!(
                                "Comando equivalente caso queira abrir outro terminal manualmente: cd \"{}\" && cargo run",
                                project.root.display()
                            )
                        });
                        app.set_status(msg, StatusKind::Success);
                    }
                    Err(msg) => {
                        self.console.system(format!("ℹ {msg}"));
                        app.set_status(msg, StatusKind::Warning);
                    }
                }
            }
            2 => {
                self.after_exec = AfterExec::None;
                self.request_exec(ToolCommand::BuildLinuxRelease, app);
            }
            3 => {
                self.after_exec = AfterExec::None;
                self.request_exec(ToolCommand::BuildWindowsRelease, app);
            }
            4 => {
                self.after_exec = AfterExec::None;
                self.request_exec(ToolCommand::Build, app);
            }
            _ => {}
        }
    }

    /// Cria `PLANO.md` para o projeto independente da Etapa 10 (sem sobrescrever)
    /// e abre o arquivo no editor quando não houver alterações não salvas.
    pub fn create_project_plan(&mut self, app: &mut App) {
        let en = app.is_english();
        let Some(project) = &self.project else {
            app.set_status(
                crate::i18n::bi(
                    "Abra um projeto antes de criar PLANO.md.",
                    "Open a project before creating PLANO.md.",
                    en,
                ),
                StatusKind::Warning,
            );
            return;
        };
        let (plan_path, created) = match project.write_project_plan_template() {
            Ok(path) => (path, true),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                (project.root.join("PLANO.md"), false)
            }
            Err(err) => {
                app.set_status(
                    format!(
                        "{}: {err}",
                        crate::i18n::bi("Erro ao criar PLANO.md", "Error creating PLANO.md", en)
                    ),
                    StatusKind::Warning,
                );
                return;
            }
        };
        self.files = project.source_files();
        self.workspace_index = WorkspaceIndex::reindex(project, &self.files);
        if let Some(idx) = self.files.iter().position(|p| p == &plan_path) {
            self.switch_file(idx, app);
        }
        let mut changed = false;
        let mut awarded = false;
        grant_project_milestone(
            app,
            "plan",
            25,
            crate::i18n::bi("Plano Etapa 10 (PLANO.md)", "Stage 10 Plan (PLANO.md)", en),
            &mut changed,
            &mut awarded,
        );
        maybe_award_independent_capstone(app, &mut changed, &mut awarded);
        if changed && !awarded {
            let _ = app.persist_progress();
        }
        if !awarded {
            let msg = if created {
                crate::i18n::bi(
                    "PLANO.md criado para guiar seu projeto independente (Etapa 10).",
                    "PLANO.md created to guide your independent project (Stage 10).",
                    en,
                )
            } else {
                crate::i18n::bi(
                    "PLANO.md já existe; arquivo aberto no editor.",
                    "PLANO.md already exists; file opened in editor.",
                    en,
                )
            };
            app.set_status(msg, StatusKind::Success);
        }
    }

    fn record_project_milestones(&self, label: &str, app: &mut App) {
        if !matches!(
            label,
            "cargo check"
                | "cargo build"
                | "cargo build (Linux)"
                | "cargo build (Windows)"
                | "cargo clippy"
                | "cargo test"
                | "cargo run"
        ) {
            return;
        }
        let Some(project) = &self.project else {
            return;
        };
        let en = app.is_english();
        let caps = project.inspect_capabilities();
        let mut changed = false;
        let mut awarded = false;

        let checks = [
            (
                caps.has_plan_md,
                "plan",
                25,
                crate::i18n::bi("Plano Etapa 10 (PLANO.md)", "Stage 10 Plan (PLANO.md)", en),
            ),
            (
                caps.source_file_count >= 2,
                "multi-module",
                25,
                crate::i18n::bi(
                    "Projeto organizado em múltiplos arquivos",
                    "Multi-file project organization",
                    en,
                ),
            ),
            (
                caps.has_structs_or_enums,
                "types",
                20,
                crate::i18n::bi(
                    "Tipos próprios (struct/enum) no projeto",
                    "Custom types (struct/enum) in project",
                    en,
                ),
            ),
            (
                caps.has_error_handling,
                "error-handling",
                25,
                crate::i18n::bi(
                    "Tratamento explícito com Option/Result",
                    "Explicit Option/Result error handling",
                    en,
                ),
            ),
            (
                caps.has_doc_comments,
                "docs",
                20,
                crate::i18n::bi(
                    "Documentação de código com ///",
                    "Code documentation with ///",
                    en,
                ),
            ),
        ];
        for (ok, key, xp, desc) in checks {
            if ok {
                grant_project_milestone(app, key, xp, desc, &mut changed, &mut awarded);
            }
        }
        match label {
            "cargo check" | "cargo build" | "cargo build (Linux)" | "cargo build (Windows)" => {
                grant_project_milestone(
                    app,
                    "build",
                    20,
                    crate::i18n::bi("Build limpo no projeto", "Clean project build", en),
                    &mut changed,
                    &mut awarded,
                );
            }
            "cargo clippy" => {
                grant_project_milestone(
                    app,
                    "clippy",
                    25,
                    crate::i18n::bi("Clippy aprovado no projeto", "Clean Clippy check", en),
                    &mut changed,
                    &mut awarded,
                );
            }
            "cargo test" if caps.has_unit_tests => {
                grant_project_milestone(
                    app,
                    "tests",
                    35,
                    crate::i18n::bi(
                        "Testes unitários aprovados no projeto",
                        "Unit tests passed in project",
                        en,
                    ),
                    &mut changed,
                    &mut awarded,
                );
            }
            "cargo run" => {
                grant_project_milestone(
                    app,
                    "run",
                    20,
                    crate::i18n::bi(
                        "Aplicação executada no projeto",
                        "Application ran in project",
                        en,
                    ),
                    &mut changed,
                    &mut awarded,
                );
            }
            _ => {}
        }
        maybe_award_independent_capstone(app, &mut changed, &mut awarded);
        if changed && !awarded {
            let _ = app.persist_progress();
        }
    }

    pub fn open_projects_dir_prompt(&mut self, app: &App) {
        self.completion = Default::default();
        self.prompt = Some(PromptState {
            kind: PromptKind::SetProjectsDir,
            text: app.projects_base_dir().display().to_string(),
            error: None,
        });
    }

    fn cycle_focus(&mut self, forward: bool) {
        self.focus = match (forward, self.agent.open, self.focus) {
            (true, false, Focus::Files) => Focus::Editor,
            (true, false, Focus::Editor) => Focus::Console,
            (true, false, Focus::Console | Focus::Agent) => Focus::Files,
            (true, true, Focus::Files) => Focus::Editor,
            (true, true, Focus::Editor) => Focus::Console,
            (true, true, Focus::Console) => Focus::Agent,
            (true, true, Focus::Agent) => Focus::Files,
            (false, false, Focus::Editor) => Focus::Files,
            (false, false, Focus::Files) => Focus::Console,
            (false, false, Focus::Console | Focus::Agent) => Focus::Editor,
            (false, true, Focus::Editor) => Focus::Files,
            (false, true, Focus::Files) => Focus::Console,
            (false, true, Focus::Console) => Focus::Agent,
            (false, true, Focus::Agent) => Focus::Editor,
        };
    }

    pub fn toggle_agent_panel(&mut self, app: &mut App) {
        let en = app.is_english();
        self.refresh_language_for(en);
        if !self.agent.open {
            self.agent.open = true;
            self.focus = Focus::Agent;
            app.set_status(
                crate::i18n::bi(
                    "Agente Antigravity aberto à direita (Alt+O modelo · Alt+Enter aplicar)",
                    "Antigravity Agent opened on the right (Alt+O model · Alt+Enter apply)",
                    en,
                ),
                StatusKind::Info,
            );
        } else if self.focus != Focus::Agent {
            self.focus = Focus::Agent;
            app.set_status(
                crate::i18n::bi(
                    "Foco no Agente Antigravity (Esc volta ao editor)",
                    "Focus on Antigravity Agent (Esc returns to editor)",
                    en,
                ),
                StatusKind::Info,
            );
        } else {
            self.agent.open = false;
            self.focus = Focus::Editor;
            app.set_status(
                crate::i18n::bi("Painel do Agente fechado", "Agent panel closed", en),
                StatusKind::Info,
            );
        }
    }

    pub fn agent_running(&self) -> bool {
        self.agent.manager.running
    }

    pub fn poll_agent(&mut self, app: &mut App) -> bool {
        let en = app.is_english();
        let mut changed = false;
        let events = self.agent.manager.poll_events();
        for ev in events {
            changed = true;
            match ev {
                MentorEvent::Completed { source, body, .. } => {
                    self.agent.proposed_code = crate::mentor::extract_rust_code_block(&body);
                    let has_code = self.agent.proposed_code.is_some();
                    self.agent.messages.push(AgentChatMessage {
                        role: AgentRole::Agent,
                        sender: source,
                        text: body,
                    });
                    self.agent.scroll = u16::MAX;
                    if has_code {
                        app.set_status(
                            crate::i18n::bi(
                                "Agente sugeriu código — Alt+Enter (ou /aplicar) aplica no editor",
                                "Agent suggested code — Alt+Enter (or /apply) applies to editor",
                                en,
                            ),
                            StatusKind::Success,
                        );
                    }
                }
                MentorEvent::Failed { message, .. } => {
                    self.agent.messages.push(AgentChatMessage {
                        role: AgentRole::System,
                        sender: crate::i18n::bi("Aviso", "Notice", en).to_string(),
                        text: format!(
                            "{message}\n{}",
                            crate::i18n::bi(
                                "Sua mensagem continua no histórico: reenvie ou use Alt+O para trocar de modelo.",
                                "Your message remains in history: resend or use Alt+O to switch models.",
                                en
                            )
                        ),
                    });
                    self.agent.scroll = u16::MAX;
                    app.set_status(
                        format!("{}: {message}", crate::i18n::bi("Agente", "Agent", en)),
                        StatusKind::Warning,
                    );
                }
            }
        }
        if self.agent.student_modified_since_review
            && !self.agent.manager.running
            && self.view == CreateView::Workspace
            && self.project.is_some()
            && self.agent.last_auto_review.elapsed() >= self.agent.next_auto_review_interval
        {
            self.trigger_periodic_code_review(app, false);
            changed = true;
        }
        changed
    }

    fn on_agent_key(&mut self, app: &mut App, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        match key.code {
            KeyCode::Enter if alt => self.apply_agent_code(app),
            KeyCode::Char('e') if ctrl => self.apply_agent_code(app),
            KeyCode::Enter => self.submit_agent_input(app),
            KeyCode::Backspace => {
                self.agent.input.pop();
            }
            KeyCode::PageUp => {
                self.agent.scroll = self.agent.scroll.saturating_sub(4);
            }
            KeyCode::PageDown => {
                self.agent.scroll = self.agent.scroll.saturating_add(4);
            }
            KeyCode::Home => self.agent.scroll = 0,
            KeyCode::End => self.agent.scroll = u16::MAX,
            KeyCode::Up if self.agent.input.is_empty() => {
                self.agent.scroll = self.agent.scroll.saturating_sub(3);
            }
            KeyCode::Down if self.agent.input.is_empty() => {
                self.agent.scroll = self.agent.scroll.saturating_add(3);
            }
            KeyCode::Char(c) if !ctrl && !alt && self.agent.input.chars().count() < 400 => {
                self.agent.input.push(c);
            }
            _ => {}
        }
    }

    fn submit_agent_input(&mut self, app: &mut App) {
        let en = app.is_english();
        let raw = self.agent.input.trim().to_string();
        if raw.is_empty() {
            return;
        }
        self.agent.input.clear();
        match raw.as_str() {
            "/modelo" | "/model" => {
                self.agent.model_picker.open_picker(app);
                return;
            }
            "/aplicar" | "/apply" => {
                self.apply_agent_code(app);
                return;
            }
            "/revisar" | "/review" => {
                self.trigger_periodic_code_review(app, true);
                return;
            }
            "/bevy" => {
                if let Some(bevy_idx) = crate::doc_hint::extra_libraries(en)
                    .iter()
                    .position(|l| l.name == "bevy")
                {
                    self.install_library_into_code(bevy_idx, app);
                }
                self.agent.open = true;
                self.focus = Focus::Agent;
                self.agent.proposed_code =
                    Some(crate::doc_hint::bevy_starter_template_for(en).to_string());
                self.agent.messages.push(AgentChatMessage {
                    role: AgentRole::Agent,
                    sender: crate::i18n::bi("Bevy Guia", "Bevy Guide", en).to_string(),
                    text: crate::i18n::bi(
                        "Bevy (ECS) pronto!\n\
• Dependência `bevy = \"0.20\"` + `[profile.dev]` adicionados ao Cargo.toml.\n\
• Um template 2D completo (`Camera2d`, `Sprite` e movimento WASD/setas com `Transform`) foi preparado abaixo — pressione `Alt+Enter` para colar no editor.\n\
• Para rodar: pressione `F4 → Rodar em outro terminal` (ou `cd` na pasta do projeto e `cargo run` em um segundo terminal enquanto você edita no RustLab!).",
                        "Bevy (ECS) ready!\n\
• Dependency `bevy = \"0.20\"` + `[profile.dev]` added to Cargo.toml.\n\
• A complete 2D template (`Camera2d`, `Sprite`, and WASD/arrow movement with `Transform`) is ready below — press `Alt+Enter` to insert it into the editor.\n\
• To run: press `F4 → Run in another terminal` (or `cd` into the project directory and run `cargo run` in a second terminal while editing in RustLab!).",
                        en,
                    )
                    .to_string(),
                });
                self.agent.scroll = u16::MAX;
                app.set_status(
                    crate::i18n::bi(
                        "Bevy configurado! Alt+Enter aplica template 2D · F4 roda em outro terminal",
                        "Bevy configured! Alt+Enter applies 2D template · F4 runs in another terminal",
                        en,
                    ),
                    StatusKind::Success,
                );
                return;
            }
            "/pong" | "/snake" | "/editor" | "/paint" | "/cavegame" => {
                let (label, template, summary, install_bevy) = match raw.as_str() {
                    "/pong" => (
                        "Pong Terminal (D10-P02)",
                        crate::doc_hint::pong_terminal_starter_template(),
                        crate::i18n::bi(
                            "Template `PongState` com colisão, placar, renderização ASCII e `#[cfg(test)]` preparado.",
                            "`PongState` template with collision, score, ASCII rendering, and `#[cfg(test)]` prepared.",
                            en,
                        ),
                        false,
                    ),
                    "/snake" => (
                        "Snake Terminal (D11-P02)",
                        crate::doc_hint::snake_terminal_starter_template(),
                        crate::i18n::bi(
                            "Template `SnakeGame` com seed determinística, crescimento, colisões e `#[cfg(test)]` preparado.",
                            "`SnakeGame` template with deterministic seed, growth, collisions, and `#[cfg(test)]` prepared.",
                            en,
                        ),
                        false,
                    ),
                    "/editor" => (
                        "Mini Editor (D12-P02)",
                        crate::doc_hint::mini_editor_starter_template(),
                        crate::i18n::bi(
                            "Template `MiniEditor` multi-linha com `undo`/`redo` e `#[cfg(test)]` preparado.",
                            "Multi-line `MiniEditor` template with `undo`/`redo` and `#[cfg(test)]` prepared.",
                            en,
                        ),
                        false,
                    ),
                    "/paint" => (
                        "Mini Paint 2D (PAINT-01..12)",
                        crate::doc_hint::mini_paint_starter_template(),
                        crate::i18n::bi(
                            "Template `Canvas2D` com traço contínuo, escala preservada em resize, exportação PPM e `#[cfg(test)]` preparado.",
                            "`Canvas2D` template with continuous strokes, resize scaling, PPM export, and `#[cfg(test)]` prepared.",
                            en,
                        ),
                        false,
                    ),
                    _ => (
                        "CaveGame Voxel + Bevy 0.20 (CAVE-01..18)",
                        crate::doc_hint::cavegame_voxel_starter_template(),
                        crate::i18n::bi(
                            "Dependência `bevy = \"0.20\"` + template `VoxelWorld` 24×24 chunks, culling de faces, persistência e `#[cfg(test)]` preparado.",
                            "Dependency `bevy = \"0.20\"` + `VoxelWorld` 24×24 chunks template, face culling, persistence, and `#[cfg(test)]` prepared.",
                            en,
                        ),
                        true,
                    ),
                };
                if install_bevy
                    && let Some(bevy_idx) = crate::doc_hint::extra_libraries(en)
                        .iter()
                        .position(|l| l.name == "bevy")
                {
                    self.install_library_into_code(bevy_idx, app);
                }
                self.agent.open = true;
                self.focus = Focus::Agent;
                self.agent.proposed_code = Some(template.to_string());
                self.agent.messages.push(AgentChatMessage {
                    role: AgentRole::Agent,
                    sender: crate::i18n::bi("Capstones V3", "V3 Capstones", en).to_string(),
                    text: if en {
                        format!(
                            "{label} ready!\n• {summary}\n• Press `Alt+Enter` to insert into editor and `F6` to run tests."
                        )
                    } else {
                        format!(
                            "{label} pronto!\n• {summary}\n• Pressione `Alt+Enter` para inserir no editor e `F6` para rodar os testes."
                        )
                    },
                });
                self.agent.scroll = u16::MAX;
                app.set_status(
                    if en {
                        format!("{label} ready! Alt+Enter applies in editor · F6 tests")
                    } else {
                        format!("{label} preparado! Alt+Enter aplica no editor · F6 testa")
                    },
                    StatusKind::Success,
                );
                return;
            }
            "/raiox" | "/memoria" | "/xray" => {
                self.open_lab_tab(crate::lab_tools::LabToolTab::RaioXMemoria, app);
                return;
            }
            "/lente" | "/eli10" | "/lens" => {
                self.lab.eli10_mode = raw == "/eli10";
                self.open_lab_tab(crate::lab_tools::LabToolTab::LenteCompilador, app);
                return;
            }
            "/debug" | "/depurar" => {
                self.open_lab_tab(crate::lab_tools::LabToolTab::DebuggerVisual, app);
                return;
            }
            "/bp" | "/breakpoint" => {
                self.debugger_toggle_breakpoint_at_cursor(app);
                return;
            }
            "/consertar" | "/quest" | "/golf" => {
                self.open_lab_tab(crate::lab_tools::LabToolTab::ConserteOCodigo, app);
                return;
            }
            "/playground" | "/comunidade" | "/community" => {
                self.open_lab_tab(crate::lab_tools::LabToolTab::PlaygroundComunidade, app);
                return;
            }
            "/wasm" | "/cli" | "/axum" => {
                let idx = match raw.as_str() {
                    "/wasm" => 0,
                    "/cli" => 1,
                    _ => 2,
                };
                let eco = crate::lab_tools::modern_ecosystems(en)[idx];
                self.lab.ecosystem_sel = idx;
                self.agent.open = true;
                self.focus = Focus::Agent;
                self.agent.proposed_code = Some(eco.template_code.to_string());
                self.agent.messages.push(AgentChatMessage {
                    role: AgentRole::Agent,
                    sender: crate::i18n::bi("Ecossistemas V3", "V3 Ecosystems", en).to_string(),
                    text: format!(
                        "{} ({})\n• {}\n• {}\n• {}",
                        eco.name,
                        eco.ecosystem,
                        eco.summary,
                        eco.run_instructions,
                        crate::i18n::bi(
                            "Pressione Alt+Enter para inserir no editor e F6 para testar!",
                            "Press Alt+Enter to insert into the editor and F6 to test!",
                            en
                        )
                    ),
                });
                self.agent.scroll = u16::MAX;
                app.set_status(
                    if en {
                        format!("{} ready! Alt+Enter inserts · F6 tests", eco.name)
                    } else {
                        format!("{} pronto! Alt+Enter insere · F6 testa", eco.name)
                    },
                    StatusKind::Success,
                );
                return;
            }
            "/lifetimes" | "/lt" => {
                self.lab.lifetime_gutter_enabled = true;
                self.open_lab_tab(crate::lab_tools::LabToolTab::MapaLifetimes, app);
                return;
            }
            "/layout" | "/sizeof" | "/padding" => {
                self.open_lab_tab(crate::lab_tools::LabToolTab::LayoutMemoria, app);
                return;
            }
            "/desugar" | "/expand" | "/macros" => {
                self.open_lab_tab(crate::lab_tools::LabToolTab::DesacucararMacros, app);
                return;
            }
            "/poliglota" | "/polyglot" | "/python" | "/js" | "/cpp" => {
                self.lab.polyglot_mode_flashcards = false;
                self.lab.polyglot_lang_filter = match raw.as_str() {
                    "/python" => 1,
                    "/js" => 2,
                    "/cpp" => 3,
                    _ => 0,
                };
                self.open_lab_tab(crate::lab_tools::LabToolTab::PoliglotaIntuicao, app);
                return;
            }
            "/treino" | "/train" | "/flashcard" | "/intuicao" => {
                self.lab.polyglot_mode_flashcards = true;
                self.open_lab_tab(crate::lab_tools::LabToolTab::PoliglotaIntuicao, app);
                return;
            }
            "/gdb" | "/lldb" => {
                self.open_lab_tab(crate::lab_tools::LabToolTab::DebuggerVisual, app);
                self.open_native_debugger(app);
                return;
            }
            "/limpar" | "/clear" => {
                let open = self.agent.open;
                self.agent = CreateAgentState::new_for(en);
                self.agent.open = open;
                return;
            }
            _ => {}
        }
        if self.agent.manager.running {
            self.agent.input = raw;
            app.set_status(
                crate::i18n::bi(
                    "aguarde a resposta do agente ou pressione Ctrl+K",
                    "wait for the agent response or press Ctrl+K",
                    en,
                ),
                StatusKind::Info,
            );
            return;
        }
        let prompt = match raw.as_str() {
            "/corrigir" | "/fix" | "/console" | "/erros" | "/errors" => crate::i18n::bi(
                "Analise a saída recente do Console (stderr/stdout), os diagnósticos do compilador e o código do arquivo aberto; explique a causa real e proponha a correção completa.",
                "Analyze the recent Console output (stderr/stdout), compiler diagnostics, and the open file code; explain the root cause and propose the complete fix.",
                en,
            )
            .to_string(),
            "/testes" | "/tests" => crate::i18n::bi(
                "Escreva testes unitários #[cfg(test)] completos para as funções do arquivo aberto.",
                "Write complete #[cfg(test)] unit tests for the functions in the open file.",
                en,
            )
            .to_string(),
            _ => raw.clone(),
        };
        self.agent.messages.push(AgentChatMessage {
            role: AgentRole::User,
            sender: crate::i18n::bi("Você", "You", en).to_string(),
            text: raw,
        });
        self.agent.scroll = u16::MAX;
        if app.config.ai.provider == AiProvider::Remote
            && app.config.ai.remote_consent == RemoteAiConsent::Ask
        {
            self.agent.consent_pending = Some(prompt);
            return;
        }
        self.dispatch_agent_query(app, prompt);
    }

    pub fn diagnose_error_with_agent(&mut self, app: &mut App) {
        let en = app.is_english();
        self.agent.open = true;
        self.focus = Focus::Agent;
        let prompt = crate::i18n::bi(
            "Diagnostique o último erro do Cargo/Console neste projeto e explique como corrigi-lo sem reescrever o que já está certo.",
            "Diagnose the latest Cargo/Console error in this project and explain how to fix it without breaking working code.",
            en,
        )
        .to_string();
        self.agent.messages.push(AgentChatMessage {
            role: AgentRole::User,
            sender: crate::i18n::bi("Você", "You", en).to_string(),
            text: prompt.clone(),
        });
        self.agent.scroll = u16::MAX;
        self.dispatch_agent_query(app, prompt);
    }

    fn dispatch_agent_query(&mut self, app: &mut App, prompt: String) {
        let en = app.is_english();
        let prior_len = self.agent.messages.len().saturating_sub(1);
        let history: Vec<(String, String)> = self
            .agent
            .messages
            .iter()
            .take(prior_len)
            .filter_map(|m| match m.role {
                AgentRole::User => Some(("user".to_string(), m.text.clone())),
                AgentRole::Agent => Some(("assistant".to_string(), m.text.clone())),
                AgentRole::System => None,
            })
            .collect();
        let mut ctx = crate::mentor::create_context_from_for(self, en);
        ctx.origin_language = app.config.origin_language;
        ctx.recurring_errors = app.progress.top_recurring_errors(3);
        self.agent
            .manager
            .submit_agent_chat_for(history, ctx, prompt, app.config.ai.clone(), en);
        let _ = self.poll_agent(app);
    }

    pub fn apply_agent_code(&mut self, app: &mut App) {
        let en = app.is_english();
        let Some(code) = self.agent.proposed_code.clone() else {
            app.set_status(
                crate::i18n::bi(
                    "Nenhuma sugestão de código do agente para aplicar.",
                    "No code suggestion from the agent to apply.",
                    en,
                ),
                StatusKind::Info,
            );
            return;
        };
        if self.editor.selection().is_none() {
            self.editor.select_all();
        }
        self.editor.insert_text(&code);
        for line in code.lines().map(str::trim).filter(|l| !l.is_empty()) {
            self.agent.ai_authored_lines.insert(line.to_string());
        }
        self.notify_lsp_change();
        self.agent.messages.push(AgentChatMessage {
            role: AgentRole::System,
            sender: "Antigravity".to_string(),
            text: crate::i18n::bi(
                "Código aplicado no editor (Ctrl+Z para desfazer).",
                "Code applied in editor (Ctrl+Z to undo).",
                en,
            )
            .to_string(),
        });
        self.focus = Focus::Editor;
        app.set_status(
            crate::i18n::bi(
                "Código do agente aplicado no editor (Ctrl+Z desfaz).",
                "Agent code applied in editor (Ctrl+Z undoes).",
                en,
            ),
            StatusKind::Success,
        );
    }

    /// Executa a revisão periódica (15–30 min ou `/revisar`) avaliando apenas
    /// trechos escritos pelo estudante e evitando premiar código gerado pela IA.
    pub fn trigger_periodic_code_review(&mut self, app: &mut App, manual: bool) {
        let en = app.is_english();
        self.agent.last_auto_review = std::time::Instant::now();
        self.agent.next_auto_review_interval = next_random_review_interval();
        self.agent.student_modified_since_review = false;

        let code = self.editor.buf.to_string_lossy();
        let (student_lines, ai_lines) =
            crate::mentor::classify_code_authorship(&code, &self.agent.ai_authored_lines);
        if student_lines.is_empty() {
            if manual {
                self.agent.messages.push(AgentChatMessage {
                    role: AgentRole::System,
                    sender: "Mentor".to_string(),
                    text: crate::i18n::bi(
                        "Revisão periódica: todo o código atual veio do agente ou o editor está vazio. Escreva algumas linhas suas para receber dicas e reforço personalizado.",
                        "Periodic review: all current code came from the agent or the editor is empty. Write a few lines of your own to receive tailored tips and feedback.",
                        en,
                    )
                    .to_string(),
                });
                self.agent.scroll = u16::MAX;
            }
            return;
        }

        self.agent.open = true;
        let ctx = crate::mentor::create_context_from_for(self, en);
        self.agent.manager.submit_periodic_review_for(
            ctx,
            student_lines,
            ai_lines,
            app.config.ai.clone(),
            en,
        );
        let _ = self.poll_agent(app);
        app.set_status(
            crate::i18n::bi(
                "Mentor IA revisando seu código (15–30 min) — dicas em Alt+A",
                "AI Mentor reviewing your code (15–30 min) — tips in Alt+A",
                en,
            ),
            StatusKind::Info,
        );
    }

    // ------------------------------------------------------------------
    // Hub Laboratório RustLab: Raio-X, Lente ELI10, Debugger, Quests,
    // Playground & Ecossistemas Modernos
    // ------------------------------------------------------------------

    /// Retorna as linhas mais recentes do Console com prefixo do stream e preservação
    /// do trecho final (tail) caso exceda `max_bytes`.
    pub fn recent_console_output(&self, max_lines: usize, max_bytes: usize) -> String {
        if max_lines == 0 || max_bytes == 0 {
            return String::new();
        }
        let start = self.console.lines.len().saturating_sub(max_lines);
        let mut lines: Vec<String> = self
            .console
            .lines
            .iter()
            .skip(start)
            .map(|line| {
                let tag = match line.kind {
                    StreamKind::Stdout => "[stdout]",
                    StreamKind::Stderr => "[stderr]",
                    StreamKind::System => "[sistema]",
                };
                format!("{tag} {}", line.text)
            })
            .collect();

        let has_stderr_in_console = self
            .console
            .lines
            .iter()
            .skip(start)
            .any(|l| l.kind == StreamKind::Stderr);
        if !has_stderr_in_console && !self.stderr_buf.trim().is_empty() {
            let stderr_lines: Vec<&str> = self.stderr_buf.lines().collect();
            let s_start = stderr_lines.len().saturating_sub(max_lines);
            for raw_line in stderr_lines.into_iter().skip(s_start) {
                lines.push(format!("[stderr] {raw_line}"));
            }
        }

        let joined = lines.join("\n");
        if joined.len() <= max_bytes {
            return joined;
        }
        let mut cut = joined.len().saturating_sub(max_bytes);
        while cut < joined.len() && !joined.is_char_boundary(cut) {
            cut += 1;
        }
        let tail = joined[cut..].trim_start_matches('\n');
        format!("[... linhas anteriores omitidas]\n{tail}")
    }

    pub fn diagnostics_text_summary(&self) -> String {
        let mut out = self.stderr_buf.clone();
        if out.trim().is_empty() {
            let console_tail = self.recent_console_output(60, 4_000);
            if !console_tail.trim().is_empty() {
                out.push_str(&console_tail);
            }
        }
        for d in &self.diags {
            out.push('\n');
            out.push_str(&d.message);
        }
        out
    }

    pub fn open_lab_tab(&mut self, tab: crate::lab_tools::LabToolTab, app: &mut App) {
        let en = app.is_english();
        let code = self.editor.buf.to_string_lossy();
        let diags = self.diagnostics_text_summary();
        self.lib_menu_open = false;
        self.build_menu_open = false;
        self.lab.open_tab_for(tab, &code, &diags, en);
        app.set_status(
            if en {
                format!("Lab: {} (1-0 tabs · Esc closes)", tab.title_for(true))
            } else {
                format!(
                    "Laboratório: {} (1-0 abas · Esc fecha)",
                    tab.title_for(false)
                )
            },
            StatusKind::Info,
        );
    }

    pub fn debugger_toggle_breakpoint_at_cursor(&mut self, app: &mut App) {
        let en = app.is_english();
        let line = self.editor.cursor.line + 1;
        let active = self.lab.debugger.toggle_breakpoint(line);
        let msg = if active {
            if en {
                format!("● Breakpoint set at line {line} (F10 or Alt+J debugs)")
            } else {
                format!("● Breakpoint marcado na linha {line} (F10 ou Alt+J depura)")
            }
        } else if en {
            format!("Breakpoint removed from line {line}")
        } else {
            format!("Breakpoint removido da linha {line}")
        };
        self.console.system(msg.clone());
        app.set_status(msg, StatusKind::Info);
    }

    fn ensure_debugger_started(&mut self) {
        if !self.lab.debugger.active || self.lab.debugger.steps.is_empty() {
            let code = self.editor.buf.to_string_lossy();
            self.lab.debugger.start_or_rebuild(&code);
        }
    }

    fn sync_editor_to_debug_step(&mut self, app: &mut App) {
        let en = app.is_english();
        let Some(step) = self.lab.debugger.current().cloned() else {
            app.set_status(
                crate::i18n::bi(
                    "Debugger sem passos executáveis para este arquivo.",
                    "Debugger has no executable steps for this file.",
                    en,
                ),
                StatusKind::Warning,
            );
            return;
        };
        let target_line = step.line_1based.saturating_sub(1);
        let max_line = self.editor.buf.line_count().saturating_sub(1);
        self.editor.cursor.line = target_line.min(max_line);
        self.editor.cursor.col = 0;
        self.editor.scroll_row = self.editor.cursor.line.saturating_sub(3);
        let vars_preview = if step.locals.is_empty() {
            crate::i18n::bi("sem locais", "no locals", en).to_string()
        } else {
            step.locals
                .iter()
                .take(3)
                .map(|l| format!("{}={}", l.name, l.value))
                .collect::<Vec<_>>()
                .join(", ")
        };
        app.set_status(
            format!(
                "Debug ▶ L{} [{}]: {} ({})",
                step.line_1based, step.func, step.note, vars_preview
            ),
            StatusKind::Info,
        );
    }

    pub fn debugger_step_over(&mut self, app: &mut App) {
        let was_active = self.lab.debugger.active;
        self.ensure_debugger_started();
        if was_active {
            let _ = self.lab.debugger.step_over();
        }
        self.sync_editor_to_debug_step(app);
    }

    pub fn debugger_step_into(&mut self, app: &mut App) {
        let was_active = self.lab.debugger.active;
        self.ensure_debugger_started();
        if was_active {
            let _ = self.lab.debugger.step_into();
        }
        self.sync_editor_to_debug_step(app);
    }

    pub fn debugger_continue(&mut self, app: &mut App) {
        self.ensure_debugger_started();
        let _ = self.lab.debugger.continue_to_breakpoint();
        self.sync_editor_to_debug_step(app);
    }

    pub fn open_native_debugger(&mut self, app: &mut App) {
        let en = app.is_english();
        let probe = crate::lab_tools::detect_native_debugger_for(en);
        let Some(project) = self.project.as_ref().cloned() else {
            self.lab.native_debugger_status = Some(
                crate::i18n::bi(
                    "Abra um projeto Cargo para anexar `rust-gdb` / `lldb` ao binário compilado.",
                    "Open a Cargo project to attach `rust-gdb` / `lldb` to the compiled binary.",
                    en,
                )
                .to_string(),
            );
            app.set_status(
                crate::i18n::bi(
                    "Abra um projeto Cargo antes de iniciar o debugger nativo.",
                    "Open a Cargo project before starting the native debugger.",
                    en,
                ),
                StatusKind::Warning,
            );
            return;
        };
        let pkg = package_binary_name(&project);
        let bin_path = project.root.join("target").join("debug").join(&pkg);
        let src_rel = self
            .editor
            .path
            .as_ref()
            .and_then(|p| p.strip_prefix(&project.root).ok())
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "src/main.rs".to_string());
        let args = crate::lab_tools::build_native_debugger_args(
            &probe,
            &bin_path,
            &src_rel,
            &self.lab.debugger.breakpoints,
        );
        let cmd_preview = crate::lab_tools::format_native_debugger_command(
            &probe,
            &bin_path,
            &src_rel,
            &self.lab.debugger.breakpoints,
        );

        let status_msg = if !probe.available {
            format!(
                "{} {}: {cmd_preview}",
                probe.install_hint,
                crate::i18n::bi("Comando pronto", "Command ready", en)
            )
        } else if !bin_path.is_file() && !cfg!(test) {
            if en {
                format!(
                    "Debugger `{}` ready! Build first (`F4` / `cargo build`) and run in another terminal: {cmd_preview}",
                    probe.binary_name
                )
            } else {
                format!(
                    "Debugger `{}` pronto! Compile primeiro (`F4` / `cargo build`) e rode em outro terminal: {cmd_preview}",
                    probe.binary_name
                )
            }
        } else {
            match spawn_external_terminal_command(&project.root, probe.binary_name, &args, en) {
                Ok(ok) => format!("{ok} · {cmd_preview}"),
                Err(err) => format!(
                    "{err} · {}: {cmd_preview}",
                    crate::i18n::bi("Comando", "Command", en)
                ),
            }
        };
        self.console.system(status_msg.clone());
        self.lab.native_debugger_status = Some(status_msg.clone());
        app.set_status(status_msg, StatusKind::Info);
    }

    fn apply_code_template_to_editor(
        &mut self,
        code: &str,
        label: &str,
        close_lab: bool,
        app: &mut App,
    ) {
        let en = app.is_english();
        self.editor.select_all();
        self.editor.insert_text(code);
        self.notify_lsp_change();
        if close_lab {
            self.lab.open = false;
            self.focus = Focus::Editor;
        }
        self.console.system(if en {
            format!("★ {label} loaded into editor (Ctrl+Z undoes).")
        } else {
            format!("★ {label} carregado no editor (Ctrl+Z desfaz).")
        });
        app.set_status(
            if en {
                format!("{label} loaded into editor (Ctrl+Z undoes)")
            } else {
                format!("{label} carregado no editor (Ctrl+Z desfaz)")
            },
            StatusKind::Success,
        );
    }

    fn add_dependency_to_cargo(&mut self, pkg: &str, version: &str, app: &mut App) {
        let en = app.is_english();
        let Some(project) = self.project.as_ref().cloned() else {
            app.set_status(
                crate::i18n::bi(
                    "Abra um projeto Cargo para adicionar dependências.",
                    "Open a Cargo project to add dependencies.",
                    en,
                ),
                StatusKind::Warning,
            );
            return;
        };
        let cargo_path = project.root.join("Cargo.toml");
        let Ok(raw) = project.read(&cargo_path) else {
            return;
        };
        let updated = crate::doc_hint::ensure_dependency_in_cargo_toml(&raw, pkg, version);
        if updated != raw && project.save(&cargo_path, &updated).is_ok() {
            if self.editor.path.as_deref() == Some(cargo_path.as_path()) {
                self.editor.select_all();
                self.editor.insert_text(&updated);
                self.editor.mark_saved();
            }
            self.console.system(if en {
                format!("✔ `{pkg} = \"{version}\"` added to Cargo.toml.")
            } else {
                format!("✔ `{pkg} = \"{version}\"` adicionado em Cargo.toml.")
            });
            app.set_status(
                if en {
                    format!("Dependency `{pkg} = \"{version}\"` added to Cargo.toml")
                } else {
                    format!("Dependência `{pkg} = \"{version}\"` adicionada ao Cargo.toml")
                },
                StatusKind::Success,
            );
        }
    }

    pub fn validate_active_quest(&mut self, app: &mut App) {
        let en = app.is_english();
        let quests = crate::lab_tools::troubleshooting_quests(en);
        let idx = self
            .lab
            .active_quest_id
            .and_then(|id| quests.iter().position(|q| q.id == id))
            .unwrap_or(self.lab.quest_sel.min(quests.len().saturating_sub(1)));
        let quest = quests[idx];
        let code = self.editor.buf.to_string_lossy();
        match crate::lab_tools::evaluate_quest_for(&quest, &code, en) {
            Ok(eval) => {
                self.lab.completed_quests.insert(quest.id);
                let _ = app.award_xp(format!("quest:{}", quest.id), eval.xp_awarded, quest.title);
                let _ = app.persist_progress();
                let bonus_txt = if eval.golf_bonus {
                    crate::i18n::bi(
                        " · Bônus Golfing + Zero Clones!",
                        " · Golfing + Zero Clones Bonus!",
                        en,
                    )
                } else {
                    ""
                };
                let msg = if en {
                    format!(
                        "✔ {} completed! (+{} XP · {} edited lines / par {} · Heap: {}){}",
                        quest.id,
                        eval.xp_awarded,
                        eval.modified_lines,
                        eval.par_lines,
                        eval.heap_allocations,
                        bonus_txt
                    )
                } else {
                    format!(
                        "✔ {} concluída! (+{} XP · {} linhas editadas / par {} · Heap: {}){}",
                        quest.id,
                        eval.xp_awarded,
                        eval.modified_lines,
                        eval.par_lines,
                        eval.heap_allocations,
                        bonus_txt
                    )
                };
                self.lab.last_quest_feedback = Some(msg.clone());
                self.console.system(msg.clone());
                app.set_status(msg, StatusKind::Success);
            }
            Err(reason) => {
                let msg = format!("✘ {}: {reason}", quest.id);
                self.lab.last_quest_feedback = Some(msg.clone());
                app.set_status(msg, StatusKind::Warning);
            }
        }
    }

    fn on_lab_modal_key(&mut self, app: &mut App, key: KeyEvent) {
        let en = app.is_english();
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        match key.code {
            KeyCode::Esc => {
                self.lab.open = false;
                return;
            }
            KeyCode::Char('x') if alt => {
                self.lab.open = false;
                return;
            }
            KeyCode::Tab | KeyCode::Right => {
                let next = self.lab.tab.next();
                self.open_lab_tab(next, app);
                return;
            }
            KeyCode::BackTab | KeyCode::Left => {
                let prev = self.lab.tab.prev();
                self.open_lab_tab(prev, app);
                return;
            }
            KeyCode::Char(d @ '0'..='9') => {
                if let Some(tab) = crate::lab_tools::LabToolTab::from_digit(d) {
                    self.open_lab_tab(tab, app);
                }
                return;
            }
            KeyCode::PageUp => {
                self.lab.scroll = self.lab.scroll.saturating_sub(4);
                return;
            }
            KeyCode::PageDown => {
                self.lab.scroll = self.lab.scroll.saturating_add(4);
                return;
            }
            _ => {}
        }

        let code = self.editor.buf.to_string_lossy();
        let diag_text = self.diagnostics_text_summary();
        match self.lab.tab {
            crate::lab_tools::LabToolTab::RaioXMemoria => match key.code {
                KeyCode::Char('r' | 'R') => {
                    self.lab.refresh_for(&code, &diag_text, en);
                    app.set_status(
                        crate::i18n::bi(
                            "Raio-X de Memória atualizado.",
                            "Memory X-Ray updated.",
                            en,
                        ),
                        StatusKind::Info,
                    );
                }
                KeyCode::Char('[' | 'p' | 'P') => {
                    if let Some(xray) = self.lab.last_xray.as_mut() {
                        xray.step_prev();
                        if let Some(step) = xray.timeline.get(xray.timeline_cursor) {
                            app.set_status(step.explanation.clone(), StatusKind::Info);
                        }
                    }
                }
                KeyCode::Char(']' | 'n' | 'N') => {
                    if let Some(xray) = self.lab.last_xray.as_mut() {
                        xray.step_next();
                        if let Some(step) = xray.timeline.get(xray.timeline_cursor) {
                            app.set_status(step.explanation.clone(), StatusKind::Info);
                        }
                    }
                }
                _ => {}
            },
            crate::lab_tools::LabToolTab::LenteCompilador => match key.code {
                KeyCode::Char('s' | 'S') => {
                    self.lab.eli10_mode = !self.lab.eli10_mode;
                    self.lab.scroll = 0;
                    app.set_status(
                        if self.lab.eli10_mode {
                            crate::i18n::bi(
                                "Modo 'Como se eu tivesse 10 anos' (ELI10) ativado!",
                                "'Explain Like I'm 10' (ELI10) mode enabled!",
                                en,
                            )
                        } else {
                            crate::i18n::bi(
                                "Modo técnico socrático ativado.",
                                "Socratic technical mode enabled.",
                                en,
                            )
                        },
                        StatusKind::Info,
                    );
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.lab.selected_fix = self.lab.selected_fix.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.lab.selected_fix = (self.lab.selected_fix + 1).min(2);
                }
                KeyCode::Enter => {
                    let lens = self.lab.last_lens.clone().unwrap_or_else(|| {
                        crate::lab_tools::SocraticErrorLens::analyze_for(&diag_text, &code, en)
                    });
                    let fix = lens.fixes[self.lab.selected_fix.min(2)];
                    self.agent.open = true;
                    self.agent.proposed_code = Some(fix.after.to_string());
                    self.lab.open = false;
                    self.focus = Focus::Agent;
                    self.agent.messages.push(AgentChatMessage {
                        role: AgentRole::Agent,
                        sender: crate::i18n::bi("Lente RustLab", "RustLab Lens", en).to_string(),
                        text: format!(
                            "{}\n{}\n```rust\n{}\n```\n{}",
                            fix.label,
                            fix.explanation,
                            fix.after,
                            crate::i18n::bi(
                                "Alt+Enter aplica no editor (Ctrl+Z desfaz).",
                                "Alt+Enter applies in editor (Ctrl+Z undoes).",
                                en
                            )
                        ),
                    });
                    self.agent.scroll = u16::MAX;
                    app.set_status(
                        crate::i18n::bi(
                            "Solução idiomática enviada ao painel do Agente.",
                            "Idiomatic fix sent to Agent panel.",
                            en,
                        ),
                        StatusKind::Success,
                    );
                }
                _ => {}
            },
            crate::lab_tools::LabToolTab::DebuggerVisual => match key.code {
                KeyCode::Char('n' | 'N') | KeyCode::F(10) => self.debugger_step_over(app),
                KeyCode::Char('s' | 'S') => self.debugger_step_into(app),
                KeyCode::Char('c' | 'C') => self.debugger_continue(app),
                KeyCode::Char('b' | 'B') => self.debugger_toggle_breakpoint_at_cursor(app),
                KeyCode::Char('g' | 'G') => self.open_native_debugger(app),
                KeyCode::Char('r' | 'R') => {
                    self.lab.debugger.start_or_rebuild(&code);
                    self.sync_editor_to_debug_step(app);
                }
                KeyCode::Char('x' | 'X') => {
                    self.lab.debugger.stop();
                    app.set_status(
                        crate::i18n::bi("Sessão de debug encerrada.", "Debug session stopped.", en),
                        StatusKind::Info,
                    );
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.lab.scroll = self.lab.scroll.saturating_sub(2);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.lab.scroll = self.lab.scroll.saturating_add(2);
                }
                _ => {}
            },
            crate::lab_tools::LabToolTab::ConserteOCodigo => {
                let quests = crate::lab_tools::troubleshooting_quests(en);
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.lab.quest_sel = self.lab.quest_sel.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let max = quests.len().saturating_sub(1);
                        self.lab.quest_sel = (self.lab.quest_sel + 1).min(max);
                    }
                    KeyCode::Enter => {
                        let quest = quests[self.lab.quest_sel.min(quests.len() - 1)];
                        self.lab.active_quest_id = Some(quest.id);
                        self.lab.last_quest_feedback = None;
                        self.apply_code_template_to_editor(
                            quest.broken_code,
                            quest.title,
                            true,
                            app,
                        );
                    }
                    KeyCode::Char('v' | 'V') => self.validate_active_quest(app),
                    _ => {}
                }
            }
            crate::lab_tools::LabToolTab::PlaygroundComunidade => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.lab.community_sel = self.lab.community_sel.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.lab.community_sel = (self.lab.community_sel + 1).min(2);
                }
                KeyCode::Char('o' | 'O') => {
                    let url = crate::lab_tools::build_playground_url(&code);
                    let msg = open_documentation_url(&url, en);
                    app.set_status(msg, StatusKind::Info);
                }
                KeyCode::Char('e' | 'E') => {
                    let dir = self
                        .project
                        .as_ref()
                        .map(|p| p.root.clone())
                        .unwrap_or_else(|| app.projects_base_dir());
                    let _ = std::fs::create_dir_all(&dir);
                    match crate::lab_tools::export_project_share_card_for(&dir, &code, en) {
                        Ok(path) => {
                            self.lab.last_export_path = Some(path.clone());
                            self.refresh_files(app);
                            app.set_status(
                                format!(
                                    "{}: {}",
                                    crate::i18n::bi("Exportado", "Exported", en),
                                    path.display()
                                ),
                                StatusKind::Success,
                            );
                        }
                        Err(e) => {
                            app.set_status(
                                format!(
                                    "{}: {e}",
                                    crate::i18n::bi("Falha ao exportar", "Export failed", en)
                                ),
                                StatusKind::Warning,
                            );
                        }
                    }
                }
                KeyCode::Char('t' | 'T') | KeyCode::Enter => {
                    let sols = crate::lab_tools::community_solutions_for_lang(&code, en);
                    let pick = sols[self.lab.community_sel.min(2)];
                    self.apply_code_template_to_editor(pick.code, pick.name, true, app);
                }
                _ => {}
            },
            crate::lab_tools::LabToolTab::EcossistemasModernos => {
                let ecos = crate::lab_tools::modern_ecosystems(en);
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.lab.ecosystem_sel = self.lab.ecosystem_sel.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let max = ecos.len().saturating_sub(1);
                        self.lab.ecosystem_sel = (self.lab.ecosystem_sel + 1).min(max);
                    }
                    KeyCode::Char('d' | 'D') => {
                        let eco = ecos[self.lab.ecosystem_sel.min(ecos.len() - 1)];
                        self.add_dependency_to_cargo(eco.dep_name, eco.dep_version, app);
                    }
                    KeyCode::Char('t' | 'T') | KeyCode::Enter => {
                        let eco = ecos[self.lab.ecosystem_sel.min(ecos.len() - 1)];
                        self.apply_code_template_to_editor(eco.template_code, eco.name, true, app);
                    }
                    KeyCode::Char('g' | 'G') => {
                        let tracks = crate::lab_tools::guided_project_tracks(en);
                        let t_idx = self
                            .lab
                            .guided_track_sel
                            .min(tracks.len().saturating_sub(1));
                        let cp_idx = self
                            .lab
                            .guided_checkpoint_sel
                            .min(tracks[t_idx].checkpoints.len().saturating_sub(1));
                        let cp = &tracks[t_idx].checkpoints[cp_idx];
                        self.apply_code_template_to_editor(cp.starter_code, cp.title, true, app);
                        self.lab.guided_feedback = Some(cp.goal.to_string());
                    }
                    KeyCode::Char('h' | 'H') => {
                        let tracks = crate::lab_tools::guided_project_tracks(en);
                        self.lab.guided_track_sel = (self.lab.guided_track_sel + 1) % tracks.len();
                        self.lab.guided_checkpoint_sel = 0;
                        self.lab.guided_feedback = None;
                    }
                    KeyCode::Char('c' | 'C') => {
                        let tracks = crate::lab_tools::guided_project_tracks(en);
                        let t_idx = self
                            .lab
                            .guided_track_sel
                            .min(tracks.len().saturating_sub(1));
                        self.lab.guided_checkpoint_sel =
                            (self.lab.guided_checkpoint_sel + 1) % tracks[t_idx].checkpoints.len();
                        self.lab.guided_feedback = None;
                    }
                    KeyCode::Char('v' | 'V') => {
                        let tracks = crate::lab_tools::guided_project_tracks(en);
                        let t_idx = self
                            .lab
                            .guided_track_sel
                            .min(tracks.len().saturating_sub(1));
                        let cp_idx = self
                            .lab
                            .guided_checkpoint_sel
                            .min(tracks[t_idx].checkpoints.len().saturating_sub(1));
                        let cp = &tracks[t_idx].checkpoints[cp_idx];
                        let (ok, fb) =
                            crate::lab_tools::validate_project_checkpoint_for(cp, &code, en);
                        self.lab.guided_feedback = Some(fb.clone());
                        if ok {
                            self.lab.completed_checkpoints.insert(cp.id);
                            app.award_xp(format!("checkpoint:{}", cp.id), cp.xp_reward, cp.title);
                            if cp_idx + 1 < tracks[t_idx].checkpoints.len() {
                                self.lab.guided_checkpoint_sel = cp_idx + 1;
                            }
                            let _ = app.persist_progress();
                            app.set_status(fb, StatusKind::Success);
                        } else {
                            app.set_status(fb, StatusKind::Warning);
                        }
                    }
                    _ => {}
                }
            }
            crate::lab_tools::LabToolTab::MapaLifetimes => match key.code {
                KeyCode::Char('l' | 'L') => {
                    self.lab.lifetime_gutter_enabled = !self.lab.lifetime_gutter_enabled;
                    app.set_status(
                        if self.lab.lifetime_gutter_enabled {
                            crate::i18n::bi(
                                "Marcadores de lifetime (┣/┃/✘) ativados na margem do editor.",
                                "Lifetime gutter markers (┣/┃/✘) enabled in editor.",
                                en,
                            )
                        } else {
                            crate::i18n::bi(
                                "Marcadores de lifetime na margem desativados.",
                                "Lifetime gutter markers disabled.",
                                en,
                            )
                        },
                        StatusKind::Info,
                    );
                }
                KeyCode::Char('r' | 'R') => {
                    self.lab.refresh_for(&code, &diag_text, en);
                    app.set_status(
                        crate::i18n::bi(
                            "Mapa de Lifetimes recalculado.",
                            "Lifetime Map recalculated.",
                            en,
                        ),
                        StatusKind::Info,
                    );
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.lab.scroll = self.lab.scroll.saturating_sub(2);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.lab.scroll = self.lab.scroll.saturating_add(2);
                }
                _ => {}
            },
            crate::lab_tools::LabToolTab::LayoutMemoria => match key.code {
                KeyCode::Char('r' | 'R') => {
                    self.lab.refresh_for(&code, &diag_text, en);
                    app.set_status(
                        crate::i18n::bi(
                            "Layout de Memória recalculado.",
                            "Memory Layout recalculated.",
                            en,
                        ),
                        StatusKind::Info,
                    );
                }
                KeyCode::Char('v' | 'V') => {
                    let mut rep = self.lab.last_layout.take().unwrap_or_else(|| {
                        crate::lab_tools::MemoryLayoutReport::analyze_for(&code, en)
                    });
                    let ok = rep.verify_struct_layouts_live(&code);
                    self.lab.last_layout = Some(rep);
                    app.set_status(
                        if ok {
                            crate::i18n::bi(
                                "Tamanhos verificados em tempo real via rustc (size_of).",
                                "Struct sizes live-verified via rustc (size_of).",
                                en,
                            )
                        } else {
                            crate::i18n::bi(
                                "Usando cálculo estático offline (defina uma struct simples ou verifique se rustc está no PATH).",
                                "Using offline static calculation (define a simple struct or ensure rustc is in PATH).",
                                en,
                            )
                        },
                        StatusKind::Info,
                    );
                }
                KeyCode::Char('o' | 'O') | KeyCode::Enter => {
                    let report = self.lab.last_layout.clone().unwrap_or_else(|| {
                        crate::lab_tools::MemoryLayoutReport::analyze_for(&code, en)
                    });
                    if let Some(st) = report.user_types.first() {
                        self.agent.open = true;
                        self.agent.proposed_code = Some(st.reordered_preview.clone());
                        self.lab.open = false;
                        self.focus = Focus::Agent;
                        self.agent.messages.push(AgentChatMessage {
                            role: AgentRole::Agent,
                            sender: "Layout X-Ray".to_string(),
                            text: if en {
                                format!(
                                    "Descending-alignment field ordering for `{}` (saves {} bytes in repr(C)):\n```rust\n{}\n```",
                                    st.name, st.bytes_saved, st.reordered_preview
                                )
                            } else {
                                format!(
                                    "Sugestão de ordenação por alinhamento decrescente para `{}` (economiza {} bytes em repr(C)):\n```rust\n{}\n```",
                                    st.name, st.bytes_saved, st.reordered_preview
                                )
                            },
                        });
                        self.agent.scroll = u16::MAX;
                        app.set_status(
                            crate::i18n::bi(
                                "Ordem otimizada da struct enviada ao painel do Agente.",
                                "Optimized struct field order sent to Agent panel.",
                                en,
                            ),
                            StatusKind::Success,
                        );
                    } else {
                        app.set_status(
                            crate::i18n::bi(
                                "Defina uma `struct Nome { ... }` no editor para sugerir reordenação.",
                                "Define a `struct Name { ... }` in the editor to suggest reordering.",
                                en,
                            ),
                            StatusKind::Info,
                        );
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.lab.scroll = self.lab.scroll.saturating_sub(2);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.lab.scroll = self.lab.scroll.saturating_add(2);
                }
                _ => {}
            },
            crate::lab_tools::LabToolTab::DesacucararMacros => {
                let report = self
                    .lab
                    .last_desugar
                    .clone()
                    .unwrap_or_else(|| crate::lab_tools::DesugarReport::analyze_for(&code, en));
                let max = report.items.len().saturating_sub(1);
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.lab.desugar_sel = self.lab.desugar_sel.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.lab.desugar_sel = (self.lab.desugar_sel + 1).min(max);
                    }
                    KeyCode::Char('e' | 'E') => {
                        let expanded = crate::lab_tools::try_cargo_expand_snippet(
                            self.project.as_ref().map(|p| p.root.as_path()),
                            &code,
                            en,
                        );
                        let mut rep = report.clone();
                        rep.live_expanded = expanded.clone();
                        self.lab.last_desugar = Some(rep);
                        app.set_status(
                            if expanded.is_some() {
                                crate::i18n::bi(
                                    "Expansão real carregada via cargo-expand.",
                                    "Live macro expansion loaded via cargo-expand.",
                                    en,
                                )
                            } else {
                                crate::i18n::bi(
                                    "cargo-expand não encontrado; mantendo expansão didática offline.",
                                    "cargo-expand not found; keeping offline educational expansion.",
                                    en,
                                )
                            },
                            StatusKind::Info,
                        );
                    }
                    KeyCode::Enter => {
                        if let Some(item) = report.items.get(self.lab.desugar_sel.min(max)) {
                            self.agent.open = true;
                            self.lab.open = false;
                            self.focus = Focus::Agent;
                            self.agent.messages.push(AgentChatMessage {
                                role: AgentRole::Agent,
                                sender: crate::i18n::bi(
                                    "Desaçucarador RustLab",
                                    "RustLab Desugar",
                                    en,
                                )
                                .to_string(),
                                text: if en {
                                    format!(
                                        "{}\nOriginal: `{}`\nCompiler expansion:\n```rust\n{}\n```\n{}",
                                        item.title,
                                        item.sugar_snippet,
                                        item.expanded_rust,
                                        item.explanation
                                    )
                                } else {
                                    format!(
                                        "{}\nOriginal: `{}`\nExpansão do compilador:\n```rust\n{}\n```\n{}",
                                        item.title,
                                        item.sugar_snippet,
                                        item.expanded_rust,
                                        item.explanation
                                    )
                                },
                            });
                            self.agent.scroll = u16::MAX;
                            app.set_status(
                                crate::i18n::bi(
                                    "Expansão da macro enviada ao Agente.",
                                    "Macro expansion sent to Agent.",
                                    en,
                                ),
                                StatusKind::Info,
                            );
                        }
                    }
                    _ => {}
                }
            }
            crate::lab_tools::LabToolTab::PoliglotaIntuicao => match key.code {
                KeyCode::Char('t' | 'T' | ' ') => {
                    self.lab.polyglot_mode_flashcards = !self.lab.polyglot_mode_flashcards;
                    self.lab.scroll = 0;
                    app.set_status(
                        if self.lab.polyglot_mode_flashcards {
                            crate::i18n::bi(
                                "Modo Micro-Treino Flashcards ativado (C = Compila · E = Erro).",
                                "Micro-Drill Flashcards mode enabled (C = Compiles · E = Error).",
                                en,
                            )
                        } else {
                            crate::i18n::bi(
                                "Modo Dicionário Poliglota ativado (Python/JS/C++ -> Rust).",
                                "Polyglot Dictionary mode enabled (Python/JS/C++ -> Rust).",
                                en,
                            )
                        },
                        StatusKind::Info,
                    );
                }
                KeyCode::Char('f' | 'F') if !self.lab.polyglot_mode_flashcards => {
                    self.lab.polyglot_lang_filter = (self.lab.polyglot_lang_filter + 1) % 4;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.lab.polyglot_mode_flashcards {
                        self.lab.flashcard_sel = self.lab.flashcard_sel.saturating_sub(1);
                        self.lab.flashcard_feedback = None;
                    } else {
                        self.lab.polyglot_sel = self.lab.polyglot_sel.saturating_sub(1);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.lab.polyglot_mode_flashcards {
                        let max = crate::lab_tools::INTUITION_FLASHCARDS
                            .len()
                            .saturating_sub(1);
                        self.lab.flashcard_sel = (self.lab.flashcard_sel + 1).min(max);
                        self.lab.flashcard_feedback = None;
                    } else {
                        let dict = crate::lab_tools::polyglot_dictionary(en);
                        let max = dict.len().saturating_sub(1);
                        self.lab.polyglot_sel = (self.lab.polyglot_sel + 1).min(max);
                    }
                }
                KeyCode::Enter if !self.lab.polyglot_mode_flashcards => {
                    let dict = crate::lab_tools::polyglot_dictionary(en);
                    let entry = dict[self.lab.polyglot_sel.min(dict.len() - 1)];
                    self.apply_code_template_to_editor(entry.rust_code, entry.concept, true, app);
                }
                KeyCode::Char('c' | 'C' | 'e' | 'E') if self.lab.polyglot_mode_flashcards => {
                    let user_compiles = matches!(key.code, KeyCode::Char('c' | 'C'));
                    let card = crate::lab_tools::intuition_flashcard(self.lab.flashcard_sel, en);
                    let (ok, msg) =
                        crate::lab_tools::evaluate_flashcard_answer_for(&card, user_compiles, en);
                    if ok {
                        self.lab.completed_flashcards.insert(card.id);
                        let _ = app.award_xp(format!("flashcard:{}", card.id), 25, card.title);
                        app.progress.record_srs_mastery(card.id, true);
                        let _ = app.persist_progress();
                    }
                    self.lab.flashcard_feedback = Some((ok, msg.clone()));
                    app.set_status(
                        msg,
                        if ok {
                            StatusKind::Success
                        } else {
                            StatusKind::Warning
                        },
                    );
                }
                _ => {}
            },
        }
    }
}

fn grant_project_milestone(
    app: &mut App,
    key: &str,
    xp: u32,
    label: &str,
    changed: &mut bool,
    awarded: &mut bool,
) {
    if app.progress.project_milestones.insert(key.to_string()) {
        *changed = true;
    }
    if app.award_xp(format!("project:{key}"), xp, label) {
        *awarded = true;
    }
}

fn maybe_award_independent_capstone(app: &mut App, changed: &mut bool, awarded: &mut bool) {
    const REQUIRED: [&str; 5] = ["plan", "build", "clippy", "tests", "run"];
    if REQUIRED
        .iter()
        .all(|key| app.progress.project_milestones.contains(*key))
    {
        if app
            .progress
            .project_milestones
            .insert("independent-complete".to_string())
        {
            *changed = true;
        }
        let label = crate::i18n::bi(
            "Projeto independente (Etapa 10) concluído",
            "Independent project (Stage 10) completed",
            app.is_english(),
        );
        if app.award_xp("project:independent-complete".into(), 50, label) {
            *awarded = true;
        }
    }
}

fn secs_of(label: &str) -> u64 {
    match label {
        "cargo run" => 30,
        "cargo test" => 120,
        _ => 180,
    }
}

pub fn count_diags(diags: &[Diagnostic]) -> (usize, usize) {
    (
        diags.iter().filter(|d| d.level == DiagLevel::Error).count(),
        diags
            .iter()
            .filter(|d| d.level == DiagLevel::Warning)
            .count(),
    )
}

fn rect_contains(r: ratatui::layout::Rect, x: u16, y: u16) -> bool {
    r.width > 0
        && r.height > 0
        && x >= r.x
        && y >= r.y
        && x < r.x.saturating_add(r.width)
        && y < r.y.saturating_add(r.height)
}

fn package_binary_name(project: &Project) -> String {
    let Ok(toml) = std::fs::read_to_string(project.root.join("Cargo.toml")) else {
        return project.name().to_string();
    };
    let mut in_package = false;
    for raw_line in toml.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_package = line == "[package]";
            continue;
        }
        if in_package
            && let Some(rest) = line.strip_prefix("name")
            && let Some(val) = rest.trim_start().strip_prefix('=')
        {
            let cleaned = val.trim().trim_matches('"').trim_matches('\'');
            if !cleaned.is_empty() {
                return cleaned.to_string();
            }
        }
    }
    project.name().to_string()
}

fn spawn_external_app(
    bin_path: &std::path::Path,
    workdir: &std::path::Path,
    en: bool,
) -> Result<String, String> {
    if cfg!(test) || std::env::var_os("RUSTLAB_DISABLE_EXTERNAL").is_some() {
        return Ok(if en {
            format!("External window ready to launch: {}", bin_path.display())
        } else {
            format!(
                "Janela externa pronta para executar: {}",
                bin_path.display()
            )
        });
    }
    if !bin_path.is_file() {
        return Err(if en {
            format!(
                "Binary not found at {} (verify the project produces an executable)",
                bin_path.display()
            )
        } else {
            format!(
                "Binário não encontrado em {} (verifique se o projeto gera executável)",
                bin_path.display()
            )
        });
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    if std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return Err(if en {
            format!(
                "No active graphical server (DISPLAY/WAYLAND). Run in terminal: {}",
                bin_path.display()
            )
        } else {
            format!(
                "Sem servidor gráfico (DISPLAY/WAYLAND) ativo. Execute no terminal: {}",
                bin_path.display()
            )
        });
    }
    std::process::Command::new(bin_path)
        .current_dir(workdir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| {
            if en {
                format!("App opened in separate window: {}", bin_path.display())
            } else {
                format!("App aberto em janela separada: {}", bin_path.display())
            }
        })
        .map_err(|e| {
            if en {
                format!("Failed to open external window: {e}")
            } else {
                format!("Falha ao abrir janela externa: {e}")
            }
        })
}

fn spawn_external_cargo_terminal(workdir: &std::path::Path, en: bool) -> Result<String, String> {
    spawn_external_terminal_command(workdir, "cargo", &["run".to_string()], en)
}

fn spawn_external_terminal_command(
    workdir: &std::path::Path,
    program: &str,
    cmd_args: &[String],
    en: bool,
) -> Result<String, String> {
    let preview = if cmd_args.is_empty() {
        program.to_string()
    } else {
        format!("{program} {}", cmd_args.join(" "))
    };
    if cfg!(test) || std::env::var_os("RUSTLAB_DISABLE_EXTERNAL").is_some() {
        return Ok(if en {
            format!(
                "External terminal ready for `{preview}` in {}",
                workdir.display()
            )
        } else {
            format!(
                "Terminal externo pronto para `{preview}` em {}",
                workdir.display()
            )
        });
    }
    #[cfg(target_os = "windows")]
    {
        return std::process::Command::new("cmd")
            .args(["/C", "start", program])
            .args(cmd_args)
            .current_dir(workdir)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map(|_| {
                if en {
                    format!(
                        "External terminal started with `{preview}` in {}",
                        workdir.display()
                    )
                } else {
                    format!(
                        "Terminal externo iniciado com `{preview}` em {}",
                        workdir.display()
                    )
                }
            })
            .map_err(|_| {
                if en {
                    format!(
                        "Open another terminal and run: cd \"{}\" && {preview}",
                        workdir.display()
                    )
                } else {
                    format!(
                        "Abra outro terminal e execute: cd \"{}\" && {preview}",
                        workdir.display()
                    )
                }
            });
    }
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "tell application \"Terminal\" to do script \"cd '{}' && {}\"",
            workdir.display().to_string().replace('\'', "'\\''"),
            preview.replace('"', "\\\"")
        );
        return std::process::Command::new("osascript")
            .args(["-e", &script])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map(|_| {
                if en {
                    format!(
                        "Terminal.app started with `{preview}` in {}",
                        workdir.display()
                    )
                } else {
                    format!(
                        "Terminal.app iniciado com `{preview}` em {}",
                        workdir.display()
                    )
                }
            })
            .map_err(|_| {
                if en {
                    format!(
                        "Open another terminal and run: cd \"{}\" && {preview}",
                        workdir.display()
                    )
                } else {
                    format!(
                        "Abra outro terminal e execute: cd \"{}\" && {preview}",
                        workdir.display()
                    )
                }
            });
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none() {
            return Err(if en {
                format!(
                    "No active graphical server. Open another terminal and run: cd \"{}\" && {preview}",
                    workdir.display()
                )
            } else {
                format!(
                    "Sem servidor gráfico ativo. Abra outro terminal paralelo e rode: cd \"{}\" && {preview}",
                    workdir.display()
                )
            });
        }
        const TERMINALS: &[(&str, &[&str])] = &[
            ("x-terminal-emulator", &["-e"]),
            ("gnome-terminal", &["--"]),
            ("konsole", &["-e"]),
            ("xfce4-terminal", &["-x"]),
            ("kitty", &[]),
            ("alacritty", &["-e"]),
            ("wezterm", &["start", "--"]),
            ("foot", &[]),
            ("xterm", &["-e"]),
        ];
        for (term, prefix) in TERMINALS {
            if std::process::Command::new(term)
                .args(*prefix)
                .arg(program)
                .args(cmd_args)
                .current_dir(workdir)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .is_ok()
            {
                return Ok(if en {
                    format!(
                        "`{preview}` launched in terminal `{term}` at {}",
                        workdir.display()
                    )
                } else {
                    format!(
                        "`{preview}` iniciado no terminal `{term}` em {}",
                        workdir.display()
                    )
                });
            }
        }
        Err(if en {
            format!(
                "No terminal emulator found. Open another terminal and run: cd \"{}\" && {preview}",
                workdir.display()
            )
        } else {
            format!(
                "Nenhum emulador gráfico encontrado. Abra outro terminal e execute: cd \"{}\" && {preview}",
                workdir.display()
            )
        })
    }
}

fn open_documentation_url(url: &str, en: bool) -> String {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return if en {
            format!("Invalid documentation URL: {url}")
        } else {
            format!("URL de documentação inválida: {url}")
        };
    }
    if cfg!(test) || std::env::var_os("RUSTLAB_DISABLE_BROWSER").is_some() {
        return if en {
            format!("Documentation ready: {url}")
        } else {
            format!("Documentação pronta: {url}")
        };
    }
    #[cfg(target_os = "macos")]
    let res = std::process::Command::new("open")
        .arg(url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    #[cfg(target_os = "windows")]
    let res = std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let res = std::process::Command::new("xdg-open")
        .arg(url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    match res {
        Ok(_) => {
            if en {
                format!("Documentation opened in browser: {url}")
            } else {
                format!("Documentação aberta no navegador: {url}")
            }
        }
        Err(_) => {
            if en {
                format!("Open in browser: {url}")
            } else {
                format!("Abra no navegador: {url}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyEvent;

    #[test]
    fn autocomplete_por_teclado_aceita_e_escape_nao_sai_do_editor() {
        let mut app = crate::app::App::for_snapshot();
        let mut state = CreateState {
            view: CreateView::Workspace,
            ..Default::default()
        };
        for ch in "pri".chars() {
            state.on_key(&mut app, KeyEvent::from(KeyCode::Char(ch)));
        }
        assert!(state.completion.items.iter().any(|s| s == "println!"));
        state.on_key(&mut app, KeyEvent::from(KeyCode::Esc));
        assert_eq!(state.focus, Focus::Editor);
        assert!(state.completion.items.is_empty());
        state.on_key(
            &mut app,
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL),
        );
        state.on_key(&mut app, KeyEvent::from(KeyCode::Enter));
        assert_eq!(state.editor.buf.to_string_lossy(), "print!");
    }

    #[test]
    fn spawn_failed_mostra_motivo_no_estado() {
        let mut app = crate::app::App::for_snapshot();
        let mut state = CreateState::default();
        state.handle_exec_event(
            ExecEvent::SpawnFailed {
                label: "cargo new".into(),
                reason: "teste".into(),
            },
            &mut app,
        );
        assert_eq!(
            state.last_failure.as_deref(),
            Some("cargo new não pôde iniciar: teste")
        );
    }

    #[test]
    fn nome_reservado_cancela_antes_de_executar_cargo() {
        let mut app = crate::app::App::for_snapshot();
        let mut state = CreateState {
            prompt: Some(PromptState {
                kind: PromptKind::NewProject,
                text: "test".into(),
                error: None,
            }),
            ..CreateState::default()
        };
        state.on_key(&mut app, KeyEvent::from(KeyCode::Enter));
        assert!(state.prompt.is_some());
        assert!(
            state
                .prompt
                .as_ref()
                .unwrap()
                .error
                .as_deref()
                .unwrap()
                .contains("reservado pelo Rust")
        );
        assert!(!app.exec.running);
        assert_eq!(state.after_exec, AfterExec::None);
    }

    #[test]
    fn hover_em_println_encontra_documentacao() {
        let mut state = CreateState {
            view: CreateView::Workspace,
            editor: Editor::new("println!(\"oi\");", None),
            focus: Focus::Editor,
            editor_text_area: ratatui::layout::Rect {
                x: 10,
                y: 5,
                width: 40,
                height: 5,
            },
            ..CreateState::default()
        };
        // A coluna 3, após moldura e gutter, cai dentro de `println`.
        state.on_mouse_move(16, 5);
        assert_eq!(
            state.hovered_doc.as_ref().map(|x| x.0.as_str()),
            Some("println")
        );
    }

    #[test]
    fn falha_de_salvamento_bloqueia_troca_de_arquivo_projeto_e_saida() {
        let mut app = App::for_snapshot();
        let mut state = CreateState {
            view: CreateView::Workspace,
            ..Default::default()
        };
        let root = std::env::temp_dir().join(format!("rustlab-save-block-{}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='test_save'\nversion='0.1.0'\n",
        )
        .unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
        app.set_config_path(root.join("config.toml"));
        state.open_project(&root, &mut app);
        // Remover o arquivo e criar um diretório no mesmo caminho simula erro de escrita.
        let file = root.join("src/main.rs");
        std::fs::remove_file(&file).unwrap();
        std::fs::create_dir(&file).unwrap();
        state.editor.insert_text("// não perder\n");
        let code = state.editor.buf.to_string_lossy();
        state.switch_file(0, &mut app);
        assert!(state.editor.dirty);
        assert_eq!(state.editor.buf.to_string_lossy(), code);
        state.open_project(&root, &mut app);
        assert_eq!(state.editor.buf.to_string_lossy(), code);
        assert!(!state.save_before_exit(&mut app));
        assert_eq!(app.config.workspace.recent_projects.len(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn selecionar_arquivo_nao_muda_caminho_aberto_ate_enter() {
        let mut app = App::for_snapshot();
        let mut state = CreateState {
            view: CreateView::Workspace,
            focus: Focus::Files,
            files: vec!["Cargo.toml".into(), "src/main.rs".into()],
            open_idx: 1,
            selected_file: 1,
            editor: Editor::new("fn main(){}", Some("src/main.rs".into())),
            ..Default::default()
        };
        state.on_key(&mut app, KeyEvent::from(KeyCode::Up));
        assert_eq!(state.selected_file, 0);
        assert_eq!(state.open_idx, 1);
        assert_eq!(state.editor.path, Some("src/main.rs".into()));
        state.on_key(&mut app, KeyEvent::from(KeyCode::BackTab));
        assert_eq!(state.focus, Focus::Console);
    }

    #[test]
    fn diagnostico_abre_arquivo_e_formatacao_nao_descarta_edicao_nova() {
        let root = std::env::temp_dir().join(format!("rustlab-jump-{}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='jump'\nversion='0.1.0'\n",
        )
        .unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {\n    erro();\n}\n").unwrap();
        let mut app = App::for_snapshot();
        app.set_config_path(root.join("config.toml"));
        let mut state = CreateState::new();
        state.open_project(&root, &mut app);
        state.switch_file(0, &mut app);
        state.diags = vec![Diagnostic {
            level: DiagLevel::Error,
            message: "não encontrado".into(),
            file: Some("src/main.rs".into()),
            line: Some(2),
            col: Some(5),
        }];
        state.on_key(&mut app, KeyEvent::from(KeyCode::F(12)));
        assert_eq!(state.editor.path, Some(root.join("src/main.rs")));
        assert_eq!(state.cursor_line_col(), (2, 5));
        state.after_exec_reload = true;
        state.editor.insert_text("// resposta ainda não salva");
        let code = state.editor.buf.to_string_lossy();
        state.handle_exec_event(
            ExecEvent::Finished {
                label: "cargo fmt".into(),
                success: true,
                code: Some(0),
            },
            &mut app,
        );
        assert!(state.editor.dirty);
        assert_eq!(state.editor.buf.to_string_lossy(), code);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn f11_vai_para_definicao_f1_mostra_doc_e_ctrl_g_busca_no_projeto() {
        let root = std::env::temp_dir().join(format!("rustlab-ide-p7-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='idep7'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        std::fs::write(
            root.join("src/main.rs"),
            "mod ajuda;\nfn main() {\n    let x = ajuda::saudar();\n}\n",
        )
        .unwrap();
        std::fs::write(
            root.join("src/ajuda.rs"),
            "/// Produz uma saudação acolhedora.\npub fn saudar() -> &'static str {\n    \"oi\"\n}\n",
        )
        .unwrap();

        let mut app = App::for_snapshot();
        app.set_config_path(root.join("config.toml"));
        let mut state = CreateState::new();
        state.open_project(&root, &mut app);
        assert_eq!(state.editor.path, Some(root.join("src/main.rs")));

        // Cursor sobre `saudar` em src/main.rs (linha 2, col 21)
        state.editor.cursor.line = 2;
        state.editor.cursor.col = 21;

        // F1 abre documentação local do símbolo do workspace
        state.on_key(&mut app, KeyEvent::from(KeyCode::F(1)));
        let doc = state.doc_popup.as_ref().expect("F1 deve abrir doc_popup");
        assert_eq!(doc.title, "saudar");
        assert!(doc.body.contains("saudação acolhedora"));

        // F1 novamente fecha o popup
        state.on_key(&mut app, KeyEvent::from(KeyCode::F(1)));
        assert!(state.doc_popup.is_none());

        // F11 salta para a definição em src/ajuda.rs
        state.on_key(&mut app, KeyEvent::from(KeyCode::F(11)));
        assert_eq!(state.editor.path, Some(root.join("src/ajuda.rs")));
        assert_eq!(state.cursor_line_col().0, 2);

        // Ctrl+G abre busca global e encontra nos 2 arquivos
        state.run_project_search("saudar", &mut app);
        assert_eq!(state.project_matches.len(), 2);

        // Marcador de diagnóstico por linha no editor aberto
        state.diags.push(Diagnostic {
            level: DiagLevel::Error,
            message: "erro de teste".into(),
            file: Some("src/ajuda.rs".into()),
            line: Some(2),
            col: Some(1),
        });
        assert_eq!(
            state.diagnostic_level_for_open_line(2),
            Some(DiagLevel::Error)
        );
        assert_eq!(state.diagnostic_level_for_open_line(1), None);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn alt_m_cria_plano_e_execucao_limpa_registra_marcos_etapa_10() {
        let root = std::env::temp_dir().join(format!("rustlab-etapa10-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='etapa10'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        std::fs::write(
            root.join("src/main.rs"),
            "/// Entrada.\nmod dados;\nfn main() -> Result<(), String> { Ok(()) }\n",
        )
        .unwrap();
        std::fs::write(
            root.join("src/dados.rs"),
            "pub struct Item { pub id: u32 }\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn ok() {}\n}\n",
        )
        .unwrap();

        let mut app = App::for_snapshot();
        app.set_config_path(root.join("config.toml"));
        app.set_storage_path(root.join("progress.toml"));
        let mut state = CreateState::new();
        state.open_project(&root, &mut app);

        // Alt+M cria PLANO.md, abre o arquivo e registra marco `plan`.
        state.on_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::ALT),
        );
        assert!(root.join("PLANO.md").exists());
        assert_eq!(state.editor.path, Some(root.join("PLANO.md")));
        assert!(app.progress.project_milestones.contains("plan"));

        // Execução bem-sucedida de cargo test registra capacidades + testes.
        state.handle_exec_event(
            ExecEvent::Finished {
                label: "cargo test".into(),
                success: true,
                code: Some(0),
            },
            &mut app,
        );
        for expected in [
            "plan",
            "multi-module",
            "types",
            "error-handling",
            "docs",
            "tests",
        ] {
            assert!(
                app.progress.project_milestones.contains(expected),
                "marco esperado ausente: {expected}"
            );
        }

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn agente_lateral_alterna_responde_e_aplica_com_undo() {
        let root = std::env::temp_dir().join(format!("rustlab-agent-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='agentdemo'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        std::fs::write(
            root.join("src/main.rs"),
            "fn soma(a: i32, b: i32) -> i32 { a + b }\nfn main() {}\n",
        )
        .unwrap();

        let mut app = App::for_snapshot();
        app.set_config_path(root.join("config.toml"));
        app.set_storage_path(root.join("progress.toml"));
        let mut state = CreateState::new();
        state.open_project(&root, &mut app);
        let original = state.editor.buf.to_string_lossy();

        // Alt+A abre o chat do agente na direita e foca o painel.
        state.on_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT),
        );
        assert!(state.agent.open);
        assert_eq!(state.focus, Focus::Agent);

        // Envia /testes no guia local determinístico -> gera bloco ```rust``` aplicável.
        for ch in "/testes".chars() {
            state.on_key(&mut app, KeyEvent::from(KeyCode::Char(ch)));
        }
        state.on_key(&mut app, KeyEvent::from(KeyCode::Enter));
        assert!(state.agent.proposed_code.is_some());

        // Alt+Enter aplica no editor e devolve foco; Ctrl+Z desfaz.
        state.on_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
        assert_eq!(state.focus, Focus::Editor);
        assert!(state.editor.buf.to_string_lossy().contains("#[cfg(test)]"));
        state.on_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL),
        );
        assert_eq!(state.editor.buf.to_string_lossy(), original);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn novo_projeto_aceita_pasta_diferente_e_define_pasta_padrao() {
        let root = std::env::temp_dir().join(format!("rustlab-dirs-{}", std::process::id()));
        let custom_dir = root.join("meus_apps");
        let _ = std::fs::remove_dir_all(&root);

        let mut app = App::for_snapshot();
        app.set_config_path(root.join("config.toml"));
        let mut state = CreateState::new();

        // Define pasta padrão pelo prompt ('d' na tela inicial).
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('d')));
        assert_eq!(
            state.prompt.as_ref().map(|p| p.kind),
            Some(PromptKind::SetProjectsDir)
        );
        state.prompt.as_mut().unwrap().text = custom_dir.display().to_string();
        state.on_key(&mut app, KeyEvent::from(KeyCode::Enter));
        assert!(custom_dir.is_dir());
        assert_eq!(app.projects_base_dir(), custom_dir);

        // Prompt de novo projeto aceita outra subpasta específica (`cli/calc`) sem rodar cargo aqui.
        state.prompt = Some(PromptState {
            kind: PromptKind::NewProject,
            text: "cli/calc".into(),
            error: None,
        });
        state.on_key(&mut app, KeyEvent::from(KeyCode::Enter));
        assert_eq!(state.pending_new_project_dir, Some(custom_dir.join("cli")));
        assert_eq!(
            state.consent_request,
            Some(ToolCommand::NewProject {
                name: "calc".into()
            })
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn esc_no_workspace_fecha_projeto_e_volta_para_tela_inicial() {
        let root = std::env::temp_dir().join(format!("rustlab-esc-close-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='escclose'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();

        let mut app = App::for_snapshot();
        app.set_config_path(root.join("config.toml"));
        app.screen = Screen::Create;
        let mut state = CreateState::new();
        state.open_project(&root, &mut app);
        assert_eq!(state.view, CreateView::Workspace);
        assert!(state.project.is_some());

        // Esc fecha o projeto atual e leva para a tela inicial do Modo Criar.
        state.on_key(&mut app, KeyEvent::from(KeyCode::Esc));
        assert_eq!(state.view, CreateView::Start);
        assert!(state.project.is_none());
        assert_eq!(app.screen, Screen::Create);

        // Esc na tela inicial volta ao menu principal.
        state.on_key(&mut app, KeyEvent::from(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Menu);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn menu_bibliotecas_insere_no_cargo_toml_e_topo_e_clique_abre_doc() {
        let root = std::env::temp_dir().join(format!("rustlab-libs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='meujogo'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        std::fs::write(
            root.join("src/main.rs"),
            "fn main() {\n    let pontos = 10;\n    println!(\"{pontos}\");\n}\n",
        )
        .unwrap();

        let mut app = App::for_snapshot();
        app.set_config_path(root.join("config.toml"));
        app.set_storage_path(root.join("progress.toml"));
        app.screen = Screen::Create;
        let mut state = CreateState::new();
        state.open_project(&root, &mut app);

        // F9 abre menu central de bibliotecas; Enter instala a primeira (macroquad).
        state.on_key(&mut app, KeyEvent::from(KeyCode::F(9)));
        assert!(state.lib_menu_open);
        state.on_key(&mut app, KeyEvent::from(KeyCode::Enter));
        assert!(!state.lib_menu_open);

        let cargo_disk = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
        assert!(cargo_disk.contains("macroquad = \"0.4\""));
        let code_now = state.editor.buf.to_string_lossy();
        assert!(code_now.starts_with("use macroquad::prelude::*;"));
        assert!(code_now.contains("let pontos = 10;"));

        // Clicar na variável local `pontos` abre popup com doc em português.
        state.editor_text_area = ratatui::layout::Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 20,
        };
        let gutter = (state.editor.buf.line_count().to_string().len().max(2) + 1) as u16;
        let (line_idx, col_idx) = code_now
            .lines()
            .enumerate()
            .find_map(|(i, l)| l.find("pontos").map(|c| (i as u16, c as u16)))
            .unwrap();
        state.on_mouse_down(gutter + col_idx, line_idx, &mut app);
        let popup = state.doc_popup.as_ref().expect("popup de clique esperado");
        assert!(popup.title.contains("pontos"));
        assert!(popup.body.contains("imutável"));
        assert!(popup.url.as_deref().unwrap().contains("rust-book-pt-br"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn revisao_periodica_dispara_sem_premiar_codigo_da_ia() {
        let root = std::env::temp_dir().join(format!("rustlab-review-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='revapp'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();

        let mut app = App::for_snapshot();
        app.set_config_path(root.join("config.toml"));
        app.set_storage_path(root.join("progress.toml"));
        app.config.ai.provider = AiProvider::Disabled;
        app.screen = Screen::Create;
        let mut state = CreateState::new();
        state.open_project(&root, &mut app);

        // Quando o código inteiro vem da IA, não há elogio indevido.
        state.agent.proposed_code = Some("fn bloco_ia() -> i32 {\n    42\n}".to_string());
        state.apply_agent_code(&mut app);
        assert!(state.agent.ai_authored_lines.contains("42"));
        assert!(!state.agent.student_modified_since_review);

        state.trigger_periodic_code_review(&mut app, true);
        let last_sys = state.agent.messages.last().unwrap();
        assert!(last_sys.text.contains("veio do agente"));

        // Quando o estudante digita código próprio e passa o intervalo (15-30 min),
        // poll_agent dispara a revisão periódica automaticamente.
        state.editor.cursor = crate::editor::Pos { line: 2, col: 0 };
        state.on_key(&mut app, KeyEvent::from(KeyCode::Enter));
        for ch in "fn soma_aluno() { let total = 3; }".chars() {
            state.on_key(&mut app, KeyEvent::from(KeyCode::Char(ch)));
        }
        assert!(state.agent.student_modified_since_review);
        state.agent.last_auto_review =
            std::time::Instant::now() - std::time::Duration::from_secs(1900);
        assert!(state.poll_agent(&mut app));
        assert!(!state.agent.student_modified_since_review);
        assert!(state.agent.open);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn bevy_suporte_completo_menu_template_e_terminal_externo() {
        let root = std::env::temp_dir().join(format!("rustlab-bevy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='jogobevy'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();

        let mut app = App::for_snapshot();
        app.set_config_path(root.join("config.toml"));
        app.set_storage_path(root.join("progress.toml"));
        app.screen = Screen::Create;
        let mut state = CreateState::new();
        state.open_project(&root, &mut app);

        // F9 -> Down (bevy) -> T (template inicial 2D ECS + Cargo.toml com [profile.dev]).
        state.on_key(&mut app, KeyEvent::from(KeyCode::F(9)));
        state.on_key(&mut app, KeyEvent::from(KeyCode::Down));
        assert_eq!(
            crate::doc_hint::EXTRA_LIBRARIES[state.lib_menu_sel].name,
            "bevy"
        );
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('t')));
        assert!(!state.lib_menu_open);

        let cargo_disk = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
        assert!(cargo_disk.contains("bevy = \"0.20\""));
        assert!(cargo_disk.contains("[profile.dev]"));
        let code_now = state.editor.buf.to_string_lossy();
        assert!(code_now.contains("use bevy::prelude::*;"));
        assert!(code_now.contains("App::new()"));
        assert!(code_now.contains("DefaultPlugins"));
        assert!(code_now.contains("Camera2d"));

        // F4 -> Down (opção 1: outro terminal) registra o comando externo sem erros.
        state.on_key(&mut app, KeyEvent::from(KeyCode::F(4)));
        assert!(state.build_menu_open);
        state.on_key(&mut app, KeyEvent::from(KeyCode::Down));
        state.on_key(&mut app, KeyEvent::from(KeyCode::Enter));
        assert!(!state.build_menu_open);
        assert!(
            state
                .console
                .lines
                .iter()
                .any(|l| l.text.contains("cargo run"))
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn hub_laboratorio_integra_raiox_lente_debugger_quests_playground_e_ecossistemas() {
        let root = std::env::temp_dir().join(format!("rustlab-hub-lab-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='hublab'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        std::fs::write(
            root.join("src/main.rs"),
            "fn main() {\n    let mut n = 10;\n    n += 5;\n    let s = String::from(\"oi\");\n    println!(\"{n} {s}\");\n}\n",
        )
        .unwrap();

        let mut app = App::for_snapshot();
        app.set_config_path(root.join("config.toml"));
        app.set_storage_path(root.join("progress.toml"));
        app.screen = Screen::Create;
        let mut state = CreateState::new();
        state.open_project(&root, &mut app);

        // 1. Alt+X abre Raio-X de Memória e calcula Stack/Heap
        state.on_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT),
        );
        assert!(state.lab.open);
        assert_eq!(state.lab.tab, crate::lab_tools::LabToolTab::RaioXMemoria);
        assert!(
            state
                .lab
                .last_xray
                .as_ref()
                .unwrap()
                .metrics
                .heap_allocations
                >= 1
        );

        // 2. Tecla '2' vai para Lente do Compilador; 's' ativa Modo 10 Anos (ELI10); Enter envia p/ Agente
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('2')));
        assert_eq!(state.lab.tab, crate::lab_tools::LabToolTab::LenteCompilador);
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('s')));
        assert!(state.lab.eli10_mode);
        state.on_key(&mut app, KeyEvent::from(KeyCode::Enter));
        assert!(!state.lab.open);
        assert!(state.agent.open);
        assert!(state.agent.proposed_code.is_some());

        // 3. Alt+B marca Breakpoint ● na linha 3 e F10 entra/avança no Debugger Visual
        state.editor.cursor = crate::editor::Pos { line: 2, col: 0 };
        state.on_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::ALT),
        );
        assert!(state.lab.debugger.breakpoints.contains(&3));
        state.on_key(&mut app, KeyEvent::from(KeyCode::F(10)));
        assert!(state.lab.debugger.active);
        assert_eq!(state.lab.debugger.active_line(), Some(2));
        state.on_key(&mut app, KeyEvent::from(KeyCode::F(10)));
        assert_eq!(state.lab.debugger.active_line(), Some(3));

        // 4. Quest "Conserte o Código" (FIX-03) -> carrega quebrado, conserta 1 linha e ganha XP
        state.on_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT),
        );
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('4')));
        state.on_key(&mut app, KeyEvent::from(KeyCode::Down));
        state.on_key(&mut app, KeyEvent::from(KeyCode::Down));
        assert_eq!(
            crate::lab_tools::TROUBLESHOOTING_QUESTS[state.lab.quest_sel].id,
            "FIX-03"
        );
        state.on_key(&mut app, KeyEvent::from(KeyCode::Enter));
        assert!(!state.lab.open);
        let fixed = crate::lab_tools::TROUBLESHOOTING_QUESTS[state.lab.quest_sel].fixed_reference;
        state.editor.select_all();
        state.editor.insert_text(fixed);
        state.on_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT),
        );
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('4')));
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('v')));
        assert!(state.lab.completed_quests.contains("FIX-03"));
        assert!(app.xp() >= 80);

        // 5. Aba 5 Playground exporta COMPARTILHAR_RUSTLAB.md
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('5')));
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('e')));
        assert!(root.join("COMPARTILHAR_RUSTLAB.md").is_file());

        // 6. Aba 6 Ecossistemas Modernos adiciona dep e aplica template Axum
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('6')));
        state.on_key(&mut app, KeyEvent::from(KeyCode::Down));
        state.on_key(&mut app, KeyEvent::from(KeyCode::Down));
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('d')));
        let toml = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
        assert!(toml.contains("axum = \"0.8\""));
        state.on_key(&mut app, KeyEvent::from(KeyCode::Enter));
        assert!(
            state
                .editor
                .buf
                .to_string_lossy()
                .contains("rotear_requisicao")
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn hub_novas_ferramentas_lifetimes_layout_macros_poliglota_flashcards_e_gdb() {
        let root = std::env::temp_dir().join(format!(
            "rustlab-lab6-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='lab6'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
        let mut app = App::for_snapshot();
        app.set_config_path(root.join("config.toml"));
        app.set_storage_path(root.join("prog.toml"));
        app.screen = Screen::Create;
        let mut state = CreateState::new();
        state.open_project(&root, &mut app);
        assert!(state.project.is_some());
        state.view = CreateView::Workspace;
        state.focus = Focus::Editor;
        state.editor.select_all();
        state.editor.insert_text(
            "#[repr(C)]\nstruct Caixa { a: bool, b: u64, c: u8 }\nfn main() {\n    let s = String::from(\"rust\");\n    let r = &s;\n    println!(\"{r}\");\n}\n",
        );

        // Alt+L abre Mapa de Lifetimes e ativa marcadores na margem
        state.on_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('l'), KeyModifiers::ALT),
        );
        assert!(state.lab.open);
        assert_eq!(state.lab.tab, crate::lab_tools::LabToolTab::MapaLifetimes);
        assert!(state.lab.lifetime_gutter_enabled);

        // Tecla '8' vai para Layout de Memória e calcula padding
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('8')));
        assert_eq!(state.lab.tab, crate::lab_tools::LabToolTab::LayoutMemoria);
        let layout = state.lab.last_layout.as_ref().unwrap();
        assert_eq!(layout.user_types[0].declared_size, 24);
        assert_eq!(layout.user_types[0].optimal_size, 16);

        // Tecla '9' abre Desaçucarador de Macros
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('9')));
        assert_eq!(
            state.lab.tab,
            crate::lab_tools::LabToolTab::DesacucararMacros
        );
        assert!(
            state
                .lab
                .last_desugar
                .as_ref()
                .unwrap()
                .items
                .iter()
                .any(|i| i.id == "macro-print")
        );

        // Tecla '0' abre Dicionário Poliglota & Flashcards, e 't' alterna para Flashcards
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('0')));
        assert_eq!(
            state.lab.tab,
            crate::lab_tools::LabToolTab::PoliglotaIntuicao
        );
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('t')));
        assert!(state.lab.polyglot_mode_flashcards);
        // FLASH-01 não compila -> tecla 'e' acerta e premia +25 XP + SRS
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('e')));
        assert!(state.lab.completed_flashcards.contains("FLASH-01"));
        assert!(app.xp() >= 25);
        assert!(app.progress.srs_cards.contains_key("FLASH-01"));

        // Tecla '3' -> 'g' prepara comando de debugger nativo (rust-gdb / lldb)
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('3')));
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('g')));
        assert!(state.lab.native_debugger_status.is_some());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn agente_le_console_stderr_e_envia_contexto_completo_para_ollama() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let root = std::env::temp_dir().join(format!(
            "rustlab-agent-console-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='meubevy'\nversion='0.1.0'\nedition='2024'\n[dependencies]\nbevy = \"0.20\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join("src/main.rs"),
            "use bevy::prelude::*;\nfn main() { App::new().run(); }\n",
        )
        .unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let captured = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let captured_srv = std::sync::Arc::clone(&captured);
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            let mut content_len = None;
            let mut header_end = None;
            loop {
                let n = stream.read(&mut chunk).unwrap_or(0);
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&chunk[..n]);
                if header_end.is_none()
                    && let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n")
                {
                    header_end = Some(pos + 4);
                    let hdrs = String::from_utf8_lossy(&buf[..pos]);
                    for line in hdrs.lines() {
                        if let Some((k, v)) = line.split_once(':')
                            && k.trim().eq_ignore_ascii_case("content-length")
                        {
                            content_len = v.trim().parse::<usize>().ok();
                        }
                    }
                }
                if let (Some(he), Some(cl)) = (header_end, content_len)
                    && buf.len() >= he + cl
                {
                    break;
                }
            }
            *captured_srv.lock().unwrap() = String::from_utf8_lossy(&buf).to_string();
            let body = r#"{"message":{"content":"Falta libudev-dev / libudev-devel via zypper no openSUSE Tumbleweed."}}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
        });

        let mut app = App::for_snapshot();
        app.set_config_path(root.join("config.toml"));
        app.set_storage_path(root.join("progress.toml"));
        app.config.ai.provider = AiProvider::Ollama;
        app.config.ai.ollama_url = format!("http://127.0.0.1:{port}");
        app.config.ai.ollama_model = "gemma4:e4b".to_string();
        app.screen = Screen::Create;

        let mut state = CreateState::new();
        state.open_project(&root, &mut app);
        state.handle_exec_event(
            ExecEvent::Chunk {
                kind: StreamKind::Stdout,
                text: "Compiling libudev-sys v0.1.4".to_string(),
            },
            &mut app,
        );
        state.handle_exec_event(
            ExecEvent::Chunk {
                kind: StreamKind::Stderr,
                text: "thread 'main' panicked at libudev-sys/build.rs:38:41:\nPackage libudev was not found in the pkg-config search path\nerror: failed to run custom build command for `libudev-sys v0.1.4`".to_string(),
            },
            &mut app,
        );
        state.handle_exec_event(
            ExecEvent::Finished {
                label: "cargo build".to_string(),
                code: Some(101),
                success: false,
            },
            &mut app,
        );

        let console_txt = state.recent_console_output(50, 4_000);
        assert!(console_txt.contains("libudev was not found"));
        assert!(console_txt.contains("zypper install"));

        let ctx = crate::mentor::create_context_from(&state);
        assert!(ctx.console.contains("libudev was not found"));
        assert!(ctx.summary.contains("bevy"));
        assert!(ctx.errors.contains("libudev-sys"));

        state.on_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT),
        );
        state.agent.input = "me ajude a entender esses erros no opensuse tumbleweed".to_string();
        state.on_key(&mut app, KeyEvent::from(KeyCode::Enter));

        server.join().unwrap();
        let req_raw = captured.lock().unwrap().clone();
        assert!(req_raw.contains("libudev was not found"));
        assert!(req_raw.contains("zypper install"));
        assert!(req_raw.contains("me ajude a entender esses erros no opensuse tumbleweed"));
        assert_eq!(
            req_raw
                .matches("me ajude a entender esses erros no opensuse tumbleweed")
                .count(),
            1,
            "pergunta do usuário não deve ser duplicada no payload do Ollama"
        );

        // Poll até chegar a resposta da thread
        for _ in 0..50 {
            if state.poll_agent(&mut app) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(
            state
                .agent
                .messages
                .iter()
                .any(|m| m.text.contains("libudev-devel"))
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn alt_e_diagnostica_console_e_xray_e_checkpoints_guiados_funcionam() {
        let mut app = crate::ui::tests_util::test_app();
        let mut state = CreateState::new_for(true);
        state.view = CreateView::Workspace;
        state.editor = Editor::new(
            "fn main() {\n    let count = 42;\n    let s = String::from(\"rust\");\n    let r = &s;\n}\n",
            None,
        );
        state.refresh_inlay_hints();
        assert!(state.inlay_hints.iter().any(|h| h.label == ": i32"));
        assert!(state.inlay_hints.iter().any(|h| h.label == ": String"));

        // Alt+E abre painel do agente e envia diagnóstico em 1 clique
        state.console.push(
            StreamKind::Stderr,
            "error[E0382]: borrow of moved value: `s`",
        );
        state.on_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::ALT),
        );
        assert!(state.agent.open);
        assert_eq!(state.focus, Focus::Agent);
        assert!(state.agent.messages.len() >= 2);

        // X-Ray passos '[' e ']'
        state.open_lab_tab(crate::lab_tools::LabToolTab::RaioXMemoria, &mut app);
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char(']')));
        assert_eq!(state.lab.last_xray.as_ref().unwrap().timeline_cursor, 1);
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('[')));
        assert_eq!(state.lab.last_xray.as_ref().unwrap().timeline_cursor, 0);

        // Guided Checkpoint g + v
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('6')));
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('g')));
        state.open_lab_tab(crate::lab_tools::LabToolTab::EcossistemasModernos, &mut app);
        state.on_key(&mut app, KeyEvent::from(KeyCode::Char('v')));
        assert!(state.lab.completed_checkpoints.contains("minigrep-cp1"));
    }
}
