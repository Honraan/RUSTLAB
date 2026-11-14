use std::path::PathBuf;
use std::time::{Duration, Instant};

use rustlab::execution::{ExecEvent, ExecutionManager, ToolCommand};

fn cargo() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let path = home.join(".cargo/bin/cargo");
    path.is_file().then_some(path)
}

fn wait(manager: &mut ExecutionManager, limit: Duration) -> Vec<ExecEvent> {
    let end = Instant::now() + limit;
    let mut all = Vec::new();
    while Instant::now() < end {
        manager.poll_events(&mut all);
        if !manager.running {
            return all;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("execução não terminou no prazo; eventos: {all:?}");
}

#[test]
fn cargo_real_check_e_run_capturam_saida_e_diagnostico() {
    let Some(cargo) = cargo() else {
        eprintln!("skip: cargo não instalado");
        return;
    };
    // Executor usa PATH; adiciona o diretório somente neste processo de teste.
    let parent = cargo.parent().unwrap();
    let path = format!(
        "{}:{}",
        parent.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    // SAFETY: teste single-thread nesta integração.
    unsafe {
        std::env::set_var("PATH", path);
    }
    let root =
        std::env::temp_dir().join(format!("rustlab-cargo-integration-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='0.1.0'\nedition='2021'\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/main.rs"),
        "fn main() { let _: u32 = \"bad\"; }\n",
    )
    .unwrap();
    let mut manager = ExecutionManager::new();
    manager
        .submit(
            ToolCommand::NewProject {
                name: "generated".into(),
            },
            root.clone(),
        )
        .unwrap();
    let generated = wait(&mut manager, Duration::from_secs(30));
    assert!(
        generated
            .iter()
            .any(|e| matches!(e, ExecEvent::Finished { success: true, .. }))
    );
    assert!(root.join("generated/Cargo.toml").is_file());
    manager.submit(ToolCommand::Check, root.clone()).unwrap();
    let failed = wait(&mut manager, Duration::from_secs(60));
    assert!(
        failed
            .iter()
            .any(|e| matches!(e, ExecEvent::Finished { success: false, .. }))
    );
    let stderr = failed
        .iter()
        .filter_map(|e| match e {
            ExecEvent::Chunk {
                kind: rustlab::execution::StreamKind::Stderr,
                text,
            } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(stderr.contains("src/main.rs:1:"), "stderr: {stderr}");
    std::fs::write(
        root.join("src/main.rs"),
        "fn main() { println!(\"Hello, world!\"); }\n",
    )
    .unwrap();
    manager.submit(ToolCommand::Run, root.clone()).unwrap();
    let passed = wait(&mut manager, Duration::from_secs(60));
    assert!(
        passed
            .iter()
            .any(|e| matches!(e, ExecEvent::Finished { success: true, .. })),
        "{passed:?}"
    );
    assert!(
        passed
            .iter()
            .any(|e| matches!(e, ExecEvent::Chunk { text, .. } if text == "Hello, world!")),
        "{passed:?}"
    );
    std::fs::write(
        root.join("src/main.rs"),
        "fn main() { loop { std::thread::sleep(std::time::Duration::from_secs(1)); } }\n",
    )
    .unwrap();
    manager.submit(ToolCommand::Run, root.clone()).unwrap();
    let started_by = Instant::now() + Duration::from_secs(20);
    let mut started = false;
    while Instant::now() < started_by && !started {
        let mut events = Vec::new();
        manager.poll_events(&mut events);
        started = events
            .iter()
            .any(|e| matches!(e, ExecEvent::Started { .. }));
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(started, "cargo run não iniciou para teste de cancelamento");
    manager.cancel();
    let cancelled = wait(&mut manager, Duration::from_secs(5));
    assert!(
        cancelled
            .iter()
            .any(|e| matches!(e, ExecEvent::Cancelled { .. })),
        "{cancelled:?}"
    );
    let curriculum = rustlab::curriculum::Curriculum::embedded().unwrap();
    let exercise_dir = root.join("exercise");
    for (id, solution) in [
        ("soma", "pub fn soma(a: i32, b: i32) -> i32 { a + b }"),
        ("dobro", "pub fn dobro(n: i32) -> i32 { n * 2 }"),
        (
            "media",
            "pub fn media(a: f64, b: f64) -> f64 { (a + b) / 2.0 }",
        ),
        (
            "intervalo",
            "pub fn no_intervalo(n: i32, minimo: i32, maximo: i32) -> bool { n >= minimo && n <= maximo }",
        ),
        ("paridade", "pub fn eh_par(n: i32) -> bool { n % 2 == 0 }"),
        (
            "maior",
            "pub fn maior(a: i32, b: i32) -> i32 { if a > b { a } else { b } }",
        ),
        (
            "contar-vogais",
            "pub fn contar_vogais(texto: &str) -> usize { texto.chars().filter(|c| matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u')).count() }",
        ),
        (
            "filtrar-positivos",
            "pub fn filtrar_positivos(nums: &[i32]) -> Vec<i32> { nums.iter().copied().filter(|&n| n > 0).collect() }",
        ),
        (
            "primeira-palavra",
            "pub fn primeira_palavra(frase: &str) -> &str { frase.split_whitespace().next().unwrap_or(\"\") }",
        ),
        (
            "retangulo-area",
            "pub struct Retangulo { pub largura: u32, pub altura: u32 }\nimpl Retangulo { pub fn area(&self) -> u32 { self.largura * self.altura } pub fn eh_quadrado(&self) -> bool { self.largura > 0 && self.largura == self.altura } }",
        ),
        (
            "divisao-segura",
            "pub fn divisao_segura(a: i32, b: i32) -> Option<i32> { if b == 0 { None } else { Some(a / b) } }",
        ),
        (
            "soma-quadrados-pares",
            "pub fn soma_quadrados_pares(nums: &[i32]) -> i32 { nums.iter().copied().filter(|n| n % 2 == 0).map(|n| n * n).sum() }",
        ),
    ] {
        let exercise = curriculum.exercise(id).unwrap();
        rustlab::exercises::prepare_fixture(&exercise_dir, exercise, &exercise.skeleton).unwrap();
        manager
            .submit(ToolCommand::ExerciseTest, exercise_dir.clone())
            .unwrap();
        let failed_attempt = wait(&mut manager, Duration::from_secs(60));
        assert!(
            failed_attempt
                .iter()
                .any(|e| matches!(e, ExecEvent::Finished { success: false, .. })),
            "starter {id} must fail: {failed_attempt:?}"
        );
        rustlab::exercises::prepare_fixture(&exercise_dir, exercise, solution).unwrap();
        manager
            .submit(ToolCommand::ExerciseTest, exercise_dir.clone())
            .unwrap();
        let passed_attempt = wait(&mut manager, Duration::from_secs(60));
        assert!(
            passed_attempt
                .iter()
                .any(|e| matches!(e, ExecEvent::Finished { success: true, .. })),
            "implementation {id} must pass: {passed_attempt:?}"
        );
    }
    // Erros comuns que compilam, mas não podem concluir o exercício.
    for (id, wrong) in [
        ("dobro", "pub fn dobro(n: i32) -> i32 { n + 2 }"),
        (
            "media",
            "pub fn media(a: f64, b: f64) -> f64 { a + b / 2.0 }",
        ),
        (
            "intervalo",
            "pub fn no_intervalo(n: i32, minimo: i32, maximo: i32) -> bool { n >= minimo || n <= maximo }",
        ),
    ] {
        rustlab::exercises::prepare_fixture(&exercise_dir, curriculum.exercise(id).unwrap(), wrong)
            .unwrap();
        manager
            .submit(ToolCommand::ExerciseTest, exercise_dir.clone())
            .unwrap();
        let events = wait(&mut manager, Duration::from_secs(60));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, ExecEvent::Finished { success: false, .. })),
            "solução incorreta de {id} passou: {events:?}"
        );
    }
    let _ = std::fs::remove_dir_all(exercise_dir);
    let _ = std::fs::remove_dir_all(root);
}
