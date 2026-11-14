//! Analisador e serializador JSON mínimo sem dependências externas.
//!
//! Atende às necessidades do cliente LSP (JSON-RPC 2.0) e dos provedores de IA
//! locais/remotos (Ollama e APIs compatíveis com OpenAI).

use std::fmt;

const MAX_DEPTH: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "JSON inválido na posição {}: {}",
            self.offset, self.message
        )
    }
}

impl std::error::Error for JsonError {}

impl JsonValue {
    pub fn null() -> Self {
        Self::Null
    }

    pub fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    pub fn int(value: i64) -> Self {
        Self::Number(value.to_string())
    }

    pub fn uint(value: u64) -> Self {
        Self::Number(value.to_string())
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn array(items: Vec<JsonValue>) -> Self {
        Self::Array(items)
    }

    pub fn object<K: Into<String>>(pairs: Vec<(K, JsonValue)>) -> Self {
        Self::Object(pairs.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    pub fn parse(input: &str) -> Result<Self, JsonError> {
        let mut parser = Parser::new(input);
        parser.skip_ws();
        let value = parser.parse_value(0)?;
        parser.skip_ws();
        if !parser.is_eof() {
            return Err(parser.err("conteúdo extra após o valor JSON"));
        }
        Ok(value)
    }

    pub fn to_json_string(&self) -> String {
        let mut out = String::new();
        self.write_json(&mut out);
        out
    }

    pub fn escape(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 2);
        write_escaped_string(s, &mut out);
        out
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        let Self::Object(entries) = self else {
            return None;
        };
        entries.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn get_path(&self, path: &[&str]) -> Option<&JsonValue> {
        let mut current = self;
        for segment in path {
            current = current.get(segment)?;
        }
        Some(current)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Number(raw) => raw.parse::<i64>().ok(),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Number(raw) => raw.parse::<u64>().ok(),
            _ => None,
        }
    }

    pub fn as_usize(&self) -> Option<usize> {
        match self {
            Self::Number(raw) => raw.parse::<usize>().ok(),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            Self::Array(items) => Some(items.as_slice()),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&[(String, JsonValue)]> {
        match self {
            Self::Object(entries) => Some(entries.as_slice()),
            _ => None,
        }
    }

    fn write_json(&self, out: &mut String) {
        match self {
            Self::Null => out.push_str("null"),
            Self::Bool(true) => out.push_str("true"),
            Self::Bool(false) => out.push_str("false"),
            Self::Number(raw) => out.push_str(raw),
            Self::String(text) => write_escaped_string(text, out),
            Self::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    item.write_json(out);
                }
                out.push(']');
            }
            Self::Object(entries) => {
                out.push('{');
                for (i, (key, value)) in entries.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_escaped_string(key, out);
                    out.push(':');
                    value.write_json(out);
                }
                out.push('}');
            }
        }
    }
}

fn write_escaped_string(text: &str, out: &mut String) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            bytes: input.as_bytes(),
            pos: 0,
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.pos += 1;
        Some(byte)
    }

    fn err(&self, message: impl Into<String>) -> JsonError {
        JsonError {
            offset: self.pos,
            message: message.into(),
        }
    }

    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            if matches!(b, b' ' | b'\t' | b'\n' | b'\r') {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        if depth > MAX_DEPTH {
            return Err(self.err("profundidade máxima de aninhamento excedida"));
        }
        self.skip_ws();
        let Some(b) = self.peek() else {
            return Err(self.err("fim inesperado ao esperar um valor JSON"));
        };
        match b {
            b'n' => {
                self.expect_literal(b"null")?;
                Ok(JsonValue::Null)
            }
            b't' => {
                self.expect_literal(b"true")?;
                Ok(JsonValue::Bool(true))
            }
            b'f' => {
                self.expect_literal(b"false")?;
                Ok(JsonValue::Bool(false))
            }
            b'"' => self.parse_string().map(JsonValue::String),
            b'[' => self.parse_array(depth + 1),
            b'{' => self.parse_object(depth + 1),
            b'-' | b'0'..=b'9' => self.parse_number(),
            _ => Err(self.err(format!("caractere inesperado '{}'", b as char))),
        }
    }

    fn expect_literal(&mut self, literal: &[u8]) -> Result<(), JsonError> {
        if self.bytes[self.pos..].starts_with(literal) {
            self.pos += literal.len();
            Ok(())
        } else {
            Err(self.err("palavra-chave JSON inválida"))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, JsonError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        match self.peek() {
            Some(b'0') => {
                self.pos += 1;
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(self.err("zero à esquerda não é permitido em JSON"));
                }
            }
            Some(b'1'..=b'9') => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => return Err(self.err("dígito esperado no número JSON")),
        }
        if self.peek() == Some(b'.') {
            self.pos += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.err("dígito esperado após ponto decimal"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.err("dígito esperado no expoente"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        let raw = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| self.err("UTF-8 inválido no número"))?;
        Ok(JsonValue::Number(raw.to_string()))
    }

    fn parse_string(&mut self) -> Result<String, JsonError> {
        if self.bump() != Some(b'"') {
            return Err(self.err("esperava aspa inicial"));
        }
        let mut out = String::new();
        while !self.is_eof() {
            let b = self.bytes[self.pos];
            match b {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.pos += 1;
                    let esc = self
                        .bump()
                        .ok_or_else(|| self.err("escape incompleto em string"))?;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let code = self.parse_hex4()?;
                            if (0xD800..=0xDBFF).contains(&code) {
                                if !self.bytes[self.pos..].starts_with(b"\\u") {
                                    return Err(self.err("par substituto UTF-16 incompleto"));
                                }
                                self.pos += 2;
                                let low = self.parse_hex4()?;
                                if !(0xDC00..=0xDFFF).contains(&low) {
                                    return Err(self.err("substituto baixo UTF-16 inválido"));
                                }
                                let combined = 0x1_0000
                                    + (((u32::from(code) - 0xD800) << 10)
                                        | (u32::from(low) - 0xDC00));
                                let ch = char::from_u32(combined)
                                    .ok_or_else(|| self.err("ponto de código Unicode inválido"))?;
                                out.push(ch);
                            } else if (0xDC00..=0xDFFF).contains(&code) {
                                return Err(self.err("substituto baixo UTF-16 inesperado"));
                            } else {
                                let ch = char::from_u32(u32::from(code))
                                    .ok_or_else(|| self.err("ponto de código Unicode inválido"))?;
                                out.push(ch);
                            }
                        }
                        _ => return Err(self.err("sequência de escape JSON inválida")),
                    }
                }
                0x00..=0x1F => {
                    return Err(self.err("caractere de controle não escapado em string"));
                }
                _ => {
                    let rest = std::str::from_utf8(&self.bytes[self.pos..])
                        .map_err(|_| self.err("UTF-8 inválido em string"))?;
                    let ch = rest
                        .chars()
                        .next()
                        .ok_or_else(|| self.err("fim inesperado em string"))?;
                    out.push(ch);
                    self.pos += ch.len_utf8();
                }
            }
        }
        Err(self.err("string JSON sem aspa de fechamento"))
    }

    fn parse_hex4(&mut self) -> Result<u16, JsonError> {
        if self.pos + 4 > self.bytes.len() {
            return Err(self.err("escape \\uXXXX incompleto"));
        }
        let hex = std::str::from_utf8(&self.bytes[self.pos..self.pos + 4])
            .map_err(|_| self.err("escape unicode inválido"))?;
        let val = u16::from_str_radix(hex, 16)
            .map_err(|_| self.err("dígitos hexadecimais inválidos em \\uXXXX"))?;
        self.pos += 4;
        Ok(val)
    }

    fn parse_array(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.pos += 1; // '['
        self.skip_ws();
        let mut items = Vec::new();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            items.push(self.parse_value(depth)?);
            self.skip_ws();
            match self.bump() {
                Some(b',') => self.skip_ws(),
                Some(b']') => return Ok(JsonValue::Array(items)),
                _ => return Err(self.err("esperava ',' ou ']' no array JSON")),
            }
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.pos += 1; // '{'
        self.skip_ws();
        let mut entries = Vec::new();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(entries));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(self.err("chave do objeto JSON deve ser uma string"));
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.bump() != Some(b':') {
                return Err(self.err("esperava ':' após chave do objeto JSON"));
            }
            let value = self.parse_value(depth)?;
            entries.push((key, value));
            self.skip_ws();
            match self.bump() {
                Some(b',') => self.skip_ws(),
                Some(b'}') => return Ok(JsonValue::Object(entries)),
                _ => return Err(self.err("esperava ',' ou '}' no objeto JSON")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_e_serializa_objeto_aninhado() {
        let raw = r#"{
            "id": 7,
            "ok": true,
            "vazio": null,
            "texto": "olá\n\"rust\"\u0021 \uD83E\uDD80",
            "lista": [1, -2, 3.5e2, {"nested": false}]
        }"#;
        let parsed = JsonValue::parse(raw).unwrap();
        assert_eq!(parsed.get("id").and_then(JsonValue::as_u64), Some(7));
        assert_eq!(parsed.get("ok").and_then(JsonValue::as_bool), Some(true));
        assert_eq!(
            parsed.get("texto").and_then(JsonValue::as_str),
            Some("olá\n\"rust\"! 🦀")
        );
        assert_eq!(
            parsed
                .get_path(&["lista"])
                .and_then(JsonValue::as_array)
                .and_then(|arr| arr.get(3))
                .and_then(|v| v.get("nested"))
                .and_then(JsonValue::as_bool),
            Some(false)
        );
        let serialized = parsed.to_json_string();
        let reparsed = JsonValue::parse(&serialized).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn rejeita_json_malformado() {
        assert!(JsonValue::parse("").is_err());
        assert!(JsonValue::parse("{a:1}").is_err());
        assert!(JsonValue::parse("[1, 2,]").is_err());
        assert!(JsonValue::parse("01").is_err());
        assert!(JsonValue::parse("{\"a\":1} lixo").is_err());
    }

    #[test]
    fn escape_string_preserva_controles() {
        assert_eq!(JsonValue::escape("a\nb\tc\"\\"), "\"a\\nb\\tc\\\"\\\\\"");
    }
}
