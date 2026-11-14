//! Snapshot dourado das telas: renderiza em `TestBackend` e compara com
//! `tests/snapshots/*.txt`. Para regenerar após mudanças visuais legítimas:
//!
//! ```text
//! SNAPSHOTS_ATUALIZAR=1 cargo test --test ui_snapshot
//! ```

use std::path::PathBuf;

use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn dump(
    w: u16,
    h: u16,
    mut f: impl FnMut(&mut ratatui::Frame<'_>, &mut rustlab::app::App),
) -> String {
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    let mut app = test_app();
    terminal.draw(|frame| f(frame, &mut app)).unwrap();
    let mut out = String::new();
    let buffer = terminal.backend().buffer().clone();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            out.push_str(buffer[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

fn test_app() -> rustlab::app::App {
    rustlab::app::App::for_snapshot()
}

#[track_caller]
fn check(name: &str, rendered: String) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(format!("{name}.txt"));
    if std::env::var_os("SNAPSHOTS_ATUALIZAR").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &rendered).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("snapshot {name} ausente — rode SNAPSHOTS_ATUALIZAR=1"));
    // Células vazias são cheias com espaço: normaliza trailing para o diff
    // ficar legível em revisão manual.
    let norm = |s: &str| -> String {
        s.lines()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    };
    assert_eq!(
        norm(&expected),
        norm(&rendered),
        "snapshot {name} divergente"
    );
}

#[test]
fn welcome_90x30() {
    check("welcome_90x30", dump(90, 30, rustlab::ui::draw));
}

#[test]
fn menu_90x30() {
    check(
        "menu_90x30",
        dump(90, 30, |f, app| {
            app.screen = rustlab::app::Screen::Menu;
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn progress_90x30() {
    check(
        "progress_90x30",
        dump(90, 30, |f, app| {
            app.screen = rustlab::app::Screen::Progress;
            app.progress.completed.insert("hello-world".into());
            app.progress.exercises_completed.insert("soma".into());
            app.progress.hints_used.insert("soma".into(), 2);
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn settings_90x30() {
    check(
        "settings_90x30",
        dump(90, 30, |f, app| {
            app.screen = rustlab::app::Screen::Settings;
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn help_90x30() {
    check(
        "help_90x30",
        dump(90, 30, |f, app| {
            app.screen = rustlab::app::Screen::Help;
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn create_90x30() {
    check(
        "create_90x30",
        dump(90, 30, |f, app| {
            app.screen = rustlab::app::Screen::Create;
            app.create = Some(rustlab::create::CreateState::new());
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn create_workspace_key_hints_90x30() {
    check(
        "create_workspace_key_hints_90x30",
        dump(90, 30, |f, app| {
            let mut state = rustlab::create::CreateState::default();
            state.view = rustlab::create::CreateView::Workspace;
            app.create = Some(state);
            app.screen = rustlab::app::Screen::Create;
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn create_recentes_90x30() {
    check(
        "create_recentes_90x30",
        dump(90, 30, |frame, app| {
            app.screen = rustlab::app::Screen::Create;
            app.config.workspace.recent_projects =
                vec!["/projetos/calculadora".into(), "/projetos/jogo".into()];
            let mut state = rustlab::create::CreateState::new();
            state.start_sel = 3;
            app.create = Some(state);
            rustlab::ui::draw(frame, app);
        }),
    );
}

#[test]
fn create_dependencia_90x30() {
    check(
        "create_dependencia_90x30",
        dump(90, 30, |frame, app| {
            app.screen = rustlab::app::Screen::Create;
            let mut state = rustlab::create::CreateState::new();
            state.view = rustlab::create::CreateView::Workspace;
            state.prompt = Some(rustlab::create::PromptState {
                kind: rustlab::create::PromptKind::AddDependency,
                text: "serde@1".into(),
                error: None,
            });
            app.create = Some(state);
            rustlab::ui::draw(frame, app);
        }),
    );
}

#[test]
fn create_console_longo_90x30() {
    check(
        "create_console_longo_90x30",
        dump(90, 30, |frame, app| {
            app.screen = rustlab::app::Screen::Create;
            let mut state = rustlab::create::CreateState::new();
            state.view = rustlab::create::CreateView::Workspace;
            state.focus = rustlab::create::Focus::Console;
            state.console.system(
                "Mensagem de diagnóstico longa para revisar por linhas visuais. ".repeat(20)
                    + "FIM DA SAIDA",
            );
            app.create = Some(state);
            rustlab::ui::draw(frame, app);
        }),
    );
}

#[test]
#[allow(clippy::field_reassign_with_default)]
fn create_hover_documentacao_90x30() {
    check(
        "create_hover_documentacao_90x30",
        dump(90, 30, |f, app| {
            let mut state = rustlab::create::CreateState::default();
            state.view = rustlab::create::CreateView::Workspace;
            state.focus = rustlab::create::Focus::Editor;
            state.editor = rustlab::editor::Editor::new("println!(\"oi\");", None);
            state.hovered_doc = Some(("println".into(), 35, 7));
            app.create = Some(state);
            app.screen = rustlab::app::Screen::Create;
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn create_autocomplete_90x30() {
    check(
        "create_autocomplete_90x30",
        dump(90, 30, |f, app| {
            let mut state = rustlab::create::CreateState::default();
            state.view = rustlab::create::CreateView::Workspace;
            state.editor = rustlab::editor::Editor::new("pri", None);
            state.editor.move_end(false);
            state.completion.refresh(&state.editor, true);
            app.create = Some(state);
            app.screen = rustlab::app::Screen::Create;
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
#[allow(clippy::field_reassign_with_default)]
fn create_bibliotecas_90x30() {
    check(
        "create_bibliotecas_90x30",
        dump(90, 30, |f, app| {
            let mut state = rustlab::create::CreateState::default();
            state.view = rustlab::create::CreateView::Workspace;
            state.lib_menu_open = true;
            app.create = Some(state);
            app.screen = rustlab::app::Screen::Create;
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn browser_90x30() {
    check(
        "browser_90x30",
        dump(90, 30, |f, app| {
            app.screen = rustlab::app::Screen::Browser;
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn exercise_list_90x30() {
    check(
        "exercise_list_90x30",
        dump(90, 30, |f, app| {
            app.screen = rustlab::app::Screen::Exercises;
            app.exercises = Some(rustlab::exercises::ExerciseState::default());
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn exercise_work_90x30() {
    check(
        "exercise_work_90x30",
        dump(90, 30, |f, app| {
            app.screen = rustlab::app::Screen::Exercises;
            let mut state = rustlab::exercises::ExerciseState::default();
            state.open_selected(app);
            app.exercises = Some(state);
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn exercise_resposta_pendente_90x30() {
    check(
        "exercise_resposta_pendente_90x30",
        dump(90, 30, |f, app| {
            app.screen = rustlab::app::Screen::Exercises;
            let mut state = rustlab::exercises::ExerciseState::default();
            state.open_selected(app);
            state.on_paste("// resposta editada\n");
            app.exercises = Some(state);
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn exercise_revisao_90x30() {
    check(
        "exercise_revisao_90x30",
        dump(90, 30, |f, app| {
            app.screen = rustlab::app::Screen::Exercises;
            let mut state = rustlab::exercises::ExerciseState::default();
            state.open_selected(app);
            let session = state.session.as_mut().unwrap();
            session.shown_hints = 2;
            session.hint_index = 0;
            session.output = (1..=30)
                .map(|n| format!("Diagnóstico de teste: linha {n}"))
                .collect();
            session.output_scroll = 10;
            session.outcome = rustlab::exercises::ExerciseOutcome::TestsFailed;
            app.exercises = Some(state);
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn exercise_textos_longos_60x24() {
    check(
        "exercise_textos_longos_60x24",
        dump(60, 24, |f, app| {
            app.screen = rustlab::app::Screen::Exercises;
            let mut state = rustlab::exercises::ExerciseState {
                selected: 2,
                ..Default::default()
            }; // média
            state.open_selected(app);
            let session = state.session.as_mut().unwrap();
            session.shown_hints = 4;
            session.hint_index = 3;
            session.hint_scroll = 2;
            session.statement_scroll = 2;
            session.output =
                vec!["mensagem de erro muito longa: ".repeat(8) + "FIM DO DIAGNOSTICO"];
            session.outcome = rustlab::exercises::ExerciseOutcome::CompileFailed;
            app.exercises = Some(state);
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn lesson_90x30() {
    check(
        "lesson_90x30",
        dump(90, 30, |f, app| {
            rustlab::learn::open_lesson(app, 0, 0); // primeira lição, no topo
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn lesson_pratica_relacionada_90x30() {
    check(
        "lesson_pratica_relacionada_90x30",
        dump(90, 30, |f, app| {
            let (chapter, lesson, _) = app
                .curriculum
                .as_ref()
                .unwrap()
                .lesson_order()
                .into_iter()
                .find(|(_, _, id)| *id == "tipos-numericos")
                .unwrap();
            rustlab::learn::open_lesson(app, chapter, lesson);
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn catalogo_vindo_da_licao_90x30() {
    check(
        "catalogo_vindo_da_licao_90x30",
        dump(90, 30, |f, app| {
            let (chapter, lesson, _) = app
                .curriculum
                .as_ref()
                .unwrap()
                .lesson_order()
                .into_iter()
                .find(|(_, _, id)| *id == "tipos-numericos")
                .unwrap();
            rustlab::learn::open_lesson(app, chapter, lesson);
            app.handle_input(ratatui::crossterm::event::KeyEvent::from(
                ratatui::crossterm::event::KeyCode::Char('e'),
            ));
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn lesson_quiz_resolvido_90x30() {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    check(
        "lesson_quiz_90x30",
        dump(90, 30, |f, app| {
            rustlab::learn::open_lesson(app, 0, 0);
            // resolve o quiz (correta = índice 1 → tecla '2') e rola até ele
            app.handle_input(KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE));
            if let Some(v) = app.lesson_view.as_mut() {
                v.scroll = 22;
            }
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn lesson_pratica_ide_90x30() {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    check(
        "lesson_pratica_ide_90x30",
        dump(90, 30, |f, app| {
            rustlab::learn::open_lesson(app, 0, 3); // hello-world -> B-P01
            app.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn lesson_narrow_50x12() {
    check(
        "lesson_narrow_50x12",
        dump(50, 12, |f, app| {
            rustlab::learn::open_lesson(app, 0, 3);
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn narrow_50x12() {
    check(
        "narrow_50x12",
        dump(50, 12, |f, app| {
            app.screen = rustlab::app::Screen::Menu;
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn create_agente_90x30() {
    check(
        "create_agente_90x30",
        dump(90, 30, |f, app| {
            app.screen = rustlab::app::Screen::Create;
            app.config.ai.provider = rustlab::config::AiProvider::Ollama;
            app.config.ai.ollama_model = "qwen2.5-coder:1.5b".to_string();
            let mut state = rustlab::create::CreateState::new();
            state.view = rustlab::create::CreateView::Workspace;
            state.focus = rustlab::create::Focus::Agent;
            state.editor = rustlab::editor::Editor::new("fn main() {\n    todo!()\n}\n", None);
            state.agent.open = true;
            state
                .agent
                .messages
                .push(rustlab::create::AgentChatMessage {
                    role: rustlab::create::AgentRole::User,
                    sender: "Você".to_string(),
                    text: "implemente olá mundo".to_string(),
                });
            state.agent.messages.push(rustlab::create::AgentChatMessage {
                role: rustlab::create::AgentRole::Agent,
                sender: "Antigravity".to_string(),
                text: "Sugestão para `main`:\n```rust\nfn main() {\n    println!(\"Olá, RustLab!\");\n}\n```".to_string(),
            });
            state.agent.proposed_code =
                Some("fn main() {\n    println!(\"Olá, RustLab!\");\n}".to_string());
            app.create = Some(state);
            rustlab::ui::draw(f, app);
        }),
    );
}

#[test]
fn mentor_90x30() {
    check(
        "mentor_90x30",
        dump(90, 30, |f, app| {
            app.screen = rustlab::app::Screen::Menu;
            app.toggle_mentor();
            rustlab::ui::draw(f, app);
        }),
    );
}

/// Garante que o motor de animação não é tocado no caminho dos snapshots
/// (testa o próprio harness, não o app).
#[test]
fn harness_sem_animacao() {
    let app = test_app();
    assert!(!app.anim.is_running());
    assert!(!app.anim.is_running());
}
