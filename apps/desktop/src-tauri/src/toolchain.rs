use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde::Serialize;

use crate::error::{CmdResult, CommandError};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsCapabilityReport {
    pub os_caption: String,
    pub os_build: String,
    pub arch: String,
    pub webview2_version: Option<String>,
    pub offline_curriculum_ready: bool,
    pub rustup_found: bool,
    pub rustc_version: Option<String>,
    pub cargo_version: Option<String>,
    pub cargo_path: Option<String>,
    pub default_host_triple: Option<String>,
    pub rustfmt_available: bool,
    pub clippy_available: bool,
    pub rust_analyzer_available: bool,
    pub msvc_linker_available: bool,
    pub gnu_linker_available: bool,
    pub windows_sdk_available: bool,
    pub can_compile_native: bool,
    pub conpty_supported: bool,
    pub recommended_action_pt: String,
    pub copyable_fixes: Vec<ToolchainFix>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolchainFix {
    pub title: String,
    pub description: String,
    pub command: Option<String>,
    pub docs_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HelloWorldCheckResult {
    pub success: bool,
    pub category: String,
    pub stdout: String,
    pub stderr: String,
    pub suggestion_pt: String,
}

fn hidden_cmd(bin: &str, args: &[&str]) -> Option<String> {
    let mut cmd = Command::new(bin);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let out = cmd.output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

fn resolve_tool(name: &str) -> Option<PathBuf> {
    let bin = if cfg!(windows) && !name.ends_with(".exe") {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let cand = dir.join(&bin);
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)?;
    let cargo_cand = home.join(".cargo").join("bin").join(&bin);
    if cargo_cand.is_file() {
        return Some(cargo_cand);
    }
    None
}

fn detect_webview2_version() -> Option<String> {
    #[cfg(windows)]
    {
        for key in [
            r"HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
            r"HKCU\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
        ] {
            if let Some(out) = hidden_cmd("reg", &["query", key, "/v", "pv"]) {
                for line in out.lines() {
                    if line.contains("pv")
                        && let Some(ver) = line.split_whitespace().last()
                        && ver.contains('.')
                    {
                        return Some(ver.to_string());
                    }
                }
            }
        }
    }
    None
}

fn detect_msvc_and_sdk() -> (bool, bool) {
    #[cfg(not(windows))]
    {
        return (false, false);
    }
    #[cfg(windows)]
    {
        let mut has_linker = resolve_tool("link").is_some() && resolve_tool("cl").is_some();
        let mut has_sdk = false;

        if !has_linker {
            let vswhere_candidates = [
                r"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe",
                r"C:\Program Files\Microsoft Visual Studio\Installer\vswhere.exe",
            ];
            for vswhere in vswhere_candidates {
                if std::path::Path::new(vswhere).is_file()
                    && let Some(install_path) = hidden_cmd(
                        vswhere,
                        &[
                            "-latest",
                            "-products",
                            "*",
                            "-requires",
                            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                            "-property",
                            "installationPath",
                        ],
                    )
                    && !install_path.trim().is_empty()
                {
                    has_linker = true;
                }
            }
        }

        for kits_dir in [
            r"C:\Program Files (x86)\Windows Kits\10\Lib",
            r"C:\Program Files\Windows Kits\10\Lib",
        ] {
            if std::path::Path::new(kits_dir).is_dir() {
                has_sdk = true;
            }
        }
        (has_linker, has_sdk)
    }
}

fn detect_gnu_linker() -> bool {
    if resolve_tool("x86_64-w64-mingw32-clang").is_some()
        || resolve_tool("x86_64-w64-mingw32-gcc").is_some()
    {
        return true;
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let cand = PathBuf::from(local)
            .join("Programs")
            .join("llvm-mingw")
            .join("bin")
            .join("x86_64-w64-mingw32-clang.exe");
        if cand.is_file() {
            return true;
        }
    }
    false
}

pub fn inspect_capabilities() -> WindowsCapabilityReport {
    let cargo_path = rustlab::execution::resolve_cargo_path();
    let cargo_bin = cargo_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "cargo".to_string());
    let rustc_bin = resolve_tool("rustc")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "rustc".to_string());
    let rustup_bin = resolve_tool("rustup")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "rustup".to_string());

    let cargo_version = if cargo_path.is_some() {
        hidden_cmd(&cargo_bin, &["--version"])
    } else {
        None
    };
    let rustc_version = hidden_cmd(&rustc_bin, &["--version"]);
    let rustup_show = hidden_cmd(&rustup_bin, &["show"]);
    let rustup_found = rustup_show.is_some();
    let default_host_triple = rustup_show.as_deref().and_then(|txt| {
        txt.lines()
            .find_map(|l| l.strip_prefix("Default host:"))
            .map(|s| s.trim().to_string())
    });

    let rustfmt_available = hidden_cmd(&cargo_bin, &["fmt", "--version"]).is_some();
    let clippy_available = hidden_cmd(&cargo_bin, &["clippy", "--version"]).is_some();
    let rust_analyzer_available = rustlab::lsp::find_rust_analyzer().is_some();
    let (msvc_linker_available, windows_sdk_available) = detect_msvc_and_sdk();
    let gnu_linker_available = detect_gnu_linker();

    let uses_gnu = default_host_triple
        .as_deref()
        .is_some_and(|t| t.contains("gnu"));
    let linker_ok = if uses_gnu {
        gnu_linker_available
    } else {
        msvc_linker_available && windows_sdk_available
    };
    let can_compile_native = cargo_version.is_some() && rustc_version.is_some() && linker_ok;

    let mut fixes = Vec::new();
    if cargo_version.is_none() || rustc_version.is_none() {
        fixes.push(ToolchainFix {
            title: "Instalar Rust e Cargo (rustup)".into(),
            description: "Baixe o instalador oficial do Rust para Windows e reinicie o RustLab depois de concluir.".into(),
            command: Some("winget install --id Rustlang.Rustup -e".into()),
            docs_url: Some("https://rust-lang.org/install".into()),
        });
    }
    if !uses_gnu && (!msvc_linker_available || !windows_sdk_available) {
        fixes.push(ToolchainFix {
            title: "Instalar Visual Studio C++ Build Tools + Windows SDK".into(),
            description: "Para o alvo padrão MSVC no Windows, o `rustc` precisa do linker `link.exe` e do Windows SDK (carga 'Desenvolvimento para desktop com C++').".into(),
            command: Some("winget install --id Microsoft.VisualStudio.2022.BuildTools --source winget --override \"--add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows11SDK.22621\"".into()),
            docs_url: Some("https://rust-lang.github.io/rustup/installation/windows-msvc.html".into()),
        });
    }
    if uses_gnu && !gnu_linker_available {
        fixes.push(ToolchainFix {
            title: "Linker MinGW/LLVM ausente para alvo -gnu/-gnullvm".into(),
            description: "Seu rustup usa alvo GNU/LLVM sem linker disponível no PATH. Instale o llvm-mingw ou mude para a toolchain stable-msvc.".into(),
            command: Some("rustup toolchain install stable-x86_64-pc-windows-msvc && rustup default stable-x86_64-pc-windows-msvc".into()),
            docs_url: Some("https://rust-lang.github.io/rustup/installation/windows-msvc.html".into()),
        });
    }
    if !rustfmt_available || !clippy_available || !rust_analyzer_available {
        fixes.push(ToolchainFix {
            title: "Componentes opcionais de qualidade e LSP".into(),
            description: "Adicione `rustfmt`, `clippy` e `rust-analyzer` para formatação, lints e autocompletar completo.".into(),
            command: Some("rustup component add rustfmt clippy rust-analyzer".into()),
            docs_url: None,
        });
    }

    let recommended_action_pt = if can_compile_native {
        "Ambiente pronto para compilar e executar projetos Rust localmente no Windows.".into()
    } else if cargo_version.is_none() || rustc_version.is_none() {
        "Rust/Cargo não detectados. Você já pode ler todas as lições e responder quizzes offline; instale o Rust quando quiser compilar código.".into()
    } else {
        "Rust detectado, mas faltam pré-requisitos de linker (C++ Build Tools / Windows SDK ou linker MinGW) para gerar executáveis.".into()
    };

    let (os_caption, os_build) = read_os_info();

    WindowsCapabilityReport {
        os_caption,
        os_build,
        arch: std::env::consts::ARCH.to_string(),
        webview2_version: detect_webview2_version(),
        offline_curriculum_ready: true,
        rustup_found,
        rustc_version,
        cargo_version,
        cargo_path: cargo_path.map(|p| p.to_string_lossy().to_string()),
        default_host_triple,
        rustfmt_available,
        clippy_available,
        rust_analyzer_available,
        msvc_linker_available,
        gnu_linker_available,
        windows_sdk_available,
        can_compile_native,
        conpty_supported: cfg!(windows),
        recommended_action_pt,
        copyable_fixes: fixes,
    }
}

