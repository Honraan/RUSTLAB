use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};

use rustlab::config::ExecConsent;
use rustlab::execution::{DiagLevel, ExecEvent, StreamKind, ToolCommand, parse_diagnostics};
use rustlab::lsp::{
    LspClient, LspEvent, WorkspaceIndex, infer_local_inlay_hints, search_project_text,
};
use rustlab::projects::{Project, normalize_display_path, resolve_new_project_target};

use crate::error::{CmdResult, CommandError};
use crate::practice_cmd::DiagnosticDto;
use crate::state::{AppState, LspSession, TaskHistoryEntry};
use crate::templates;

static TASK_SEQ: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDto {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfoDto {
    pub name: String,
    pub root_dir: String,
    pub trusted: bool,
    pub files: Vec<String>,
    pub capabilities: ProjectCapabilitiesDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCapabilitiesDto {
    pub has_plan_md: bool,
    pub source_file_count: usize,
    pub has_structs_or_enums: bool,
    pub has_error_handling: bool,
    pub has_unit_tests: bool,
    pub has_doc_comments: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContentDto {
    pub rel_path: String,
    pub content: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveFileResultDto {
    pub saved: bool,
    pub conflict: bool,
    pub disk_sha256: String,
    pub disk_content: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHitDto {
    pub rel_path: String,
    pub line: usize,
    pub column: usize,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceExecResultDto {
    pub status: String,
    pub code: Option<i32>,
    pub output_lines: Vec<String>,
    pub diagnostics: Vec<DiagnosticDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStreamEvent {
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InlayHintDto {
    pub line: usize,
    pub col: usize,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspInspectDto {
    pub lsp_active: bool,
    pub completions: Vec<String>,
    pub inlay_hints: Vec<InlayHintDto>,
    pub hover_doc: Option<String>,
}

fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn ensure_safe_project_parent(dir: &Path) -> CmdResult<()> {
    let norm = normalize_display_path(dir)
        .to_string_lossy()
        .to_ascii_lowercase();
    for forbidden in [
        r"\program files",
        r"\program files (x86)",
        r"\windows\system32",
    ] {
        if norm.contains(forbidden) {
            return Err(CommandError::forbidden(
                "Por segurança, não crie nem abra projetos dentro de pastas do sistema ou Program Files.",
            ));
        }
    }
    Ok(())
}

pub fn get_default_projects_dir() -> String {
    rustlab::projects::default_workspace_dir()
        .to_string_lossy()
        .to_string()
}

pub fn list_project_templates() -> Vec<TemplateDto> {
    templates::list_templates()
        .into_iter()
        .map(|t| TemplateDto {
            id: t.id.to_string(),
            name: t.name.to_string(),
            description: t.description.to_string(),
        })
        .collect()
}

pub fn create_project(
    state: &AppState,
    parent_dir: &str,
    raw_name: &str,
    template_id: &str,
) -> CmdResult<ProjectInfoDto> {
    let base = if parent_dir.trim().is_empty() {
        rustlab::projects::default_workspace_dir()
    } else {
        PathBuf::from(parent_dir.trim())
    };
    let (resolved_parent, valid_name) =
        resolve_new_project_target(raw_name, &base).map_err(CommandError::invalid)?;
    ensure_safe_project_parent(&resolved_parent)?;

    let project_root = resolved_parent.join(&valid_name);
    if project_root.exists() {
        return Err(CommandError::invalid(format!(
            "A pasta `{}` já existe.",
            project_root.display()
        )));
    }
    let tmpl = templates::find_template(template_id)
        .ok_or_else(|| CommandError::not_found("Template não encontrado."))?;

    std::fs::create_dir_all(project_root.join("src"))
        .map_err(|e| CommandError::io("Falha ao criar pasta do projeto", e))?;

    let cargo_toml = format!(
        "[package]\nname = \"{valid_name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n{}",
        tmpl.cargo_toml_extra
    );
    std::fs::write(project_root.join("Cargo.toml"), cargo_toml)
        .map_err(|e| CommandError::io("Falha ao gravar Cargo.toml", e))?;
    std::fs::write(project_root.join("src/main.rs"), tmpl.main_rs)
        .map_err(|e| CommandError::io("Falha ao gravar src/main.rs", e))?;

    let project = Project::detect(&project_root)
        .ok_or_else(|| CommandError::io("Projeto criado inválido", "Cargo.toml ausente"))?;
    let canon = project.root.to_string_lossy().to_string();

    {
        let mut inner = state
            .inner
            .lock()
            .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
        if !inner.config.desktop.trusted_projects.contains(&canon) {
            inner.config.desktop.trusted_projects.push(canon.clone());
        }
        inner.config.workspace.last_project = Some(canon.clone());
        inner
            .config
            .workspace
            .recent_projects
            .retain(|p| p != &canon);
        inner.config.workspace.recent_projects.insert(0, canon);
        inner.config.workspace.recent_projects.truncate(8);
        let _ = inner.save_config();
        inner.append_log(
            "workspace",
            &format!("Projeto criado: {valid_name} ({template_id})"),
        );
    }

    open_project(state, &project.root.to_string_lossy())
}

pub fn open_project(state: &AppState, root_dir: &str) -> CmdResult<ProjectInfoDto> {
    let project = Project::detect(root_dir)
        .ok_or_else(|| CommandError::not_found("A pasta selecionada não contém Cargo.toml."))?;
    ensure_safe_project_parent(&project.root)?;
    let canon = project.root.to_string_lossy().to_string();

    let files: Vec<String> = project
        .source_files()
        .into_iter()
        .filter_map(|p| {
            normalize_display_path(&p)
                .strip_prefix(&project.root)
                .ok()
                .map(|rel| rel.to_string_lossy().replace('\\', "/"))
        })
        .collect();
    let caps = project.inspect_capabilities();

    let trusted = {
        let mut inner = state
            .inner
            .lock()
            .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
        let is_trusted = inner
            .config
            .desktop
            .trusted_projects
            .iter()
            .any(|p| p == &canon);
        inner.config.workspace.last_project = Some(canon.clone());
        inner
            .config
            .workspace
            .recent_projects
            .retain(|p| p != &canon);
        inner
            .config
            .workspace
            .recent_projects
            .insert(0, canon.clone());
        inner.config.workspace.recent_projects.truncate(8);
        let _ = inner.save_config();
        is_trusted
    };

    Ok(ProjectInfoDto {
        name: project.name(),
        root_dir: canon,
        trusted,
        files,
        capabilities: ProjectCapabilitiesDto {
            has_plan_md: caps.has_plan_md,
            source_file_count: caps.source_file_count,
            has_structs_or_enums: caps.has_structs_or_enums,
            has_error_handling: caps.has_error_handling,
            has_unit_tests: caps.has_unit_tests,
            has_doc_comments: caps.has_doc_comments,
        },
    })
}

pub fn trust_project(state: &AppState, root_dir: &str) -> CmdResult<()> {
    let project = Project::detect(root_dir)
        .ok_or_else(|| CommandError::not_found("Projeto não encontrado."))?;
    let canon = project.root.to_string_lossy().to_string();
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    if !inner.config.desktop.trusted_projects.contains(&canon) {
        inner.config.desktop.trusted_projects.push(canon);
        inner
            .save_config()
            .map_err(|e| CommandError::io("Falha ao gravar confiança do projeto", e))?;
    }
    Ok(())
}

pub fn get_recent_projects(state: &AppState) -> CmdResult<Vec<String>> {
    let inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    Ok(inner
        .config
        .workspace
        .recent_projects
        .iter()
        .filter(|p| Path::new(p).join("Cargo.toml").is_file())
        .cloned()
        .collect())
}

pub fn read_project_file(root_dir: &str, rel_path: &str) -> CmdResult<FileContentDto> {
    let project = Project::detect(root_dir)
        .ok_or_else(|| CommandError::not_found("Projeto não encontrado."))?;
    let target = project.root.join(rel_path);
    let content = project
        .read(&target)
        .map_err(|e| CommandError::forbidden(format!("Leitura recusada: {e}")))?;
    let sha256 = sha256_hex(&content);
    Ok(FileContentDto {
        rel_path: rel_path.replace('\\', "/"),
        content,
        sha256,
    })
}

pub fn save_project_file(
    root_dir: &str,
    rel_path: &str,
    content: &str,
    expected_sha256: Option<&str>,
    force: bool,
) -> CmdResult<SaveFileResultDto> {
    let project = Project::detect(root_dir)
        .ok_or_else(|| CommandError::not_found("Projeto não encontrado."))?;
    let target = project.root.join(rel_path);
    if !force
        && let Some(expected) = expected_sha256
        && target.is_file()
        && let Ok(disk_now) = project.read(&target)
    {
        let disk_hash = sha256_hex(&disk_now);
        if disk_hash != expected {
            return Ok(SaveFileResultDto {
                saved: false,
                conflict: true,
                disk_sha256: disk_hash,
                disk_content: Some(disk_now),
            });
        }
    }
    project
        .save(&target, content)
        .map_err(|e| CommandError::forbidden(format!("Falha ao salvar arquivo: {e}")))?;
    Ok(SaveFileResultDto {
        saved: true,
        conflict: false,
        disk_sha256: sha256_hex(content),
        disk_content: None,
    })
}

pub fn create_project_file(root_dir: &str, rel_path: &str) -> CmdResult<String> {
    let project = Project::detect(root_dir)
        .ok_or_else(|| CommandError::not_found("Projeto não encontrado."))?;
    let created = project
        .create_file(rel_path)
        .map_err(|e| CommandError::invalid(format!("{e}")))?;
    let rel = normalize_display_path(&created)
        .strip_prefix(&project.root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| rel_path.to_string());
    Ok(rel)
}

pub fn rename_project_file(root_dir: &str, old_rel: &str, new_rel: &str) -> CmdResult<String> {
    let project = Project::detect(root_dir)
        .ok_or_else(|| CommandError::not_found("Projeto não encontrado."))?;
    if old_rel == "Cargo.toml" || new_rel == "Cargo.toml" {
        return Err(CommandError::forbidden(
            "Cargo.toml não pode ser renomeado.",
        ));
    }
    let old_abs = project.root.join(old_rel);
    let _ = project
        .read(&old_abs)
        .map_err(|e| CommandError::forbidden(format!("{e}")))?;
    let created = project
        .create_file(new_rel)
        .map_err(|e| CommandError::invalid(format!("{e}")))?;
    let old_content = std::fs::read_to_string(&old_abs)
        .map_err(|e| CommandError::io("Falha ao ler arquivo original", e))?;
    if let Err(e) = project.save(&created, &old_content) {
        let _ = std::fs::remove_file(&created);
        return Err(CommandError::io("Falha ao gravar arquivo renomeado", e));
    }
    let _ = std::fs::remove_file(&old_abs);
    Ok(new_rel.replace('\\', "/"))
}

pub fn delete_project_file(root_dir: &str, rel_path: &str) -> CmdResult<()> {
    let project = Project::detect(root_dir)
        .ok_or_else(|| CommandError::not_found("Projeto não encontrado."))?;
    if rel_path == "Cargo.toml" || rel_path == "src/main.rs" {
        return Err(CommandError::forbidden(
            "Cargo.toml e src/main.rs são protegidos contra exclusão.",
        ));
    }
    let abs = project.root.join(rel_path);
    let _ = project
        .read(&abs)
        .map_err(|e| CommandError::forbidden(format!("{e}")))?;
    std::fs::remove_file(abs).map_err(|e| CommandError::io("Falha ao remover arquivo", e))?;
    Ok(())
}

pub fn create_project_plan_md(root_dir: &str) -> CmdResult<String> {
    let project = Project::detect(root_dir)
        .ok_or_else(|| CommandError::not_found("Projeto não encontrado."))?;
    let _ = project
        .write_project_plan_template()
        .map_err(|e| CommandError::io("PLANO.md já existe ou não pôde ser criado", e))?;
    Ok("PLANO.md".to_string())
}

pub fn search_in_project(root_dir: &str, query: &str) -> CmdResult<Vec<SearchHitDto>> {
    let project = Project::detect(root_dir)
        .ok_or_else(|| CommandError::not_found("Projeto não encontrado."))?;
    let files = project.source_files();
    let hits = search_project_text(&project, &files, query, 80);
    Ok(hits
        .into_iter()
        .filter_map(|h| {
            normalize_display_path(&h.file)
                .strip_prefix(&project.root)
                .ok()
                .map(|rel| SearchHitDto {
                    rel_path: rel.to_string_lossy().replace('\\', "/"),
                    line: h.line + 1,
                    column: h.column + 1,
                    preview: h.preview,
                })
        })
        .collect())
}

pub fn run_workspace_cargo(
    app: Option<&AppHandle>,
    state: &AppState,
    root_dir: &str,
    action: &str,
) -> CmdResult<WorkspaceExecResultDto> {
    let cmd = match action {
        "check" => ToolCommand::Check,
        "build" => ToolCommand::Build,
        "build-release" => ToolCommand::BuildLinuxRelease,
        "test" => ToolCommand::Test,
        "fmt" => ToolCommand::Fmt,
        "clippy" => ToolCommand::Clippy,
        "run" => ToolCommand::Run,
        _ => return Err(CommandError::invalid("Comando Cargo desconhecido.")),
    };

    let project = Project::detect(root_dir)
        .ok_or_else(|| CommandError::not_found("Projeto não encontrado."))?;
    let canon = project.root.to_string_lossy().to_string();

    {
        let mut inner = state
            .inner
            .lock()
            .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
        if inner.config.execution.consent != ExecConsent::Granted {
            return Err(CommandError::consent(
                "Consentimento de execução necessário antes de rodar Cargo.",
            ));
        }
        if !inner
            .config
            .desktop
            .trusted_projects
            .iter()
            .any(|p| p == &canon)
        {
            return Err(CommandError::consent(
                "Este projeto externo ainda não foi marcado como confiável (build scripts/proc-macros rodam com sua conta).",
            ));
        }
        if inner.exec.running {
            return Err(CommandError::busy(
                "Já existe um comando Cargo em execução.",
            ));
        }
        inner
            .exec
            .submit(cmd.clone(), project.root.clone())
            .map_err(|e| CommandError::new("INTERNAL", e, None))?;
    }

    let mut output_lines = Vec::new();
    let mut stderr_lines = Vec::new();
    let mut final_ev: Option<ExecEvent> = None;
    let deadline = Instant::now() + cmd.timeout() + Duration::from_secs(5);

    while Instant::now() < deadline {
        let mut batch = Vec::new();
        let still_running = {
            let mut inner = state
                .inner
                .lock()
                .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
            inner.exec.poll_events(&mut batch);
            inner.exec.running
        };
        for ev in batch {
            match &ev {
                ExecEvent::Chunk { kind, text } => {
                    let kind_str = match kind {
                        StreamKind::Stdout => "stdout",
                        StreamKind::Stderr => {
                            stderr_lines.push(text.clone());
                            "stderr"
                        }
                        StreamKind::System => "system",
                    };
                    output_lines.push(text.clone());
                    if let Some(handle) = app {
                        let _ = handle.emit(
                            "workspace://exec",
                            WorkspaceStreamEvent {
                                kind: kind_str.to_string(),
                                text: text.clone(),
                            },
                        );
                    }
                }
                ExecEvent::Finished { .. }
                | ExecEvent::TimedOut { .. }
                | ExecEvent::Cancelled { .. }
                | ExecEvent::SpawnFailed { .. } => {
                    final_ev = Some(ev);
                }
                ExecEvent::Started { .. } => {}
            }
        }
        if final_ev.is_some() && !still_running {
            break;
        }
        std::thread::sleep(Duration::from_millis(15));
    }

    let (status, code) = match final_ev {
        Some(ExecEvent::Finished { success, code, .. }) => {
            (if success { "ok" } else { "failed" }.to_string(), code)
        }
        Some(ExecEvent::TimedOut { .. }) => ("timeout".to_string(), None),
        Some(ExecEvent::Cancelled { .. }) => ("cancelled".to_string(), None),
        Some(ExecEvent::SpawnFailed { reason, .. }) => {
            output_lines.push(reason);
            ("spawn-failed".to_string(), None)
        }
        _ => ("timeout".to_string(), None),
    };

    let diagnostics: Vec<DiagnosticDto> = parse_diagnostics(&stderr_lines.join("\n"))
        .into_iter()
        .map(|d| DiagnosticDto {
            level: match d.level {
                DiagLevel::Error => "error",
                DiagLevel::Warning => "warning",
                DiagLevel::Note => "note",
            }
            .to_string(),
            message: d.message,
            file: d.file.map(|f| {
                let p = Path::new(&f);
                if let Ok(rel) = normalize_display_path(p).strip_prefix(&project.root) {
                    rel.to_string_lossy().replace('\\', "/")
                } else {
                    f.replace('\\', "/")
                }
            }),
            line: d.line,
            col: d.col,
        })
        .collect();

    if let Ok(mut inner) = state.inner.lock() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        inner.push_task_history(TaskHistoryEntry {
            id: format!("task-{}", TASK_SEQ.fetch_add(1, Ordering::Relaxed)),
            command: cmd.label().to_string(),
            cwd: canon,
            status: status.clone(),
            timestamp: format!("{ts}"),
            output_preview: output_lines
                .iter()
                .rev()
                .take(4)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n"),
        });
    }

    Ok(WorkspaceExecResultDto {
        status,
        code,
        output_lines,
        diagnostics,
    })
}

pub fn launch_gui_app(
    app: Option<&AppHandle>,
    state: &AppState,
    root_dir: &str,
) -> CmdResult<String> {
    let build_res = run_workspace_cargo(app, state, root_dir, "build")?;
    if build_res.status != "ok" {
        return Err(CommandError::new(
            "INTERNAL",
            "Falha ao compilar o projeto antes de abrir a janela GUI.",
            Some(build_res.output_lines.join("\n")),
        ));
    }

    let project = Project::detect(root_dir)
        .ok_or_else(|| CommandError::not_found("Projeto não encontrado."))?;
    let pkg_name =
        read_package_name(&project.root.join("Cargo.toml")).unwrap_or_else(|| project.name());
    let exe_name = if cfg!(windows) {
        format!("{pkg_name}.exe")
    } else {
        pkg_name.clone()
    };

    let target_base = {
        let inner = state
            .inner
            .lock()
            .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
        inner
            .exec
            .shared_target_dir()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| project.root.join("target"))
    };

    let bin_path = target_base.join("debug").join(&exe_name);
    if !bin_path.is_file() {
        return Err(CommandError::not_found(format!(
            "Binário compilado `{}` não encontrado em `{}`.",
            exe_name,
            bin_path.display()
        )));
    }

    // Lançamento direto sem shell (zero cmd.exe!) para janela Windows nativa separada.
    std::process::Command::new(&bin_path)
        .current_dir(&project.root)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| CommandError::io("Falha ao abrir janela do aplicativo do aluno", e))?;

    if let Ok(inner) = state.inner.lock() {
        inner.append_log("gui", &format!("Janela separada iniciada: {exe_name}"));
    }
    Ok(format!(
        "Aplicativo `{exe_name}` aberto em janela separada!"
    ))
}

fn read_package_name(cargo_toml: &Path) -> Option<String> {
    let txt = std::fs::read_to_string(cargo_toml).ok()?;
    let val: toml::Value = toml::from_str(&txt).ok()?;
    val.get("package")?
        .get("name")?
        .as_str()
        .map(str::to_string)
}

pub fn get_task_history(state: &AppState) -> CmdResult<Vec<TaskHistoryEntry>> {
    let inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    Ok(inner.task_history.clone())
}

pub fn lsp_inspect_file(
    state: &AppState,
    root_dir: &str,
    rel_path: &str,
    content: &str,
    word_under_cursor: Option<&str>,
) -> CmdResult<LspInspectDto> {
    let project = Project::detect(root_dir)
        .ok_or_else(|| CommandError::not_found("Projeto não encontrado."))?;
    let files = project.source_files();
    let idx = WorkspaceIndex::reindex(&project, &files);
    let mut completions = rustlab::completion::rust_keywords()
        .iter()
        .map(|s| (*s).to_string())
        .collect::<Vec<_>>();
    for sym in idx.symbol_names() {
        if !completions.contains(&sym) {
            completions.push(sym);
        }
    }

    let inlay_hints = infer_local_inlay_hints(content)
        .into_iter()
        .map(|h| InlayHintDto {
            line: h.line + 1,
            col: h.col + 1,
            label: h.label,
        })
        .collect();

    let (en, trusted) = {
        let inner = state
            .inner
            .lock()
            .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
        (
            inner.is_english(),
            inner
                .config
                .desktop
                .trusted_projects
                .iter()
                .any(|p| p == &project.root.to_string_lossy()),
        )
    };

    let hover_doc = word_under_cursor.and_then(|w| {
        idx.find_doc(w).or_else(|| {
            rustlab::doc_hint::lookup_for(w, en).map(|d| format!("{} ({})", d.summary, d.url))
        })
    });

    let mut lsp_active = false;
    if trusted {
        let mut inner = state
            .inner
            .lock()
            .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
        let need_start = inner.lsp.as_ref().is_none_or(|s| s.root != project.root);
        if need_start {
            if let Some(mut old) = inner.lsp.take()
                && let Some(mut client) = old.client.take()
            {
                client.shutdown();
            }
            let client = LspClient::start(&project.root).ok();
            inner.lsp = Some(LspSession {
                root: project.root.clone(),
                client,
                version_counter: 1,
            });
        }
        if let Some(sess) = inner.lsp.as_mut()
            && let Some(client) = sess.client.as_mut()
        {
            for ev in client.poll_events() {
                if matches!(ev, LspEvent::Initialized) {
                    client.did_open(&project.root.join(rel_path), content);
                }
            }
            if client.is_ready() {
                lsp_active = true;
                sess.version_counter += 1;
                client.did_change(&project.root.join(rel_path), sess.version_counter, content);
            }
        }
    }

    Ok(LspInspectDto {
        lsp_active,
        completions,
        inlay_hints,
        hover_doc,
    })
}
