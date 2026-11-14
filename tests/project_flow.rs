//! Projetos independentes: console por linhas, EOF, cancelamento e dependências.
use rustlab::execution::{ExecEvent, ExecutionManager, StreamKind, ToolCommand};
use std::time::{Duration, Instant};

fn until(
    manager: &mut ExecutionManager,
    all: &mut Vec<ExecEvent>,
    condition: impl Fn(&[ExecEvent]) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        manager.poll_events(all);
        if condition(all) {
            return;
        }
        assert!(Instant::now() < deadline, "timeout: {all:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn cargo_interativo_exibe_prompt_sem_newline_recebe_linha_e_eof_sem_vazar_para_proximo_run() {
    let root = std::env::temp_dir().join(format!("rustlab-project-flow-{}", std::process::id()));
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='interactive_fixture'\nversion='0.1.0'\nedition='2021'\n[workspace]\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/main.rs"),
        r#"
use std::io::{self, Write};
fn main() {
    print!("Seu nome? "); io::stdout().flush().unwrap();
    let mut name = String::new(); io::stdin().read_line(&mut name).unwrap();
    println!("Olá, {}!", name.trim());
    let mut end = String::new();
    assert_eq!(io::stdin().read_line(&mut end).unwrap(), 0);
    println!("EOF confirmado");
}
"#,
    )
    .unwrap();
    let mut manager = ExecutionManager::new();
    manager
        .submit(ToolCommand::RunInteractive, root.clone())
        .unwrap();
    assert!(manager.accepts_input());
    let mut all = Vec::new();
    until(&mut manager, &mut all, |events| {
        events.iter().any(|event| matches!(event, ExecEvent::Chunk { kind: StreamKind::Stdout, text } if text.contains("Seu nome?")))
    });
    assert!(manager.send_line("duas\nlinhas").is_err());
    assert!(manager.send_line(&"x".repeat(4097)).is_err());
    manager.send_line("João").unwrap();
    manager.close_input().unwrap();
    assert!(!manager.accepts_input());
    until(&mut manager, &mut all, |events| {
        events
            .iter()
            .any(|event| matches!(event, ExecEvent::Finished { .. }))
    });
    assert!(
        all.iter()
            .any(|event| matches!(event, ExecEvent::Finished { success: true, .. })),
        "{all:?}"
    );
    assert!(
        all.iter().any(
            |event| matches!(event, ExecEvent::Chunk { text, .. } if text.contains("Olá, João!"))
        ),
        "{all:?}"
    );
    assert!(
        all.iter().any(
            |event| matches!(event, ExecEvent::Chunk { text, .. } if text == "EOF confirmado")
        )
    );
    assert!(!manager.accepts_input());
    manager
        .submit(ToolCommand::RunInteractive, root.clone())
        .unwrap();
    all.clear();
    until(&mut manager, &mut all, |events| {
        events.iter().any(
            |event| matches!(event, ExecEvent::Chunk { text, .. } if text.contains("Seu nome?")),
        )
    });
    manager.cancel();
    until(&mut manager, &mut all, |events| {
        events
            .iter()
            .any(|event| matches!(event, ExecEvent::Cancelled { .. }))
    });
    assert!(!manager.accepts_input());

    // cargo remove é exercitado sem downloads, com dependência local.
    std::fs::create_dir_all(root.join("local_dep/src")).unwrap();
    std::fs::write(
        root.join("local_dep/Cargo.toml"),
        "[package]\nname='local_dep'\nversion='0.1.0'\n",
    )
    .unwrap();
    std::fs::write(root.join("local_dep/src/lib.rs"), "").unwrap();
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        format!("{manifest}\n[dependencies]\nlocal_dep = {{ path = 'local_dep' }}\n"),
    )
    .unwrap();
    manager
        .submit(
            ToolCommand::RemoveDependency {
                name: "local_dep".into(),
            },
            root.clone(),
        )
        .unwrap();
    all.clear();
    until(&mut manager, &mut all, |events| {
        events
            .iter()
            .any(|event| matches!(event, ExecEvent::Finished { .. }))
    });
    assert!(
        all.iter()
            .any(|event| matches!(event, ExecEvent::Finished { success: true, .. })),
        "{all:?}"
    );
    assert!(
        !std::fs::read_to_string(root.join("Cargo.toml"))
            .unwrap()
            .contains("local_dep")
    );
    std::fs::remove_dir_all(root).unwrap();
}
