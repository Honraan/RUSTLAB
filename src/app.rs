//! Núcleo da aplicação: estado, máquina de telas e o loop de eventos.
//!
//! O processamento de teclas é deliberadamente separado do desenho e da
//! I/O de terminal (`handle_key` não conhece `Terminal`), para que a
//! lógica de navegação seja testável com testes unitários comuns.

use std::io;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use ratatui::Terminal;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::animation::{AnimEngine, ScreenFx};
use crate::audio::{self, AudioReport};
use crate::config::{self, Config, MotionMode, ThemeId};
use crate::event::{Event, Events};
use crate::theme::Theme;
use crate::ui;

/// Telas de fases futuras, rotuladas com honestidade (requisito 18:
/// "placeholders claramente rotulados" — nada aqui finge estar pronto).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placeholder {
    pub title_key: &'static str,
    pub phase: u8,
    pub summary: &'static str,
    pub planned: &'static [&'static str],
}

pub const PLACEHOLDER_CREATE: Placeholder = Placeholder {
    title_key: "menu.create",
    phase: 3,
    summary: "Modo Criar com editor e execução Cargo",
    planned: &["Fase 7 — rust-analyzer / IDE avançada"],
};

pub const PLACEHOLDER_PROGRESS: Placeholder = Placeholder {
    title_key: "menu.progress",
    phase: 5,
    summary: "XP, níveis, conquistas e marcos de aprendizado",
    planned: &[
        "Fase 4 — persistência do progresso de aprendizado",
        "Fase 5 — gamificação e celebrações",
    ],
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Menu,
    Settings,
    Help,
    Browser,
    LessonView,
    Create,
    Exercises,
    Progress,
    Mentor,
    Placeholder(Placeholder),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Success,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Status {
    pub message: String,
    pub kind: StatusKind,
    pub since: Instant,
}

impl Status {
    pub fn new(message: impl Into<String>, kind: StatusKind) -> Self {
        Self {
            message: message.into(),
            kind,
            since: Instant::now(),
        }
    }
}

/// Itens do menu principal, na ordem em que aparecem.
pub const MENU_ITEMS: [MenuAction; 6] = [
    MenuAction::Learn,
    MenuAction::Create,
    MenuAction::Progress,
    MenuAction::Settings,
    MenuAction::Help,
    MenuAction::Quit,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Learn,
    Create,
    Progress,
    Settings,
    Help,
    Quit,
}

/// Posição de edição na tela de configurações.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingRow {
    Theme,
    Animations,
    Sound,
    Volume,
    Gamification,
    Language,
    Ai,
    OllamaModel,
}

pub const SETTING_ROWS: [SettingRow; 8] = [
    SettingRow::Theme,
    SettingRow::Animations,
    SettingRow::Sound,
    SettingRow::Volume,
    SettingRow::Gamification,
    SettingRow::Language,
    SettingRow::Ai,
    SettingRow::OllamaModel,
];

pub struct App {
    pub config: Config,
    pub theme: Theme,
    pub anim: AnimEngine,
    pub screen: Screen,
    pub menu_index: usize,
    pub progress_scroll: usize,
    pub settings_row: usize,
    pub status: Option<Status>,
    pub audio_report: AudioReport,
    pub unicode: bool,
    pub quit: bool,
    /// Aviso de carregamento de config (malformada, ilegível etc.)
    pub load_warning: Option<String>,
    /// Currículo carregado do conteúdo embutido (Fase 2). `None` apenas
    /// em apps de teste que não o carregam — a UI avisa honestamente.
    pub curriculum: Option<crate::curriculum::Curriculum>,
    /// Progresso da sessão atual (persistência: Fase 4).
    pub progress: crate::learn::Progress,
    pub browser: crate::learn::BrowserState,
    pub lesson_view: Option<crate::learn::LessonView>,
    pub lesson_drafts: std::collections::HashMap<String, String>,
    pub create: Option<crate::create::CreateState>,
    pub exercises: Option<crate::exercises::ExerciseState>,
    pub mentor: Option<crate::mentor::MentorState>,
    pub exec: crate::execution::ExecutionManager,
    pub ollama_models: Vec<String>,
    pub ollama_models_fetched: bool,
    storage_path_override: Option<std::path::PathBuf>,
    /// Onde `persist_settings` grava. `None` (app real) => caminho padrão
    /// XDG; testes apontam para um arquivo temporário.
    config_path_override: Option<std::path::PathBuf>,
    dirty: bool,
}

