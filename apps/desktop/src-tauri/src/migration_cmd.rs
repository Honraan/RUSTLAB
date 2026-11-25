use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use rustlab::config::{
    AiProvider, Config, ExecConsent, GamificationVisibility, MotionMode, OriginLanguage,
    RemoteAiConsent, ThemeId,
};
use rustlab::curriculum::v3_blueprint::{
    V3Stage, entries_for_stage, is_v3_entry_completed, v3_curriculum_blueprint,
};
use rustlab::learn::Progress;
use rustlab::storage::{self, ProgressData};

use crate::error::{CmdResult, CommandError};
use crate::mentor_cmd;
use crate::state::AppState;

const ALLOWED_URL_HOSTS: &[&str] = &[
    "doc.rust-lang.org",
    "docs.rs",
    "rust-lang.org",
    "www.rust-lang.org",
    "rust-lang.github.io",
    "v2.tauri.app",
    "bevy.org",
    "github.com",
    "rustup.rs",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressSummaryDto {
    pub xp: u32,
    pub level: u32,
    pub completed_v3: usize,
    pub total_v3: usize,
    pub completed_exercises: usize,
    pub independent_completions: usize,
    pub due_srs_items: Vec<String>,
    pub top_errors: Vec<(String, u32)>,
    pub stage_progress: Vec<StageProgressDto>,
    pub achievements: Vec<AchievementDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StageProgressDto {
    pub id: String,
    pub label: String,
    pub completed: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementDto {
    pub id: String,
    pub title: String,
    pub unlocked: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    pub language: String,
    pub origin_language: String,
    pub theme: String,
    pub desktop_theme: String,
    pub animations: String,
    pub gamification_visible: bool,
    pub focus_mode: bool,
    pub sound_enabled: bool,
    pub sound_volume: u8,
    pub ui_scale: u16,
    pub editor_font_size: u16,
    pub first_run_completed: bool,
    pub exec_consent_granted: bool,
    pub ai_provider: String,
    pub ollama_url: String,
    pub ollama_model: String,
    pub remote_url: String,
    pub remote_model: String,
    pub remote_consent: String,
    pub has_remote_api_key: bool,
    pub data_root: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatchDto {
    pub language: Option<String>,
    pub origin_language: Option<String>,
    pub desktop_theme: Option<String>,
    pub animations: Option<String>,
    pub gamification_visible: Option<bool>,
    pub focus_mode: Option<bool>,
    pub sound_enabled: Option<bool>,
    pub sound_volume: Option<u8>,
    pub ui_scale: Option<u16>,
    pub editor_font_size: Option<u16>,
    pub first_run_completed: Option<bool>,
    pub ai_provider: Option<String>,
    pub ollama_url: Option<String>,
    pub ollama_model: Option<String>,
    pub remote_url: Option<String>,
    pub remote_model: Option<String>,
    pub remote_consent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableBackupV1 {
    pub schema_version: u32,
    pub app_version: String,
    pub exported_unix_secs: u64,
    pub sha256_progress: String,
    pub progress: ProgressData,
    pub safe_config: Option<Config>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreviewDto {
    pub source_format: String,
    pub incoming_lessons: usize,
    pub incoming_exercises: usize,
    pub incoming_xp: u32,
    pub new_xp_after_merge: u32,
    pub warning: Option<String>,
}

pub fn get_progress_summary(state: &AppState) -> CmdResult<ProgressSummaryDto> {
    let inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    let en = inner.is_english();
    let bp = v3_curriculum_blueprint();
    let completed_v3 = bp
        .iter()
        .filter(|e| is_v3_entry_completed(&inner.progress, e))
        .count();
    let independent_completions = inner
        .progress
        .xp_events
        .keys()
        .filter(|k| k.ends_with(":independent"))
        .count();

    let stage_progress = V3Stage::ALL
        .iter()
        .map(|st| {
            let entries = entries_for_stage(*st);
            let done = entries
                .iter()
                .filter(|e| is_v3_entry_completed(&inner.progress, e))
                .count();
            StageProgressDto {
                id: crate::curriculum_cmd::stage_id_str(*st).to_string(),
                label: st.label_for(en).to_string(),
                completed: done,
                total: entries.len(),
            }
        })
        .collect();

    let achievements = vec![
        AchievementDto {
            id: "first-lesson".into(),
            title: "Primeiro Passo (1 lição)".into(),
            unlocked: completed_v3 >= 1,
        },
        AchievementDto {
            id: "first-practice".into(),
            title: "Primeiro Cargo Verde".into(),
            unlocked: !inner.progress.exercises_completed.is_empty(),
        },
        AchievementDto {
            id: "independent".into(),
            title: "Autonomia Pura (100+100 XP sem dicas)".into(),
            unlocked: independent_completions >= 1,
        },
        AchievementDto {
            id: "basico".into(),
            title: "Mestre de O BÁSICO (14/14)".into(),
            unlocked: entries_for_stage(V3Stage::Basico)
                .iter()
                .all(|e| is_v3_entry_completed(&inner.progress, e)),
        },
        AchievementDto {
            id: "ten-practices".into(),
            title: "Prática Constante (10 práticas/exercícios)".into(),
            unlocked: inner.progress.exercises_completed.len() >= 10,
        },
    ];

    Ok(ProgressSummaryDto {
        xp: inner.xp(),
        level: inner.level(),
        completed_v3,
        total_v3: bp.len(),
        completed_exercises: inner.progress.exercises_completed.len(),
        independent_completions,
        due_srs_items: inner.progress.due_srs_items(),
        top_errors: inner.progress.top_recurring_errors(5),
        stage_progress,
        achievements,
    })
}

pub fn get_settings(state: &AppState) -> CmdResult<SettingsDto> {
    let inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    Ok(settings_to_dto(&inner.config, &inner.data_root))
}

fn settings_to_dto(cfg: &Config, data_root: &Path) -> SettingsDto {
    SettingsDto {
        language: cfg.language.clone(),
        origin_language: match cfg.origin_language {
            OriginLanguage::None => "none",
            OriginLanguage::Python => "python",
            OriginLanguage::JavaScriptTs => "js-ts",
            OriginLanguage::CCpp => "c-cpp",
            OriginLanguage::Go => "go",
            OriginLanguage::JavaCSharp => "java-csharp",
        }
        .to_string(),
        theme: format!("{:?}", cfg.ui.theme).to_lowercase(),
        desktop_theme: cfg.desktop.desktop_theme.clone(),
        animations: match cfg.ui.animations {
            MotionMode::Full => "full",
            MotionMode::Reduced => "reduced",
            MotionMode::Off => "off",
        }
        .to_string(),
        gamification_visible: cfg.ui.gamification == GamificationVisibility::Visible,
        focus_mode: cfg.desktop.focus_mode,
        sound_enabled: cfg.sound.enabled,
        sound_volume: cfg.sound.volume,
        ui_scale: cfg.desktop.ui_scale,
        editor_font_size: cfg.desktop.editor_font_size,
        first_run_completed: cfg.desktop.first_run_completed,
        exec_consent_granted: cfg.execution.consent == ExecConsent::Granted,
        ai_provider: match cfg.ai.provider {
            AiProvider::Disabled => "disabled",
            AiProvider::Ollama => "ollama",
            AiProvider::Remote => "remote",
        }
        .to_string(),
        ollama_url: cfg.ai.ollama_url.clone(),
        ollama_model: cfg.ai.ollama_model.clone(),
        remote_url: cfg.ai.remote_url.clone(),
        remote_model: cfg.ai.remote_model.clone(),
        remote_consent: match cfg.ai.remote_consent {
            RemoteAiConsent::Ask => "ask",
            RemoteAiConsent::Granted => "granted",
            RemoteAiConsent::Denied => "denied",
        }
        .to_string(),
        has_remote_api_key: mentor_cmd::has_api_key(),
        data_root: data_root.to_string_lossy().to_string(),
    }
}

pub fn update_settings(state: &AppState, patch: SettingsPatchDto) -> CmdResult<SettingsDto> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    let mut lang_changed = false;
    if let Some(lang) = patch.language {
        let next = if lang.trim().to_ascii_lowercase().starts_with("en") {
            "en-US".to_string()
        } else {
            "pt-BR".to_string()
        };
        if inner.config.language != next {
            inner.config.language = next;
            lang_changed = true;
        }
    }
    if let Some(orig) = patch.origin_language {
        inner.config.origin_language = match orig.as_str() {
            "python" => OriginLanguage::Python,
            "js-ts" => OriginLanguage::JavaScriptTs,
            "c-cpp" => OriginLanguage::CCpp,
            "go" => OriginLanguage::Go,
            "java-csharp" => OriginLanguage::JavaCSharp,
            _ => OriginLanguage::None,
        };
    }
    if let Some(dt) = patch.desktop_theme {
        inner.config.desktop.desktop_theme = match dt.as_str() {
            "light" | "dark" | "high-contrast" | "system" => dt,
            _ => "dark".to_string(),
        };
        inner.config.ui.theme = match inner.config.desktop.desktop_theme.as_str() {
            "high-contrast" => ThemeId::Mono,
            _ => ThemeId::Cuia,
        };
    }
    if let Some(anim) = patch.animations {
        inner.config.ui.animations = match anim.as_str() {
            "off" => MotionMode::Off,
            "reduced" => MotionMode::Reduced,
            _ => MotionMode::Full,
        };
    }
    if let Some(vis) = patch.gamification_visible {
        inner.config.ui.gamification = if vis {
            GamificationVisibility::Visible
        } else {
            GamificationVisibility::Hidden
        };
    }
    if let Some(fm) = patch.focus_mode {
        inner.config.desktop.focus_mode = fm;
    }
    if let Some(se) = patch.sound_enabled {
        inner.config.sound.enabled = se;
    }
    if let Some(sv) = patch.sound_volume {
        inner.config.sound.volume = sv.min(100);
    }
    if let Some(sc) = patch.ui_scale {
        inner.config.desktop.ui_scale = sc.clamp(90, 150);
    }
    if let Some(fs) = patch.editor_font_size {
        inner.config.desktop.editor_font_size = fs.clamp(12, 24);
    }
    if let Some(done) = patch.first_run_completed {
        inner.config.desktop.first_run_completed = done;
        inner.progress.onboarding_completed = done;
        let _ = inner.save_progress();
    }
    if let Some(p) = patch.ai_provider {
        inner.config.ai.provider = match p.as_str() {
            "ollama" => AiProvider::Ollama,
            "remote" => AiProvider::Remote,
            _ => AiProvider::Disabled,
        };
    }
    if let Some(u) = patch.ollama_url {
        inner.config.ai.ollama_url = u;
    }
    if let Some(m) = patch.ollama_model {
        inner.config.ai.ollama_model = m;
    }
    if let Some(u) = patch.remote_url {
        inner.config.ai.remote_url = u;
    }
    if let Some(m) = patch.remote_model {
        inner.config.ai.remote_model = m;
    }
    if let Some(c) = patch.remote_consent {
        inner.config.ai.remote_consent = match c.as_str() {
            "granted" => RemoteAiConsent::Granted,
            "denied" => RemoteAiConsent::Denied,
            _ => RemoteAiConsent::Ask,
        };
    }
    if lang_changed {
        inner.reload_curriculum_lang();
    }
    inner
        .save_config()
        .map_err(|e| CommandError::io("Falha ao gravar config.toml", e))?;
    Ok(settings_to_dto(&inner.config, &inner.data_root))
}

pub fn grant_exec_consent(state: &AppState) -> CmdResult<()> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    inner.config.execution.consent = ExecConsent::Granted;
    inner
        .save_config()
        .map_err(|e| CommandError::io("Falha ao salvar consentimento", e))?;
    inner.append_log(
        "security",
        "Consentimento de execução local concedido pelo usuário",
    );
    Ok(())
}

pub fn reset_progress(state: &AppState) -> CmdResult<()> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    backup_current_progress(&inner.progress_path, &inner.data_root.join("backups"));
    inner.progress = Progress::default();
    inner
        .save_progress()
        .map_err(|e| CommandError::io("Falha ao redefinir progresso", e))?;
    inner.append_log("data", "Progresso local redefinido após backup");
    Ok(())
}

fn backup_current_progress(src: &Path, backups_dir: &Path) {
    if !src.is_file() {
        return;
    }
    let _ = std::fs::create_dir_all(backups_dir);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let dst = backups_dir.join(format!("progresso-backup-{ts}.toml"));
    let _ = std::fs::copy(src, dst);
}

fn hash_progress_data(data: &ProgressData) -> String {
    let toml_txt = toml::to_string(data).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(toml_txt.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn export_backup(state: &AppState, dest_path: &str) -> CmdResult<()> {
    let inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    let progress = inner.progress.to_data();
    let mut safe_cfg = inner.config.clone();
    safe_cfg.workspace.last_project = None;
    safe_cfg.workspace.recent_projects.clear();

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let bundle = PortableBackupV1 {
        schema_version: 1,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        exported_unix_secs: ts,
        sha256_progress: hash_progress_data(&progress),
        progress,
        safe_config: Some(safe_cfg),
    };
    let json = serde_json::to_string_pretty(&bundle)
        .map_err(|e| CommandError::io("Falha ao serializar backup", e))?;
    std::fs::write(dest_path, json)
        .map_err(|e| CommandError::io("Falha ao gravar arquivo de backup", e))?;
    inner.append_log("data", "Backup portátil exportado");
    Ok(())
}

fn parse_backup_file(
    src_path: &str,
) -> CmdResult<(String, ProgressData, Option<Config>, Option<String>)> {
    let raw = std::fs::read_to_string(src_path)
        .map_err(|e| CommandError::io("Falha ao ler arquivo de importação", e))?;
    if raw.trim_start().starts_with('{') {
        let bundle: PortableBackupV1 = serde_json::from_str(&raw)
            .map_err(|e| CommandError::invalid(format!("Backup JSON inválido: {e}")))?;
        let computed = hash_progress_data(&bundle.progress);
        let warn = if computed != bundle.sha256_progress {
            Some("Aviso: checksum SHA-256 difere do original; confira os dados na prévia.".into())
        } else {
            None
        };
        Ok((
            "rustlab-backup-v1.json".into(),
            bundle.progress,
            bundle.safe_config,
            warn,
        ))
    } else {
        let (data, warn) = storage::load_from(Path::new(src_path));
        if warn.is_some()
            && data.completed_lessons.is_empty()
            && data.completed_exercises.is_empty()
        {
            return Err(CommandError::invalid(
                warn.unwrap_or_else(|| "Arquivo TOML inválido.".into()),
            ));
        }
        Ok(("progresso.toml (TUI V3)".into(), data, None, warn))
    }
}

fn merge_xp_maps(
    current: &BTreeMap<String, u32>,
    incoming: &BTreeMap<String, u32>,
) -> BTreeMap<String, u32> {
    let mut merged = current.clone();
    for (k, v) in incoming {
        let entry = merged.entry(k.clone()).or_insert(0);
        *entry = (*entry).max(*v);
    }
    merged
}

pub fn preview_import_backup(state: &AppState, src_path: &str) -> CmdResult<ImportPreviewDto> {
    let (fmt, incoming, _, warn) = parse_backup_file(src_path)?;
    let inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    let current_data = inner.progress.to_data();
    let merged_xp = merge_xp_maps(&current_data.xp_events, &incoming.xp_events);
    let incoming_xp: u32 = incoming
        .xp_events
        .values()
        .copied()
        .fold(0, u32::saturating_add);
    let new_xp_after_merge: u32 = merged_xp.values().copied().fold(0, u32::saturating_add);
    Ok(ImportPreviewDto {
        source_format: fmt,
        incoming_lessons: incoming.completed_lessons.len(),
        incoming_exercises: incoming.completed_exercises.len(),
        incoming_xp,
        new_xp_after_merge,
        warning: warn,
    })
}

pub fn apply_import_backup(
    state: &AppState,
    src_path: &str,
    mode: &str,
) -> CmdResult<ProgressSummaryDto> {
    let (_, incoming, maybe_cfg, _) = parse_backup_file(src_path)?;
    {
        let mut inner = state
            .inner
            .lock()
            .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
        backup_current_progress(&inner.progress_path, &inner.data_root.join("backups"));

        let next_data = if mode == "replace" {
            incoming
        } else {
            let mut cur = inner.progress.to_data();
            cur.read_lessons.extend(incoming.read_lessons);
            cur.completed_lessons.extend(incoming.completed_lessons);
            cur.completed_exercises.extend(incoming.completed_exercises);
            for (k, v) in incoming.hints_used {
                let e = cur.hints_used.entry(k).or_insert(0);
                *e = (*e).max(v);
            }
            for (k, v) in incoming.exercise_answers {
                cur.exercise_answers.entry(k).or_insert(v);
            }
            cur.xp_events = merge_xp_maps(&cur.xp_events, &incoming.xp_events);
            cur.project_milestones.extend(incoming.project_milestones);
            for (k, v) in incoming.srs_cards {
                cur.srs_cards.entry(k).or_insert(v);
            }
            for (k, v) in incoming.error_counts {
                *cur.error_counts.entry(k).or_insert(0) += v;
            }
            cur.onboarding_completed |= incoming.onboarding_completed;
            cur.last_entry_id = cur.last_entry_id.or(incoming.last_entry_id);
            cur
        };

        inner.progress = Progress::from_data(next_data);
        inner
            .save_progress()
            .map_err(|e| CommandError::io("Falha ao gravar progresso importado", e))?;

        if let Some(mut cfg) = maybe_cfg {
            // Sanitiza caminhos POSIX inválidos no Windows
            if cfg!(windows) {
                cfg.workspace
                    .recent_projects
                    .retain(|p| Path::new(p).exists());
                if cfg
                    .workspace
                    .last_project
                    .as_deref()
                    .is_some_and(|p| !Path::new(p).exists())
                {
                    cfg.workspace.last_project = None;
                }
                if cfg
                    .execution
                    .target_cache_dir
                    .as_deref()
                    .is_some_and(|p| p.starts_with('/'))
                {
                    cfg.execution.target_cache_dir = None;
                }
            }
            inner.config = cfg;
            inner.reload_curriculum_lang();
            let _ = inner.save_config();
        }
        inner.append_log("data", &format!("Importação aplicada (modo={mode})"));
    }
    get_progress_summary(state)
}

pub fn validate_safe_external_url(raw: &str) -> CmdResult<String> {
    let url = raw.trim();
    if url.contains(['\r', '\n', '\0']) || !url.starts_with("https://") {
        return Err(CommandError::forbidden(
            "Apenas URLs HTTPS oficiais são permitidas.",
        ));
    }
    let after_scheme = &url["https://".len()..];
    let host = after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if !ALLOWED_URL_HOSTS
        .iter()
        .any(|allowed| host == *allowed || host.ends_with(&format!(".{allowed}")))
    {
        return Err(CommandError::forbidden(format!(
            "Domínio `{host}` não está na lista segura de documentação."
        )));
    }
    Ok(url.to_string())
}

pub fn export_diagnostics_log(state: &AppState, dest_path: &str) -> CmdResult<()> {
    let caps = crate::toolchain::inspect_capabilities();
    let (log_text, history_count) = {
        let inner = state
            .inner
            .lock()
            .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
        let log_path = inner.data_root.join("logs").join("rustlab-desktop.log");
        let txt = std::fs::read_to_string(log_path).unwrap_or_default();
        (txt, inner.task_history.len())
    };
    let report = serde_json::json!({
        "appVersion": env!("CARGO_PKG_VERSION"),
        "capabilities": caps,
        "taskHistoryEntries": history_count,
        "recentLogTail": log_text.lines().rev().take(100).collect::<Vec<_>>(),
    });
    let pretty = serde_json::to_string_pretty(&report)
        .map_err(|e| CommandError::io("Falha ao formatar diagnóstico", e))?;
    std::fs::write(dest_path, pretty)
        .map_err(|e| CommandError::io("Falha ao salvar diagnóstico", e))?;
    Ok(())
}