fn read_os_info() -> (String, String) {
    #[cfg(windows)]
    {
        let out = hidden_cmd(
            "reg",
            &[
                "query",
                r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion",
                "/v",
                "CurrentBuildNumber",
            ],
        );
        let build = out
            .as_deref()
            .and_then(|t| {
                t.lines()
                    .find(|l| l.contains("CurrentBuildNumber"))
                    .and_then(|l| l.split_whitespace().last())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "desconhecido".to_string());
        let build_num: u32 = build.parse().unwrap_or(0);
        let caption = if build_num >= 22000 {
            "Microsoft Windows 11".to_string()
        } else {
            "Microsoft Windows 10".to_string()
        };
        (caption, build)
    }
    #[cfg(not(windows))]
    {
        (std::env::consts::OS.to_string(), String::new())
    }
}

pub fn run_hello_world_test() -> CmdResult<HelloWorldCheckResult> {
    let Some(cargo) = rustlab::execution::resolve_cargo_path() else {
        return Ok(HelloWorldCheckResult {
            success: false,
            category: "toolchain".into(),
            stdout: String::new(),
            stderr: "cargo não encontrado no sistema.".into(),
            suggestion_pt: "Instale o Rust em https://rust-lang.org/install e reabra o RustLab."
                .into(),
        });
    };
    let dir = std::env::temp_dir().join(format!("rustlab-doctor-hello-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src"))
        .map_err(|e| CommandError::io("Falha ao criar projeto temporário de teste", e))?;
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"rustlab_doctor_hello\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n",
    )
    .map_err(|e| CommandError::io("Falha ao gravar Cargo.toml", e))?;
    std::fs::write(
        dir.join("src/main.rs"),
        "fn main() {\n    println!(\"RustLab Desktop OK!\");\n}\n",
    )
    .map_err(|e| CommandError::io("Falha ao gravar src/main.rs", e))?;

    let mut cmd = Command::new(&cargo);
    cmd.args(["run", "--offline"])
        .current_dir(&dir)
        .env("CARGO_TERM_COLOR", "never")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(parent) = cargo.parent() {
        let mut paths = vec![parent.to_path_buf()];
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let mingw = PathBuf::from(local)
                .join("Programs")
                .join("llvm-mingw")
                .join("bin");
            if mingw.is_dir() {
                paths.push(mingw);
            }
        }
        if let Some(p) = std::env::var_os("PATH") {
            paths.extend(std::env::split_paths(&p));
        }
        if let Ok(joined) = std::env::join_paths(paths) {
            cmd.env("PATH", joined);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }

    let output = cmd.output();
    let _ = std::fs::remove_dir_all(&dir);

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            let success = out.status.success();
            let (category, suggestion_pt) = if success {
                (
                    "ok".to_string(),
                    "Compilação e execução funcionaram perfeitamente!".to_string(),
                )
            } else if stderr.contains("link.exe")
                || stderr.contains("linker")
                || stderr.contains("mingw")
            {
                (
                    "linker".to_string(),
                    "Erro de linker: verifique os pré-requisitos C++ Build Tools + Windows SDK (MSVC) ou o linker MinGW no PATH.".to_string(),
                )
            } else if stderr.contains("Access is denied") || stderr.contains("Acesso negado") {
                (
                    "permissions".to_string(),
                    "Permissão negada ou bloqueio de antivírus na pasta temporária. Tente adicionar exceção para seus projetos.".to_string(),
                )
            } else if stderr.contains("could not resolve") || stderr.contains("proxy") {
                (
                    "network".to_string(),
                    "Falha de rede ou proxy ao acessar crates.".to_string(),
                )
            } else {
                (
                    "toolchain".to_string(),
                    "Revise os detalhes de saída e verifique `rustup show`.".to_string(),
                )
            };
            Ok(HelloWorldCheckResult {
                success,
                category,
                stdout,
                stderr,
                suggestion_pt,
            })
        }
        Err(e) => Ok(HelloWorldCheckResult {
            success: false,
            category: "path".into(),
            stdout: String::new(),
            stderr: e.to_string(),
            suggestion_pt: "Não foi possível iniciar o Cargo. Confirme que ele está instalado e acessível no PATH.".into(),
        }),
    }
}