impl App {
    /// Carrega configuração do disco e monta o app. Nunca falha por
    /// config corrompida — recupera com padrões e guarda o aviso.
    pub fn new() -> io::Result<Self> {
        let outcome = config::load();
        let unicode_lookup = |k: &str| std::env::var(k).ok();
        let unicode = config::supports_unicode(&unicode_lookup);
        let audio_report = audio::detect();
        let mut app = Self::from_config(outcome.config, outcome.warning, unicode, audio_report);
        // Currículo embutido: validado na compilação dos dados. Um erro
        // aqui é bug de desenvolvimento, não do usuário — avisamos e
        // seguimos sem lições, em vez de derrubar o app.
        match crate::curriculum::Curriculum::embedded() {
            Ok(c) => app.curriculum = Some(c),
            Err(err) => {
                let msg = format!("currículo embutido inválido: {err}");
                app.load_warning = Some(match app.load_warning.take() {
                    Some(prev) => format!("{prev} · {msg}"),
                    None => msg,
                });
            }
        }
        let (progress, progress_warning) = crate::storage::load();
        app.progress = crate::learn::Progress::from_data(progress);
        if let Some(warning) = progress_warning {
            app.load_warning = Some(match app.load_warning.take() {
                Some(existing) => format!("{existing} · {warning}"),
                None => warning,
            });
        }
        // `RUSTLAB_CONFIG` aponta para um arquivo alternativo (testes e
        // desenvolvimento; também serve de base para um futuro `--config`).
        if let Some(path) = std::env::var_os("RUSTLAB_CONFIG") {
            app.set_config_path(std::path::PathBuf::from(path));
        }
        // Sem alarme: apenas informa uma vez que o som é opcional.
        if !app.audio_report.available() {
            let msg = app
                .bi(
                    "Áudio não detectado; o RustLab funciona normalmente sem som.",
                    "Audio not detected; RustLab works normally without sound.",
                )
                .to_string();
            app.status = Some(Status::new(msg, StatusKind::Info));
        }
        Ok(app)
    }

    /// Construção explícita para testes e para futuras telas "resetar
    /// configurações".
    pub fn from_config(
        config: Config,
        load_warning: Option<String>,
        unicode: bool,
        audio_report: AudioReport,
    ) -> Self {
        let theme = Theme::for_id(config.ui.theme);
        let anim = AnimEngine::new(config.ui.animations);
        let mut app = Self {
            config,
            theme,
            anim,
            screen: Screen::Welcome,
            menu_index: 0,
            progress_scroll: 0,
            settings_row: 0,
            status: None,
            audio_report,
            unicode,
            quit: false,
            load_warning,
            curriculum: None,
            progress: crate::learn::Progress::default(),
            browser: crate::learn::BrowserState::default(),
            lesson_view: None,
            lesson_drafts: std::collections::HashMap::new(),
            create: None,
            exercises: None,
            mentor: None,
            exec: crate::execution::ExecutionManager::new(),
            ollama_models: Vec::new(),
            ollama_models_fetched: false,
            storage_path_override: None,
            config_path_override: None,
            dirty: true,
        };
        app.anim.play(ScreenFx::Welcome, app.theme.accent);
        app
    }

    pub fn is_english(&self) -> bool {
        self.config.is_english()
    }

