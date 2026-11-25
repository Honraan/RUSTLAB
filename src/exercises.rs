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
    pub clippy_tips: Vec<String>,
    pub show_idiomatic: bool,
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
    pub bugfix_mode: bool,
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
            bugfix_mode: false,
            drafts: Default::default(),
        }
    }
}

impl ExerciseState {
    pub fn active_exercises(&self, app: &App) -> Vec<Exercise> {
        if self.bugfix_mode {
            bugfix_challenges(app.is_english())
        } else {
            app.curriculum
                .as_ref()
                .map(|c| c.exercises().cloned().collect())
                .unwrap_or_default()
        }
    }

    pub fn resolve_exercise(&self, app: &App, id: &str) -> Option<Exercise> {
        if let Some(ex) = app.curriculum.as_ref().and_then(|c| c.exercise(id)) {
            return Some(ex.clone());
        }
        bugfix_challenges(app.is_english())
            .into_iter()
            .find(|ex| ex.id == id)
    }

    pub fn open_selected(&mut self, app: &App) {
        let list = self.active_exercises(app);
        let Some(ex) = list.get(self.selected) else {
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
                clippy_tips: Vec::new(),
                show_idiomatic: false,
            }
        }));
        self.view = ExerciseView::Work;
    }

    pub fn on_key(&mut self, app: &mut App, key: KeyEvent) {
        let en = app.is_english();
        if self.consent_pending {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('s') => {
                    self.consent_pending = false;
                    app.config.execution.consent = ExecConsent::Granted;
                    if let Err(e) = app.save_config() {
                        app.set_status(
                            if en {
                                format!("failed to save consent: {e}")
                            } else {
                                format!("não salvei consentimento: {e}")
                            },
                            StatusKind::Warning,
                        );
                    }
                    self.run_tests(app);
                }
                KeyCode::Esc | KeyCode::Char('n') => {
                    self.consent_pending = false;
                    app.set_status(
                        if en {
                            "no tests were run"
                        } else {
                            "nenhum teste foi executado"
                        },
                        StatusKind::Info,
                    );
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
                    let n = self.active_exercises(app).len();
                    self.selected = (self.selected + 1).min(n.saturating_sub(1));
                }
                KeyCode::Char('b') | KeyCode::Char('B') => {
                    self.bugfix_mode = !self.bugfix_mode;
                    self.selected = 0;
                    app.set_status(
                        match (self.bugfix_mode, en) {
                            (true, true) => "Mode: Fix the Bug micro-challenges (2 min)",
                            (true, false) => "Modo: Micro-desafios Conserte o Bug (2 min)",
                            (false, true) => "Mode: Classic exercises",
                            (false, false) => "Modo: Exercícios clássicos",
                        },
                        StatusKind::Info,
                    );
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
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let en = app.is_english();
        if key.code == KeyCode::Esc {
            if self.session.as_ref().is_some_and(|s| s.show_idiomatic) {
                if let Some(s) = self.session.as_mut() {
                    s.show_idiomatic = false;
                }
                return;
            }
            if self
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
        }
        match key.code {
            KeyCode::F(6) => {
                self.toggle_idiomatic_modal(app);
            }
            KeyCode::Char('i') if alt => {
                self.toggle_idiomatic_modal(app);
            }
            KeyCode::Char('e') if alt => {
                self.diagnose_current_error_with_mentor(app);
            }
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
                    app.set_status(
                        if en {
                            "answers saved to disk"
                        } else {
                            "respostas salvas em disco"
                        },
                        StatusKind::Success,
                    );
                }
            }
            KeyCode::Char(c) if !ctrl && !alt => self.with_editor(|e| e.insert_char(c)),
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

    fn toggle_idiomatic_modal(&mut self, app: &mut App) {
        let en = app.is_english();
        let Some(s) = self.session.as_mut() else {
            return;
        };
        if s.outcome != ExerciseOutcome::Passed {
            app.set_status(
                if en {
                    "Complete and pass all tests (F5) first to unlock idiomatic comparison."
                } else {
                    "Passe em todos os testes (F5) primeiro para liberar a comparação idiomática."
                },
                StatusKind::Warning,
            );
            return;
        }
        s.show_idiomatic = !s.show_idiomatic;
    }

    pub fn diagnose_current_error_with_mentor(&mut self, app: &mut App) {
        let en = app.is_english();
        let Some(ex) = self.exercise(app) else {
            return;
        };
        let Some(s) = self.session.as_mut() else {
            return;
        };
        s.ai_used = true;
        let output = s.output.join("\n");
        let ctx = crate::mentor::MentorContext {
            origin: format!(
                "{}: {}",
                if en { "Exercise" } else { "Exercício" },
                ex.title
            ),
            topic_title: Some(ex.title),
            summary: ex.statement,
            code: s.editor.buf.to_string_lossy(),
            errors: output.clone(),
            console: output,
            origin_language: app.config.origin_language,
            recurring_errors: app.progress.top_recurring_errors(3),
        };
        let mut mentor = crate::mentor::MentorState::open_for(Screen::Exercises, ctx, en);
        mentor.dispatch(
            app,
            crate::mentor::MentorAction::ExplainCompilerError,
            String::new(),
        );
        app.mentor = Some(mentor);
        app.screen = Screen::Mentor;
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
                s.clippy_tips.clear();
                s.show_idiomatic = false;
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
        let en = app.is_english();
        if let Err(err) = app.persist_progress() {
            app.set_status(
                if en {
                    format!("failed to save answers: {err}. Ctrl+S retries; your code stays in the editor.")
                } else {
                    format!("não salvei as respostas: {err}. Ctrl+S tenta novamente; o código continua no editor.")
                },
                StatusKind::Warning,
            );
            return Err(err);
        }
        for session in self.drafts.values_mut().chain(self.session.iter_mut()) {
            session.editor.mark_saved();
        }
        Ok(())
    }

    fn reveal_hint(&mut self, app: &mut App) {
        let en = app.is_english();
        let Some(id) = self.session.as_ref().map(|s| s.id.clone()) else {
            return;
        };
        let Some(ex) = self.resolve_exercise(app, &id) else {
            return;
        };
        let Some(s) = self.session.as_mut() else {
            return;
        };
        if s.shown_hints >= ex.hints.len() {
            app.set_status(
                if en {
                    "all available hints have already been revealed"
                } else {
                    "todas as dicas disponíveis já foram reveladas"
                },
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
                if en {
                    format!("hint shown, but progress not saved: {e}")
                } else {
                    format!("dica mostrada, mas progresso não salvo: {e}")
                },
                StatusKind::Warning,
            );
        }
    }

    fn run_tests(&mut self, app: &mut App) {
        let en = app.is_english();
        if app.exec.running {
            app.set_status(
                if en {
                    "a Cargo command is already running; please wait or press Ctrl+K"
                } else {
                    "já há um comando Cargo rodando; aguarde ou use Ctrl+K"
                },
                StatusKind::Warning,
            );
            return;
        }
        if self.save_answers(app).is_err() {
            return;
        }
        let Some(id) = self.session.as_ref().map(|s| s.id.clone()) else {
            return;
        };
        let Some(ex) = self.resolve_exercise(app, &id) else {
            return;
        };
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let dir = match fresh_temp_dir(&ex.id) {
            Ok(dir) => dir,
            Err(e) => {
                let reason = if en {
                    format!("could not create temp directory: {e}")
                } else {
                    format!("não consegui criar diretório temporário: {e}")
                };
                session.outcome = ExerciseOutcome::StartFailed;
                session.output = vec![reason.clone()];
                session.output_scroll = 0;
                app.set_status(reason, StatusKind::Warning);
                return;
            }
        };
        if let Err(e) = prepare_fixture(&dir, &ex, &session.editor.buf.to_string_lossy()) {
            let _ = std::fs::remove_dir_all(&dir);
            let reason = if en {
                format!("could not prepare tests: {e}")
            } else {
                format!("não consegui preparar testes: {e}")
            };
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
                if en {
                    "running deterministic exercise tests…"
                } else {
                    "rodando testes determinísticos do exercício…"
                },
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
        let en = app.is_english();
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
                                if en {
                                    "exercise completed"
                                } else {
                                    "exercício concluído"
                                },
                            );
                            if bonus > 0 {
                                app.award_xp(
                                    format!("exercise:{}:independent", session.id),
                                    bonus,
                                    if en {
                                        "no-hints bonus"
                                    } else {
                                        "bônus sem dicas"
                                    },
                                );
                            }
                        } else {
                            app.award_xp(
                                format!("exercise:{}", session.id),
                                30,
                                if en {
                                    "exercise completed"
                                } else {
                                    "exercício concluído"
                                },
                            );
                        }
                    }
                    app.progress.record_srs_mastery(
                        &session.id,
                        session.shown_hints == 0 && !session.ai_used,
                    );
                    let student_code = session.editor.buf.to_string_lossy();
                    let clippy = pedagogical_clippy_for(&student_code, en);
                    session.clippy_tips = clippy.clone();
                    for tip in clippy {
                        session.push_output(format!("Clippy: {tip}"));
                    }
                    if idiomatic_solution_for(&session.id, en).is_some() {
                        session.push_output(
                            if en {
                                "Unlocked: press F6 or Alt+I to compare with an idiomatic Rust solution."
                            } else {
                                "Liberado: aperte F6 ou Alt+I para comparar com uma solução idiomática."
                            }
                            .into(),
                        );
                    }
                    if let Err(e) = app.persist_progress() {
                        app.set_status(
                            if en {
                                format!("exercise passed, but progress not saved: {e}")
                            } else {
                                format!("exercício passou, mas progresso não salvo: {e}")
                            },
                            StatusKind::Warning,
                        );
                    } else {
                        app.set_status(
                            if en {
                                "tests passed; exercise completed and progress saved"
                            } else {
                                "testes passaram; exercício concluído e progresso salvo"
                            },
                            StatusKind::Success,
                        );
                    }
                } else {
                    if compile_failed {
                        let full_out = session.output.join("\n");
                        let _ = app.progress.record_compiler_errors(&full_out);
                        let _ = app.persist_progress();
                        if let Some(help) = crate::learn::explain_compiler_error_for(&full_out, en)
                        {
                            session.push_output(format!(
                                "{}: {help}",
                                if en {
                                    "Ferris explains"
                                } else {
                                    "Explicação do Ferris"
                                }
                            ));
                        }
                    }
                    let message = match (compile_failed, en) {
                        (true, true) => "code did not compile; see diagnostics",
                        (true, false) => "o código não compilou; veja o diagnóstico",
                        (false, true) => "one or more tests failed; see output",
                        (false, false) => "um ou mais testes falharam; veja a saída",
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
                    if en {
                        "Time limit reached; process terminated. Check for infinite loops."
                    } else {
                        "Tempo limite atingido; processo encerrado. Verifique loops sem fim."
                    }
                    .into(),
                );
                session.output_scroll = 0;
                app.set_status(
                    if en {
                        "tests exceeded the time limit"
                    } else {
                        "testes excederam o limite de tempo"
                    },
                    StatusKind::Warning,
                );
                if let Some(dir) = session.pending_dir.take() {
                    let _ = std::fs::remove_dir_all(dir);
                }
            }
            ExecEvent::Cancelled { .. } => {
                session.outcome = ExerciseOutcome::Cancelled;
                session.push_output(
                    if en {
                        "Execution cancelled by user; no new result was recorded."
                    } else {
                        "Execução cancelada pelo usuário; nenhum novo resultado foi registrado."
                    }
                    .into(),
                );
                session.output_scroll = 0;
                app.set_status(
                    if en {
                        "tests cancelled"
                    } else {
                        "testes cancelados"
                    },
                    StatusKind::Info,
                );
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

    pub fn exercise(&self, app: &App) -> Option<Exercise> {
        let id = &self.session.as_ref()?.id;
        self.resolve_exercise(app, id)
    }
}

