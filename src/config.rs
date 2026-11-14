//! Configurações persistidas em TOML (`~/.config/rustlab/config.toml`).
//!
//! Regras da especificação (seções 14 e 16): valores recuperam defaults
//! quando ausentes, um arquivo malformado não impede o app de iniciar —
//! reportamos o problema como aviso e usamos os padrões.

use std::fmt;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Nível de animação desejado pelo usuário.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MotionMode {
    /// Transições e efeitos completos (padrão).
    #[default]
    Full,
    /// Sem movimento: fades instantâneos/curtos, nada em deslocamento.
    /// Requisito de acessibilidade "reduced motion".
    Reduced,
    /// Nenhuma animação é executada.
    Off,
}

impl MotionMode {
    /// Ordem intencional: do menos para o mais intenso, para que ← = menos
    /// movimento e → = mais movimento (o clamp acontece nas pontas).
    pub const ALL: [MotionMode; 3] = [MotionMode::Off, MotionMode::Reduced, MotionMode::Full];

    pub fn label_key(self) -> &'static str {
        match self {
            MotionMode::Full => "settings.anim_full",
            MotionMode::Reduced => "settings.anim_reduced",
            MotionMode::Off => "settings.anim_off",
        }
    }

    /// Se animações devem realmente rodar neste modo.
    pub fn animate(self) -> bool {
        self != MotionMode::Off
    }

    /// Multiplicador de duração: modo reduzido encurta e simplifica.
    pub fn duration_scale(self) -> u32 {
        match self {
            MotionMode::Full => 100,
            MotionMode::Reduced => 40,
            MotionMode::Off => 0,
        }
    }
}

/// Identificador de tema. As paletas ficam em `crate::theme`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeId {
    /// Padrão: fundo escuro, acento ferrugem/laranja.
    #[default]
    Cuia,
    /// Varia mais fria, inspirada em forno de fundição.
    Ferrugem,
    /// Acessível: sem cor, apenas negrito/traçado sobre a cor do terminal.
    Mono,
    /// Varia em tons de ciano e azul profundo para leitura prolongada.
    Oceano,
}

impl ThemeId {
    pub const ALL: [ThemeId; 4] = [
        ThemeId::Cuia,
        ThemeId::Ferrugem,
        ThemeId::Mono,
        ThemeId::Oceano,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ThemeId::Cuia => "Cuia",
            ThemeId::Ferrugem => "Ferrugem",
            ThemeId::Mono => "Mono (sem cor)",
            ThemeId::Oceano => "Oceano",
        }
    }

    pub fn label_for(self, english: bool) -> &'static str {
        if english && self == ThemeId::Mono {
            "Mono (no color)"
        } else {
            self.label()
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GamificationVisibility {
    #[default]
    Visible,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct UiSettings {
    pub theme: ThemeId,
    pub animations: MotionMode,
    pub gamification: GamificationVisibility,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct SoundSettings {
    /// `false` silencia os sons de XP.
    pub enabled: bool,
    /// 0..=100: amplitude do tom suave de XP; zero silencia.
    pub volume: u8,
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 70,
        }
    }
}

/// Consentimento explícito antes de enviar trechos de código ou erros
/// a uma API remota compatível com OpenAI.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteAiConsent {
    #[default]
    Ask,
    Granted,
    Denied,
}

/// Configuração do mentor opcional de IA (Fase 8).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct AiSettings {
    pub provider: AiProvider,
    pub ollama_url: String,
    pub ollama_model: String,
    pub remote_url: String,
    pub remote_model: String,
    pub remote_consent: RemoteAiConsent,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            provider: AiProvider::Disabled,
            ollama_url: "http://127.0.0.1:11434".to_string(),
            ollama_model: "qwen2.5-coder:3b".to_string(),
            remote_url: "https://api.openai.com/v1".to_string(),
            remote_model: "gpt-4o-mini".to_string(),
            remote_consent: RemoteAiConsent::Ask,
        }
    }
}

