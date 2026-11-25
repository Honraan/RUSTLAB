use rustlab::doc_hint::extra_libraries;
use serde::Serialize;
use toml_edit::{DocumentMut, Item, Table, value};

#[derive(Serialize)]
pub struct Plan {
    pub before: String,
    pub after: String,
    pub present: bool,
}

pub fn plan(name: &str, before: String) -> Result<Plan, String> {
    let entry = extra_libraries(false)
        .iter()
        .find(|entry| entry.name == name)
        .ok_or("Biblioteca fora do catálogo F9.")?;
    let mut doc = before
        .parse::<DocumentMut>()
        .map_err(|_| "O Cargo.toml precisa ser um TOML válido.")?;
    if !doc.contains_key("package") {
        return Err("Abra a pasta do pacote Cargo que vai receber a dependência.".into());
    }
    if !doc.contains_key("dependencies") {
        doc["dependencies"] = Item::Table(Table::new());
    }
    let deps = doc["dependencies"]
        .as_table_like_mut()
        .ok_or("A seção de dependências está inválida.")?;
    let present = deps.get(entry.name).is_some();
    if !present {
        deps.insert(entry.name, value(entry.version));
    }
    let after = if present {
        before.clone()
    } else {
        doc.to_string()
    };
    Ok(Plan {
        before,
        after,
        present,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_preview_keeps_comments_and_existing_features() {
        let manifest = "# comentário\n[package]\nname = \"exemplo\"\n[dependencies]\nrand = { version = \"0.9\", features = [\"std\"] }\n";
        assert!(plan("rand", manifest.into()).unwrap().present);
        let change = plan("macroquad", manifest.into()).unwrap();
        assert_eq!(change.before, manifest);
        assert!(change.after.starts_with(manifest));
        assert!(change.after.ends_with("macroquad = \"0.4\"\n"));
        assert!(plan("lib-inexistente", manifest.into()).is_err());
    }
}