    pub fn t(&self, key: &'static str) -> &'static str {
        crate::i18n::t_for(key, self.is_english())
    }

    pub fn tf(&self, key: &'static str, args: &[&dyn std::fmt::Display]) -> String {
        crate::i18n::tf_for(key, self.is_english(), args)
    }

    pub fn bi<'a>(&self, pt: &'a str, en: &'a str) -> &'a str {
        crate::i18n::bi(pt, en, self.is_english())
    }

    /// Redireciona o arquivo de config (usado por testes; também útil
    /// para um futuro `--config`).
    pub fn set_config_path(&mut self, path: std::path::PathBuf) {
        self.config_path_override = Some(path);
    }

    pub fn save_config(&self) -> Result<(), config::SaveError> {
        match &self.config_path_override {
            Some(path) => config::save_to(&self.config, path),
            None => config::save(&self.config),
        }
    }

    pub fn config_path(&self) -> std::path::PathBuf {
        self.config_path_override
            .clone()
            .or_else(config::config_path)
            .unwrap_or_else(|| std::path::PathBuf::from("rustlab-config.toml"))
    }

    pub fn set_storage_path(&mut self, path: std::path::PathBuf) {
        self.storage_path_override = Some(path);
    }

    pub fn persist_progress(&self) -> std::io::Result<()> {
        let data = self.progress.to_data();
        match &self.storage_path_override {
            Some(path) => crate::storage::save_to(&data, path),
            None => crate::storage::save(&data),
        }
    }

    /// Registra recompensa somente para eventos novos; o ledger é persistido,
    /// portanto repetir uma tarefa ou reiniciar não volta a conceder XP.
    pub fn award_xp(&mut self, event: String, amount: u32, label: &str) -> bool {
        if self.progress.xp_events.contains_key(&event) {
            return false;
        }
        let old_level = self.level();
        let milestone = self.unlocked_by_event(&event);
        self.progress.xp_events.insert(event, amount);
        let new_level = self.level();
        let message =
            if self.config.ui.gamification == crate::config::GamificationVisibility::Visible {
                if new_level > old_level {
                    format!(
                        "{label}! +{amount} XP · nível {new_level} alcançado{}",
                        milestone
                            .map(|m| format!(" · marco: {m}"))
                            .unwrap_or_default()
                    )
                } else {
                    format!(
                        "{label}! +{amount} XP{}",
                        milestone
                            .map(|m| format!(" · marco: {m}"))
                            .unwrap_or_default()
                    )
                }
            } else {
                label.to_string()
            };
        self.set_status(message, StatusKind::Success);
        self.anim
            .play(crate::animation::ScreenFx::Celebration, self.theme.accent);
        if self.config.sound.enabled {
            crate::audio::play_cue(self.config.sound.volume);
        }
        if let Err(err) = self.persist_progress() {
            self.set_status(
                format!("XP contabilizado nesta sessão, mas não salvo: {err}"),
                StatusKind::Warning,
            );
        }
        true
    }

    pub fn xp(&self) -> u32 {
        self.progress
            .xp_events
            .values()
            .copied()
            .fold(0, u32::saturating_add)
    }

    pub fn level(&self) -> u32 {
        self.xp() / 100 + 1
    }

    pub fn has_independent_project_complete(&self) -> bool {
        ["plan", "build", "clippy", "tests", "run"]
            .iter()
            .all(|k| self.progress.project_milestones.contains(*k))
    }

    pub fn achievement_count(&self) -> usize {
        let (lessons, total_lessons, exercises, total_exercises) =
            self.curriculum.as_ref().map_or((0, 0, 0, 0), |curriculum| {
                let order = curriculum.lesson_order();
                let lessons = order
                    .iter()
                    .filter(|(_, _, id)| self.progress.completed.contains(*id))
                    .count();
                let all_ex: Vec<_> = curriculum.exercises().collect();
                let exercises = all_ex
                    .iter()
                    .filter(|ex| self.progress.exercises_completed.contains(&ex.id))
                    .count();
                (lessons, order.len(), exercises, all_ex.len())
            });
        usize::from(lessons >= 1)
            + usize::from(lessons >= 5)
            + usize::from(total_lessons > 0 && lessons == total_lessons)
            + usize::from(exercises >= 1)
            + usize::from(exercises >= 6)
            + usize::from(total_exercises > 0 && exercises == total_exercises)
            + usize::from(lessons + exercises >= 10)
            + usize::from(self.has_independent_project_complete())
    }

    fn unlocked_by_event(&self, event: &str) -> Option<&'static str> {
        if event == "project:independent-complete" {
            return self
                .has_independent_project_complete()
                .then_some("Projeto independente concluído");
        }
        if event.starts_with("project:") {
            return None;
        }
        let curriculum = self.curriculum.as_ref()?;
        let order = curriculum.lesson_order();
        let lesson_count = order
            .iter()
            .filter(|(_, _, id)| self.progress.completed.contains(*id))
            .count();
        let total_lessons = order.len();
        let all_ex: Vec<_> = curriculum.exercises().collect();
        let exercise_count = all_ex
            .iter()
            .filter(|ex| self.progress.exercises_completed.contains(&ex.id))
            .count();
        let total_exercises = all_ex.len();
        if event.starts_with("lesson:") {
            match lesson_count {
                1 => Some("Primeiros passos"),
                5 => Some("Cinco lições"),
                count if total_lessons > 0 && count == total_lessons => Some("Trilha completa"),
                _ if lesson_count + exercise_count == 10 => Some("Prática constante"),
                _ => None,
            }
        } else if event.starts_with("exercise:") {
            match exercise_count {
                1 => Some("Primeiro desafio"),
                6 => Some("Seis desafios"),
                count if total_exercises > 0 && count == total_exercises => {
                    Some("Todos os desafios")
                }
                _ if lesson_count + exercise_count == 10 => Some("Prática constante"),
                _ => None,
            }
        } else {
            None
        }
    }

    /// App determinístico para snapshots de UI (tests/ui_snapshot.rs):
    /// padrões de config, sem animação, sem tocar no disco do usuário.
    pub fn for_snapshot() -> Self {
        let mut app = Self::for_snapshot_with_animation();
        app.anim.set_mode(MotionMode::Off);
        app
    }

    /// Igual a `for_snapshot`, mas a animação de abertura continua ativa
    /// (usado pelo teste de término de efeitos `fx_finish`).
    pub fn for_snapshot_with_animation() -> Self {
        let mut app = Self::from_config(
            Config {
                language: "pt-BR".to_string(),
                ..Config::defaults()
            },
            None,
            true,
            crate::audio::AudioReport {
                backend: crate::audio::AudioBackend::None,
                detail: String::new(),
            },
        );
        app.status = None;
        app.curriculum = crate::curriculum::Curriculum::embedded().ok();
        app.set_config_path(std::path::PathBuf::from(
            "/tmp/rustlab-snapshot-config.toml",
        ));
        app.set_storage_path(std::path::PathBuf::from(
            "/tmp/rustlab-snapshot-progress.toml",
        ));
        app
    }

    // ------------------------------------------------------------------
    // Loop principal
    // ------------------------------------------------------------------

