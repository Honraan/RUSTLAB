//! Fluxo real da Fase 4: consentimento, testes, diagnóstico e retomada.
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rustlab::{
    app::App,
    exercises::{ExerciseOutcome, ExerciseState},
    storage,
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

fn key(state: &mut ExerciseState, app: &mut App, code: KeyCode) {
    state.on_key(app, KeyEvent::new(code, KeyModifiers::NONE));
}

fn wait(state: &mut ExerciseState, app: &mut App) {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let mut events = Vec::new();
        app.exec.poll_events(&mut events);
        for event in events {
            state.handle_exec_event(event, app);
        }
        if !app.exec.running {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "Cargo não terminou: {:?}",
            state.session
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        state.session.as_ref().unwrap().pending_dir.is_none(),
        "fixture não foi limpo"
    );
}

fn replace(state: &mut ExerciseState, app: &mut App, source: &str) {
    state.on_key(
        app,
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
    );
    state.on_paste(source);
}

#[test]
fn consentimento_falha_compilacao_sucesso_e_retomada_com_cargo_real() {
    let root = std::env::temp_dir().join(format!("rustlab-flow-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path: PathBuf = root.join("progresso.toml");
    let mut app = App::for_snapshot();
    app.set_storage_path(path.clone());
    app.set_config_path(root.join("config.toml"));
    let mut state = ExerciseState::default();
    state.open_selected(&app);
    assert_eq!(state.session.as_ref().unwrap().id, "soma");

    key(&mut state, &mut app, KeyCode::F(5));
    assert!(state.consent_pending);
    assert!(!app.exec.running);
    key(&mut state, &mut app, KeyCode::Esc);
    assert!(!app.exec.running);
    key(&mut state, &mut app, KeyCode::F(5));
    key(&mut state, &mut app, KeyCode::Enter);
    assert!(app.exec.running);
    let fixture = state.session.as_ref().unwrap().pending_dir.clone().unwrap();
    wait(&mut state, &mut app);
    assert!(!fixture.exists());
    assert_eq!(
        state.session.as_ref().unwrap().outcome,
        ExerciseOutcome::TestsFailed
    );
    assert!(!app.progress.exercises_completed.contains("soma"));

    // Erro de sintaxe sem código E0xxx, antes tratado como teste falho.
    replace(
        &mut state,
        &mut app,
        "pub fn soma(a: i32, b: i32) -> i32 { a + }",
    );
    key(&mut state, &mut app, KeyCode::F(5));
    wait(&mut state, &mut app);
    assert_eq!(
        state.session.as_ref().unwrap().outcome,
        ExerciseOutcome::CompileFailed
    );

    let solution = "pub fn soma(a: i32, b: i32) -> i32 { a + b }\n";
    replace(&mut state, &mut app, solution);
    key(&mut state, &mut app, KeyCode::F(1));
    key(&mut state, &mut app, KeyCode::F(5));
    wait(&mut state, &mut app);
    assert_eq!(
        state.session.as_ref().unwrap().outcome,
        ExerciseOutcome::Passed
    );
    assert_eq!(app.xp(), 30, "a primeira aprovação concede XP uma vez");
    let (data, warning) = storage::load_from(&path);
    assert!(warning.is_none());
    assert!(data.completed_exercises.contains("soma"));
    assert_eq!(data.xp_events.get("exercise:soma"), Some(&30));
    assert_eq!(data.exercise_answers["soma"], solution);
    assert_eq!(data.hints_used["soma"], 1);

    let mut restarted = App::for_snapshot();
    restarted.progress = rustlab::learn::Progress::from_data(data);
    restarted.set_storage_path(path);
    let mut resumed = ExerciseState::default();
    resumed.open_selected(&restarted);
    assert_eq!(
        resumed
            .session
            .as_ref()
            .unwrap()
            .editor
            .buf
            .to_string_lossy(),
        solution
    );
    assert_eq!(
        resumed.session.as_ref().unwrap().outcome,
        ExerciseOutcome::NotRun
    );
    assert_eq!(resumed.session.as_ref().unwrap().shown_hints, 1);
    assert!(!restarted.exec.running);

    // Cancelar uma nova tentativa não apaga a conclusão histórica.
    replace(
        &mut resumed,
        &mut restarted,
        "pub fn soma(_:i32,_:i32)->i32 { loop { std::thread::sleep(std::time::Duration::from_millis(10)); } }",
    );
    restarted.config.execution.consent = rustlab::config::ExecConsent::Granted;
    key(&mut resumed, &mut restarted, KeyCode::F(5));
    resumed.on_key(
        &mut restarted,
        KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL),
    );
    wait(&mut resumed, &mut restarted);
    assert_eq!(
        resumed.session.as_ref().unwrap().outcome,
        ExerciseOutcome::Cancelled
    );
    assert!(restarted.progress.exercises_completed.contains("soma"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn pratica_embutida_bp01_executa_cargo_real_e_premia_100_mais_100_xp() {
    let root = std::env::temp_dir().join(format!("rustlab-bp01-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("progresso.toml");

    let mut app = App::for_snapshot();
    app.set_config_path(root.join("config.toml"));
    app.set_storage_path(path.clone());
    app.config.execution.consent = rustlab::config::ExecConsent::Granted;

    rustlab::learn::open_lesson(&mut app, 0, 3); // hello-world -> B-P01
    app.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    assert!(app.lesson_view.as_ref().unwrap().practice_focused);

    let wait_lesson = |app: &mut App| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        let mut events = Vec::new();
        while app.exec.running && std::time::Instant::now() < deadline {
            app.exec.poll_events(&mut events);
            for ev in events.drain(..) {
                rustlab::learn::on_lesson_exec_event(app, ev);
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        app.exec.poll_events(&mut events);
        for ev in events {
            rustlab::learn::on_lesson_exec_event(app, ev);
        }
        assert!(!app.exec.running, "execução Cargo excedeu o tempo");
    };

    // 1. Código inalterado (placeholder `"..."`) deve rodar no Cargo real e ser rejeitado pela validação
    app.handle_input(KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE));
    wait_lesson(&mut app);
    let view = app.lesson_view.as_ref().unwrap();
    let p = view.practice.as_ref().unwrap();
    assert_eq!(p.outcome, rustlab::learn::PracticeOutcome::ValidationFailed);
    assert_eq!(view.ferris, rustlab::learn::FerrisMood::Confused);
    assert_eq!(app.xp(), 0);

    // 2. Código válido do estudante executa via `cargo run` real e concede 100 + 100 XP sem dicas
    {
        let v = app.lesson_view.as_mut().unwrap();
        let p = v.practice.as_mut().unwrap();
        p.editor = rustlab::editor::Editor::new(
            "fn main() {\n    println!(\"Olá, RustLab V3!\");\n}\n",
            None,
        );
    }
    app.handle_input(KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE));
    wait_lesson(&mut app);
    let view = app.lesson_view.as_ref().unwrap();
    let p = view.practice.as_ref().unwrap();
    assert_eq!(p.outcome, rustlab::learn::PracticeOutcome::Passed);
    assert_eq!(view.ferris, rustlab::learn::FerrisMood::Happy);
    assert_eq!(app.xp(), 200);
    assert_eq!(app.progress.xp_events.get("practice:B-P01"), Some(&100));
    assert_eq!(
        app.progress.xp_events.get("practice:B-P01:independent"),
        Some(&100)
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn pratica_embutida_cargo_test_e_srs_funcionam_end_to_end() {
    let root = std::env::temp_dir().join(format!("rustlab-bt09p-srs-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("progresso.toml");

    let mut app = App::for_snapshot();
    app.set_config_path(root.join("config.toml"));
    app.set_storage_path(path.clone());
    app.config.execution.consent = rustlab::config::ExecConsent::Granted;

    let (ci, li) = {
        let cur = app.curriculum.as_ref().unwrap();
        cur.chapters
            .iter()
            .enumerate()
            .find_map(|(ci, ch)| {
                ch.lessons
                    .iter()
                    .position(|id| id == "mutabilidade")
                    .map(|li| (ci, li))
            })
            .expect("lição mutabilidade deve existir no currículo")
    };
    rustlab::learn::open_lesson(&mut app, ci, li);
    app.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    assert!(app.lesson_view.as_ref().unwrap().practice_focused);

    let wait_lesson = |app: &mut App| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        let mut events = Vec::new();
        while app.exec.running && std::time::Instant::now() < deadline {
            app.exec.poll_events(&mut events);
            for ev in events.drain(..) {
                rustlab::learn::on_lesson_exec_event(app, ev);
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        app.exec.poll_events(&mut events);
        for ev in events {
            rustlab::learn::on_lesson_exec_event(app, ev);
        }
        assert!(!app.exec.running, "execução Cargo excedeu o tempo");
    };

    // 1. Código sem `mut` deve falhar compilação (E0384) e receber explicação pt-BR sem consumir dicas/XP
    {
        let v = app.lesson_view.as_mut().unwrap();
        let p = v.practice.as_mut().unwrap();
        assert_eq!(p.spec.id, "B-T09-P");
        p.editor = rustlab::editor::Editor::new(
            "pub fn incrementar(valor: i32, passo: i32) -> i32 {\n    valor += passo;\n    valor\n}\n",
            None,
        );
    }
    app.handle_input(KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE));
    wait_lesson(&mut app);
    {
        let view = app.lesson_view.as_ref().unwrap();
        let p = view.practice.as_ref().unwrap();
        assert_eq!(p.outcome, rustlab::learn::PracticeOutcome::CompileFailed);
        assert_eq!(view.ferris, rustlab::learn::FerrisMood::Encouraging);
        assert_eq!(p.shown_hints, 0);
        let joined = p.output.join("\n");
        assert!(
            joined.contains("Explicação do Ferris:"),
            "esperava explicação offline pt-BR, obteve:\n{joined}"
        );
        assert!(
            joined.contains("E0384"),
            "esperava menção a E0384:\n{joined}"
        );
    }
    assert_eq!(app.xp(), 0);

    // 2. Solução válida com `mut` passa no `cargo test` real, concede 100+100 XP e salva SrsCard em progresso.toml
    {
        let v = app.lesson_view.as_mut().unwrap();
        let p = v.practice.as_mut().unwrap();
        p.editor = rustlab::editor::Editor::new(
            "pub fn incrementar(mut valor: i32, passo: i32) -> i32 {\n    valor += passo;\n    valor\n}\n",
            None,
        );
    }
    app.handle_input(KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE));
    wait_lesson(&mut app);
    {
        let view = app.lesson_view.as_ref().unwrap();
        let p = view.practice.as_ref().unwrap();
        assert_eq!(p.outcome, rustlab::learn::PracticeOutcome::Passed);
        assert_eq!(view.ferris, rustlab::learn::FerrisMood::Happy);
    }
    assert_eq!(app.xp(), 200);
    assert_eq!(
        app.progress
            .srs_cards
            .get("B-T09-P")
            .map(|c| c.mastery_level),
        Some(1)
    );

    let (loaded, warning) = rustlab::storage::load_from(&path);
    assert!(warning.is_none(), "warning inesperado: {warning:?}");
    let saved_card = loaded
        .srs_cards
        .get("B-T09-P")
        .expect("cartão SRS B-T09-P deve persistir em progresso.toml");
    assert_eq!(saved_card.mastery_level, 1);
    assert_eq!(saved_card.independent_passes, 1);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn pratica_interativa_stdin_d09_p01_executa_cargo_real() {
    let root = std::env::temp_dir().join(format!("rustlab-d09-p01-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let mut app = App::for_snapshot();
    app.set_config_path(root.join("config.toml"));
    app.set_storage_path(root.join("progresso.toml"));
    app.config.execution.consent = rustlab::config::ExecConsent::Granted;

    rustlab::learn::open_lesson(&mut app, 0, 0);
    let spec = rustlab::learn::interactive_practice_spec_d09();
    let solucao = concat!(
        "use std::io::{self, BufRead};\n",
        "fn main() {\n",
        "    for linha in io::stdin().lock().lines() {\n",
        "        let op = linha.unwrap_or_default();\n",
        "        match op.trim() {\n",
        "            \"1\" => println!(\"Ajuda\"),\n",
        "            \"2\" => println!(\"Pontos: 10\"),\n",
        "            \"0\" => {\n",
        "                println!(\"Saindo\");\n",
        "                break;\n",
        "            }\n",
        "            _ => {}\n",
        "        }\n",
        "    }\n",
        "}\n",
    );
    {
        let view = app.lesson_view.as_mut().unwrap();
        view.practice = Some(rustlab::learn::LessonPractice {
            spec,
            editor: rustlab::editor::Editor::new(solucao, None),
            shown_hints: 0,
            hint_index: 0,
            hint_scroll: 0,
            ai_used: false,
            outcome: rustlab::learn::PracticeOutcome::NotRun,
            output: Vec::new(),
            stdout_lines: Vec::new(),
            output_scroll: 0,
            compile_error_seen: false,
            consent_pending: false,
            pending_dir: None,
        });
        view.practice_focused = true;
    }

    app.handle_input(KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    let mut events = Vec::new();
    while app.exec.running && std::time::Instant::now() < deadline {
        app.exec.poll_events(&mut events);
        for ev in events.drain(..) {
            rustlab::learn::on_lesson_exec_event(&mut app, ev);
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    app.exec.poll_events(&mut events);
    for ev in events {
        rustlab::learn::on_lesson_exec_event(&mut app, ev);
    }
    assert!(!app.exec.running, "execução interativa excedeu o tempo");

    let view = app.lesson_view.as_ref().unwrap();
    let p = view.practice.as_ref().unwrap();
    assert_eq!(
        p.outcome,
        rustlab::learn::PracticeOutcome::Passed,
        "saída obtida:\n{}",
        p.output.join("\n")
    );
    assert_eq!(view.ferris, rustlab::learn::FerrisMood::Happy);
    assert_eq!(app.xp(), 200);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn templates_capstone_v3_compilam_e_passam_cargo_test_real() {
    let root =
        std::env::temp_dir().join(format!("rustlab-capstones-v3-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();

    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"rustlab_capstones_v3\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\ndoctest = false\n\n[workspace]\n",
    )
    .unwrap();
    let lib_src = format!(
        "#![allow(dead_code)]\n\npub mod pong {{\n{}\n}}\n\npub mod snake {{\n{}\n}}\n\npub mod editor {{\n{}\n}}\n\npub mod paint {{\n{}\n}}\n\npub mod cavegame {{\n{}\n}}\n",
        rustlab::doc_hint::pong_terminal_starter_template(),
        rustlab::doc_hint::snake_terminal_starter_template(),
        rustlab::doc_hint::mini_editor_starter_template(),
        rustlab::doc_hint::mini_paint_starter_template(),
        rustlab::doc_hint::cavegame_voxel_starter_template(),
    );
    std::fs::write(root.join("src/lib.rs"), lib_src).unwrap();

    let mut exec = rustlab::execution::ExecutionManager::new();
    exec.submit(rustlab::execution::ToolCommand::Test, root.clone())
        .unwrap();

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(90);
    let mut events = Vec::new();
    let mut output_log = String::new();
    let mut finished_ok = None;
    while exec.running && std::time::Instant::now() < deadline {
        exec.poll_events(&mut events);
        for ev in events.drain(..) {
            match ev {
                rustlab::execution::ExecEvent::Chunk { text, .. } => {
                    output_log.push_str(&text);
                    output_log.push('\n');
                }
                rustlab::execution::ExecEvent::Finished { success, .. } => {
                    finished_ok = Some(success);
                }
                rustlab::execution::ExecEvent::TimedOut { .. }
                | rustlab::execution::ExecEvent::Cancelled { .. }
                | rustlab::execution::ExecEvent::SpawnFailed { .. } => {
                    finished_ok = Some(false);
                }
                rustlab::execution::ExecEvent::Started { .. } => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    exec.poll_events(&mut events);
    for ev in events {
        match ev {
            rustlab::execution::ExecEvent::Chunk { text, .. } => {
                output_log.push_str(&text);
                output_log.push('\n');
            }
            rustlab::execution::ExecEvent::Finished { success, .. } => {
                finished_ok = Some(success);
            }
            _ => {}
        }
    }
    assert_eq!(
        finished_ok,
        Some(true),
        "capstones V3 falharam no cargo test:\n{output_log}"
    );
    assert!(
        output_log.contains("5 passed"),
        "esperava 5 testes de capstones aprovados:\n{output_log}"
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn templates_ecossistemas_modernos_e_quests_v3_compilam_no_cargo_real() {
    let root = std::env::temp_dir().join(format!("rustlab-labtools-real-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();

    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"rustlab_modern_quests\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\ndoctest = false\n\n[workspace]\n",
    )
    .unwrap();

    let mut lib_src = String::from("#![allow(dead_code)]\n\n");
    for (idx, eco) in rustlab::lab_tools::MODERN_ECOSYSTEMS.iter().enumerate() {
        lib_src.push_str(&format!(
            "pub mod eco_{idx} {{\n{}\n}}\n\n",
            eco.template_code
        ));
    }
    for (idx, quest) in rustlab::lab_tools::TROUBLESHOOTING_QUESTS
        .iter()
        .enumerate()
    {
        lib_src.push_str(&format!(
            "pub mod quest_{idx} {{\n{}\n}}\n\n",
            quest.fixed_reference
        ));
    }
    std::fs::write(root.join("src/lib.rs"), lib_src).unwrap();

    let mut exec = rustlab::execution::ExecutionManager::new();
    exec.submit(rustlab::execution::ToolCommand::Test, root.clone())
        .unwrap();

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(90);
    let mut events = Vec::new();
    let mut output_log = String::new();
    let mut finished_ok = None;
    while exec.running && std::time::Instant::now() < deadline {
        exec.poll_events(&mut events);
        for ev in events.drain(..) {
            match ev {
                rustlab::execution::ExecEvent::Chunk { text, .. } => {
                    output_log.push_str(&text);
                    output_log.push('\n');
                }
                rustlab::execution::ExecEvent::Finished { success, .. } => {
                    finished_ok = Some(success);
                }
                rustlab::execution::ExecEvent::TimedOut { .. }
                | rustlab::execution::ExecEvent::Cancelled { .. }
                | rustlab::execution::ExecEvent::SpawnFailed { .. } => {
                    finished_ok = Some(false);
                }
                rustlab::execution::ExecEvent::Started { .. } => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    exec.poll_events(&mut events);
    for ev in events {
        if let rustlab::execution::ExecEvent::Finished { success, .. } = ev {
            finished_ok = Some(success);
        }
    }
    assert_eq!(
        finished_ok,
        Some(true),
        "templates/quests falharam no cargo test real:\n{output_log}"
    );
    assert!(
        output_log.contains("3 passed"),
        "esperava 3 testes dos ecossistemas modernos aprovados:\n{output_log}"
    );

    let _ = std::fs::remove_dir_all(root);
}
