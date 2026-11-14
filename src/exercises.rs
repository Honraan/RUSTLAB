//! Fluxo de exercícios com testes reais do Cargo e dicas progressivas.

use std::path::{Path, PathBuf};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Screen, StatusKind};
use crate::config::ExecConsent;
use crate::curriculum::Exercise;
use crate::editor::Editor;
use crate::execution::{ExecEvent, StreamKind, ToolCommand};

const MAX_OUTPUT: usize = 500;
static TEMP_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExerciseOutcome {
    NotRun,
    Running,
    Passed,
    TestsFailed,
    CompileFailed,
    TimedOut,
    Cancelled,
    StartFailed,
}

#[derive(Debug, Clone)]
pub struct ExerciseSession {
    pub id: String,
    pub editor: Editor,
    pub shown_hints: usize,
    pub hint_index: usize,
    pub output: Vec<String>,
    /// Linhas acima do fim da saída (0 acompanha o final).
    pub output_scroll: usize,
    pub output_page_rows: usize,
    pub output_width: u16,
    pub compile_error_seen: bool,
    pub statement_scroll: u16,
    pub hint_scroll: u16,
    pub outcome: ExerciseOutcome,
    pub pending_dir: Option<PathBuf>,
    pub ai_used: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExerciseView {
    List,
    Work,
}

#[derive(Debug, Clone)]
pub struct ExerciseState {
    pub view: ExerciseView,
    pub selected: usize,
    pub return_screen: Screen,
    pub session: Option<ExerciseSession>,
    pub consent_pending: bool,
    /// Respostas preservadas ao alternar desafios durante esta sessão.
    pub drafts: std::collections::HashMap<String, ExerciseSession>,
}

impl Default for ExerciseState {
    fn default() -> Self {
        Self {
            view: ExerciseView::List,
            selected: 0,
            return_screen: Screen::Browser,
            session: None,
            consent_pending: false,
            drafts: Default::default(),
        }
    }
}

impl ExerciseState {
    pub fn open_selected(&mut self, app: &App) {
        let Some(ex) = app
            .curriculum
            .as_ref()
            .and_then(|c| c.exercises().nth(self.selected))
        else {
            return;
        };
        if self
            .session
            .as_ref()
            .is_some_and(|session| session.outcome == ExerciseOutcome::Running)
        {
            return;
        }
        if let Some(session) = self.session.take() {
            self.drafts.insert(session.id.clone(), session);
        }
        let shown_hints = (app.progress.hints_used.get(&ex.id).copied().unwrap_or(0) as usize)
            .min(ex.hints.len());
        self.session = Some(self.drafts.remove(&ex.id).unwrap_or_else(|| {
            ExerciseSession {
                id: ex.id.clone(),
                editor: Editor::new(
                    app.progress
                        .exercise_answers
                        .get(&ex.id)
                        .map(String::as_str)
                        .unwrap_or(&ex.skeleton),
                    None,
                ),
                shown_hints,
                hint_index: shown_hints.saturating_sub(1),
                output: Vec::new(),
                output_scroll: 0,
                output_page_rows: 1,
                output_width: 80,
                compile_error_seen: false,
                statement_scroll: 0,
                hint_scroll: 0,
                outcome: ExerciseOutcome::NotRun,
                pending_dir: None,
                ai_used: false,
            }
        }));
        self.view = ExerciseView::Work;
    }

