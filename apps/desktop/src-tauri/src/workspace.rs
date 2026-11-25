use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use rustlab::execution::sanitize_project_name;
use rustlab::projects::Project;
use serde::Serialize;

use crate::course::MAX_SOURCE;

#[derive(Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub root: String,
    pub files: Vec<String>,
}

pub struct Workspace {
    project: Project,
}

impl Workspace {
    pub fn open(path: &Path) -> Result<Self, String> {
        let project = Project::detect(path).ok_or("Escolha uma pasta com Cargo.toml.")?;
        let workspace = Self { project };
        workspace.file_path("Cargo.toml")?;
        Ok(workspace)
    }

    pub fn create(parent: &Path, raw_name: &str) -> Result<Self, String> {
        let name = sanitize_project_name(raw_name)
            .ok_or("Use letras sem acento, números, hífen ou sublinhado no nome do crate.")?;
        validate_component(&name)?;
        let parent = parent.canonicalize().map_err(error_text)?;
        let staging = tempfile::tempdir_in(&parent).map_err(error_text)?;
        let root = parent.join(&name);
        if root.try_exists().map_err(error_text)? {
            return Err("Já existe uma pasta com esse nome.".into());
        }
        fs::create_dir(staging.path().join("src")).map_err(error_text)?;
        fs::write(staging.path().join("Cargo.toml"), format!(
            "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n\n[workspace]\n"
        )).map_err(error_text)?;
        fs::write(
            staging.path().join("src/main.rs"),
            "fn main() {\n    println!(\"Olá, Rust!\");\n}\n",
        )
        .map_err(error_text)?;
        // On Windows this operation fails when the destination directory exists.
        fs::rename(staging.path(), &root).map_err(error_text)?;
        Self::open(&root)
    }

    pub fn root(&self) -> &Path {
        &self.project.root
    }

    pub fn info(&self) -> ProjectInfo {
        let files = self
            .project
            .source_files()
            .into_iter()
            .take(1000)
            .filter_map(|path| {
                let relative = path
                    .strip_prefix(self.root())
                    .ok()?
                    .to_str()?
                    .replace('\\', "/");
                self.file_path(&relative).ok()?;
                Some(relative)
            })
            .collect();
        ProjectInfo {
            name: self.project.name(),
            root: self.root().display().to_string(),
            files,
        }
    }

    fn file_path(&self, relative: &str) -> Result<PathBuf, String> {
        let path = Path::new(relative);
        validate_relative(path)?;
        let mut resolved = self.root().to_owned();
        if resolved.canonicalize().map_err(error_text)? != resolved {
            return Err("A pasta do projeto mudou. Abra o projeto novamente.".into());
        }
        for part in path {
            resolved.push(part);
            let metadata = fs::symlink_metadata(&resolved).map_err(error_text)?;
            if is_redirect(&metadata) {
                return Err("Links e pontos de reanálise não são editáveis.".into());
            }
        }
        if !resolved.is_file()
            || !resolved
                .canonicalize()
                .map_err(error_text)?
                .starts_with(self.root())
        {
            return Err("Arquivo fora do projeto ou indisponível.".into());
        }
        Ok(resolved)
    }

    pub fn read(&self, relative: &str) -> Result<String, String> {
        let path = self.file_path(relative)?;
        let mut bytes = Vec::new();
        File::open(path)
            .and_then(|file| file.take((MAX_SOURCE + 1) as u64).read_to_end(&mut bytes))
            .map_err(error_text)?;
        if bytes.len() > MAX_SOURCE {
            return Err("O arquivo excede 256 KiB.".into());
        }
        String::from_utf8(bytes).map_err(|_| "O editor aceita arquivos UTF-8.".into())
    }

    pub fn save(&self, relative: &str, expected: &str, source: &str) -> Result<(), String> {
        if source.len() > MAX_SOURCE || expected.len() > MAX_SOURCE {
            return Err("O arquivo excede 256 KiB.".into());
        }
        let path = self.file_path(relative)?;
        if self.read(relative)? != expected {
            return Err(
                "O arquivo mudou no disco. Seu texto continua aberto. Copie-o antes de recarregar."
                    .into(),
            );
        }
        self.project.save(&path, source).map_err(error_text)
    }

    pub fn create_source(&self, relative: &str) -> Result<ProjectInfo, String> {
        let path = Path::new(relative);
        validate_relative(path)?;
        let mut parent = self.root().to_owned();
        for part in path.parent().into_iter().flat_map(Path::iter) {
            parent.push(part);
            match fs::symlink_metadata(&parent) {
                Ok(meta) if is_redirect(&meta) => return Err("Pasta redirecionada.".into()),
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
                Err(error) => return Err(error_text(error)),
            }
        }
        self.project.create_file(relative).map_err(error_text)?;
        Ok(self.info())
    }
}

fn is_redirect(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    meta.is_symlink()
}

fn validate_relative(path: &Path) -> Result<(), String> {
    let allowed_top_file = ["Cargo.toml", "build.rs", "PLANO.md"]
        .iter()
        .any(|file| path == Path::new(file));
    if !allowed_top_file
        && (!path.extension().is_some_and(|ext| ext == "rs")
            || !["src", "tests", "examples", "benches"]
                .iter()
                .any(|dir| path.starts_with(dir)))
    {
        return Err("Selecione um arquivo Rust ou o manifesto do projeto.".into());
    }
    for component in path.components() {
        let Component::Normal(component) = component else {
            return Err("Caminho de arquivo inválido.".into());
        };
        validate_component(component.to_str().ok_or("Nome de arquivo inválido.")?)?;
    }
    Ok(())
}

fn validate_component(component: &str) -> Result<(), String> {
    let base = component
        .split('.')
        .next()
        .unwrap_or("")
        .trim_end()
        .to_ascii_uppercase();
    let device = matches!(
        base.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) || ["COM", "LPT"].iter().any(|prefix| {
        base.strip_prefix(prefix).is_some_and(|number| {
            matches!(
                number,
                "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    });
    if device
        || component.starts_with('.')
        || component.ends_with([' ', '.'])
        || component
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
    {
        Err("O nome contém caracteres ou nomes reservados no Windows.".into())
    } else {
        Ok(())
    }
}

fn error_text(error: std::io::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_creation_editing_and_external_conflict() {
        let parent = tempfile::tempdir().unwrap();
        let project = Workspace::create(parent.path(), "minha_app").unwrap();
        assert_eq!(project.info().files, ["Cargo.toml", "src/main.rs"]);
        assert!(Workspace::create(parent.path(), "minha_app").is_err());
        project.create_source("src/util.rs").unwrap();
        let base = project.read("src/main.rs").unwrap();
        project.save("src/main.rs", &base, "fn main() {}").unwrap();
        assert!(
            project
                .save("src/main.rs", &base, "perder alteração")
                .is_err()
        );
        assert!(project.read("src/../../outside.rs").is_err());
        assert!(project.read("src/main.rs:stream").is_err());
        assert!(project.create_source("src/nul.rs").is_err());
        assert_eq!(project.read("src/main.rs").unwrap(), "fn main() {}");
    }
}
