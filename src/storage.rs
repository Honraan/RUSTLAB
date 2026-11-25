//! Persistência local do progresso de aprendizagem (Fase 4).
//! Arquivo versionado e tolerante a corrupção. Respostas são texto armazenado,
//! nunca executado durante o carregamento.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
static FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SrsCard {
    pub mastery_level: u8,
    pub interval_steps: u32,
    pub due_at_step: u32,
    pub independent_passes: u32,
    pub review_count: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProgressData {
    pub version: u32,
    pub read_lessons: BTreeSet<String>,
    pub completed_lessons: BTreeSet<String>,
    pub completed_exercises: BTreeSet<String>,
    pub hints_used: BTreeMap<String, u8>,
    pub exercise_answers: BTreeMap<String, String>,
    /// Eventos premiados uma única vez: `lesson:<id>` / `exercise:<id>` / `project:<id>`.
    pub xp_events: BTreeMap<String, u32>,
    /// Marcos verificados de projeto independente (Etapa 10).
    pub project_milestones: BTreeSet<String>,
    /// Cartões de maestria e revisão espaçada (V3).
    pub srs_cards: BTreeMap<String, SrsCard>,
    /// Frequência de códigos de erro do compilador (ex.: E0382 -> 3) para foco pedagógico.
    pub error_counts: BTreeMap<String, u32>,
    /// Se o tour de onboarding inicial foi concluído.
    pub onboarding_completed: bool,
}

impl ProgressData {
    pub fn current() -> Self {
        Self {
            version: 1,
            ..Self::default()
        }
    }
}

pub fn default_path() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("rustlab").join("progresso.toml")
}

pub fn load() -> (ProgressData, Option<String>) {
    load_from(&default_path())
}