    pub fn on_key(&mut self, app: &mut App, key: KeyEvent) {
        if self.consent_pending {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('s') => {
                    self.consent_pending = false;
                    app.config.execution.consent = ExecConsent::Granted;
                    if let Err(e) = app.save_config() {
                        app.set_status(
                            format!("não salvei consentimento: {e}"),
                            StatusKind::Warning,
                        );
                    }
                    self.run_tests(app);
                }
                KeyCode::Esc | KeyCode::Char('n') => {
                    self.consent_pending = false;
                    app.set_status("nenhum teste foi executado", StatusKind::Info);
                }
                _ => {}
            }
            return;
        }
        match self.view {
            ExerciseView::List => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => app.screen = self.return_screen,
                KeyCode::Up | KeyCode::Char('k') => self.selected = self.selected.saturating_sub(1),
                KeyCode::Down | KeyCode::Char('j') => {
                    let n = app
                        .curriculum
                        .as_ref()
                        .map(|c| c.exercises().count())
                        .unwrap_or(0);
                    self.selected = (self.selected + 1).min(n.saturating_sub(1));
                }
                KeyCode::Enter => self.open_selected(app),
                _ => {}
            },
            ExerciseView::Work => self.work_key(app, key),
        }
    }

    fn work_key(&mut self, app: &mut App, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        if key.code == KeyCode::Esc
            && self
                .session
                .as_ref()
                .is_some_and(|s| s.outcome != ExerciseOutcome::Running)
        {
            if self.save_answers(app).is_err() {
                return;
            }
            self.view = ExerciseView::List;
            return;
        }
        match key.code {
            KeyCode::PageUp => {
                if let Some(s) = self.session.as_mut() {
                    s.output_scroll = s
                        .output_scroll
                        .saturating_add(s.output_page_rows.max(1))
                        .min(s.output_line_count().saturating_sub(s.output_page_rows));
                }
            }
            KeyCode::PageDown => {
                if let Some(s) = self.session.as_mut() {
                    s.output_scroll = s.output_scroll.saturating_sub(s.output_page_rows.max(1));
                }
            }
            KeyCode::F(2) => {
                if let Some(s) = self.session.as_mut() {
                    s.hint_index = s.hint_index.saturating_sub(1);
                    s.hint_scroll = 0;
                }
            }
            KeyCode::F(3) => {
                if let Some(s) = self.session.as_mut() {
                    s.hint_index = (s.hint_index + 1).min(s.shown_hints.saturating_sub(1));
                    s.hint_scroll = 0;
                }
            }
            KeyCode::F(4) => {
                if let Some(s) = self.session.as_mut() {
                    s.hint_scroll = if shift {
                        s.hint_scroll.saturating_sub(1)
                    } else {
                        s.hint_scroll.saturating_add(1)
                    };
                }
            }
            KeyCode::F(9) => {
                if let Some(s) = self.session.as_mut() {
                    s.statement_scroll = if shift {
                        s.statement_scroll.saturating_sub(1)
                    } else {
                        s.statement_scroll.saturating_add(1)
                    };
                }
            }
            KeyCode::Char('k') if ctrl => {
                if app.exec.running {
                    app.exec.cancel();
                }
            }
            KeyCode::F(5) => {
                if app.config.execution.consent == ExecConsent::Granted {
                    self.run_tests(app);
                } else {
                    self.consent_pending = true;
                }
            }
            KeyCode::F(1) => self.reveal_hint(app),
            KeyCode::Char('s') if ctrl => {
                if self.save_answers(app).is_ok() {
                    app.set_status("respostas salvas em disco", StatusKind::Success);
                }
            }
            KeyCode::Char(c) if !ctrl => self.with_editor(|e| e.insert_char(c)),
            KeyCode::Enter => self.with_editor(Editor::newline),
            KeyCode::Backspace => self.with_editor(Editor::backspace),
            KeyCode::Delete => self.with_editor(Editor::delete),
            KeyCode::Tab => self.with_editor(|e| e.insert_text("    ")),
            KeyCode::Left => self.with_editor(|e| e.move_cursor(-1, 0, shift, ctrl)),
            KeyCode::Right => self.with_editor(|e| e.move_cursor(1, 0, shift, ctrl)),
            KeyCode::Up => self.with_editor(|e| e.move_cursor(0, -1, shift, false)),
            KeyCode::Down => self.with_editor(|e| e.move_cursor(0, 1, shift, false)),
            KeyCode::Home if ctrl => self.with_editor(|e| e.move_doc(false, shift)),
            KeyCode::End if ctrl => self.with_editor(|e| e.move_doc(true, shift)),
            KeyCode::Home => self.with_editor(|e| e.move_home(shift)),
            KeyCode::End => self.with_editor(|e| e.move_end(shift)),
            KeyCode::Char('a') if ctrl => self.with_editor(Editor::select_all),
            KeyCode::Char('c') if ctrl => self.with_editor(|e| {
                e.copy();
            }),
            KeyCode::Char('x') if ctrl => self.with_editor(|e| {
                e.cut();
            }),
            KeyCode::Char('v') if ctrl => self.with_editor(Editor::paste),
            KeyCode::Char('z') if ctrl && shift => self.with_editor(Editor::redo),
            KeyCode::Char('z') if ctrl => self.with_editor(Editor::undo),
            KeyCode::Char('y') if ctrl => self.with_editor(Editor::redo),
            _ => {}
        }
    }

    fn with_editor(&mut self, f: impl FnOnce(&mut Editor)) {
        if let Some(s) = self.session.as_mut() {
            if s.outcome == ExerciseOutcome::Running {
                return;
            }
            let before = s.editor.buf.to_string_lossy();
            f(&mut s.editor);
            if s.editor.buf.to_string_lossy() != before {
                s.outcome = ExerciseOutcome::NotRun;
                s.output.clear();
                s.output_scroll = 0;
                s.compile_error_seen = false;
            }
        }
    }

    pub fn on_paste(&mut self, text: &str) {
        if self.view == ExerciseView::Work && !self.consent_pending {
            self.with_editor(|editor| editor.insert_text(text));
        }
    }

    /// Salva todas as respostas abertas; falhas mantêm os editores sujos.
    pub fn save_answers(&mut self, app: &mut App) -> std::io::Result<()> {
        if self.session.is_none() && self.drafts.is_empty() {
            return Ok(());
        }
        let sessions = self.drafts.values().chain(self.session.iter());
        for session in sessions {
            app.progress
                .exercise_answers
                .insert(session.id.clone(), session.editor.buf.to_string_lossy());
        }
        if let Err(err) = app.persist_progress() {
            app.set_status(format!("não salvei as respostas: {err}. Ctrl+S tenta novamente; o código continua no editor."), StatusKind::Warning);
            return Err(err);
        }
        for session in self.drafts.values_mut().chain(self.session.iter_mut()) {
            session.editor.mark_saved();
        }
        Ok(())
    }

    fn reveal_hint(&mut self, app: &mut App) {
        let Some(s) = self.session.as_mut() else {
            return;
        };
        let Some(ex) = app.curriculum.as_ref().and_then(|c| c.exercise(&s.id)) else {
            return;
        };
        if s.shown_hints >= ex.hints.len() {
            app.set_status(
                "todas as dicas disponíveis já foram reveladas",
                StatusKind::Info,
            );
            return;
        }
        s.shown_hints += 1;
        s.hint_index = s.shown_hints - 1;
        s.hint_scroll = 0;
        app.progress
            .hints_used
            .insert(s.id.clone(), s.shown_hints.min(u8::MAX as usize) as u8);
        if let Err(e) = app.persist_progress() {
            app.set_status(
                format!("dica mostrada, mas progresso não salvo: {e}"),
                StatusKind::Warning,
            );
        }
    }

    fn run_tests(&mut self, app: &mut App) {
        if app.exec.running {
            app.set_status(
                "já há um comando Cargo rodando; aguarde ou use Ctrl+K",
                StatusKind::Warning,
            );
            return;
        }
        if self.save_answers(app).is_err() {
            return;
        }
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let Some(ex) = app
            .curriculum
            .as_ref()
            .and_then(|c| c.exercise(&session.id))
        else {
            return;
        };
        let dir = match fresh_temp_dir(&ex.id) {
            Ok(dir) => dir,
            Err(e) => {
                let reason = format!("não consegui criar diretório temporário: {e}");
                session.outcome = ExerciseOutcome::StartFailed;
                session.output = vec![reason.clone()];
                session.output_scroll = 0;
                app.set_status(reason, StatusKind::Warning);
                return;
            }
        };
        if let Err(e) = prepare_fixture(&dir, ex, &session.editor.buf.to_string_lossy()) {
            let _ = std::fs::remove_dir_all(&dir);
            let reason = format!("não consegui preparar testes: {e}");
            session.outcome = ExerciseOutcome::StartFailed;
            session.output = vec![reason.clone()];
            session.output_scroll = 0;
            app.set_status(reason, StatusKind::Warning);
            return;
        }
        session.pending_dir = Some(dir.clone());
        session.output.clear();
        session.output_scroll = 0;
        session.compile_error_seen = false;
        session.outcome = ExerciseOutcome::Running;
        match app.exec.submit(ToolCommand::ExerciseTest, dir) {
            Ok(()) => app.set_status(
                "rodando testes determinísticos do exercício…",
                StatusKind::Info,
            ),
            Err(e) => {
                session.outcome = ExerciseOutcome::StartFailed;
                session.output.push(e.to_string());
                if let Some(dir) = session.pending_dir.take() {
                    let _ = std::fs::remove_dir_all(dir);
                }
                app.set_status(e, StatusKind::Warning);
            }
        }
    }

    pub fn handle_exec_event(&mut self, ev: ExecEvent, app: &mut App) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        match ev {
            ExecEvent::Chunk { kind, text } => {
                if kind == StreamKind::Stderr && is_compile_error(&text) {
                    session.compile_error_seen = true;
                }
                session.push_output(text);
            }
            ExecEvent::Finished { success, .. } => {
                let compile_failed = session.compile_error_seen;
                session.outcome = if success {
                    ExerciseOutcome::Passed
                } else if compile_failed {
                    ExerciseOutcome::CompileFailed
                } else {
                    ExerciseOutcome::TestsFailed
                };
                if success {
                    let newly_completed =
                        app.progress.exercises_completed.insert(session.id.clone());
                    if newly_completed {
                        if session.shown_hints == 0 && !session.ai_used {
                            let (base, bonus) = crate::learn::calculate_v3_practice_xp(
                                session.shown_hints,
                                session.ai_used,
                            );
                            app.award_xp(
                                format!("exercise:{}", session.id),
                                base,
                                "exercício concluído",
                            );
                            if bonus > 0 {
                                app.award_xp(
                                    format!("exercise:{}:independent", session.id),
                                    bonus,
                                    "bônus sem dicas",
                                );
                            }
                        } else {
                            app.award_xp(
                                format!("exercise:{}", session.id),
                                30,
                                "exercício concluído",
                            );
                        }
                    }
                    app.progress.record_srs_mastery(
                        &session.id,
                        session.shown_hints == 0 && !session.ai_used,
                    );
                    if let Err(e) = app.persist_progress() {
                        app.set_status(
                            format!("exercício passou, mas progresso não salvo: {e}"),
                            StatusKind::Warning,
                        );
                    } else {
                        app.set_status(
                            "testes passaram; exercício concluído e progresso salvo",
                            StatusKind::Success,
                        );
                    }
                } else {
                    if compile_failed
                        && let Some(help) =
                            crate::learn::explain_compiler_error_pt_br(&session.output.join("\n"))
                    {
                        session.push_output(format!("Explicação do Ferris: {help}"));
                    }
                    let message = if compile_failed {
                        "o código não compilou; veja o diagnóstico"
                    } else {
                        "um ou mais testes falharam; veja a saída"
                    };
                    app.set_status(message, StatusKind::Warning);
                }
                if let Some(dir) = session.pending_dir.take() {
                    let _ = std::fs::remove_dir_all(dir);
                }
            }
            ExecEvent::TimedOut { .. } => {
                session.outcome = ExerciseOutcome::TimedOut;
                session.push_output(
                    "Tempo limite atingido; processo encerrado. Verifique loops sem fim.".into(),
                );
                session.output_scroll = 0;
                app.set_status("testes excederam o limite de tempo", StatusKind::Warning);
                if let Some(dir) = session.pending_dir.take() {
                    let _ = std::fs::remove_dir_all(dir);
                }
            }
            ExecEvent::Cancelled { .. } => {
                session.outcome = ExerciseOutcome::Cancelled;
                session.push_output(
                    "Execução cancelada pelo usuário; nenhum novo resultado foi registrado.".into(),
                );
                session.output_scroll = 0;
                app.set_status("testes cancelados", StatusKind::Info);
                if let Some(dir) = session.pending_dir.take() {
                    let _ = std::fs::remove_dir_all(dir);
                }
            }
            ExecEvent::SpawnFailed { reason, .. } => {
                session.outcome = ExerciseOutcome::StartFailed;
                session.push_output(reason.clone());
                session.output_scroll = 0;
                app.set_status(reason, StatusKind::Warning);
                if let Some(dir) = session.pending_dir.take() {
                    let _ = std::fs::remove_dir_all(dir);
                }
            }
            ExecEvent::Started { .. } => {}
        }
    }

    pub fn exercise<'a>(&self, app: &'a App) -> Option<&'a Exercise> {
        let id = &self.session.as_ref()?.id;
        app.curriculum.as_ref()?.exercise(id)
    }
}

