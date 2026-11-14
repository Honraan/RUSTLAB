//! Projetos independentes, descoberta de fontes e escrita local protegida.

use std::path::{Path, PathBuf};

/// Capacidades técnicas verificáveis em um projeto independente da Etapa 10.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ProjectCapabilities {
    pub has_plan_md: bool,
    pub source_file_count: usize,
    pub has_structs_or_enums: bool,
    pub has_error_handling: bool,
    pub has_unit_tests: bool,
    pub has_doc_comments: bool,
}

/// Um projeto Cargo aberto: raiz + arquivos-fonte conhecidos.
#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
}

impl Project {
    /// `Some` apenas se `root/Cargo.toml` existe.
    pub fn detect(root: impl AsRef<Path>) -> Option<Project> {
        let root = expand_home(root.as_ref());
        if root.join("Cargo.toml").is_file() {
            Some(Project {
                root: root.canonicalize().ok()?,
            })
        } else {
            None
        }
    }

    pub fn name(&self) -> String {
        self.root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "projeto".into())
    }

    /// Arquivos editáveis: Cargo.toml + todos os .rs sob src/ (sem
    /// target/, .git/, profundidade limitada). Ordem estável: Cargo.toml
    /// primeiro, depois src em ordem alfabética.
    pub fn source_files(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let cargo = self.root.join("Cargo.toml");
        if cargo.is_file() {
            out.push(cargo);
        }
        let src = self.root.join("src");
        if src.is_dir() {
            collect_rs(&src, &mut out, 6);
        }
        for dir in ["tests", "examples", "benches"] {
            collect_rs(&self.root.join(dir), &mut out, 6);
        }
        if self.root.join("build.rs").is_file() {
            out.push(self.root.join("build.rs"));
        }
        if self.root.join("PLANO.md").is_file() {
            out.push(self.root.join("PLANO.md"));
        }
        out
    }

    /// Gera um modelo `PLANO.md` para a Etapa 10 sem sobrescrever se já existir.
    pub fn write_project_plan_template(&self) -> std::io::Result<PathBuf> {
        use std::io::Write;
        let path = self.root.join("PLANO.md");
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        let template = format!(
            "# Plano do Projeto Independente — {}\n\n\
             ## 1. Problema e objetivo\n\
             - O que este programa resolve e para quem?\n\n\
             ## 2. Entradas, saídas e interface\n\
             - Argumentos CLI, arquivos ou comandos interativos:\n\n\
             ## 3. Arquitetura (módulos, structs e enums)\n\
             - `src/main.rs`: ponto de entrada e interface\n\
             - `src/lib.rs` ou submódulos: lógica pura e tratamento de erros (`Option`/`Result`)\n\n\
             ## 4. Marcos verificáveis (Etapa 10)\n\
             - [ ] Código formatado (`cargo fmt`, F8)\n\
             - [ ] Sem avisos relevantes (`cargo clippy`, Ctrl+L / Alt+C)\n\
             - [ ] Testes automatizados (`#[test]`, F6) cobrindo casos de borda\n\
             - [ ] Documentação `///` nos itens públicos e execução validada (`F5`)\n",
            self.name()
        );
        file.write_all(template.as_bytes())?;
        Ok(path)
    }

    /// Inspeciona capacidades reais demonstradas no código do projeto.
    pub fn inspect_capabilities(&self) -> ProjectCapabilities {
        let files = self.source_files();
        let rs_files: Vec<&PathBuf> = files
            .iter()
            .filter(|p| p.extension().is_some_and(|e| e == "rs"))
            .collect();
        let mut caps = ProjectCapabilities {
            has_plan_md: self.root.join("PLANO.md").is_file(),
            source_file_count: rs_files.len(),
            ..ProjectCapabilities::default()
        };
        for file in rs_files {
            let Ok(text) = std::fs::read_to_string(file) else {
                continue;
            };
            if text.contains("struct ") || text.contains("enum ") {
                caps.has_structs_or_enums = true;
            }
            if text.contains("Result<") || text.contains("Option<") || text.contains("Err(") {
                caps.has_error_handling = true;
            }
            if text.contains("#[test]") {
                caps.has_unit_tests = true;
            }
            if text.contains("///") {
                caps.has_doc_comments = true;
            }
        }
        caps
    }

    pub fn read(&self, path: &Path) -> std::io::Result<String> {
        self.check_path(path)?;
        std::fs::read_to_string(path)
    }

    /// Salva criando diretórios pais se preciso; grava em .tmp e renomeia
    /// (preserva o trabalho em caso de falha no meio da escrita).
    pub fn save(&self, path: &Path, content: &str) -> std::io::Result<()> {
        use std::io::Write;
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        self.check_path(path)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension(format!(
            "rlab-tmp-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)?;
        let result = (|| {
            if let Ok(metadata) = std::fs::metadata(path) {
                file.set_permissions(metadata.permissions())?;
            }
            file.write_all(content.as_bytes())?;
            file.sync_all()?;
            std::fs::rename(&tmp, path)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(tmp);
        }
        result
    }

    fn check_path(&self, path: &Path) -> std::io::Result<()> {
        let root = self.root.canonicalize()?;
        let resolved = path.canonicalize().or_else(|_| {
            Ok::<_, std::io::Error>(
                path.parent()
                    .ok_or_else(|| std::io::Error::other("caminho sem diretório"))?
                    .canonicalize()?
                    .join(path.file_name().unwrap_or_default()),
            )
        })?;
        if !resolved.starts_with(root) {
            return Err(std::io::Error::other("arquivo fora do projeto"));
        }
        Ok(())
    }

    /// Cria somente caminhos relativos, sem sobrescrever arquivos existentes.
    pub fn create_file(&self, relative: &str) -> std::io::Result<PathBuf> {
        use std::io::Write;
        use std::path::Component;
        let path = Path::new(relative);
        if relative.is_empty()
            || !path.components().all(|c| matches!(c, Component::Normal(_)))
            || !path.extension().is_some_and(|e| e == "rs")
        {
            return Err(std::io::Error::other(
                "use um caminho relativo .rs, sem .. (ex.: src/util.rs)",
            ));
        }
        if relative != "build.rs"
            && !["src", "tests", "examples", "benches"]
                .iter()
                .any(|dir| path.starts_with(dir))
        {
            return Err(std::io::Error::other(
                "crie em src/, tests/, examples/ ou benches/ (ou build.rs)",
            ));
        }
        if path.components().count() > 7
            || path
                .components()
                .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
        {
            return Err(std::io::Error::other(
                "caminho profundo demais ou oculto; use até cinco subdiretórios",
            ));
        }
        let root = self.root.canonicalize()?;
        let mut parent = root.clone();
        for part in path.parent().into_iter().flat_map(Path::components) {
            parent.push(part);
            if !parent.exists() {
                std::fs::create_dir(&parent)?;
            }
            if !parent.canonicalize()?.starts_with(&root) {
                return Err(std::io::Error::other("diretório fora do projeto"));
            }
        }
        let target = parent.join(path.file_name().unwrap());
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)?;
        file.write_all(b"// Novo arquivo Rust. Declare o modulo no arquivo pai, se necessario.\n")?;
        Ok(target)
    }
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>, depth: u8) {
    if depth == 0 || std::fs::symlink_metadata(dir).map_or(true, |m| m.is_symlink()) {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        if e.file_type().map_or(true, |kind| kind.is_symlink()) {
            continue;
        }
        let p = e.path();
        let name = e.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || name == "target" {
            continue;
        }
        if p.is_dir() {
            collect_rs(&p, out, depth - 1);
        } else if name.ends_with(".rs") {
            out.push(p);
        }
    }
}

/// Expande um `~` inicial (não usamos shell, então fazemos aqui).
pub fn expand_home(p: &Path) -> PathBuf {
    let s = p.to_string_lossy();
    if (s == "~" || s.starts_with("~/"))
        && let Some(home) = std::env::var_os("HOME")
    {
        let mut base = PathBuf::from(home);
        for part in p.components().skip(1) {
            base.push(part);
        }
        return base;
    }
    p.to_path_buf()
}

/// Diretório padrão onde o RustLab cria projetos do aluno.
pub fn default_workspace_dir() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share")))
        .unwrap_or_else(|| PathBuf::from("."));
    workspace_dir_for(&base)
}