pub fn load_from(path: &Path) -> (ProgressData, Option<String>) {
    match std::fs::read_to_string(path) {
        Ok(text) => match toml::from_str::<ProgressData>(&text) {
            Ok(data) if data.version <= 1 => (data, None),
            Ok(_) => (
                ProgressData::current(),
                Some("versão de progresso desconhecida; usando sessão vazia e bloqueando gravação para preservar o arquivo".into()),
            ),
            Err(err) => (
                ProgressData::current(),
                Some(format!("progresso inválido ({err}); usando valores vazios. O original será preservado como .corrupt-* ao salvar")),
            ),
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => (ProgressData::current(), None),
        Err(err) => (
            ProgressData::current(),
            Some(format!("não consegui ler progresso: {err}")),
        ),
    }
}

pub fn save(data: &ProgressData) -> std::io::Result<()> {
    save_to(data, &default_path())
}

pub fn save_to(data: &ProgressData, path: &Path) -> std::io::Result<()> {
    if data.version > 1 {
        return Err(std::io::Error::other(
            "não é possível gravar esta versão de progresso",
        ));
    }
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let text = toml::to_string_pretty(data).map_err(std::io::Error::other)?;
    match std::fs::read(path) {
        Ok(original) => {
            let parsed = std::str::from_utf8(&original)
                .ok()
                .and_then(|s| toml::from_str::<ProgressData>(s).ok());
            match parsed {
                Some(previous) if previous.version > 1 => {
                    return Err(std::io::Error::other(
                        "versão de progresso mais nova; arquivo preservado, salvamento bloqueado",
                    ));
                }
                None => {
                    let (backup_path, mut backup) = create_sibling(path, "corrupt")?;
                    if let Err(err) = backup.write_all(&original).and_then(|()| backup.sync_all()) {
                        let _ = std::fs::remove_file(backup_path);
                        return Err(err);
                    }
                }
                _ => {}
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    let (tmp, mut file) = create_sibling(path, "tmp")?;
    let result = file
        .write_all(text.as_bytes())
        .and_then(|()| file.sync_all());
    drop(file);
    let result = result.and_then(|()| std::fs::rename(&tmp, path));
    if result.is_err() {
        let _ = std::fs::remove_file(tmp);
    }
    result
}

fn create_sibling(path: &Path, kind: &str) -> std::io::Result<(PathBuf, std::fs::File)> {
    for _ in 0..100 {
        let mut name = path.file_name().unwrap_or_default().to_os_string();
        name.push(format!(
            ".{kind}-{}-{}",
            std::process::id(),
            FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let sibling = path.with_file_name(name);
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        match options.open(&sibling) {
            Ok(file) => return Ok((sibling, file)),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }
    Err(std::io::Error::other(
        "não foi possível reservar arquivo temporário de progresso",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("rustlab-storage-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn roundtrip_grava_progresso_e_hints() {
        let dir = dir("roundtrip");
        let path = dir.join("progresso.toml");
        let mut data = ProgressData::current();
        data.read_lessons.insert("hello".into());
        data.completed_lessons.insert("hello".into());
        data.completed_exercises.insert("soma".into());
        data.hints_used.insert("soma".into(), 2);
        data.exercise_answers.insert(
            "soma".into(),
            "pub fn soma(a: i32, b: i32) -> i32 {\n    a + b\n}\n".into(),
        );
        save_to(&data, &path).unwrap();
        assert_eq!(load_from(&path).0, data);
        assert!(!path.with_extension("toml.tmp").exists());
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn arquivo_corrompido_recupera_defaults_com_aviso() {
        let dir = dir("corrupt");
        let path = dir.join("progresso.toml");
        std::fs::write(&path, "[isso não é toml").unwrap();
        let (data, warning) = load_from(&path);
        assert!(warning.is_some());
        assert!(data.completed_lessons.is_empty());
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn arquivo_ausente_e_primeiro_uso_sem_aviso() {
        let dir = dir("missing");
        let (data, warning) = load_from(&dir.join("missing.toml"));
        assert!(warning.is_none());
        assert_eq!(data.version, 1);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn progresso_antigo_sem_respostas_continua_compativel() {
        let dir = dir("legacy");
        let path = dir.join("progresso.toml");
        std::fs::write(&path, "version = 1\ncompleted_exercises = ['soma']\n").unwrap();
        let (data, warning) = load_from(&path);
        assert!(warning.is_none());
        assert!(data.completed_exercises.contains("soma"));
        assert!(data.exercise_answers.is_empty());
        assert!(data.xp_events.is_empty());
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn salvar_preserva_bytes_corrompidos_em_backup() {
        let dir = dir("backup");
        let path = dir.join("progresso.toml");
        let original = b"nao e TOML\xff";
        std::fs::write(&path, original).unwrap();
        save_to(&ProgressData::current(), &path).unwrap();
        assert!(load_from(&path).1.is_none());
        let backup = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains(".corrupt-")
            })
            .unwrap();
        assert_eq!(std::fs::read(backup).unwrap(), original);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn versao_futura_nunca_e_sobrescrita() {
        let dir = dir("future");
        let path = dir.join("progresso.toml");
        let original = "version = 999\ncompleted_exercises = ['soma']\n";
        std::fs::write(&path, original).unwrap();
        assert!(load_from(&path).1.is_some());
        assert!(save_to(&ProgressData::current(), &path).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn gravacoes_simultaneas_nao_compartilham_temporario() {
        let dir = dir("parallel");
        let path = dir.join("progresso.toml");
        std::thread::scope(|scope| {
            for i in 0..8 {
                let path = &path;
                scope.spawn(move || {
                    let mut data = ProgressData::current();
                    data.exercise_answers
                        .insert("soma".into(), format!("resposta {i}"));
                    save_to(&data, path).unwrap();
                });
            }
        });
        let (data, warning) = load_from(&path);
        assert!(warning.is_none());
        assert!(data.exercise_answers["soma"].starts_with("resposta "));
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn srs_cards_roundtrip_e_compatibilidade_retroativa() {
        let dir = dir("srs-roundtrip");
        let path = dir.join("progresso.toml");

        // Arquivo v1 legado sem o campo `srs_cards` continua carregando limpo.
        std::fs::write(&path, "version = 1\ncompleted_lessons = ['hello-world']\n").unwrap();
        let (legacy, warning) = load_from(&path);
        assert!(warning.is_none());
        assert!(legacy.srs_cards.is_empty());

        let mut data = legacy;
        data.srs_cards.insert(
            "B-P01".into(),
            SrsCard {
                mastery_level: 2,
                interval_steps: 3,
                due_at_step: 4,
                independent_passes: 2,
                review_count: 2,
            },
        );
        save_to(&data, &path).unwrap();
        let (loaded, warning) = load_from(&path);
        assert!(warning.is_none());
        assert_eq!(loaded.srs_cards, data.srs_cards);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn error_counts_e_onboarding_persistem_com_compatibilidade() {
        let dir = dir("errors-onboarding");
        let path = dir.join("progresso.toml");

        std::fs::write(&path, "version = 1\n").unwrap();
        let (mut data, warning) = load_from(&path);
        assert!(warning.is_none());
        assert!(data.error_counts.is_empty());
        assert!(!data.onboarding_completed);

        data.error_counts.insert("E0382".into(), 4);
        data.onboarding_completed = true;
        save_to(&data, &path).unwrap();
        let (loaded, warning) = load_from(&path);
        assert!(warning.is_none());
        assert_eq!(loaded.error_counts.get("E0382").copied(), Some(4));
        assert!(loaded.onboarding_completed);
        std::fs::remove_dir_all(dir).ok();
    }
}
