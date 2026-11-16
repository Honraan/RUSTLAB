//! Diagnóstico local: não envia código, consulta modelos nem grava configurações.

use std::io::{self, Write};
use std::process::{Command, Stdio};

use crate::config::{self, AiProvider, RemoteAiConsent};

fn available(command: &str, args: &[&str]) -> bool {
    Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Retorna se Cargo e rustc estão disponíveis para as práticas.
pub fn report(out: &mut impl Write) -> io::Result<bool> {
    writeln!(out, "RustLab — diagnóstico local / local diagnostics")?;
    let cargo = available("cargo", &["--version"]);
    let rustc = available("rustc", &["--version"]);
    for (name, ok, hint) in [
        ("Cargo", cargo, "instale Rust: https://rustup.rs"),
        ("rustc", rustc, "instale Rust: https://rustup.rs"),
        (
            "rust-analyzer (opcional / optional)",
            crate::lsp::find_rust_analyzer().is_some(),
            "rustup component add rust-analyzer; fallback: índice local",
        ),
        (
            "curl / HTTPS (opcional / optional)",
            available("curl", &["--version"]),
            "instale curl e adicione ao PATH para usar IA via HTTPS",
        ),
    ] {
        if ok {
            writeln!(out, "[OK] {name}")?;
        } else {
            writeln!(out, "[--] {name}: {hint}")?;
        }
    }
    let loaded = config::load();
    if loaded.warning.is_some() {
        writeln!(
            out,
            "[--] Configuração: usando padrões; confira o arquivo TOML."
        )?;
    }
    match loaded.config.ai.provider {
        AiProvider::Disabled => writeln!(out, "[OK] Mentor: guia offline / offline guide")?,
        AiProvider::Ollama => writeln!(
            out,
            "[INFO] Mentor: Ollama; servidor e modelos não consultados / not queried."
        )?,
        AiProvider::Remote => {
            let key_present = ["RUSTLAB_AI_API_KEY", "OPENAI_API_KEY"]
                .iter()
                .any(|name| std::env::var(name).is_ok_and(|value| !value.trim().is_empty()));
            writeln!(
                out,
                "[INFO] Mentor: API remota / remote API; token: {}; consentimento: {}.",
                if key_present { "presente" } else { "ausente" },
                if loaded.config.ai.remote_consent == RemoteAiConsent::Granted {
                    "concedido"
                } else {
                    "pendente ou negado"
                }
            )?;
        }
    }
    writeln!(
        out,
        "Nenhuma requisição de rede realizada / No network requests made."
    )?;
    Ok(cargo && rustc)
}