impl ExerciseSession {
    /// Conta linhas visuais, incluindo quebras de linhas longas no terminal.
    pub fn output_line_count(&self) -> usize {
        ratatui::widgets::Paragraph::new(self.output.join("\n"))
            .wrap(ratatui::widgets::Wrap { trim: false })
            .line_count(self.output_width.max(1))
    }

    fn push_output(&mut self, text: String) {
        if self.output_scroll > 0 {
            let added = ratatui::widgets::Paragraph::new(text.as_str())
                .wrap(ratatui::widgets::Wrap { trim: false })
                .line_count(self.output_width.max(1));
            self.output_scroll = self.output_scroll.saturating_add(added);
        }
        self.output.push(text);
        while self.output.len() > MAX_OUTPUT {
            self.output.remove(0);
        }
        // O renderer ajusta o limite após truncamento/redimensionamento.
    }
}

fn is_compile_error(text: &str) -> bool {
    let text = text.trim();
    text.contains("could not compile")
        || text.contains("error[E")
        || (text.starts_with("error:") && !text.starts_with("error: test failed"))
        || (text.contains(": error:") && !text.contains("test failed"))
}

pub fn prepare_fixture(dir: &Path, exercise: &Exercise, source: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::create_dir_all(dir.join("tests"))?;
    std::fs::write(
        dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"rustlab_exercise_{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
            exercise.id.replace('-', "_")
        ),
    )?;
    std::fs::write(dir.join("src/lib.rs"), source)?;
    // O aluno edita a biblioteca; a suíte fica separada e é selecionada
    // explicitamente no Cargo. Isso não transforma execução local em sandbox.
    let tests = exercise.tests.replace(
        "use super::",
        &format!("use rustlab_exercise_{}::", exercise.id.replace('-', "_")),
    );
    std::fs::write(dir.join("tests/rustlab_validation.rs"), tests)
}