/// Consentimento de execução de código (requisito 10: execução local NÃO
/// é sandbox; o usuário precisa aceitar explicitamente antes do primeiro
/// `cargo run`). `pending` pergunta de novo a cada sessão/primeira ação.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecConsent {
    #[default]
    Pending,
    Granted,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct ExecSettings {
    pub consent: ExecConsent,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct WorkspaceSettings {
    /// Último projeto aberto (reabre no Modo Criar).
    pub last_project: Option<String>,
    /// Mais recentes primeiro, no máximo oito caminhos canônicos.
    pub recent_projects: Vec<String>,
    /// Diretório padrão escolhido para novos projetos (`None` usa o padrão XDG).
    pub projects_dir: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AiProvider {
    #[default]
    Disabled,
    Ollama,
    Remote,
}

pub const SUPPORTED_LANGUAGES: [&str; 2] = ["en-US", "pt-BR"];

pub fn default_language() -> String {
    "en-US".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct Config {
    #[serde(default = "default_language")]
    pub language: String,
    pub ui: UiSettings,
    pub sound: SoundSettings,
    pub ai: AiSettings,
    pub execution: ExecSettings,
    pub workspace: WorkspaceSettings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: default_language(),
            ui: UiSettings::default(),
            sound: SoundSettings::default(),
            ai: AiSettings::default(),
            execution: ExecSettings::default(),
            workspace: WorkspaceSettings::default(),
        }
    }
}

impl Config {
    /// Construção a partir dos padrões; inglês (`en-US`) é o padrão inicial.
    pub fn defaults() -> Self {
        Self::default()
    }

    pub fn is_english(&self) -> bool {
        !self.language.trim().to_ascii_lowercase().starts_with("pt")
    }

    pub fn language_label(&self) -> &'static str {
        if self.is_english() {
            "English (en-US)"
        } else {
            "Português (pt-BR)"
        }
    }
}

/// Resultado do carregamento, preservando avisos para a UI exibir.
pub struct LoadOutcome {
    pub config: Config,
    /// Alguma coisa deu errado (arquivo ilegível/malformado) e usamos defaults.
    pub warning: Option<String>,
}

pub fn config_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("RUSTLAB_CONFIG") {
        return Some(PathBuf::from(path));
    }
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
        let dir = PathBuf::from(dir);
        if dir.is_absolute() {
            return Some(dir.join("rustlab").join("config.toml"));
        }
    }
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("rustlab")
            .join("config.toml"),
    )
}

pub fn load() -> LoadOutcome {
    let Some(path) = config_path() else {
        return LoadOutcome {
            config: Config::defaults(),
            warning: Some("sem HOME configurado; usando padrões desta sessão".into()),
        };
    };
    load_from_path(&path)
}

/// Lê um arquivo específico — separação que `load()` usa e os testes
/// exercitam sem tocar no HOME real.
pub fn load_from_path(path: &std::path::Path) -> LoadOutcome {
    match std::fs::read_to_string(path) {
        Ok(text) => toml::from_str::<Config>(&text)
            .map(|config| LoadOutcome {
                config,
                warning: None,
            })
            .unwrap_or_else(|err| LoadOutcome {
                config: Config::defaults(),
                warning: Some(format!(
                    "config.toml malformado ({}); usei padrões",
                    err.message()
                )),
            }),
        Err(err) if err.kind() == io::ErrorKind::NotFound => LoadOutcome {
            config: Config::defaults(),
            warning: None,
        },
        Err(err) => LoadOutcome {
            config: Config::defaults(),
            warning: Some(format!("não consegui ler {}: {err}", path.display())),
        },
    }
}

pub fn save(config: &Config) -> Result<(), SaveError> {
    let path = config_path().ok_or(SaveError::NoHomeDir)?;
    save_to(config, &path)
}

/// Grava em um caminho específico (quase-atômico: .tmp + rename).
pub fn save_to(config: &Config, path: &std::path::Path) -> Result<(), SaveError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = toml::to_string_pretty(config)?;
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[derive(Debug)]
pub enum SaveError {
    Io(io::Error),
    Serde(toml::ser::Error),
    NoHomeDir,
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Io(e) => write!(f, "{e}"),
            SaveError::Serde(e) => write!(f, "erro ao serializar: {e}"),
            SaveError::NoHomeDir => write!(f, "HOME/XDG_CONFIG_HOME não definados"),
        }
    }
}

impl From<io::Error> for SaveError {
    fn from(e: io::Error) -> Self {
        SaveError::Io(e)
    }
}

impl From<toml::ser::Error> for SaveError {
    fn from(e: toml::ser::Error) -> Self {
        SaveError::Serde(e)
    }
}