    pub fn run(
        &mut self,
        terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    ) -> io::Result<ExitCode> {
        let events = Events::new();
        loop {
            // Redesenha se algo mudou OU se há animação rodando. Quando
            // está tudo parado, aguarda eventos com timeout longo e custo
            // de CPU ~zero (requisito 16).
            if self.dirty || self.anim.is_running() {
                terminal.draw(|frame| ui::draw(frame, self))?;
                self.dirty = false;
            }

            let mut exec_events = Vec::new();
            self.exec.poll_events(&mut exec_events);
            if !exec_events.is_empty() {
                let lesson_owns_process = self
                    .lesson_view
                    .as_ref()
                    .and_then(|v| v.practice.as_ref())
                    .is_some_and(|p| p.outcome == crate::learn::PracticeOutcome::Running);
                let exercise_owns_process = self
                    .exercises
                    .as_ref()
                    .and_then(|s| s.session.as_ref())
                    .is_some_and(|s| s.outcome == crate::exercises::ExerciseOutcome::Running);
                if lesson_owns_process {
                    for event in exec_events {
                        crate::learn::on_lesson_exec_event(self, event);
                    }
                } else if exercise_owns_process {
                    if let Some(mut exercises) = self.exercises.take() {
                        for event in exec_events {
                            exercises.handle_exec_event(event, self);
                        }
                        self.exercises = Some(exercises);
                    }
                } else if let Some(mut create) = self.create.take() {
                    for event in exec_events {
                        create.handle_exec_event(event, self);
                    }
                    self.create = Some(create);
                }
                self.dirty = true;
            }
            if let Some(mut create) = self.create.take() {
                if create.poll_lsp(self) {
                    self.dirty = true;
                }
                if create.poll_agent(self) {
                    self.dirty = true;
                }
                self.create = Some(create);
            }
            if let Some(mentor) = self.mentor.as_mut()
                && mentor.poll_events()
            {
                self.dirty = true;
            }

            let timeout = if self.exec.running
                || self.anim.is_running()
                || self
                    .create
                    .as_ref()
                    .is_some_and(|c| c.lsp_pending() || c.agent_running())
                || self.mentor.as_ref().is_some_and(|m| m.is_running())
            {
                Duration::from_millis(40)
            } else {
                Duration::from_millis(500)
            };
            match events.next(timeout) {
                Event::Key(key) => self.handle_input(key),
                Event::Mouse(mouse) => {
                    use ratatui::crossterm::event::{MouseButton, MouseEventKind};
                    if self.screen == Screen::Create {
                        match mouse.kind {
                            MouseEventKind::Moved => {
                                if let Some(mut create) = self.create.take() {
                                    create.on_mouse_move(mouse.column, mouse.row);
                                    self.create = Some(create);
                                    self.dirty = true;
                                }
                            }
                            MouseEventKind::Down(MouseButton::Left) => {
                                if let Some(mut create) = self.create.take() {
                                    create.on_mouse_down(mouse.column, mouse.row, self);
                                    self.create = Some(create);
                                    self.dirty = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::Resize => self.dirty = true,
                Event::Paste(text) => {
                    if self.screen == Screen::Create
                        && let Some(mut create) = self.create.take()
                    {
                        create.on_paste(&text);
                        self.create = Some(create);
                    }
                    if self.screen == Screen::Exercises
                        && let Some(state) = self.exercises.as_mut()
                    {
                        state.on_paste(&text);
                    }
                    if self.screen == Screen::LessonView {
                        crate::learn::on_lesson_paste(self, &text);
                    }
                    self.dirty = true;
                }
                Event::Tick => {
                    // Expiração de notificações temporárias.
                    if self
                        .status
                        .as_ref()
                        .is_some_and(|s| s.since.elapsed() > Duration::from_secs(4))
                    {
                        self.status = None;
                        self.dirty = true;
                    }
                    if self.anim.is_running() {
                        self.dirty = true;
                    }
                }
            }

            if self.quit {
                if self.exec.running {
                    self.exec.cancel();
                    continue; // drenar o evento final e limpar o fixture antes de sair
                }
                return Ok(ExitCode::SUCCESS);
            }
        }
    }

    // ------------------------------------------------------------------
    // Entrada
    // ------------------------------------------------------------------

    pub fn handle_input(&mut self, key: KeyEvent) {
        // crossterm pode reportar Release/Repeat no Windows; só reagimos
        // ao Press para evitar navegação dupla.
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
            if let Some(mut create) = self.create.take() {
                let saved = create.save_before_exit(self);
                self.create = Some(create);
                if !saved {
                    self.dirty = true;
                    return;
                }
            }
            if let Some(mut exercises) = self.exercises.take() {
                let saved = exercises.save_answers(self);
                self.exercises = Some(exercises);
                if saved.is_err() {
                    self.dirty = true;
                    return;
                }
            }
            crate::learn::save_active_practice_draft(self);
            if self.exec.running {
                self.exec.cancel();
            }
            self.quit = true;
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('t')) {
            self.dirty = true;
            if self.screen == Screen::Create
                && self
                    .create
                    .as_ref()
                    .is_some_and(|c| c.view == crate::create::CreateView::Workspace)
            {
                if let Some(mut create) = self.create.take() {
                    create.toggle_agent_panel(self);
                    self.create = Some(create);
                }
            } else {
                self.toggle_mentor();
            }
            return;
        }
        // Ajuda de desenvolvimento: RUSTLAB_TRACE_KEYS=1 registra as teclas
        // e a tela ativa (útil ao depurar navegação; nunca em uso normal).
        if std::env::var_os("RUSTLAB_TRACE_KEYS").is_some() {
            eprintln!("key screen={:?} code={:?}", self.screen, key.code);
        }
        self.dirty = true;
        match self.screen {
            Screen::Welcome => self.on_welcome(&key),
            Screen::Menu => self.on_menu(&key),
            Screen::Settings => self.on_settings(&key),
            Screen::Help => self.on_help(&key),
            Screen::Progress => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => self.back_to_menu(),
                KeyCode::Up | KeyCode::Char('k') => {
                    self.progress_scroll = self.progress_scroll.saturating_sub(1)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.progress_scroll = self.progress_scroll.saturating_add(1)
                }
                KeyCode::Home => self.progress_scroll = 0,
                KeyCode::Char('e') => {
                    self.exercises
                        .get_or_insert_with(crate::exercises::ExerciseState::default)
                        .return_screen = Screen::Progress;
                    self.screen = Screen::Exercises;
                }
                KeyCode::Char('m') => self.toggle_mentor(),
                _ => {}
            },
            Screen::Browser => {
                if key.code == KeyCode::Char('e') {
                    self.exercises
                        .get_or_insert_with(crate::exercises::ExerciseState::default)
                        .return_screen = Screen::Browser;
                    self.screen = Screen::Exercises;
                } else if key.code == KeyCode::Char('m') {
                    crate::learn::mark_lesson_ai_used(self);
                    self.toggle_mentor();
                } else {
                    crate::learn::on_browser_key(self, &key);
                }
            }
            Screen::LessonView => {
                if key.code == KeyCode::Char('m')
                    && key.modifiers == KeyModifiers::NONE
                    && !self
                        .lesson_view
                        .as_ref()
                        .is_some_and(|v| v.practice_focused)
                {
                    crate::learn::mark_lesson_ai_used(self);
                    self.toggle_mentor();
                } else {
                    crate::learn::on_lesson_key(self, key);
                }
            }
            Screen::Create => {
                if let Some(mut create) = self.create.take() {
                    create.on_key(self, key);
                    self.create = Some(create);
                }
            }
            Screen::Exercises => {
                if let Some(mut state) = self.exercises.take() {
                    state.on_key(self, key);
                    self.exercises = Some(state);
                }
            }
            Screen::Mentor => {
                if let Some(mut mentor) = self.mentor.take() {
                    mentor.on_key(self, key);
                    self.mentor = Some(mentor);
                }
            }
            Screen::Placeholder(_) => self.on_placeholder(&key),
        }
    }

    pub fn projects_base_dir(&self) -> std::path::PathBuf {
        match self
            .config
            .workspace
            .projects_dir
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            Some(dir) => crate::projects::expand_home(std::path::Path::new(dir)),
            None => crate::projects::default_workspace_dir(),
        }
    }

    pub fn refresh_ollama_models(&mut self) -> Result<usize, String> {
        self.ollama_models_fetched = true;
        match crate::mentor::fetch_ollama_models(&self.config.ai.ollama_url) {
            Ok(models) => {
                let count = models.len();
                self.ollama_models = models;
                Ok(count)
            }
            Err(err) => Err(err),
        }
    }

    pub fn available_ollama_models(&self) -> Vec<String> {
        crate::mentor::candidate_ollama_models(&self.ollama_models, &self.config.ai.ollama_model)
    }

    pub fn toggle_mentor(&mut self) {
        if self.screen == Screen::Mentor {
            if let Some(mentor) = &self.mentor {
                self.screen = mentor.return_screen;
            } else {
                self.screen = Screen::Menu;
            }
            return;
        }
        if matches!(self.screen, Screen::Welcome) {
            return;
        }
        let return_screen = self.screen;
        let ctx = crate::mentor::build_context(self);
        match self.mentor.as_mut() {
            Some(mentor) => mentor.refresh_context(return_screen, ctx),
            None => self.mentor = Some(crate::mentor::MentorState::open(return_screen, ctx)),
        }
        self.screen = Screen::Mentor;
    }

    fn on_welcome(&mut self, key: &KeyEvent) {
        // Qualquer tecla avança — a tela de boas-vindas é uma cortina.
        if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
            // Mas 'q' e Esc ainda significam "sair do programa" na cortina.
            self.quit = true;
            return;
        }
        self.screen = Screen::Menu;
        self.anim.play(ScreenFx::Enter, self.theme.accent);
    }

    fn on_menu(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Up | KeyCode::Char('k') => {
                self.menu_index = (self.menu_index + MENU_ITEMS.len() - 1) % MENU_ITEMS.len();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.menu_index = (self.menu_index + 1) % MENU_ITEMS.len();
            }
            KeyCode::Enter => {
                self.activate_menu_item(MENU_ITEMS[self.menu_index]);
            }
            KeyCode::Char('m') => self.toggle_mentor(),
            _ => {}
        }
    }