fn workspace_dir_for(base: &Path) -> PathBuf {
    base.join("rustlab").join("projetos")
}

/// Resolve alvo de criação (`(pasta_pai, nome_do_projeto)`) aceitando tanto
/// um nome simples quanto caminhos completos ou relativos (ex.: `~/apps/jogo`).
pub fn resolve_new_project_target(
    raw: &str,
    default_base: &Path,
) -> Result<(PathBuf, String), String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("digite o nome do projeto ou pasta/nome (ex.: ~/apps/meu_app)".into());
    }
    let has_sep = trimmed.contains('/') || trimmed.contains(std::path::MAIN_SEPARATOR);
    if has_sep {
        let expanded = expand_home(Path::new(trimmed));
        if expanded
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err("caminho inválido: não use '..'".into());
        }
        let full = if expanded.is_absolute() {
            expanded
        } else {
            default_base.join(expanded)
        };
        let raw_name = full.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
            "indique o nome do projeto após a pasta (ex.: ~/apps/meu_app)".to_string()
        })?;
        let valid_name = validate_project_name_err(raw_name)?;
        let parent = full
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .ok_or_else(|| "diretório pai inválido".to_string())?;
        Ok((parent.to_path_buf(), valid_name))
    } else {
        let valid_name = validate_project_name_err(trimmed)?;
        Ok((default_base.to_path_buf(), valid_name))
    }
}

