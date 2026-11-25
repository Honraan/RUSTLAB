use std::process::Command;

#[test]
fn ajuda_e_versao_funcionam_sem_terminal() {
    for arg in ["--help", "--version"] {
        let output = Command::new(env!("CARGO_BIN_EXE_rustlab"))
            .arg(arg)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(!output.stdout.contains(&0x1b));
    }
}

#[test]
fn diagnostico_sem_toolchain_explica_falha_sem_iniciar_tui() {
    let output = Command::new(env!("CARGO_BIN_EXE_rustlab"))
        .arg("--doctor")
        .env("PATH", "")
        .env("HOME", "/rustlab-missing-home")
        .env("USERPROFILE", r"C:\rustlab-missing-home")
        .env("LOCALAPPDATA", r"C:\rustlab-missing-local")
        .env("APPDATA", r"C:\rustlab-missing-appdata")
        .env("RUSTLAB_CONFIG", "/rustlab-missing-config.toml")
        .env_remove("RUSTLAB_RUST_ANALYZER")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("[--] Cargo"));
    assert!(text.contains("https://rustup.rs"));
    assert!(text.contains("No network requests made"));
    assert!(!text.contains('\u{1b}'));
}