    fn activate_menu_item(&mut self, action: MenuAction) {
        match action {
            MenuAction::Quit => self.quit = true,
            MenuAction::Learn => crate::learn::open_browser(self),
            MenuAction::Create => {
                self.screen = Screen::Create;
                self.create
                    .get_or_insert_with(crate::create::CreateState::new);
            }
            MenuAction::Progress => {
                self.screen = Screen::Progress;
                self.progress_scroll = 0;
            }
            MenuAction::Settings => {
                self.screen = Screen::Settings;
                self.anim.play(ScreenFx::Enter, self.theme.accent);
            }
            MenuAction::Help => {
                self.screen = Screen::Help;
                self.anim.play(ScreenFx::Enter, self.theme.accent);
            }
        }
    }

    fn on_placeholder(&mut self, key: &KeyEvent) {
        if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
            self.back_to_menu();
        }
    }

    fn back_to_menu(&mut self) {
        self.screen = Screen::Menu;
        self.anim.play(ScreenFx::Leave, self.theme.accent);
    }

    fn on_help(&mut self, key: &KeyEvent) {
        if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
            self.back_to_menu();
        }
    }

    fn on_settings(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.back_to_menu(),
            KeyCode::Up | KeyCode::Char('k') => {
                self.settings_row =
                    (self.settings_row + SETTING_ROWS.len() - 1) % SETTING_ROWS.len();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.settings_row = (self.settings_row + 1) % SETTING_ROWS.len();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.adjust_setting(SETTING_ROWS[self.settings_row], -1);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.adjust_setting(SETTING_ROWS[self.settings_row], 1);
            }
            KeyCode::Char('r') => match self.refresh_ollama_models() {
                Ok(n) if n > 0 => self.set_status(
                    if self.is_english() {
                        format!("Ollama: {n} local model(s) found.")
                    } else {
                        format!("Ollama: {n} modelo(s) local(is) encontrado(s).")
                    },
                    StatusKind::Success,
                ),
                Ok(_) => self.set_status(
                    self.bi(
                        "Ollama ativo, mas sem modelos baixados.",
                        "Ollama is running, but no models are downloaded.",
                    )
                    .to_string(),
                    StatusKind::Info,
                ),
                Err(err) => self.set_status(
                    if self.is_english() {
                        format!("Could not query Ollama models ({err}).")
                    } else {
                        format!("Não consegui consultar modelos do Ollama ({err}).")
                    },
                    StatusKind::Warning,
                ),
            },
            _ => {}
        }
    }

    fn adjust_setting(&mut self, row: SettingRow, direction: i32) {
        // Índice com clamp nas pontas: envolver de "Completas" para
        // "Desligadas" com ← confundiria um iniciante.
        let step = |v: usize, delta: i32, len: usize| -> usize {
            (v as i32 + delta).clamp(0, len as i32 - 1) as usize
        };
        match row {
            SettingRow::Theme => {
                let cur = ThemeId::ALL
                    .iter()
                    .position(|x| *x == self.config.ui.theme)
                    .unwrap_or(0);
                self.config.ui.theme = ThemeId::ALL[step(cur, direction, ThemeId::ALL.len())];
                self.theme = Theme::for_id(self.config.ui.theme);
            }
            SettingRow::Animations => {
                let cur = MotionMode::ALL
                    .iter()
                    .position(|x| *x == self.config.ui.animations)
                    .unwrap_or(0);
                let next = MotionMode::ALL[step(cur, direction, MotionMode::ALL.len())];
                self.config.ui.animations = next;
                self.anim.set_mode(next);
            }
            SettingRow::Sound => {
                self.config.sound.enabled = !self.config.sound.enabled;
            }
            SettingRow::Volume => {
                let delta = direction * 10;
                self.config.sound.volume =
                    (self.config.sound.volume as i32 + delta).clamp(0, 100) as u8;
            }
            SettingRow::Gamification => {
                use crate::config::GamificationVisibility as G;
                self.config.ui.gamification = match self.config.ui.gamification {
                    G::Visible => G::Hidden,
                    G::Hidden => G::Visible,
                };
            }
            SettingRow::Language => {
                let cur = usize::from(!self.config.is_english());
                let next = step(cur, direction, config::SUPPORTED_LANGUAGES.len());
                self.config.language = config::SUPPORTED_LANGUAGES[next].to_string();
            }
            SettingRow::Ai => {
                use crate::config::AiProvider as P;
                let providers = [P::Disabled, P::Ollama, P::Remote];
                let cur = providers
                    .iter()
                    .position(|x| *x == self.config.ai.provider)
                    .unwrap_or(0);
                self.config.ai.provider = providers[step(cur, direction, providers.len())];
            }
            SettingRow::OllamaModel => {
                if !self.ollama_models_fetched {
                    let _ = self.refresh_ollama_models();
                }
                let candidates = self.available_ollama_models();
                if !candidates.is_empty() {
                    let cur = candidates
                        .iter()
                        .position(|m| m == &self.config.ai.ollama_model)
                        .unwrap_or(0);
                    let next = step(cur, direction, candidates.len());
                    self.config.ai.ollama_model = candidates[next].clone();
                    self.config.ai.provider = crate::config::AiProvider::Ollama;
                }
            }
        }
        self.persist_settings();
    }

    fn persist_settings(&mut self) {
        let result = match &self.config_path_override {
            Some(path) => config::save_to(&self.config, path),
            None => config::save(&self.config),
        };
        match result {
            Ok(()) => self.set_status(self.t("settings.saved").to_string(), StatusKind::Success),
            Err(err) => self.set_status(
                format!("{}: {err}", self.t("settings.save_error")),
                StatusKind::Warning,
            ),
        }
    }

    pub fn set_status(&mut self, message: impl Into<String>, kind: StatusKind) {
        self.status = Some(Status::new(message, kind));
    }

    pub fn menu_action(&self) -> MenuAction {
        MENU_ITEMS[self.menu_index]
    }
}

