//! Autocomplete local por prefixo; não faz inferência de tipos.
use crate::editor::Editor;

#[derive(Debug, Default)]
pub struct Completion {
    pub items: Vec<String>,
    pub selected: usize,
    start: usize,
    end: usize,
}

pub fn rust_keywords() -> &'static [&'static str] {
    &[
        "println!",
        "print!",
        "eprintln!",
        "format!",
        "vec!",
        "assert_eq!",
        "let",
        "mut",
        "fn",
        "pub",
        "use",
        "mod",
        "struct",
        "enum",
        "impl",
        "match",
        "if",
        "else",
        "for",
        "while",
        "loop",
        "return",
        "break",
        "continue",
        "true",
        "false",
        "String",
        "Vec",
        "Option",
        "Result",
        "Some",
        "None",
        "Ok",
        "Err",
        "usize",
        "i32",
        "u32",
        "bool",
        "str",
    ]
}

impl Completion {
    pub fn refresh(&mut self, editor: &Editor, explicit: bool) {
        self.refresh_with_extras(editor, explicit, &[]);
    }

    pub fn refresh_with_extras(&mut self, editor: &Editor, explicit: bool, extras: &[String]) {
        *self = Self::default();
        if editor.selection().is_some() {
            return;
        }
        let line = editor.buf.line(editor.cursor.line).unwrap_or_default();
        let end = editor.cursor.col.min(line.len());
        // Não sugerir dentro de strings e comentários comuns da linha.
        let mut quoted = false;
        let mut escaped = false;
        for (i, ch) in line.iter().take(end).enumerate() {
            if !quoted && *ch == '/' && line.get(i + 1) == Some(&'/') {
                return;
            }
            if *ch == '"' && !escaped {
                quoted = !quoted;
            }
            escaped = *ch == '\\' && !escaped;
        }
        if quoted {
            return;
        }
        let mut start = end;
        while start > 0 && ident(line[start - 1]) {
            start -= 1;
        }
        let prefix: String = line[start..end].iter().collect();
        if prefix.is_empty() || (!explicit && end - start < 2) {
            return;
        }
        let mut candidates: std::collections::BTreeSet<String> =
            rust_keywords().iter().copied().map(String::from).collect();
        let full_code = editor.buf.to_string_lossy();
        for word in full_code.split(|ch: char| !ident(ch)) {
            if word
                .chars()
                .next()
                .is_some_and(|ch| ch.is_alphabetic() || ch == '_')
            {
                candidates.insert(word.to_string());
            }
        }
        if full_code.contains("bevy") {
            const BEVY_SYMBOLS: &[&str] = &[
                "App",
                "DefaultPlugins",
                "MinimalPlugins",
                "Startup",
                "Update",
                "FixedUpdate",
                "Commands",
                "Query",
                "Res",
                "ResMut",
                "Component",
                "Resource",
                "Event",
                "Entity",
                "Transform",
                "GlobalTransform",
                "Camera2d",
                "Camera3d",
                "Sprite",
                "ButtonInput",
                "KeyCode",
                "Time",
                "AssetServer",
                "Handle",
                "Vec2",
                "Vec3",
                "Color",
                "With",
                "Without",
                "add_plugins",
                "add_systems",
                "init_resource",
                "insert_resource",
                "spawn",
                "despawn",
                "single",
                "single_mut",
                "delta_secs",
                "pressed",
                "just_pressed",
                "from_xyz",
            ];
            for sym in BEVY_SYMBOLS {
                candidates.insert((*sym).to_string());
            }
        }
        for extra in extras {
            if !extra.is_empty() {
                candidates.insert(extra.clone());
            }
        }
        self.items = candidates
            .into_iter()
            .filter(|word| word.starts_with(&prefix) && word != &prefix)
            .take(8)
            .collect();
        self.start = start;
        self.end = end;
    }

    pub fn accept(&mut self, editor: &mut Editor) {
        if let Some(item) = self.items.get(self.selected).cloned() {
            // Selecionar só o prefixo: a inserção continua usando o undo do editor.
            editor.move_cursor(-((self.end - self.start) as i32), 0, true, false);
            let suffix = editor.buf.line(editor.cursor.line).unwrap_or_default();
            let insertion = if item.ends_with('!') && suffix.get(self.end) == Some(&'!') {
                item.trim_end_matches('!')
            } else {
                &item
            };
            editor.insert_text(insertion);
        }
        *self = Self::default();
    }
}

fn ident(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completa_macro_e_undo_restaura_prefixo() {
        let mut editor = Editor::new("pri", None);
        editor.move_end(false);
        let mut completion = Completion::default();
        completion.refresh(&editor, true);
        completion.selected = completion
            .items
            .iter()
            .position(|s| s == "println!")
            .unwrap();
        completion.accept(&mut editor);
        assert_eq!(editor.buf.to_string_lossy(), "println!");
        editor.undo();
        assert_eq!(editor.buf.to_string_lossy(), "pri");
    }
    #[test]
    fn nao_sugere_em_string_ou_comentario() {
        for source in ["// pri", "\"pri"] {
            let mut editor = Editor::new(source, None);
            editor.move_end(false);
            let mut completion = Completion::default();
            completion.refresh(&editor, true);
            assert!(completion.items.is_empty());
        }
    }

    #[test]
    fn completa_simbolos_do_workspace() {
        let mut editor = Editor::new("cal", None);
        editor.move_end(false);
        let mut completion = Completion::default();
        completion.refresh_with_extras(&editor, true, &["calcular_media".to_string()]);
        assert_eq!(completion.items, vec!["calcular_media".to_string()]);
    }

    #[test]
    fn completa_simbolos_bevy_quando_importado() {
        let mut editor = Editor::new("use bevy::prelude::*;\nfn s(mut cmd: Com", None);
        editor.cursor = crate::editor::Pos {
            line: 1,
            col: "fn s(mut cmd: Com".chars().count(),
        };
        let mut completion = Completion::default();
        completion.refresh(&editor, true);
        assert!(completion.items.contains(&"Commands".to_string()));
    }
}