/// Detecção de suporte a Unicode/UTF-8 para escolher glifos seguros.
pub fn supports_unicode(env_lookup: &dyn Fn(&str) -> Option<String>) -> bool {
    let lang = env_lookup("LANG")
        .or_else(|| env_lookup("LC_ALL"))
        .unwrap_or_default();
    let up = lang.to_uppercase();
    up.contains("UTF-8") || up.contains("UTF8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_toml() {
        let cfg = Config {
            language: "pt-BR".into(),
            ui: UiSettings {
                theme: ThemeId::Ferrugem,
                animations: MotionMode::Reduced,
                gamification: GamificationVisibility::Hidden,
            },
            sound: SoundSettings {
                enabled: false,
                volume: 30,
            },
            ai: AiSettings::default(),
            execution: ExecSettings::default(),
            workspace: WorkspaceSettings::default(),
        };
        let text = toml::to_string_pretty(&cfg).unwrap();
        let back: Config = toml::from_str(&text).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn vazio_usa_defaults() {
        let cfg: Config = toml::from_str("").unwrap();
        assert_eq!(cfg.ui.theme, ThemeId::default());
        assert_eq!(cfg.ui.animations, MotionMode::default());
        assert_eq!(cfg.sound.volume, 70);
        assert!(cfg.workspace.recent_projects.is_empty());
    }

    #[test]
    fn recentes_roundtrip_sem_quebrar_config_antiga() {
        let mut config: Config =
            toml::from_str("[workspace]\nlast-project='/projetos/antigo'\n").unwrap();
        assert!(config.workspace.recent_projects.is_empty());
        config.workspace.recent_projects = vec!["/projetos/um".into(), "/projetos/dois".into()];
        let restored: Config = toml::from_str(&toml::to_string(&config).unwrap()).unwrap();
        assert_eq!(config, restored);
    }

    #[test]
    fn partial_file_merges_defaults() {
        let cfg: Config = toml::from_str("language = \"en-US\"\n[ui]\ntheme = \"mono\"\n").unwrap();
        assert_eq!(cfg.language, "en-US");
        assert_eq!(cfg.ui.theme, ThemeId::Mono);
        assert_eq!(cfg.ui.animations, MotionMode::default());
    }

    #[test]
    fn malformed_e_recuperado_pelo_load_como_warning() {
        let dir = std::env::temp_dir().join(format!("rustlab-cfg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("broken.toml");
        std::fs::write(&bad, "ui = [isto nao é toml").unwrap();
        let outcome = load_from_path(&bad);
        assert!(
            outcome.warning.is_some(),
            "arquivo corrompido deve gerar aviso"
        );
        assert_eq!(outcome.config.ui.animations, MotionMode::default());

        let missing = dir.join("inexistente.toml");
        let outcome = load_from_path(&missing);
        assert!(
            outcome.warning.is_none(),
            "ausente não é erro, é primeiro uso"
        );

        let ok = dir.join("bom.toml");
        save_to(&Config::defaults(), &ok).unwrap();
        let outcome = load_from_path(&ok);
        assert!(outcome.warning.is_none());
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn motion_modes() {
        assert!(!MotionMode::Off.animate());
        assert_eq!(MotionMode::Off.duration_scale(), 0);
        assert!(MotionMode::Reduced.animate());
        assert!(MotionMode::Reduced.duration_scale() < MotionMode::Full.duration_scale());
    }

    #[test]
    fn unicode_detect() {
        let lookup = |k: &str| match k {
            "LANG" => Some("pt_BR.UTF-8".to_string()),
            _ => None,
        };
        assert!(supports_unicode(&lookup));
        let lookup2 = |_: &str| Some("C".to_string());
        assert!(!supports_unicode(&lookup2));
    }

    #[test]
    fn escala_de_duracao_aplicada() {
        // Sanidade: full preserva, reduced encurta.
        let scaled = |mode: MotionMode, ms: u32| ms * mode.duration_scale() / 100;
        assert_eq!(scaled(MotionMode::Full, 900), 900);
        assert_eq!(scaled(MotionMode::Reduced, 900), 360);
        assert_eq!(scaled(MotionMode::Off, 900), 0);
    }

    #[test]
    fn config_ai_mantem_defaults_e_roundtrip() {
        let partial: Config = toml::from_str("[ai]\nprovider = \"ollama\"\n").unwrap();
        assert_eq!(partial.ai.provider, AiProvider::Ollama);
        assert_eq!(partial.ai.ollama_url, "http://127.0.0.1:11434");
        assert_eq!(partial.ai.ollama_model, "qwen2.5-coder:3b");
        assert_eq!(partial.ai.remote_consent, RemoteAiConsent::Ask);

        let mut cfg = Config::defaults();
        cfg.ai.provider = AiProvider::Remote;
        cfg.ai.remote_consent = RemoteAiConsent::Granted;
        cfg.ai.remote_model = "gpt-4.1-mini".to_string();
        let serialized = toml::to_string(&cfg).unwrap();
        let restored: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(cfg.ai, restored.ai);
    }

    #[test]
    fn idioma_padrao_e_ingles_na_primeira_abertura() {
        let cfg = Config::default();
        assert_eq!(cfg.language, "en-US");
        assert!(cfg.is_english());
        assert_eq!(Config::defaults().language, "en-US");

        let partial: Config = toml::from_str("[ui]\nanimations = \"off\"\n").unwrap();
        assert_eq!(partial.language, "en-US");
        assert!(partial.is_english());

        let missing =
            std::env::temp_dir().join(format!("rustlab-missing-{}.toml", std::process::id()));
        let outcome = load_from_path(&missing);
        assert_eq!(outcome.config.language, "en-US");
        assert!(outcome.config.is_english());
    }
}
