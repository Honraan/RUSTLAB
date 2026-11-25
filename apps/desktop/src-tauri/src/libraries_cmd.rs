use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use rustlab::doc_hint;
use rustlab::projects::Project;

use crate::error::{CmdResult, CommandError};
use crate::state::{AppState, LibraryRevertSnapshot};

static REVERT_SEQ: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemDto {
    pub name: String,
    pub version: String,
    pub category: String,
    pub summary: String,
    pub docs_pt: String,
    pub use_line: String,
    pub doc_url: String,
    pub associated_v3_lessons: Vec<String>,
    pub common_pitfalls: String,
    pub windows_notes: String,
    pub installed_in_project: bool,
    pub version_note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryChangePreviewDto {
    pub name: String,
    pub version: String,
    pub cargo_toml_before: String,
    pub cargo_toml_after: String,
    pub target_rs_rel: String,
    pub target_rs_before: String,
    pub target_rs_after: String,
    pub requires_download_note: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryApplyResultDto {
    pub revert_id: String,
    pub target_rs_rel: String,
}

fn windows_notes_for(name: &str) -> &'static str {
    match name {
        "bevy" => {
            "Requer MSVC/linker + Windows SDK e GPU com DirectX 12 ou Vulkan. Abre janela 3D própria fora do RustLab."
        }
        "eframe" => {
            "Requer driver OpenGL/WGL no Windows. Ideal para Mini Paint GUI em janela própria."
        }
        "minifb" => "Usa Win32 GDI/janela nativa sem dependências externas pesadas.",
        "macroquad" => "Requer driver OpenGL no Windows; abre janela 2D própria.",
        "rodio" => {
            "Usa WASAPI nativo do Windows; degrada silenciosamente se não houver dispositivo de áudio."
        }
        "tachyonfx" => {
            "Pure Rust para efeitos em terminal ConPTY. Atenção: catálogo histórico exibe 0.11, enquanto o host RustLab usa 0.25."
        }
        _ => "Pure Rust multiplataforma sem dependências nativas do Windows.",
    }
}

pub fn list_libraries(state: &AppState, root_dir: Option<&str>) -> CmdResult<Vec<LibraryItemDto>> {
    let inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    let en = inner.is_english();
    let cargo_toml_text = root_dir
        .and_then(Project::detect)
        .and_then(|p| p.read(&p.root.join("Cargo.toml")).ok())
        .unwrap_or_default();

    Ok(doc_hint::extra_libraries(en)
        .iter()
        .map(|lib| {
            let meta = lib.v3_metadata_for(en);
            let installed = !cargo_toml_text.is_empty()
                && cargo_toml_text
                    .lines()
                    .any(|l| l.trim_start().starts_with(lib.name));
            let version_note = if lib.name == "tachyonfx" {
                Some("Nota: versão 0.11 no catálogo F9; o binário RustLab usa 0.25.".to_string())
            } else {
                None
            };
            LibraryItemDto {
                name: lib.name.to_string(),
                version: lib.version.to_string(),
                category: lib.category.to_string(),
                summary: lib.summary.to_string(),
                docs_pt: lib.docs_pt.to_string(),
                use_line: lib.use_line.to_string(),
                doc_url: lib.doc_url.to_string(),
                associated_v3_lessons: meta
                    .associated_v3_lessons
                    .iter()
                    .map(|s| (*s).to_string())
                    .collect(),
                common_pitfalls: meta.common_pitfalls.to_string(),
                windows_notes: windows_notes_for(lib.name).to_string(),
                installed_in_project: installed,
                version_note,
            }
        })
        .collect())
}

pub fn preview_add_library(
    state: &AppState,
    root_dir: &str,
    lib_name: &str,
    include_template: bool,
) -> CmdResult<LibraryChangePreviewDto> {
    let inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    let en = inner.is_english();
    let lib = doc_hint::extra_libraries(en)
        .iter()
        .find(|l| l.name == lib_name)
        .ok_or_else(|| {
            CommandError::not_found(format!("Biblioteca `{lib_name}` não está no catálogo F9."))
        })?;

    let project = Project::detect(root_dir)
        .ok_or_else(|| CommandError::not_found("Projeto Cargo ativo não encontrado."))?;
    let cargo_before = project
        .read(&project.root.join("Cargo.toml"))
        .map_err(|e| CommandError::io("Falha ao ler Cargo.toml", e))?;
    let mut cargo_after =
        doc_hint::ensure_dependency_in_cargo_toml(&cargo_before, lib.name, lib.version);
    if lib.name == "bevy" {
        cargo_after = doc_hint::ensure_bevy_fast_dev_profile_in_cargo_toml(&cargo_after);
    }

    let main_rel = "src/main.rs".to_string();
    let rs_before = project
        .read(&project.root.join(&main_rel))
        .unwrap_or_default();
    let rs_after = if include_template {
        if lib.name == "bevy" {
            doc_hint::bevy_starter_template_for(en).to_string()
        } else if lib.example.contains("fn main(") {
            format!("{}\n\n{}\n", lib.use_line, lib.example)
        } else {
            let indented = lib
                .example
                .lines()
                .map(|l| format!("    {l}"))
                .collect::<Vec<_>>()
                .join("\n");
            format!("{}\n\nfn main() {{\n{indented}\n}}\n", lib.use_line)
        }
    } else {
        doc_hint::insert_use_line_in_rust_code(&rs_before, lib.use_line)
    };

    Ok(LibraryChangePreviewDto {
        name: lib.name.to_string(),
        version: lib.version.to_string(),
        cargo_toml_before: cargo_before,
        cargo_toml_after: cargo_after,
        target_rs_rel: main_rel,
        target_rs_before: rs_before,
        target_rs_after: rs_after,
        requires_download_note: format!(
            "Alterará apenas `{}` após seu consentimento. A próxima compilação Cargo pode baixar `{}` da crates.io (se não estiver no cache offline).",
            project.name(),
            lib.name
        ),
    })
}

pub fn apply_add_library(
    state: &AppState,
    root_dir: &str,
    lib_name: &str,
    include_template: bool,
) -> CmdResult<LibraryApplyResultDto> {
    let preview = preview_add_library(state, root_dir, lib_name, include_template)?;
    let project = Project::detect(root_dir)
        .ok_or_else(|| CommandError::not_found("Projeto Cargo ativo não encontrado."))?;

    project
        .save(&project.root.join("Cargo.toml"), &preview.cargo_toml_after)
        .map_err(|e| CommandError::io("Falha ao atualizar Cargo.toml", e))?;
    project
        .save(
            &project.root.join(&preview.target_rs_rel),
            &preview.target_rs_after,
        )
        .map_err(|e| CommandError::io("Falha ao atualizar src/main.rs", e))?;

    let revert_id = format!("rev-{}", REVERT_SEQ.fetch_add(1, Ordering::Relaxed));
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    inner.library_reverts.insert(
        revert_id.clone(),
        LibraryRevertSnapshot {
            root: project.root,
            cargo_toml_before: preview.cargo_toml_before,
            target_rs_rel: preview.target_rs_rel.clone(),
            target_rs_before: preview.target_rs_before,
        },
    );
    inner.append_log(
        "f9",
        &format!("Biblioteca {lib_name} adicionada (revert={revert_id})"),
    );

    Ok(LibraryApplyResultDto {
        revert_id,
        target_rs_rel: preview.target_rs_rel,
    })
}

pub fn revert_library_change(state: &AppState, revert_id: &str) -> CmdResult<()> {
    let snap = {
        let mut inner = state
            .inner
            .lock()
            .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
        inner
            .library_reverts
            .remove(revert_id)
            .ok_or_else(|| CommandError::not_found("Snapshot de reversão não encontrado."))?
    };
    let project = Project::detect(&snap.root)
        .ok_or_else(|| CommandError::not_found("Projeto não encontrado para reverter."))?;
    project
        .save(&project.root.join("Cargo.toml"), &snap.cargo_toml_before)
        .map_err(|e| CommandError::io("Falha ao restaurar Cargo.toml", e))?;
    project
        .save(
            &project.root.join(&snap.target_rs_rel),
            &snap.target_rs_before,
        )
        .map_err(|e| CommandError::io("Falha ao restaurar arquivo fonte", e))?;
    Ok(())
}