pub fn bugfix_for_error_code(code: &str, english: bool) -> Exercise {
    let all = bugfix_challenges(english);
    let idx = match code.trim() {
        "E0384" => 0,
        "E0382" => 1,
        "E0502" => 2,
        "E0106" | "E0597" => 3,
        "E0004" => 4,
        "E0308" => 5,
        "E0599" | "E0277" => 6,
        _ => 7,
    };
    all.into_iter().nth(idx).unwrap()
}

pub fn bugfix_challenges(english: bool) -> Vec<Exercise> {
    vec![
        Exercise {
            id: "bugfix-mutability".to_string(),
            order: 1001,
            lesson_ids: vec!["mutabilidade".to_string()],
            title: if english {
                "[Bugfix E0384] Accumulator cannot mutate"
            } else {
                "[Bugfix E0384] Acumulador não consegue mutar"
            }
            .to_string(),
            statement: if english {
                "Fix the compiler error E0384 in `sum_until` so `total` can be updated inside the `for` loop."
            } else {
                "Corrija o erro de compilação E0384 em `sum_until` para que `total` possa ser atualizado no laço `for`."
            }
            .to_string(),
            skeleton: "pub fn sum_until(n: i32) -> i32 {\n    let total = 0;\n    for i in 1..=n {\n        total += i;\n    }\n    total\n}\n".to_string(),
            tests: "use super::sum_until;\n#[test]\nfn check_sum() {\n    assert_eq!(sum_until(4), 10);\n    assert_eq!(sum_until(0), 0);\n}\n".to_string(),
            hints: if english {
                vec![
                    "Read the E0384 diagnostic: variables in Rust are immutable by default.".into(),
                    "Add `mut` to the `let total = 0;` binding.".into(),
                ]
            } else {
                vec![
                    "Leia o diagnóstico E0384: variáveis em Rust são imutáveis por padrão.".into(),
                    "Adicione `mut` na declaração `let total = 0;`.".into(),
                ]
            },
        },
        Exercise {
            id: "bugfix-move-clone".to_string(),
            order: 1002,
            lesson_ids: vec!["ownership-na-pratica".to_string()],
            title: if english {
                "[Bugfix E0382] String moved before second use"
            } else {
                "[Bugfix E0382] String movida antes do segundo uso"
            }
            .to_string(),
            statement: if english {
                "Fix the move error E0382 without changing the `label_pair` public signature."
            } else {
                "Corrija o erro de move E0382 sem alterar a assinatura pública de `label_pair`."
            }
            .to_string(),
            skeleton: "fn measure(s: String) -> usize {\n    s.len()\n}\n\npub fn label_pair(name: String) -> (usize, String) {\n    let len = measure(name);\n    let tag = format!(\"{name}:{len}\");\n    (len, tag)\n}\n".to_string(),
            tests: "use super::label_pair;\n#[test]\nfn check_pair() {\n    assert_eq!(label_pair(\"rust\".to_string()), (4, \"rust:4\".to_string()));\n}\n".to_string(),
            hints: if english {
                vec![
                    "`measure` takes `String` by value and drops it, so `name` is moved.".into(),
                    "Change `measure(s: &str)` and call `measure(&name)` (or clone carefully).".into(),
                ]
            } else {
                vec![
                    "`measure` recebe `String` por valor e move `name`.".into(),
                    "Mude o helper privado para `measure(s: &str)` e chame `measure(&name)`.".into(),
                ]
            },
        },
        Exercise {
            id: "bugfix-borrow-conflict".to_string(),
            order: 1003,
            lesson_ids: vec!["borrowing-e-referencias".to_string()],
            title: if english {
                "[Bugfix E0502] Mutable & immutable borrow overlap"
            } else {
                "[Bugfix E0502] Borrow mutável e imutável ao mesmo tempo"
            }
            .to_string(),
            statement: if english {
                "Reorder or extract the read of `items[0]` so `items.push(extra)` does not conflict with the active shared reference."
            } else {
                "Reordene ou copie o valor de `items[0]` para que `items.push(extra)` não conflite com a referência compartilhada ativa."
            }
            .to_string(),
            skeleton: "pub fn append_first(items: &mut Vec<i32>, extra: i32) -> i32 {\n    let first = &items[0];\n    items.push(extra);\n    *first + extra\n}\n".to_string(),
            tests: "use super::append_first;\n#[test]\nfn check_append_first() {\n    let mut v = vec![10, 20];\n    assert_eq!(append_first(&mut v, 5), 15);\n    assert_eq!(v, vec![10, 20, 5]);\n}\n".to_string(),
            hints: if english {
                vec![
                    "Copying `i32` (`let first = items[0];`) avoids holding `&items[0]` across `items.push`.".into(),
                    "Use `let first = items[0];` and return `first + extra`.".into(),
                ]
            } else {
                vec![
                    "Como `i32` é `Copy`, `let first = items[0];` evita segurar `&items[0]` durante `items.push`.".into(),
                    "Use `let first = items[0];` e retorne `first + extra`.".into(),
                ]
            },
        },
        Exercise {
            id: "bugfix-lifetime-ret".to_string(),
            order: 1004,
            lesson_ids: vec!["tempo-de-vida-introducao".to_string()],
            title: if english {
                "[Bugfix E0106] Missing lifetime annotation `'a`"
            } else {
                "[Bugfix E0106] Lifetime `'a` faltando no retorno"
            }
            .to_string(),
            statement: if english {
                "Add a shared lifetime parameter `'a` so `longest_str` tells `rustc` how long the returned `&str` lives."
            } else {
                "Adicione o parâmetro de lifetime `'a` em `longest_str` para informar ao `rustc` quanto tempo a fatia retornada vive."
            }
            .to_string(),
            skeleton: "pub fn longest_str(a: &str, b: &str) -> &str {\n    if a.len() >= b.len() { a } else { b }\n}\n".to_string(),
            tests: "use super::longest_str;\n#[test]\nfn check_longest() {\n    assert_eq!(longest_str(\"rust\", \"go\"), \"rust\");\n    assert_eq!(longest_str(\"hi\", \"cargo\"), \"cargo\");\n}\n".to_string(),
            hints: if english {
                vec![
                    "When returning a reference chosen from two input slices, elision cannot guess which lifetime applies.".into(),
                    "Write `pub fn longest_str<'a>(a: &'a str, b: &'a str) -> &'a str`.".into(),
                ]
            } else {
                vec![
                    "Ao retornar uma referência escolhida entre duas entradas `&str`, a elisão não sabe qual lifetime usar.".into(),
                    "Escreva `pub fn longest_str<'a>(a: &'a str, b: &'a str) -> &'a str`.".into(),
                ]
            },
        },
        Exercise {
            id: "bugfix-match-exhaustive".to_string(),
            order: 1005,
            lesson_ids: vec!["enums-e-pattern-matching".to_string()],
            title: if english {
                "[Bugfix E0004] Non-exhaustive match on enum"
            } else {
                "[Bugfix E0004] match não exaustivo em enum"
            }
            .to_string(),
            statement: if english {
                "Handle the missing `Light::Yellow` arm inside `action_label` so `match` covers all variants."
            } else {
                "Trate a variante `Light::Yellow` que faltou em `action_label` para tornar o `match` exaustivo."
            }
            .to_string(),
            skeleton: "pub enum Light {\n    Red,\n    Yellow,\n    Green,\n}\n\npub fn action_label(light: Light) -> &'static str {\n    match light {\n        Light::Red => \"stop\",\n        Light::Green => \"go\",\n    }\n}\n".to_string(),
            tests: "use super::{Light, action_label};\n#[test]\nfn check_lights() {\n    assert_eq!(action_label(Light::Red), \"stop\");\n    assert_eq!(action_label(Light::Yellow), \"caution\");\n    assert_eq!(action_label(Light::Green), \"go\");\n}\n".to_string(),
            hints: if english {
                vec![
                    "Check the test expectations: `Light::Yellow` must return `\"caution\"`.".into(),
                    "Add `Light::Yellow => \"caution\",` inside the `match`.".into(),
                ]
            } else {
                vec![
                    "Veja o que os testes esperam: `Light::Yellow` deve retornar `\"caution\"`.".into(),
                    "Adicione `Light::Yellow => \"caution\",` no `match`.".into(),
                ]
            },
        },
        Exercise {
            id: "bugfix-result-qmark".to_string(),
            order: 1006,
            lesson_ids: vec!["result-e-propagacao".to_string()],
            title: if english {
                "[Bugfix E0308] Propagate Result with `?` before adding"
            } else {
                "[Bugfix E0308] Propague Result com `?` antes de somar"
            }
            .to_string(),
            statement: if english {
                "Fix `parse_and_double` so `s.parse::<i32>()` propagates errors cleanly via `?` and wraps the sum in `Ok(...)`."
            } else {
                "Corrija `parse_and_double` usando `?` em `s.parse::<i32>()` e envolvendo o resultado em `Ok(...)`."
            }
            .to_string(),
            skeleton: "use std::num::ParseIntError;\n\npub fn parse_and_double(s: &str) -> Result<i32, ParseIntError> {\n    let n = s.trim().parse::<i32>();\n    n * 2\n}\n".to_string(),
            tests: "use super::parse_and_double;\n#[test]\nfn check_parse_double() {\n    assert_eq!(parse_and_double(\" 21 \"), Ok(42));\n    assert!(parse_and_double(\"abc\").is_err());\n}\n".to_string(),
            hints: if english {
                vec![
                    "`parse::<i32>()` returns `Result<i32, ParseIntError>`, which cannot be multiplied by 2 directly.".into(),
                    "Use `let n = s.trim().parse::<i32>()?;` and return `Ok(n * 2)`.".into(),
                ]
            } else {
                vec![
                    "`parse::<i32>()` retorna `Result<i32, ParseIntError>`, que não pode ser multiplicado por 2.".into(),
                    "Use `let n = s.trim().parse::<i32>()?;` e retorne `Ok(n * 2)`.".into(),
                ]
            },
        },
        Exercise {
            id: "bugfix-trait-impl".to_string(),
            order: 1007,
            lesson_ids: vec!["traits-modulos-e-cargo".to_string()],
            title: if english {
                "[Bugfix E0277] Missing `Summary` trait impl"
            } else {
                "[Bugfix E0277] Falta implementar trait `Summary`"
            }
            .to_string(),
            statement: if english {
                "Implement the `Summary` trait for `Article` so `render_summary(&article)` compiles."
            } else {
                "Implemente o trait `Summary` para `Article` para que `render_summary(&article)` compile."
            }
            .to_string(),
            skeleton: "pub trait Summary {\n    fn summarize(&self) -> String;\n}\n\npub struct Article {\n    pub title: String,\n}\n\npub fn render_summary<T: Summary>(item: &T) -> String {\n    format!(\"[Summary] {}\", item.summarize())\n}\n".to_string(),
            tests: "use super::{Article, render_summary};\n#[test]\nfn check_summary() {\n    let a = Article { title: \"Rust 2024\".into() };\n    assert_eq!(render_summary(&a), \"[Summary] Rust 2024\");\n}\n".to_string(),
            hints: if english {
                vec![
                    "`render_summary` requires `T: Summary`, so `Article` needs `impl Summary for Article`.".into(),
                    "Return `self.title.clone()` inside `fn summarize(&self) -> String`.".into(),
                ]
            } else {
                vec![
                    "`render_summary` exige `T: Summary`, logo `Article` precisa de `impl Summary for Article`.".into(),
                    "Retorne `self.title.clone()` dentro de `fn summarize(&self) -> String`.".into(),
                ]
            },
        },
        Exercise {
            id: "bugfix-vec-bounds".to_string(),
            order: 1008,
            lesson_ids: vec!["colecoes-e-iteracao".to_string()],
            title: if english {
                "[Bugfix Index] Off-by-one panic on empty or last index"
            } else {
                "[Bugfix Índice] Pânico off-by-one em fatia"
            }
            .to_string(),
            statement: if english {
                "Fix `safe_last` so it returns `Option<i32>` without indexing out of bounds on empty slices."
            } else {
                "Corrija `safe_last` para retornar `Option<i32>` sem estourar o índice em fatias vazias."
            }
            .to_string(),
            skeleton: "pub fn safe_last(nums: &[i32]) -> Option<i32> {\n    Some(nums[nums.len()])\n}\n".to_string(),
            tests: "use super::safe_last;\n#[test]\nfn check_safe_last() {\n    assert_eq!(safe_last(&[]), None);\n    assert_eq!(safe_last(&[7, 9]), Some(9));\n}\n".to_string(),
            hints: if english {
                vec![
                    "`nums[nums.len()]` is always out of bounds, and `nums` might be empty!".into(),
                    "Slices already have `nums.last().copied()`.".into(),
                ]
            } else {
                vec![
                    "`nums[nums.len()]` sempre sai dos limites, e `nums` pode estar vazio!".into(),
                    "Fatias já oferecem `nums.last().copied()`.".into(),
                ]
            },
        },
    ]
}