fn fresh_temp_dir(id: &str) -> std::io::Result<PathBuf> {
    for _ in 0..100 {
        let nonce = TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "rustlab-exercise-{}-{id}-{nonce}",
            std::process::id()
        ));
        match std::fs::create_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "não encontrei nome temporário livre",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyEvent, KeyModifiers};

    fn app(tag: &str) -> (App, PathBuf) {
        let mut app = App::from_config(
            crate::config::Config::default(),
            None,
            true,
            crate::audio::AudioReport {
                backend: crate::audio::AudioBackend::None,
                detail: String::new(),
            },
        );
        app.curriculum = crate::curriculum::Curriculum::embedded().ok();
        let progress_path = std::env::temp_dir().join(format!(
            "rustlab-ex-{}-{}-{}.toml",
            tag,
            std::process::id(),
            uuid_like()
        ));
        app.set_storage_path(progress_path.clone());
        (app, progress_path)
    }

    fn uuid_like() -> u128 {
        use std::hash::{BuildHasher, Hasher};
        std::collections::hash_map::RandomState::new()
            .build_hasher()
            .finish() as u128
    }

    #[test]
    fn fixture_compila_com_tests_injetados_sem_shell() {
        let curriculum = crate::curriculum::Curriculum::embedded().unwrap();
        let ex = curriculum.exercise("soma").unwrap();
        let dir = std::env::temp_dir().join(format!("rustlab-fixture-test-{}", std::process::id()));
        prepare_fixture(&dir, ex, "pub fn soma(a:i32,b:i32)->i32 { a+b }").unwrap();
        let src = std::fs::read_to_string(dir.join("src/lib.rs")).unwrap();
        assert!(src.contains("pub fn soma"));
        assert!(!src.contains("fn soma_positivos"));
        let tests = std::fs::read_to_string(dir.join("tests/rustlab_validation.rs")).unwrap();
        assert!(tests.contains("fn soma_positivos"));
        assert!(tests.contains("use rustlab_exercise_soma::soma"));
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn rever_dicas_nao_revela_novas_nem_altera_contagem() {
        let (mut app, path) = app("review-hints");
        let mut state = ExerciseState::default();
        state.open_selected(&app);
        for _ in 0..2 {
            state.on_key(&mut app, KeyEvent::from(KeyCode::F(1)));
        }
        assert_eq!(state.session.as_ref().unwrap().hint_index, 1);
        state.on_key(&mut app, KeyEvent::from(KeyCode::F(2)));
        assert_eq!(state.session.as_ref().unwrap().hint_index, 0);
        for _ in 0..6 {
            state.on_key(&mut app, KeyEvent::from(KeyCode::F(3)));
        }
        let session = state.session.as_ref().unwrap();
        assert_eq!(session.hint_index, 1);
        assert_eq!(session.shown_hints, 2);
        assert_eq!(app.progress.hints_used.get(&session.id), Some(&2));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn revisao_de_saida_respeita_limites_e_nova_execucao() {
        let (mut app, _) = app("review-output");
        let mut state = ExerciseState::default();
        state.open_selected(&app);
        let session = state.session.as_mut().unwrap();
        session.output = (0..20).map(|i| format!("linha {i}")).collect();
        session.output_page_rows = 5;
        for _ in 0..8 {
            state.on_key(&mut app, KeyEvent::from(KeyCode::PageUp));
        }
        assert_eq!(state.session.as_ref().unwrap().output_scroll, 15);
        state.on_key(&mut app, KeyEvent::from(KeyCode::PageDown));
        assert_eq!(state.session.as_ref().unwrap().output_scroll, 10);
        state.on_paste("// alterado\n");
        assert_eq!(state.session.as_ref().unwrap().output_scroll, 0);
        assert!(state.session.as_ref().unwrap().output.is_empty());
    }

    #[test]
    fn falha_de_inicio_e_cancelamento_nao_concluem_exercicio() {
        let (mut app, _) = app("failed-start");
        let mut state = ExerciseState::default();
        state.open_selected(&app);
        state.handle_exec_event(
            ExecEvent::SpawnFailed {
                label: "cargo test".into(),
                reason: "Cargo indisponível".into(),
            },
            &mut app,
        );
        assert_eq!(
            state.session.as_ref().unwrap().outcome,
            ExerciseOutcome::StartFailed
        );
        assert!(
            state
                .session
                .as_ref()
                .unwrap()
                .output
                .iter()
                .any(|s| s.contains("Cargo indisponível"))
        );
        state.handle_exec_event(
            ExecEvent::Cancelled {
                label: "cargo test".into(),
            },
            &mut app,
        );
        assert_eq!(
            state.session.as_ref().unwrap().outcome,
            ExerciseOutcome::Cancelled
        );
        assert!(app.progress.exercises_completed.is_empty());
    }

    #[test]
    fn erro_de_sintaxe_sem_codigo_e_lido_apenas_do_stderr() {
        let (mut app, _) = app("syntax");
        let mut state = ExerciseState::default();
        state.open_selected(&app);
        state.handle_exec_event(
            ExecEvent::Chunk {
                kind: StreamKind::Stderr,
                text: "src/lib.rs:2:1: error: expected expression".into(),
            },
            &mut app,
        );
        // A classificação permanece mesmo quando a linha original saiu do buffer.
        for _ in 0..MAX_OUTPUT + 5 {
            state.handle_exec_event(
                ExecEvent::Chunk {
                    kind: StreamKind::Stdout,
                    text: "linha".into(),
                },
                &mut app,
            );
        }
        state.handle_exec_event(
            ExecEvent::Finished {
                label: "cargo test".into(),
                success: false,
                code: Some(101),
            },
            &mut app,
        );
        assert_eq!(
            state.session.as_ref().unwrap().outcome,
            ExerciseOutcome::CompileFailed
        );
        assert!(!is_compile_error(
            "error: test failed, to rerun pass --test rustlab_validation"
        ));
    }

    #[test]
    fn linhas_longas_e_unicode_contam_para_rolagem_visual() {
        let (app, _) = app("visual-output");
        let mut state = ExerciseState::default();
        state.open_selected(&app);
        let session = state.session.as_mut().unwrap();
        session.output_width = 10;
        session.push_output("abcdefghijabcdefghijabcdefghij".into());
        assert_eq!(session.output_line_count(), 3);
        session.output_page_rows = 1;
        session.output_scroll = 2;
        session.push_output("olá".into());
        assert_eq!(session.output_scroll, 3);
    }

    #[test]
    fn executor_ocupado_nao_rouba_processo_ou_prepara_fixture() {
        let (mut app, _) = app("busy");
        app.config.execution.consent = ExecConsent::Granted;
        app.exec.running = true;
        let mut state = ExerciseState::default();
        state.open_selected(&app);
        state.on_key(&mut app, KeyEvent::from(KeyCode::F(5)));
        let session = state.session.as_ref().unwrap();
        assert_eq!(session.outcome, ExerciseOutcome::NotRun);
        assert!(session.pending_dir.is_none());
        assert!(app.status.as_ref().unwrap().message.contains("já há"));
    }

    #[test]
    fn timeout_limpa_fixture_e_nao_registra_conclusao() {
        let (mut app, _) = app("timeout");
        let mut state = ExerciseState::default();
        state.open_selected(&app);
        let dir = fresh_temp_dir("timeout-test").unwrap();
        state.session.as_mut().unwrap().pending_dir = Some(dir.clone());
        state.session.as_mut().unwrap().outcome = ExerciseOutcome::Running;
        state.handle_exec_event(
            ExecEvent::TimedOut {
                label: "cargo test".into(),
            },
            &mut app,
        );
        assert_eq!(
            state.session.as_ref().unwrap().outcome,
            ExerciseOutcome::TimedOut
        );
        assert!(!dir.exists());
        assert!(state.session.as_ref().unwrap().pending_dir.is_none());
        assert!(app.progress.exercises_completed.is_empty());
    }

    #[test]
    fn respostas_multilinha_restauram_apos_reiniciar_sem_executar_codigo() {
        let (mut app, path) = app("answers-restart");
        let mut state = ExerciseState::default();
        state.open_selected(&app);
        let id = state.session.as_ref().unwrap().id.clone();
        let code = "// resposta com acentos: olá\npub fn maior(a:i32,b:i32)->i32 { a.max(b) }\n";
        state.with_editor(Editor::select_all);
        state.on_paste(code);
        assert!(state.session.as_ref().unwrap().editor.dirty);
        state.on_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
        );
        assert!(!state.session.as_ref().unwrap().editor.dirty);
        let (loaded, warning) = crate::storage::load_from(&path);
        assert!(warning.is_none());
        assert_eq!(loaded.exercise_answers.get(&id).unwrap(), code);
        let (mut restarted_app, _) = self::app("answers-restarted");
        restarted_app.progress = crate::learn::Progress::from_data(loaded);
        let mut restarted = ExerciseState::default();
        restarted.open_selected(&restarted_app);
        assert_eq!(
            restarted
                .session
                .as_ref()
                .unwrap()
                .editor
                .buf
                .to_string_lossy(),
            code
        );
        assert_eq!(
            restarted.session.as_ref().unwrap().outcome,
            ExerciseOutcome::NotRun
        );
        assert!(!restarted_app.exec.running);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn falha_de_escrita_mantem_codigo_e_impede_sair_do_exercicio() {
        let (mut app, path) = app("answers-failure");
        // Um diretório no lugar do arquivo torna a gravação impossível.
        std::fs::create_dir_all(&path).unwrap();
        let mut state = ExerciseState::default();
        state.open_selected(&app);
        state.on_paste("// preservar\n");
        state.on_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(state.view, ExerciseView::Work);
        assert!(state.session.as_ref().unwrap().editor.dirty);
        assert!(app.status.as_ref().unwrap().message.contains("não salvei"));
        std::fs::remove_dir(&path).unwrap();
        state.on_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(state.view, ExerciseView::List);
        assert!(!state.session.as_ref().unwrap().editor.dirty);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn alternar_exercicios_preserva_resposta_sem_inventar_resultado() {
        let (mut app, _) = app("draft");
        let mut state = ExerciseState::default();
        let id = app
            .curriculum
            .as_ref()
            .unwrap()
            .exercises()
            .next()
            .unwrap()
            .id
            .clone();
        app.progress.exercises_completed.insert(id);
        state.open_selected(&app);
        assert_eq!(
            state.session.as_ref().unwrap().outcome,
            ExerciseOutcome::NotRun
        );
        state.on_paste("// minha resposta\n");
        let original = state.session.as_ref().unwrap().editor.buf.to_string_lossy();
        state.selected = 1;
        state.open_selected(&app);
        state.selected = 0;
        state.open_selected(&app);
        assert_eq!(
            state.session.as_ref().unwrap().editor.buf.to_string_lossy(),
            original
        );
    }

    #[test]
    fn editar_invalida_resultado_e_nao_altera_codigo_em_execucao() {
        let (app, _) = app("invalidate");
        let mut state = ExerciseState::default();
        state.open_selected(&app);
        state.session.as_mut().unwrap().outcome = ExerciseOutcome::Passed;
        state.on_paste("// alterado\n");
        assert_eq!(
            state.session.as_ref().unwrap().outcome,
            ExerciseOutcome::NotRun
        );
        state.session.as_mut().unwrap().outcome = ExerciseOutcome::Running;
        let before = state.session.as_ref().unwrap().editor.buf.to_string_lossy();
        state.on_paste("nao inserir");
        assert_eq!(
            state.session.as_ref().unwrap().editor.buf.to_string_lossy(),
            before
        );
    }

    #[test]
    fn hints_sao_progressivos_com_teto_quatro_e_persistem() {
        let (mut app, progress_path) = app("hints");
        let mut state = ExerciseState::default();
        state.open_selected(&app);
        for _ in 0..6 {
            state.on_key(&mut app, KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE));
        }
        assert_eq!(state.session.as_ref().unwrap().shown_hints, 4);
        assert_eq!(
            app.progress
                .hints_used
                .get(&state.session.as_ref().unwrap().id),
            Some(&4)
        );
        let (data, warning) = crate::storage::load_from(&progress_path);
        assert!(warning.is_none());
        assert_eq!(
            data.hints_used.get(&state.session.as_ref().unwrap().id),
            Some(&4)
        );
        std::fs::remove_file(progress_path).ok();
    }

    #[test]
    fn consentimento_recusado_nao_inicia_cargo() {
        let (mut app, _) = app("consent");
        let mut state = ExerciseState::default();
        state.open_selected(&app);
        state.on_key(&mut app, KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE));
        assert!(state.consent_pending);
        assert!(!app.exec.running);
        state.on_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!state.consent_pending);
        assert!(!app.exec.running);
    }

    #[test]
    fn evento_de_teste_verde_persiste_exercicio_concluido() {
        let (mut app, progress_path) = app("completion");
        let mut state = ExerciseState::default();
        state.open_selected(&app);
        let id = state.session.as_ref().unwrap().id.clone();
        state.handle_exec_event(
            ExecEvent::Finished {
                label: "cargo test".into(),
                code: Some(0),
                success: true,
            },
            &mut app,
        );
        assert!(app.progress.exercises_completed.contains(&id));
        let (data, warning) = crate::storage::load_from(&progress_path);
        assert!(warning.is_none());
        assert!(data.completed_exercises.contains(&id));
        std::fs::remove_file(progress_path).ok();
    }
}