// ----------------------------------------------------------------------
// Testes de navegação/estado (sem terminal)
// ----------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn test_app() -> App {
        let mut app = App::from_config(
            Config::default(),
            None,
            true,
            AudioReport {
                backend: audio::AudioBackend::None,
                detail: "teste".into(),
            },
        );
        // Testes nunca escrevem na config real do usuário.
        app.set_config_path(
            std::env::temp_dir().join(format!("rustlab-test-{}.toml", std::process::id())),
        );
        app.set_storage_path(
            std::env::temp_dir().join(format!("rustlab-test-progress-{}.toml", std::process::id())),
        );
        app
    }

    #[test]
    fn comeca_em_bem_vindo_e_avanca_para_menu() {
        let mut app = test_app();
        assert_eq!(app.screen, Screen::Welcome);
        app.handle_input(key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::Menu);
    }

    #[test]
    fn menu_navega_com_envolvimento() {
        let mut app = test_app();
        app.handle_input(key(KeyCode::Enter)); // welcome -> menu
        assert_eq!(app.menu_index, 0);
        app.handle_input(key(KeyCode::Down));
        app.handle_input(key(KeyCode::Down));
        assert_eq!(app.menu_action(), MenuAction::Progress);
        app.handle_input(key(KeyCode::Up));
        app.handle_input(key(KeyCode::Up));
        app.handle_input(key(KeyCode::Up));
        assert_eq!(app.menu_action(), MenuAction::Quit);
    }

    #[test]
    fn modo_criar_abre_workspace_inicial() {
        let mut app = test_app();
        app.handle_input(key(KeyCode::Enter));
        app.handle_input(key(KeyCode::Down)); // -> Criar
        app.handle_input(key(KeyCode::Enter)); // abre modo criar
        assert_eq!(app.screen, Screen::Create);
        app.handle_input(key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Menu);
    }

    #[test]
    fn progresso_real_abre_pratica_e_volta_ao_menu() {
        let mut app = test_app();
        app.activate_menu_item(MenuAction::Progress);
        assert_eq!(app.screen, Screen::Progress);
        app.handle_input(key(KeyCode::Char('e')));
        assert_eq!(app.screen, Screen::Exercises);
        app.activate_menu_item(MenuAction::Progress);
        app.handle_input(key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Menu);
    }

    #[test]
    fn ctrl_c_salva_resposta_antes_de_sair() {
        let mut app = test_app();
        app.curriculum = crate::curriculum::Curriculum::embedded().ok();
        let path =
            std::env::temp_dir().join(format!("rustlab-exit-answers-{}.toml", std::process::id()));
        app.set_storage_path(path.clone());
        let mut state = crate::exercises::ExerciseState::default();
        state.open_selected(&app);
        state.on_paste("// salvar na saida\n");
        let id = state.session.as_ref().unwrap().id.clone();
        let code = state.session.as_ref().unwrap().editor.buf.to_string_lossy();
        app.exercises = Some(state);
        app.handle_input(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(app.quit);
        let (data, warning) = crate::storage::load_from(&path);
        assert!(warning.is_none());
        assert_eq!(data.exercise_answers.get(&id), Some(&code));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn tecla_e_abre_exercicios_a_partir_do_browser() {
        let mut app = test_app();
        app.curriculum = crate::curriculum::Curriculum::embedded().ok();
        app.screen = Screen::Browser;
        app.handle_input(key(KeyCode::Char('e')));
        assert_eq!(app.screen, Screen::Exercises);
        assert!(app.exercises.is_some());
    }

    #[test]
    fn learn_sem_curriculo_avisa_e_fica_no_menu() {
        let mut app = test_app(); // test_app não carrega currículo
        app.handle_input(key(KeyCode::Enter)); // welcome -> menu
        app.handle_input(key(KeyCode::Enter)); // Learn
        assert!(matches!(app.screen, Screen::Menu));
        assert!(app.status.is_some());
    }

    #[test]
    fn ctrl_c_sai_de_qualquer_tela() {
        let mut app = test_app();
        app.screen = Screen::Settings;
        let mut k = key(KeyCode::Char('c'));
        k.modifiers = KeyModifiers::CONTROL;
        app.handle_input(k);
        assert!(app.quit);
    }

    #[test]
    fn q_sai_a_partir_do_menu() {
        let mut app = test_app();
        app.handle_input(key(KeyCode::Enter)); // -> menu
        app.handle_input(key(KeyCode::Char('q')));
        assert!(app.quit);
    }

    #[test]
    fn eventos_de_release_sao_ignorados() {
        // Evita navegação dupla em terminais que reportam Release/Repeat.
        let mut app = test_app();
        app.handle_input(key(KeyCode::Enter)); // welcome -> menu
        let mut release = key(KeyCode::Down);
        release.kind = KeyEventKind::Release;
        app.handle_input(release);
        assert_eq!(app.menu_index, 0);
    }

    #[test]
    fn tema_muda_aplicacoes_de_estilo() {
        let mut app = test_app();
        app.handle_input(key(KeyCode::Enter));
        app.screen = Screen::Settings;
        app.settings_row = 0; // Theme
        app.handle_input(key(KeyCode::Right));
        assert_eq!(app.config.ui.theme, ThemeId::Ferrugem);
        app.handle_input(key(KeyCode::Right));
        assert_eq!(app.config.ui.theme, ThemeId::Mono);
        // Ajustes tentam salvar em disco; em testes isso toca HOME real,
        // então apenas verificamos o estado em memória acima.
    }

    #[test]
    fn volume_clampa_entre_0_e_100() {
        let mut app = test_app();
        app.screen = Screen::Settings;
        app.settings_row = 3; // Volume
        for _ in 0..30 {
            app.handle_input(key(KeyCode::Left));
        }
        assert_eq!(app.config.sound.volume, 0);
        for _ in 0..40 {
            app.handle_input(key(KeyCode::Right));
        }
        assert_eq!(app.config.sound.volume, 100);
    }

    #[test]
    fn ajustar_animacao_clampa_em_ambas_as_pontas() {
        let mut app = test_app();
        app.screen = Screen::Settings;
        app.settings_row = 1; // Animações
        assert_eq!(app.config.ui.animations, MotionMode::Full);
        app.handle_input(key(KeyCode::Left));
        assert_eq!(app.config.ui.animations, MotionMode::Reduced);
        app.handle_input(key(KeyCode::Left));
        assert_eq!(app.config.ui.animations, MotionMode::Off);
        // ponta: mais um ← continua em Off
        app.handle_input(key(KeyCode::Left));
        assert_eq!(app.config.ui.animations, MotionMode::Off);
        app.handle_input(key(KeyCode::Right));
        assert_eq!(app.config.ui.animations, MotionMode::Reduced);
    }

    #[test]
    fn esc_na_configuracao_volta_ao_menu() {
        let mut app = test_app();
        app.screen = Screen::Settings;
        app.handle_input(key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Menu);
    }

    #[test]
    fn seis_longs_para_baixo_no_menu_chegam_em_sair() {
        let mut app = test_app();
        app.screen = Screen::Menu;
        app.menu_index = 0;
        for _ in 0..5 {
            app.handle_input(key(KeyCode::Down));
        }
        assert_eq!(app.menu_action(), MenuAction::Quit);
        app.handle_input(key(KeyCode::Enter));
        assert!(app.quit);
    }

    #[test]
    fn xp_e_idempotente_persistente_e_niveis_sao_deterministicos() {
        let mut app = test_app();
        app.set_storage_path(
            std::env::temp_dir().join(format!("rustlab-xp-test-{}.toml", std::process::id())),
        );
        assert!(app.award_xp("lesson:hello".into(), 20, "lição concluída"));
        assert!(!app.award_xp("lesson:hello".into(), 20, "lição concluída"));
        for i in 0..4 {
            app.award_xp(format!("exercise:{i}"), 30, "exercício concluído");
        }
        assert_eq!(app.xp(), 140);
        assert_eq!(app.level(), 2);
        let (data, warning) =
            crate::storage::load_from(&app.storage_path_override.clone().unwrap());
        assert!(warning.is_none());
        assert_eq!(data.xp_events.len(), 5);
        let mut restored = App::for_snapshot();
        restored.progress = crate::learn::Progress::from_data(data);
        assert_eq!(restored.xp(), 140);
        assert!(!restored.award_xp("lesson:hello".into(), 20, "lição concluída"));
        assert_eq!(restored.xp(), 140);
        std::fs::remove_file(app.storage_path_override.unwrap()).ok();
    }

    #[test]
    fn marcos_contam_somente_ids_do_curriculo_atual_e_ocultar_nao_apaga_xp() {
        let mut app = test_app();
        app.curriculum = crate::curriculum::Curriculum::embedded().ok();
        app.progress
            .completed
            .extend(["hello-world".into(), "obsoleta".into()]);
        app.progress.exercises_completed.insert("soma".into());
        assert_eq!(app.achievement_count(), 2);
        app.config.ui.gamification = crate::config::GamificationVisibility::Hidden;
        app.progress
            .xp_events
            .insert("lesson:hello-world".into(), 20);
        assert_eq!(app.xp(), 20);
        let text = crate::ui::progress::lines(&app)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!text.contains("XP"));
        assert!(!text.contains("Nível"));
        assert!(!text.contains("Marcos conquistados"));
    }

    #[test]
    fn ctrl_t_alterna_mentor_preservando_tela_de_retorno() {
        let mut app = test_app();
        app.screen = Screen::Progress;
        app.handle_input(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL));
        assert_eq!(app.screen, Screen::Mentor);
        assert_eq!(app.mentor.as_ref().unwrap().return_screen, Screen::Progress);
        app.handle_input(KeyEvent::from(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Progress);
    }

    #[test]
    fn idioma_alterna_nas_configuracoes_e_padrao_e_ingles() {
        let mut app = test_app();
        assert_eq!(app.config.language, "en-US");
        assert!(app.is_english());
        assert_eq!(app.t("settings.title"), "Settings");

        app.screen = Screen::Settings;
        app.settings_row = SETTING_ROWS
            .iter()
            .position(|r| *r == SettingRow::Language)
            .expect("linha de idioma");
        app.handle_input(key(KeyCode::Right));
        assert_eq!(app.config.language, "pt-BR");
        assert!(!app.is_english());
        assert_eq!(app.t("settings.title"), "Configurações");

        app.handle_input(key(KeyCode::Left));
        assert_eq!(app.config.language, "en-US");
        assert!(app.is_english());
    }
}