/// Sugestões idiomáticas suaves pós-vitória ("Clippy Pedagógico", sem descontar XP).
pub fn pedagogical_clippy_for(code: &str, english: bool) -> Vec<String> {
    let mut tips = Vec::new();
    let trimmed = code.trim();
    if trimmed.contains("for ") && trimmed.contains(".len()") {
        tips.push(if english {
            "Prefer iterating directly (`for item in slice`) or using iterator combinators (`.iter().filter(...).sum()`) instead of indexing with `0..vec.len()`."
        } else {
            "Prefira iterar diretamente (`for item in slice`) ou usar combinadores (`.iter().filter(...).sum()`) em vez de indexar `0..vec.len()`."
        }.to_string());
    }
    if trimmed.contains(".clone()") {
        tips.push(if english {
            "Check whether borrowing (`&T`) or `.copied()` eliminates `.clone()` cleanly with zero heap allocation."
        } else {
            "Verifique se empréstimo (`&T`) ou `.copied()` elimina o `.clone()` sem alocar na Heap."
        }.to_string());
    }
    if trimmed.contains("return ") {
        tips.push(if english {
            "In Rust, the final expression without `;` is returned automatically — omitting redundant `return` makes short functions cleaner."
        } else {
            "Em Rust, a última expressão sem `;` já é o retorno — omitir `return` redundante deixa funções curtas mais idiomáticas."
        }.to_string());
    }
    if trimmed.contains(".unwrap()") {
        tips.push(if english {
            "Prefer `?`, `.unwrap_or(...)`, or pattern matching (`if let`) over `.unwrap()` to keep edge cases panic-free."
        } else {
            "Prefira `?`, `.unwrap_or(...)` ou `if let` em vez de `.unwrap()` para evitar panics em entradas inválidas."
        }.to_string());
    }
    if tips.is_empty()
        && trimmed.contains("if ")
        && trimmed.contains("true")
        && trimmed.contains("false")
    {
        tips.push(if english {
            "When a condition evaluates to `bool`, return the boolean expression directly instead of `if cond { true } else { false }`."
        } else {
            "Quando a expressão já resulta em `bool`, retorne a condição diretamente em vez de `if cond { true } else { false }`."
        }.to_string());
    }
    tips.truncate(2);
    tips
}

