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
pub enum OriginLanguage {
    #[default]
    None,
    Python,
    JavaScriptTs,
    CCpp,
    Go,
    JavaCSharp,
}

impl OriginLanguage {
    pub const ALL: [OriginLanguage; 6] = [
        OriginLanguage::None,
        OriginLanguage::Python,
        OriginLanguage::JavaScriptTs,
        OriginLanguage::CCpp,
        OriginLanguage::Go,
        OriginLanguage::JavaCSharp,
    ];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&l| l == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Self {
        let idx = Self::ALL.iter().position(|&l| l == self).unwrap_or(0);
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub const fn label_for(self, english: bool) -> &'static str {
        match (self, english) {
            (OriginLanguage::None, true) => "None (fresh start)",
            (OriginLanguage::None, false) => "Nenhuma (começando do zero)",
            (OriginLanguage::Python, _) => "Python",
            (OriginLanguage::JavaScriptTs, _) => "JavaScript / TypeScript",
            (OriginLanguage::CCpp, _) => "C / C++",
            (OriginLanguage::Go, _) => "Go",
            (OriginLanguage::JavaCSharp, _) => "Java / C#",
        }
    }

    pub fn polyglot_callout_for(self, topic_hint: &str, english: bool) -> Option<String> {
        if self == OriginLanguage::None {
            return None;
        }
        let topic = topic_hint.to_ascii_lowercase();
        let ownership = topic.contains("ownership")
            || topic.contains("move")
            || topic.contains("borrow")
            || topic.contains("referenc")
            || topic.contains("lifetime")
            || topic.contains("b-t")
            || topic.contains("m0");
        let errors = topic.contains("option")
            || topic.contains("result")
            || topic.contains("erro")
            || topic.contains("interrogacao");
        let traits = topic.contains("trait")
            || topic.contains("generic")
            || topic.contains("struct")
            || topic.contains("enum");
        let async_conc =
            topic.contains("thread") || topic.contains("async") || topic.contains("concorr");

        let msg = match (self, ownership, errors, traits, async_conc, english) {
            (OriginLanguage::Python, true, _, _, _, true) => {
                "From Python: `a = b` does NOT alias a garbage-collected object for heap types like `String` or `Vec` — it moves ownership unless you borrow (`&b`) or explicitly `.clone()`."
            }
            (OriginLanguage::Python, true, _, _, _, false) => {
                "Para quem vem de Python: `a = b` NÃO cria apelido para o mesmo objeto com GC em tipos heap (`String`, `Vec`) — ele move a posse, salvo se você emprestar (`&b`) ou chamar `.clone()`."
            }
            (OriginLanguage::Python, _, true, _, _, true) => {
                "From Python: instead of `try/except` exceptions and `None` runtime errors, Rust encodes absence (`Option<T>`) and recoverable failures (`Result<T, E>`) right in return types."
            }
            (OriginLanguage::Python, _, true, _, _, false) => {
                "Para quem vem de Python: em vez de exceções `try/except` e `None` surpresa em runtime, Rust coloca ausência (`Option<T>`) e falhas (`Result<T, E>`) direto no tipo de retorno."
            }
            (OriginLanguage::JavaScriptTs, true, _, _, _, true) => {
                "From JS/TS: `const` only stops variable rebinding (objects still mutate); in Rust, `let` is deeply immutable until you opt into `mut` or `&mut`, with no GC needed."
            }
            (OriginLanguage::JavaScriptTs, true, _, _, _, false) => {
                "Para quem vem de JS/TS: `const` lá só trava reatribuição da variável; em Rust, `let` protege os dados contra mutação até você optar por `mut`/`&mut`, sem depender de Garbage Collector."
            }
            (OriginLanguage::JavaScriptTs, _, true, _, _, true) => {
                "From JS/TS: avoid `undefined`/`null` checks or `throw`; combine `Option`/`Result` with `match` and `?` so missing cases fail at compile time."
            }
            (OriginLanguage::JavaScriptTs, _, true, _, _, false) => {
                "Para quem vem de JS/TS: em vez de `null`/`undefined` e `throw`, use `Option`/`Result` com `match` e `?` para garantir tratamento ainda na compilação."
            }
            (OriginLanguage::CCpp, true, _, _, _, true) => {
                "From C/C++: Rust gives you RAII + `std::unique_ptr`-like move semantics by default (`Box`, `String`, `Vec`), and makes `&T` / `&mut T` aliasing rules compile-time checked!"
            }
            (OriginLanguage::CCpp, true, _, _, _, false) => {
                "Para quem vem de C/C++: Rust adota RAII e semântica de move por padrão (estilo `unique_ptr`), e o compilador verifica regras de aliasing (`&T` vs `&mut T`) antes de gerar binário!"
            }
            (OriginLanguage::CCpp, _, _, _, true, true) => {
                "From C/C++: `Send` and `Sync` traits turn data races into compile errors instead of undefined behavior in multithreaded code."
            }
            (OriginLanguage::CCpp, _, _, _, true, false) => {
                "Para quem vem de C/C++: os traits `Send` e `Sync` transformam data races em erro de compilação em vez de comportamento indefinido em threads."
            }
            (OriginLanguage::Go, _, true, _, _, true) => {
                "From Go: instead of ignoring `(val, err)` tuples or `if err != nil` boilerplate, Rust wraps both in `Result<T, E>` and gives you `?` plus exhaustive `match`."
            }
            (OriginLanguage::Go, _, true, _, _, false) => {
                "Para quem vem de Go: em vez de tuplas `(val, err)` e `if err != nil`, Rust unifica tudo em `Result<T, E>` com o operador `?` e `match` exaustivo."
            }
            (OriginLanguage::Go, _, _, true, _, true) => {
                "From Go: `trait` is explicit (unlike Go structural interfaces), giving you zero-cost static dispatch with generics `<T: Trait>` or `dyn Trait` when needed."
            }
            (OriginLanguage::Go, _, _, true, _, false) => {
                "Para quem vem de Go: `trait` é implementado explicitamente (diferente de interfaces implícitas de Go), permitindo despacho estático zero-cost com `<T: Trait>`."
            }
            (OriginLanguage::JavaCSharp, _, _, true, _, true) => {
                "From Java/C#: prefer composition with `struct` + `enum` + `trait` instead of class inheritance hierarchies; data (`struct`) stays separate from behavior (`impl`/`trait`)."
            }
            (OriginLanguage::JavaCSharp, _, _, true, _, false) => {
                "Para quem vem de Java/C#: prefira composição com `struct` + `enum` + `trait` em vez de herança de classes; dados (`struct`) ficam separados de comportamento (`impl`/`trait`)."
            }
            (OriginLanguage::JavaCSharp, _, true, _, _, true) => {
                "From Java/C#: there is no `NullPointerException` — use `Option<T>` when a value may be absent and `Result<T, E>` for checked errors without heavy stack unwinding."
            }
            (OriginLanguage::JavaCSharp, _, true, _, _, false) => {
                "Para quem vem de Java/C#: não existe `NullPointerException` — use `Option<T>` para ausência explícita e `Result<T, E>` para erros tratados sem custo de exceções."
            }
            (OriginLanguage::Python, _, _, _, _, true) => {
                "Tip for Python devs: Rust compiles to native machine code; explicit types let `rustc` catch mismatches early and run loops at C speed."
            }
            (OriginLanguage::Python, _, _, _, _, false) => {
                "Dica para quem vem de Python: Rust compila para código nativo; tipos explícitos permitem ao `rustc` pegar incompatibilidades cedo e rodar loops em velocidade de C."
            }
            (OriginLanguage::JavaScriptTs, _, _, _, _, true) => {
                "Tip for JS/TS devs: think of `match` + `enum` as TypeScript discriminated unions enforced everywhere at compile time."
            }
            (OriginLanguage::JavaScriptTs, _, _, _, _, false) => {
                "Dica para quem vem de JS/TS: pense em `enum` + `match` como uniões discriminadas do TypeScript verificadas exaustivamente na compilação."
            }
            (OriginLanguage::CCpp, _, _, _, _, true) => {
                "Tip for C/C++ devs: same zero-cost abstractions and deterministic Drop, without manual `free`/`delete` or dangling pointer UB in safe Rust."
            }
            (OriginLanguage::CCpp, _, _, _, _, false) => {
                "Dica para quem vem de C/C++: mesmas abstrações zero-cost e Drop determinístico, mas sem `free`/`delete` manual ou ponteiros soltos em Rust seguro."
            }
            (OriginLanguage::Go, _, _, _, _, true) => {
                "Tip for Go devs: Rust trades the garbage collector for deterministic ownership lifetimes, giving predictable latency with fearless concurrency."
            }
            (OriginLanguage::Go, _, _, _, _, false) => {
                "Dica para quem vem de Go: Rust troca o Garbage Collector por ownership determinístico, entregando latência previsível e concorrência sem data races."
            }
            (OriginLanguage::JavaCSharp, _, _, _, _, true) => {
                "Tip for Java/C# devs: `Result` + `Option` + `trait` replace null references, checked exceptions, and deep OOP hierarchies with lean value semantics."
            }
            (OriginLanguage::JavaCSharp, _, _, _, _, false) => {
                "Dica para quem vem de Java/C#: `Result`, `Option` e `trait` substituem referências nulas, exceções e hierarquias OOP pesadas por semântica de valor enxuta."
            }
            (OriginLanguage::None, _, _, _, _, _) => return None,
        };
        Some(msg.to_string())
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GamificationVisibility {
    #[default]
    Visible,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct UiSettings {
    pub theme: ThemeId,
    pub animations: MotionMode,
    pub gamification: GamificationVisibility,
    #[serde(default = "default_true")]
    pub inlay_hints: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: ThemeId::default(),
            animations: MotionMode::default(),
            gamification: GamificationVisibility::default(),
            inlay_hints: true,
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct ExecSettings {
    pub consent: ExecConsent,
    #[serde(default = "default_true")]
    pub shared_target_cache: bool,
    #[serde(default)]
    pub target_cache_dir: Option<String>,
}

impl Default for ExecSettings {
    fn default() -> Self {
        Self {
            consent: ExecConsent::default(),
            shared_target_cache: true,
            target_cache_dir: None,
        }
    }
}

impl ExecSettings {
    pub fn resolved_target_cache_dir(&self) -> Option<PathBuf> {
        if !self.shared_target_cache {
            return None;
        }
        if let Some(custom) = self
            .target_cache_dir
            .as_deref()
            .filter(|s| !s.trim().is_empty())
        {
            return Some(PathBuf::from(custom));
        }
        if let Some(cache_home) = std::env::var_os("XDG_CACHE_HOME") {
            let p = PathBuf::from(cache_home);
            if p.is_absolute() {
                return Some(p.join("rustlab").join("target-cache"));
            }
        }
        if let Some(home) = std::env::var_os("HOME") {
            return Some(
                PathBuf::from(home)
                    .join(".cache")
                    .join("rustlab")
                    .join("target-cache"),
            );
        }
        Some(std::env::temp_dir().join("rustlab-target-cache"))
    }
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
    #[serde(default)]
    pub origin_language: OriginLanguage,
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
            origin_language: OriginLanguage::default(),
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
            origin_language: OriginLanguage::Python,
            ui: UiSettings {
                theme: ThemeId::Ferrugem,
                animations: MotionMode::Reduced,
                gamification: GamificationVisibility::Hidden,
                inlay_hints: true,
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
    fn linguagem_de_origem_e_cache_compartilhado_funcionam() {
        assert_eq!(OriginLanguage::ALL.len(), 6);
        assert_eq!(OriginLanguage::None.next(), OriginLanguage::Python);
        assert_eq!(OriginLanguage::None.prev(), OriginLanguage::JavaCSharp);
        assert!(
            OriginLanguage::None
                .polyglot_callout_for("ownership", true)
                .is_none()
        );
        let py_en = OriginLanguage::Python
            .polyglot_callout_for("ownership-move", true)
            .unwrap();
        let py_pt = OriginLanguage::Python
            .polyglot_callout_for("ownership-move", false)
            .unwrap();
        assert!(py_en.contains("Python"));
        assert!(py_pt.contains("Python"));
        assert_ne!(py_en, py_pt);

        let mut exec = ExecSettings::default();
        assert!(exec.shared_target_cache);
        assert!(exec.resolved_target_cache_dir().is_some());
        exec.shared_target_cache = false;
        assert!(exec.resolved_target_cache_dir().is_none());
        exec.shared_target_cache = true;
        exec.target_cache_dir = Some("/tmp/custom-rustlab-cache".into());
        assert_eq!(
            exec.resolved_target_cache_dir().unwrap(),
            PathBuf::from("/tmp/custom-rustlab-cache")
        );

        assert!(UiSettings::default().inlay_hints);
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
