//! Realce de sintaxe Rust determinístico — um tokenizador de linha única,
//! suficiente para colorir exemplos de lição sem depender de
//! rust-analyzer (que chega na Fase 7 para edição profunda).
//!
//! Regras intencionalmente simples: identificadores são classificados por
//! listas de palavras-chave; strings/comentários/números têm precedência.
//! Não tente "entender" Rust — apenas pinte. O consumidor de verdade de
//! diagnóstico correto é o compilador.

use ratatui::style::{Modifier, Style};
use ratatui::text::Span;

use crate::theme::Theme;

const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn",
    "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
    "use", "where", "while", "async", "await",
];

const TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char", "str", "String", "Vec", "Option", "Result",
];

/// Divide uma linha de código em spans já estilizados.
pub fn highlight_line(line: &str, theme: &Theme) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let bytes: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut plain = String::new();

    // atalhos de estilo
    let kw = theme.title();
    let ty = Style::default().fg(theme.accent_dim);
    let str_ = theme.success();
    let num = theme.warning();
    let com = theme.dim().add_modifier(Modifier::ITALIC);
    let plain_style = theme.base();

    fn is_ident_c(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    while i < bytes.len() {
        let c = bytes[i];
        // comentário de linha: pinta o resto e termina
        if c == '/' && i + 1 < bytes.len() && bytes[i + 1] == '/' {
            if !plain.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut plain), plain_style));
            }
            spans.push(Span::styled(bytes[i..].iter().collect::<String>(), com));
            return spans;
        }
        // string (com escapes \"): simples, de uma linha; raw strings r"..."
        if c == '"' || (c == 'r' && i + 1 < bytes.len() && bytes[i + 1] == '"') {
            if !plain.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut plain), plain_style));
            }
            let j = if c == 'r' { i + 2 } else { i + 1 };
            let mut end = j;
            while end < bytes.len() {
                if bytes[end] == '\\' {
                    end += 2;
                    continue;
                }
                if bytes[end] == '"' {
                    end += 1;
                    break;
                }
                end += 1;
            }
            spans.push(Span::styled(
                bytes[i..end.min(bytes.len())].iter().collect::<String>(),
                str_,
            ));
            i = end;
            continue;
        }
        // número
        if c.is_ascii_digit() && (i == 0 || !is_ident_c(bytes[i - 1])) {
            if !plain.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut plain), plain_style));
            }
            let mut end = i;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || bytes[end] == '_' || bytes[end] == '.')
            {
                end += 1;
            }
            spans.push(Span::styled(bytes[i..end].iter().collect::<String>(), num));
            i = end;
            continue;
        }
        // identificador / palavra-chave
        if is_ident_c(c) && (i == 0 || !is_ident_c(bytes[i - 1])) {
            if !plain.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut plain), plain_style));
            }
            let mut end = i;
            while end < bytes.len() && is_ident_c(bytes[end]) {
                end += 1;
            }
            let mut word: String = bytes[i..end].iter().collect();
            let mut style = if KEYWORDS.contains(&word.as_str()) {
                kw
            } else if TYPES.contains(&word.as_str()) {
                ty
            } else {
                plain_style
            };
            // macro: nome terminando em "!" (println!, vec!)
            if end < bytes.len() && bytes[end] == '!' {
                word.push('!');
                end += 1;
                style = kw;
            }
            spans.push(Span::styled(word, style));
            i = end;
            continue;
        }
        plain.push(c);
        i += 1;
    }
    if !plain.is_empty() {
        spans.push(Span::styled(plain, plain_style));
    }
    spans
}

/// Versão plana (sem estilo) — usada em testes e em terminais que não
/// suportam cor (tema Mono ainda assim mantém a estrutura).
pub fn plain_line(line: &str) -> String {
    line.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    fn theme() -> Theme {
        Theme::for_id(crate::config::ThemeId::Cuia)
    }

    #[test]
    fn palavras_chave_sao_reconhecidas() {
        let t = theme();
        let spans = highlight_line("let mut x = 42;", &t);
        assert_eq!(
            spans.iter().map(|s| s.content.as_ref()).collect::<String>(),
            "let mut x = 42;"
        );
        assert_eq!(spans[0].style.fg, Some(Color::Rgb(222, 165, 132))); // keyword "let"
    }

    #[test]
    fn comentario_come_na_barra() {
        let t = theme();
        let spans = highlight_line("// oi", &t);
        assert_eq!(spans.len(), 1);
        assert!(
            spans[0]
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::ITALIC)
        );
    }

    #[test]
    fn string_com_escape() {
        let t = theme();
        let spans = highlight_line(r#"println!("a\"b");"#, &t);
        let joined: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(joined, r#"println!("a\"b");"#);
        let macro_span = spans.iter().find(|s| s.content.starts_with("println"));
        assert_eq!(macro_span.unwrap().content, "println!");
    }

    #[test]
    fn raw_string() {
        let t = theme();
        let spans = highlight_line(r#"let s = r"c:\tmp";"#, &t);
        let has = spans.iter().any(|s| s.content == r#"r"c:\tmp""#);
        assert!(has, "raw string deve ser um span só: {spans:?}");
    }

    #[test]
    fn numeros_e_tipos() {
        let t = theme();
        let spans = highlight_line("let v: u32 = 7u32;", &t);
        let joined: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(joined, "let v: u32 = 7u32;");
        assert!(
            spans
                .iter()
                .any(|s| s.content == "u32" && s.style.fg == Some(Color::Rgb(158, 106, 76)))
        );
    }

    #[test]
    fn reconstruir_a_linha_original() {
        // invariante central: a concatenação dos spans é exatamente a linha
        let t = theme();
        for line in [
            "fn main() {",
            "    let nome = \"mundo\"; // olá",
            "    println!(\"Olá, {nome}!\");",
            "}",
            "",
            "   ",
            "let x = 1.5 + 2e3;",
        ] {
            let spans = highlight_line(line, &t);
            let joined: String = spans.iter().map(|s| s.content.as_ref()).collect();
            assert_eq!(joined, line, "linha alterada: {line:?}");
        }
    }
}