/// Solução idiomática comentada (liberada APENAS após todos os testes passarem).
pub fn idiomatic_solution_for(id: &str, english: bool) -> Option<(&'static str, &'static str)> {
    Some(match (id, english) {
        ("soma", true) => (
            "pub fn soma(a: i32, b: i32) -> i32 {\n    a + b\n}",
            "Single expression without `;` returns directly with zero boilerplate.",
        ),
        ("soma", false) => (
            "pub fn soma(a: i32, b: i32) -> i32 {\n    a + b\n}",
            "Expressão única sem `;` retorna diretamente sem `return` explícito.",
        ),
        ("dobro", true) => (
            "pub fn dobro(n: i32) -> i32 {\n    n * 2\n}",
            "Clean tail expression without mutable locals.",
        ),
        ("dobro", false) => (
            "pub fn dobro(n: i32) -> i32 {\n    n * 2\n}",
            "Expressão final limpa sem variáveis mutáveis.",
        ),
        ("media", true) => (
            "pub fn media(a: f64, b: f64) -> f64 {\n    (a + b) / 2.0\n}",
            "Parentheses enforce precedence before float division by `2.0`.",
        ),
        ("media", false) => (
            "pub fn media(a: f64, b: f64) -> f64 {\n    (a + b) / 2.0\n}",
            "Parênteses garantem a soma antes da divisão em `f64`.",
        ),
        ("paridade", true) => (
            "pub fn eh_par(n: i32) -> bool {\n    n % 2 == 0\n}",
            "Boolean expression directly without `if ... { true } else { false }`.",
        ),
        ("paridade", false) => (
            "pub fn eh_par(n: i32) -> bool {\n    n % 2 == 0\n}",
            "Retorna a expressão booleana direto, sem `if ... { true } else { false }`.",
        ),
        ("maior", true) => (
            "pub fn maior(a: i32, b: i32) -> i32 {\n    a.max(b)\n}",
            "Standard library `Ord::max` communicates intent clearly.",
        ),
        ("maior", false) => (
            "pub fn maior(a: i32, b: i32) -> i32 {\n    a.max(b)\n}",
            "Usar `a.max(b)` da std deixa a intenção imediata e idiomática.",
        ),
        ("intervalo", true) => (
            "pub fn no_intervalo(n: i32, minimo: i32, maximo: i32) -> bool {\n    minimo <= maximo && (minimo..=maximo).contains(&n)\n}",
            "Inclusive range `(minimo..=maximo).contains(&n)` prevents boundary off-by-one bugs.",
        ),
        ("intervalo", false) => (
            "pub fn no_intervalo(n: i32, minimo: i32, maximo: i32) -> bool {\n    minimo <= maximo && (minimo..=maximo).contains(&n)\n}",
            "Intervalo inclusivo `(minimo..=maximo).contains(&n)` evita erros de borda.",
        ),
        ("contar-vogais", true) => (
            "pub fn contar_vogais(texto: &str) -> usize {\n    texto\n        .chars()\n        .filter(|c| matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u'))\n        .count()\n}",
            "Zero-allocation iterator pipeline using `matches!`.",
        ),
        ("contar-vogais", false) => (
            "pub fn contar_vogais(texto: &str) -> usize {\n    texto\n        .chars()\n        .filter(|c| matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u'))\n        .count()\n}",
            "Pipeline de iterador sem alocações combinado com `matches!`.",
        ),
        ("filtrar-positivos", true) => (
            "pub fn filtrar_positivos(nums: &[i32]) -> Vec<i32> {\n    nums.iter().copied().filter(|&x| x > 0).collect()\n}",
            "`.copied()` converts `&i32` items cleanly in one iterator pass.",
        ),
        ("filtrar-positivos", false) => (
            "pub fn filtrar_positivos(nums: &[i32]) -> Vec<i32> {\n    nums.iter().copied().filter(|&x| x > 0).collect()\n}",
            "`.copied()` converte `&i32` em uma única passada com `.filter()` e `.collect()`.",
        ),
        ("primeira-palavra", true) => (
            "pub fn primeira_palavra(frase: &str) -> &str {\n    frase.split_whitespace().next().unwrap_or(\"\")\n}",
            "Borrows directly from `frase` as `&str` with zero Heap allocation.",
        ),
        ("primeira-palavra", false) => (
            "pub fn primeira_palavra(frase: &str) -> &str {\n    frase.split_whitespace().next().unwrap_or(\"\")\n}",
            "Retorna fatia emprestada `&str` direto de `frase` sem alocar `String`.",
        ),
        ("retangulo-area", true) => (
            "impl Retangulo {\n    pub fn area(&self) -> u32 {\n        self.largura * self.altura\n    }\n    pub fn eh_quadrado(&self) -> bool {\n        self.largura > 0 && self.largura == self.altura\n    }\n}",
            "Shared `&self` borrows read fields without taking ownership.",
        ),
        ("retangulo-area", false) => (
            "impl Retangulo {\n    pub fn area(&self) -> u32 {\n        self.largura * self.altura\n    }\n    pub fn eh_quadrado(&self) -> bool {\n        self.largura > 0 && self.largura == self.altura\n    }\n}",
            "Métodos com `&self` leem os campos sem mover a struct.",
        ),
        ("divisao-segura", true) => (
            "pub fn divisao_segura(a: i32, b: i32) -> Option<i32> {\n    if b == 0 { None } else { Some(a / b) }\n}",
            "Uses `Option<i32>` to rule out division-by-zero panics at the type level.",
        ),
        ("divisao-segura", false) => (
            "pub fn divisao_segura(a: i32, b: i32) -> Option<i32> {\n    if b == 0 { None } else { Some(a / b) }\n}",
            "`Option<i32>` torna impossível esquecer o divisor zero.",
        ),
        ("soma-quadrados-pares", true) => (
            "pub fn soma_quadrados_pares(nums: &[i32]) -> i32 {\n    nums.iter().copied().filter(|n| n % 2 == 0).map(|n| n * n).sum()\n}",
            "Functional iterator chain compiles down to a single tight loop.",
        ),
        ("soma-quadrados-pares", false) => (
            "pub fn soma_quadrados_pares(nums: &[i32]) -> i32 {\n    nums.iter().copied().filter(|n| n % 2 == 0).map(|n| n * n).sum()\n}",
            "Cadeia de iteradores que compila para um único loop otimizado sem alocação.",
        ),
        ("bugfix-mutability", true) => (
            "pub fn sum_until(n: i32) -> i32 {\n    (1..=n).sum()\n}",
            "Even cleaner than `let mut total`: range `.sum()` works for any `n` (returning 0 when `n <= 0`).",
        ),
        ("bugfix-mutability", false) => (
            "pub fn sum_until(n: i32) -> i32 {\n    (1..=n).sum()\n}",
            "Ainda mais enxuto que `let mut total`: `(1..=n).sum()` retorna 0 naturalmente quando `n <= 0`.",
        ),
        ("bugfix-move-clone", true) => (
            "fn measure(s: &str) -> usize {\n    s.len()\n}\npub fn label_pair(name: String) -> (usize, String) {\n    let len = measure(&name);\n    let tag = format!(\"{name}:{len}\");\n    (len, tag)\n}",
            "Accepting `&str` instead of `String` lets callers borrow without losing ownership or cloning.",
        ),
        ("bugfix-move-clone", false) => (
            "fn measure(s: &str) -> usize {\n    s.len()\n}\npub fn label_pair(name: String) -> (usize, String) {\n    let len = measure(&name);\n    let tag = format!(\"{name}:{len}\");\n    (len, tag)\n}",
            "Receber `&str` em vez de `String` permite emprestar sem mover posse nem clonar.",
        ),
        ("bugfix-borrow-conflict", true) => (
            "pub fn append_first(items: &mut Vec<i32>, extra: i32) -> i32 {\n    let first = items[0];\n    items.push(extra);\n    first + extra\n}",
            "Copying a `Copy` scalar (`i32`) ends the borrow immediately before `Vec::push`.",
        ),
        ("bugfix-borrow-conflict", false) => (
            "pub fn append_first(items: &mut Vec<i32>, extra: i32) -> i32 {\n    let first = items[0];\n    items.push(extra);\n    first + extra\n}",
            "Copiar um valor `Copy` (`i32`) encerra o borrow antes de chamar `Vec::push`.",
        ),
        ("bugfix-lifetime-ret", true) => (
            "pub fn longest_str<'a>(a: &'a str, b: &'a str) -> &'a str {\n    if a.len() >= b.len() { a } else { b }\n}",
            "`'a` promises the returned slice lives as long as the shorter of `a` and `b`.",
        ),
        ("bugfix-lifetime-ret", false) => (
            "pub fn longest_str<'a>(a: &'a str, b: &'a str) -> &'a str {\n    if a.len() >= b.len() { a } else { b }\n}",
            "`'a` garante que a fatia retornada vive ao menos tanto quanto a menor entrada.",
        ),
        ("bugfix-match-exhaustive", true) => (
            "pub enum Light { Red, Yellow, Green }\npub fn action_label(light: Light) -> &'static str {\n    match light {\n        Light::Red => \"stop\",\n        Light::Yellow => \"caution\",\n        Light::Green => \"go\",\n    }\n}",
            "Listing every variant explicitly lets `rustc` warn you immediately if a new variant is added later.",
        ),
        ("bugfix-match-exhaustive", false) => (
            "pub enum Light { Red, Yellow, Green }\npub fn action_label(light: Light) -> &'static str {\n    match light {\n        Light::Red => \"stop\",\n        Light::Yellow => \"caution\",\n        Light::Green => \"go\",\n    }\n}",
            "Cobrir cada variante explicitamente faz o compilador avisar se uma nova variante surgir no futuro.",
        ),
        ("bugfix-result-qmark", true) => (
            "use std::num::ParseIntError;\npub fn parse_and_double(s: &str) -> Result<i32, ParseIntError> {\n    Ok(s.trim().parse::<i32>()? * 2)\n}",
            "`?` propagates `ParseIntError` early and keeps the happy path concise.",
        ),
        ("bugfix-result-qmark", false) => (
            "use std::num::ParseIntError;\npub fn parse_and_double(s: &str) -> Result<i32, ParseIntError> {\n    Ok(s.trim().parse::<i32>()? * 2)\n}",
            "`?` propaga `ParseIntError` antecipadamente deixando o caminho feliz enxuto.",
        ),
        ("bugfix-trait-impl", true) => (
            "pub trait Summary {\n    fn summarize(&self) -> String;\n}\npub struct Article {\n    pub title: String,\n}\nimpl Summary for Article {\n    fn summarize(&self) -> String {\n        self.title.clone()\n    }\n}\npub fn render_summary<T: Summary>(item: &T) -> String {\n    format!(\"[Summary] {}\", item.summarize())\n}",
            "Explicit `impl Trait for Type` enables static dispatch `<T: Summary>` everywhere.",
        ),
        ("bugfix-trait-impl", false) => (
            "pub trait Summary {\n    fn summarize(&self) -> String;\n}\npub struct Article {\n    pub title: String,\n}\nimpl Summary for Article {\n    fn summarize(&self) -> String {\n        self.title.clone()\n    }\n}\npub fn render_summary<T: Summary>(item: &T) -> String {\n    format!(\"[Summary] {}\", item.summarize())\n}",
            "`impl Trait for Type` explícito habilita despacho estático `<T: Summary>` em qualquer função.",
        ),
        ("bugfix-vec-bounds", true) => (
            "pub fn safe_last(nums: &[i32]) -> Option<i32> {\n    nums.last().copied()\n}",
            "`slice::last()` + `.copied()` eliminates indexing panics entirely.",
        ),
        ("bugfix-vec-bounds", false) => (
            "pub fn safe_last(nums: &[i32]) -> Option<i32> {\n    nums.last().copied()\n}",
            "`slice::last().copied()` elimina risco de panic por índice inválido.",
        ),
        _ => return None,
    })
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

pub fn is_compile_error(text: &str) -> bool {
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

pub fn fresh_temp_dir(id: &str) -> std::io::Result<PathBuf> {
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
            crate::config::Config {
                language: "pt-BR".to_string(),
                ..crate::config::Config::default()
            },
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

    #[test]
    fn bugfix_clippy_pedagogico_e_solucao_idiomatica_funcionam() {
        let bugfixes_en = bugfix_challenges(true);
        let bugfixes_pt = bugfix_challenges(false);
        assert_eq!(bugfixes_en.len(), 8);
        assert_eq!(bugfixes_pt.len(), 8);
        for ex in &bugfixes_en {
            assert!(
                idiomatic_solution_for(&ex.id, true).is_some(),
                "missing idiomatic EN for {}",
                ex.id
            );
            assert!(
                idiomatic_solution_for(&ex.id, false).is_some(),
                "missing idiomatic PT for {}",
                ex.id
            );
            assert!(!ex.tests.trim().is_empty());
        }
        assert_eq!(bugfix_for_error_code("E0382", true).id, "bugfix-move-clone");
        assert_eq!(bugfix_for_error_code("E0384", true).id, "bugfix-mutability");

        let tips = pedagogical_clippy_for(
            "fn f(v: Vec<i32>) -> usize { for i in 0..v.len() { let _ = v[i].clone(); } return v.len(); }",
            true,
        );
        assert!(!tips.is_empty());
        assert!(tips.len() <= 2);

        let (mut app, _) = app("bugfix-flow");
        let mut state = ExerciseState::default();
        state.on_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE),
        );
        assert!(state.bugfix_mode);
        state.open_selected(&app);
        // Bloqueado antes de passar nos testes
        state.on_key(&mut app, KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE));
        assert!(!state.session.as_ref().unwrap().show_idiomatic);

        // Desbloqueado após passar
        state.handle_exec_event(
            ExecEvent::Finished {
                label: "cargo test".into(),
                code: Some(0),
                success: true,
            },
            &mut app,
        );
        state.on_key(&mut app, KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE));
        assert!(state.session.as_ref().unwrap().show_idiomatic);

        // Diagnóstico em 1 clique Alt+E abre o Mentor com erro compilado
        state.handle_exec_event(
            ExecEvent::Chunk {
                kind: StreamKind::Stderr,
                text: "error[E0384]: cannot assign twice to immutable variable".into(),
            },
            &mut app,
        );
        state.on_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::ALT),
        );
        assert_eq!(app.screen, Screen::Mentor);
        assert!(app.mentor.is_some());
    }
}