fn validate_project_name_err(raw: &str) -> Result<String, String> {
    crate::execution::sanitize_project_name(raw).ok_or_else(|| {
        if raw.trim() == "test" {
            "'test' é reservado pelo Rust. Use 'meu_teste' ou 'teste'.".to_string()
        } else {
            "use só letras, números, _ e - (começando com letra ou _)".to_string()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("rustlab-proj-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn detect_precisa_de_cargo_toml() {
        let d = temp_dir("detect");
        assert!(Project::detect(&d).is_none());
        std::fs::write(d.join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();
        assert!(Project::detect(&d).is_some());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn lista_fontes_sem_target_e_ordena() {
        let d = temp_dir("list");
        std::fs::create_dir_all(d.join("src/nested")).unwrap();
        std::fs::create_dir_all(d.join("target/debug")).unwrap();
        std::fs::create_dir_all(d.join(".git")).unwrap();
        std::fs::write(d.join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();
        std::fs::write(d.join("src/main.rs"), "fn main(){}").unwrap();
        std::fs::write(d.join("src/lib.rs"), "").unwrap();
        std::fs::write(d.join("src/nested/mod.rs"), "").unwrap();
        std::fs::write(d.join("target/debug/ignore.rs"), "").unwrap();
        std::fs::write(d.join(".git/ignore.rs"), "").unwrap();
        let p = Project { root: d.clone() };
        let files: Vec<String> = p
            .source_files()
            .iter()
            .map(|f| f.strip_prefix(&d).unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(
            files,
            vec![
                "Cargo.toml",
                "src/lib.rs",
                "src/main.rs",
                "src/nested/mod.rs"
            ]
        );
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn save_escreve_e_le() {
        let d = temp_dir("save");
        std::fs::create_dir_all(d.join("src")).unwrap();
        let p = Project { root: d.clone() };
        let f = d.join("src/main.rs");
        p.save(&f, "fn main() { println!(\"oi\"); }").unwrap();
        assert_eq!(p.read(&f).unwrap(), "fn main() { println!(\"oi\"); }");
        assert!(
            !f.with_extension("rs.rlab-tmp").exists(),
            "tmp deve ser renomeado"
        );
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn expand_home_funciona() {
        let home = std::env::var("HOME").unwrap();
        let p = expand_home(Path::new("~/projetos/x"));
        assert!(p.starts_with(home));
        let abs = expand_home(Path::new("/tmp/x"));
        assert_eq!(abs, PathBuf::from("/tmp/x"));
    }

    #[test]
    fn workspace_dir_absoluto() {
        assert!(default_workspace_dir().is_absolute() || std::env::var_os("HOME").is_none());
    }

    #[test]
    fn workspace_respeita_xdg_data_home() {
        assert_eq!(
            workspace_dir_for(Path::new("/dados")),
            PathBuf::from("/dados/rustlab/projetos")
        );
    }

    #[test]
    fn novos_arquivos_nao_sobrescrevem_nem_escapam_do_projeto() {
        let dir = temp_dir("safe-new");
        let project = Project { root: dir.clone() };
        let path = project.create_file("src/tools/math.rs").unwrap();
        std::fs::write(&path, "trabalho do aluno").unwrap();
        assert!(project.create_file("src/tools/math.rs").is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "trabalho do aluno");
        for invalid in [
            "../escape.rs",
            "/tmp/escape.rs",
            "src/../../escape.rs",
            "",
            "Cargo.toml",
        ] {
            assert!(project.create_file(invalid).is_err(), "{invalid}");
        }
        let outside = temp_dir("outside");
        std::os::unix::fs::symlink(&outside, dir.join("outside")).unwrap();
        assert!(project.create_file("outside/escape.rs").is_err());
        assert!(!outside.join("escape.rs").exists());
        assert!(project.save(&outside.join("escape.rs"), "não").is_err());
        std::fs::remove_dir_all(dir).unwrap();
        std::fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    fn lista_testes_exemplos_e_ignora_diretorios_simbolicos() {
        let dir = temp_dir("more-files");
        let project = Project { root: dir.clone() };
        project.create_file("tests/integration.rs").unwrap();
        project.create_file("examples/demo.rs").unwrap();
        project.create_file("benches/bench.rs").unwrap();
        std::os::unix::fs::symlink(&dir, dir.join("src")).unwrap();
        assert_eq!(project.source_files().len(), 3);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn gera_plano_sem_sobrescrever_e_inspeciona_capacidades() {
        let dir = temp_dir("plan-capabilities");
        std::fs::create_dir_all(dir.join("src")).unwrap();
        let project = Project { root: dir.clone() };
        let plan_path = project.write_project_plan_template().unwrap();
        assert!(plan_path.exists());
        let initial = std::fs::read_to_string(&plan_path).unwrap();
        assert!(initial.contains("Problema e objetivo"));

        let second = project.write_project_plan_template();
        assert!(matches!(
            second,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists
        ));
        assert_eq!(std::fs::read_to_string(&plan_path).unwrap(), initial);

        std::fs::write(
            dir.join("src/main.rs"),
            "/// Funcao principal.\nfn main() -> Result<(), String> { Ok(()) }\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("src/model.rs"),
            "pub struct Tarefa { pub id: u32 }\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn ok() {}\n}\n",
        )
        .unwrap();

        let files = project.source_files();
        assert!(files.iter().any(|path| path.ends_with("PLANO.md")));

        let caps = project.inspect_capabilities();
        assert!(caps.has_plan_md);
        assert_eq!(caps.source_file_count, 2);
        assert!(caps.has_structs_or_enums);
        assert!(caps.has_error_handling);
        assert!(caps.has_unit_tests);
        assert!(caps.has_doc_comments);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn resolve_alvo_em_pasta_padrao_subpasta_e_caminho_absoluto() {
        let default_base = PathBuf::from("/tmp/base-projetos");

        let (dir, name) = resolve_new_project_target("meu_cli", &default_base).unwrap();
        assert_eq!(dir, default_base);
        assert_eq!(name, "meu_cli");

        let (dir, name) = resolve_new_project_target("ferramentas/calc-rs", &default_base).unwrap();
        assert_eq!(dir, default_base.join("ferramentas"));
        assert_eq!(name, "calc-rs");

        let (dir, name) = resolve_new_project_target("/tmp/outro_dir/app2", &default_base).unwrap();
        assert_eq!(dir, PathBuf::from("/tmp/outro_dir"));
        assert_eq!(name, "app2");

        assert!(resolve_new_project_target("../fora/app", &default_base).is_err());
        assert!(resolve_new_project_target("~/test", &default_base).is_err());
        assert!(resolve_new_project_target("   ", &default_base).is_err());
    }
}
