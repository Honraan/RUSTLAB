use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rustlab_desktop_lib::state::{AppState, InnerState};
use rustlab_desktop_lib::{
    curriculum_cmd, libraries_cmd, mentor_cmd, migration_cmd, practice_cmd, workspace_cmd,
};

fn test_state(tag: &str) -> (AppState, PathBuf) {
    let dir =
        std::env::temp_dir().join(format!("rustlab-desktop-test-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let dir = rustlab::projects::normalize_display_path(&std::fs::canonicalize(&dir).unwrap());
    let cfg = rustlab::config::Config::desktop_defaults();
    let cur = rustlab::curriculum::Curriculum::embedded_for(false).unwrap();
    let inner = InnerState {
        config: cfg,
        config_warning: None,
        progress: rustlab::learn::Progress::default(),
        progress_warning: None,
        curriculum: cur,
        exec: rustlab::execution::ExecutionManager::new(),
        task_history: Vec::new(),
        library_reverts: std::collections::HashMap::new(),
        pty_sessions: std::collections::HashMap::new(),
        lsp: None,
        data_root: dir.clone(),
        config_path: dir.join("config.toml"),
        progress_path: dir.join("progresso.toml"),
    };
    (
        AppState {
            inner: Arc::new(Mutex::new(inner)),
        },
        dir,
    )
}

#[test]
fn curso_tem_213_entradas_em_pt_br_e_bt01_real() {
    let (st, dir) = test_state("213");
    let ov = curriculum_cmd::get_course_overview(&st, false).unwrap();
    assert_eq!(ov.total_entries, 213);
    assert_eq!(ov.stages.len(), 6);
    let lesson = curriculum_cmd::get_lesson_detail(&st, "B-T01").unwrap();
    assert_eq!(lesson.id, "B-T01");
    assert!(lesson.title.contains("programar"));
    assert!(!lesson.blocks.is_empty());
    let quiz = curriculum_cmd::submit_quiz_answer(&st, "B-T01", 0, 0);
    assert!(quiz.is_ok());
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn pratica_cargo_real_bp01_e_d09_com_100_mais_100_xp() {
    let (st, dir) = test_state("practice-real");
    // Sem consentimento deve recusar
    let denied = practice_cmd::run_practice_check_core(
        None,
        &st,
        "practice",
        "B-P01",
        "fn main() { println!(\"Olá\"); }",
        false,
    );
    assert!(denied.is_err());

    migration_cmd::grant_exec_consent(&st).unwrap();
    let res = practice_cmd::run_practice_check_core(
        None,
        &st,
        "practice",
        "B-P01",
        "fn main() {\n    println!(\"Olá, RustLab Desktop!\");\n}\n",
        false,
    )
    .unwrap();
    assert_eq!(res.outcome, "passed", "output: {:?}", res.output_lines);
    assert_eq!(res.xp_awarded, 100);
    assert_eq!(res.independence_bonus, 100);
    assert_eq!(res.total_xp, 200);

    // Erro de compilação retorna diagnóstico e explicação do Ferris
    let err_res = practice_cmd::run_practice_check_core(
        None,
        &st,
        "practice",
        "B-P02",
        "fn main() {\n    let pontos = 0;\n    pontos = 5;\n    println!(\"{pontos}\");\n}\n",
        false,
    )
    .unwrap();
    assert_eq!(err_res.outcome, "compile-failed");
    assert!(err_res.ferris_explanation.is_some());

    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn seguranca_caminhos_unicode_e_allowlist_url() {
    let (st, dir) = test_state("sec-unicode");
    let base = dir.join("projetos usuário");
    std::fs::create_dir_all(&base).unwrap();
    let info = workspace_cmd::create_project(&st, &base.to_string_lossy(), "app_café_ok", "hello");
    // Nome de crate deve seguir alfabeto ASCII (sanitizado)
    assert!(info.is_err());
    let ok_info =
        workspace_cmd::create_project(&st, &base.to_string_lossy(), "app_cafe_1", "hello").unwrap();
    assert!(ok_info.trusted);
    assert!(workspace_cmd::read_project_file(&ok_info.root_dir, "../fora.rs").is_err());
    assert!(workspace_cmd::create_project(&st, r"C:\Program Files", "evil", "hello").is_err());
    assert!(migration_cmd::validate_safe_external_url("https://doc.rust-lang.org/std/").is_ok());
    assert!(migration_cmd::validate_safe_external_url("file:///C:/Windows/win.ini").is_err());
    assert!(migration_cmd::validate_safe_external_url("https://evil.example.com/").is_err());
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn f9_preview_apply_e_revert_funcionam() {
    let (st, dir) = test_state("f9");
    let proj =
        workspace_cmd::create_project(&st, &dir.to_string_lossy(), "f9_proj", "hello").unwrap();
    let preview = libraries_cmd::preview_add_library(&st, &proj.root_dir, "rand", false).unwrap();
    assert!(preview.cargo_toml_after.contains("rand"));
    assert!(preview.target_rs_after.contains("use rand::Rng;"));
    let applied = libraries_cmd::apply_add_library(&st, &proj.root_dir, "rand", false).unwrap();
    let toml_now = workspace_cmd::read_project_file(&proj.root_dir, "Cargo.toml").unwrap();
    assert!(toml_now.content.contains("rand"));
    libraries_cmd::revert_library_change(&st, &applied.revert_id).unwrap();
    let toml_reverted = workspace_cmd::read_project_file(&proj.root_dir, "Cargo.toml").unwrap();
    assert!(!toml_reverted.content.contains("rand"));
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn migracao_backup_v3_e_idempotencia_xp() {
    let (st, dir) = test_state("mig");
    let legacy_toml = dir.join("tui_progresso.toml");
    std::fs::write(
        &legacy_toml,
        "version = 1\ncompleted_lessons = ['B-T01']\ncompleted_exercises = ['soma']\n[xp_events]\n'lesson:B-T01' = 20\n'exercise:soma' = 30\n",
    )
    .unwrap();
    let prev = migration_cmd::preview_import_backup(&st, &legacy_toml.to_string_lossy()).unwrap();
    assert_eq!(prev.incoming_xp, 50);
    let sum1 =
        migration_cmd::apply_import_backup(&st, &legacy_toml.to_string_lossy(), "merge").unwrap();
    assert_eq!(sum1.xp, 50);
    // Reimportar não duplica XP
    let sum2 =
        migration_cmd::apply_import_backup(&st, &legacy_toml.to_string_lossy(), "merge").unwrap();
    assert_eq!(sum2.xp, 50);

    let export_path = dir.join("backup.json");
    migration_cmd::export_backup(&st, &export_path.to_string_lossy()).unwrap();
    assert!(export_path.is_file());
    std::fs::remove_dir_all(dir).ok();
}

#[tokio::test]
async fn mentor_learn_mode_bloqueia_codigo() {
    let (st, dir) = test_state("mentor");
    let reply = mentor_cmd::mentor_ask(
        &st,
        mentor_cmd::MentorAskRequest {
            learn_mode: true,
            action: "explain-error".into(),
            origin: "Lição B-P02".into(),
            topic_title: Some("Variáveis".into()),
            summary: "Mutabilidade".into(),
            code: "let x = 1; x = 2;".into(),
            errors: "error[E0384]: cannot assign twice".into(),
            console: String::new(),
            question: "Como resolvo?".into(),
        },
    )
    .await
    .unwrap();
    assert!(reply.read_only_enforced);
    assert!(reply.proposed_code.is_none());
    assert!(!reply.body.contains("```"));
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn windows_conpty_real_executa_stdin_e_stdout() {
    use portable_pty::{CommandBuilder, PtySize, native_pty_system};
    use std::io::{Read, Write};
    use std::time::{Duration, Instant};

    let pty = native_pty_system();
    let pair = pty
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let mut cmd = CommandBuilder::new("cmd.exe");
    // Mantém o processo filho vivo lendo stdin até o ConPTY drenar "CONPTY_OK" no pipe de saída.
    cmd.args(["/Q", "/C", "echo CONPTY_OK& set /p _x="]);
    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().unwrap();
    let mut writer = pair.master.take_writer().unwrap();
    let (seen_tx, seen_rx) = std::sync::mpsc::channel::<()>();
    let reader_handle = std::thread::spawn(move || {
        let mut out = String::new();
        let mut buf = [0u8; 1024];
        let mut notified = false;
        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break;
            }
            out.push_str(&String::from_utf8_lossy(&buf[..n]));
            if !notified && out.contains("CONPTY_OK") {
                notified = true;
                let _ = seen_tx.send(());
            }
        }
        out
    });

    // Responde imediatamente ao DSR (\x1b[6n) exigido por PSEUDOCONSOLE_INHERIT_CURSOR no Windows ConPTY.
    let _ = writer.write_all(b"\x1b[1;1R");
    let _ = writer.flush();

    let _ = seen_rx.recv_timeout(Duration::from_secs(10));

    let _ = writer.write_all(b"ok\r\n");
    let _ = writer.flush();
    drop(writer);

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(25));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break;
            }
        }
    }

    // Em ConPTY, dropar MasterPty chama ClosePseudoConsole e fecha o pipe para o reader encerrar.
    drop(pair.master);

    let out = reader_handle.join().unwrap();
    assert!(out.contains("CONPTY_OK"), "ConPTY output: {out:?}");
}

#[test]
fn templates_std_compilam_e_passam_cargo_test_real() {
    let (st, dir) = test_state("tpls-std");
    migration_cmd::grant_exec_consent(&st).unwrap();
    for tpl in ["hello", "pong", "snake", "editor"] {
        let proj =
            workspace_cmd::create_project(&st, &dir.to_string_lossy(), &format!("proj_{tpl}"), tpl)
                .unwrap();
        let res = workspace_cmd::run_workspace_cargo(None, &st, &proj.root_dir, "test").unwrap();
        assert_eq!(
            res.status, "ok",
            "template {tpl} falhou no cargo test: {:?}",
            res.output_lines
        );
    }
    std::fs::remove_dir_all(dir).ok();
}
