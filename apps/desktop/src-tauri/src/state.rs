use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::Serialize;

use rustlab::config::{self, Config};
use rustlab::curriculum::Curriculum;
use rustlab::execution::ExecutionManager;
use rustlab::learn::Progress;
use rustlab::lsp::LspClient;
use rustlab::storage;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskHistoryEntry {
    pub id: String,
    pub command: String,
    pub cwd: String,
    pub status: String,
    pub timestamp: String,
    pub output_preview: String,
}

#[derive(Debug, Clone)]
pub struct LibraryRevertSnapshot {
    pub root: PathBuf,
    pub cargo_toml_before: String,
    pub target_rs_rel: String,
    pub target_rs_before: String,
}

pub struct PtySessionHandle {
    pub writer: Box<dyn std::io::Write + Send>,
    pub master: Box<dyn portable_pty::MasterPty + Send>,
    pub killer: Box<dyn portable_pty::ChildKiller + Send + Sync>,
}

pub struct LspSession {
    pub root: PathBuf,
    pub client: Option<LspClient>,
    pub version_counter: i64,
}

pub struct InnerState {
    pub config: Config,
    pub config_warning: Option<String>,
    pub progress: Progress,
    pub progress_warning: Option<String>,
    pub curriculum: Curriculum,
    pub exec: ExecutionManager,
    pub task_history: Vec<TaskHistoryEntry>,
    pub library_reverts: HashMap<String, LibraryRevertSnapshot>,
    pub pty_sessions: HashMap<String, PtySessionHandle>,
    pub lsp: Option<LspSession>,
    pub data_root: PathBuf,
    pub config_path: PathBuf,
    pub progress_path: PathBuf,
}

pub struct AppState {
    pub inner: Arc<Mutex<InnerState>>,
}

impl AppState {
    pub fn new() -> Self {
        let cfg_path = config::config_path().unwrap_or_else(|| PathBuf::from("config.toml"));
        let (cfg, cfg_warn) = if cfg_path.exists() {
            let outcome = config::load_from_path(&cfg_path);
            (outcome.config, outcome.warning)
        } else {
            (Config::desktop_defaults(), None)
        };

        let prog_path = storage::default_path();
        let (prog_data, prog_warn) = storage::load_from(&prog_path);
        let progress = Progress::from_data(prog_data);

        let en = cfg.is_english();
        let curriculum = Curriculum::embedded_for(en).expect("embedded curriculum must be valid");

        let mut exec = ExecutionManager::new();
        exec.set_shared_target_dir(cfg.execution.resolved_target_cache_dir());

        let data_root = prog_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        let _ = std::fs::create_dir_all(data_root.join("drafts"));
        let _ = std::fs::create_dir_all(data_root.join("backups"));
        let _ = std::fs::create_dir_all(data_root.join("logs"));

        let state = InnerState {
            config: cfg,
            config_warning: cfg_warn,
            progress,
            progress_warning: prog_warn,
            curriculum,
            exec,
            task_history: Vec::new(),
            library_reverts: HashMap::new(),
            pty_sessions: HashMap::new(),
            lsp: None,
            data_root,
            config_path: cfg_path,
            progress_path: prog_path,
        };
        state.append_log("app", "RustLab Desktop inicializado");
        Self {
            inner: Arc::new(Mutex::new(state)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl InnerState {
    pub fn is_english(&self) -> bool {
        self.config.is_english()
    }

    pub fn reload_curriculum_lang(&mut self) {
        if let Ok(cur) = Curriculum::embedded_for(self.is_english()) {
            self.curriculum = cur;
        }
    }

    pub fn save_config(&self) -> Result<(), config::SaveError> {
        config::save_to(&self.config, &self.config_path)
    }

    pub fn save_progress(&self) -> std::io::Result<()> {
        storage::save_to(&self.progress.to_data(), &self.progress_path)
    }

    pub fn xp(&self) -> u32 {
        self.progress
            .xp_events
            .values()
            .copied()
            .fold(0, u32::saturating_add)
    }

    pub fn level(&self) -> u32 {
        self.xp() / 100 + 1
    }

    pub fn award_xp(&mut self, event: String, amount: u32) -> u32 {
        if self.progress.xp_events.contains_key(&event) {
            return 0;
        }
        self.progress.xp_events.insert(event, amount);
        amount
    }

    pub fn draft_file_path(&self, key: &str) -> PathBuf {
        let safe: String = key
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        self.data_root.join("drafts").join(format!("{safe}.rs"))
    }

    pub fn write_draft(&self, key: &str, code: &str) {
        let p = self.draft_file_path(key);
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(p, code);
    }

    pub fn read_draft(&self, key: &str) -> Option<String> {
        std::fs::read_to_string(self.draft_file_path(key)).ok()
    }

    pub fn push_task_history(&mut self, entry: TaskHistoryEntry) {
        self.task_history.insert(0, entry);
        if self.task_history.len() > 50 {
            self.task_history.truncate(50);
        }
    }

    pub fn append_log(&self, category: &str, msg: &str) {
        use std::io::Write;
        let path = self.data_root.join("logs").join("rustlab-desktop.log");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let _ = writeln!(f, "[{ts}] [{category}] {msg}");
        }
    }
}
