//! Editor de código multilinha do Modo Criar (Fase 3).
//!
//! Escopo implementado aqui (requisito 11, núcleo):
//! * edição multilinha com cursor por caractere (Unicode-safe);
//! * seleção com Shift+movemento, copiar/recortar/colar internos;
//! * undo/redo com coalescência de digitação;
//! * auto-indentação simples (herda espaços da linha; +4 após `{`);
//! * busca simples (Ctrl+F, Enter localiza, F3 próxima);
//! * indicador de alterações não salvas;
//! * colagem via bracketed paste do terminal (Event::Paste).
//!
//! Fora do escopo (Fases 6/7): LSP, autocompletar, clipboard do sistema
//! (copiar PARA fora exige ferramenta externa; documentamos isso na UI).
//! Toda a lógica é pura e testável; o desenho vive em `ui/create.rs`.

use std::path::PathBuf;

/// Posição em (linha, coluna) — coluna é índice de *caractere*, não byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

impl Pos {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
    /// Ordena duas posições (a <= b).
    pub fn min_max(a: Pos, b: Pos) -> (Pos, Pos) {
        if a.line < b.line || (a.line == b.line && a.col <= b.col) {
            (a, b)
        } else {
            (b, a)
        }
    }
}

/// Buffer de texto: linhas como `Vec<char>` para edição O(1) por caractere.
#[derive(Debug, Clone, Default)]
pub struct TextBuffer {
    lines: Vec<Vec<char>>,
}

impl TextBuffer {
    pub fn from_text(text: &str) -> Self {
        let lines: Vec<Vec<char>> = text
            .split('\n')
            .map(|l| l.trim_end_matches('\r').chars().collect())
            .collect();
        Self {
            lines: if lines.is_empty() {
                vec![Vec::new()]
            } else {
                lines
            },
        }
    }

    pub fn to_string_lossy(&self) -> String {
        self.lines
            .iter()
            .map(|l| l.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn line(&self, i: usize) -> Option<&[char]> {
        self.lines.get(i).map(|l| l.as_slice())
    }

    pub fn line_str(&self, i: usize) -> String {
        self.lines
            .get(i)
            .map(|l| l.iter().collect())
            .unwrap_or_default()
    }

    pub fn line_len(&self, i: usize) -> usize {
        self.lines.get(i).map(|l| l.len()).unwrap_or(0)
    }

    pub fn clamp(&self, p: Pos) -> Pos {
        let line = p.line.min(self.lines.len().saturating_sub(1));
        let col = p.col.min(self.line_len(line));
        Pos::new(line, col)
    }

    pub fn insert_char(&mut self, p: Pos, c: char) -> Pos {
        let p = self.clamp(p);
        self.lines[p.line].insert(p.col, c);
        Pos::new(p.line, p.col + 1)
    }

    /// Insere texto multilinha; retorna a nova posição do cursor.
    pub fn insert_str(&mut self, p: Pos, text: &str) -> Pos {
        let mut p = self.clamp(p);
        let chunks: Vec<&str> = text.split('\n').collect();
        for (index, chunk) in chunks.iter().enumerate() {
            for c in chunk.trim_end_matches('\r').chars() {
                self.lines[p.line].insert(p.col, c);
                p.col += 1;
            }
            if index + 1 < chunks.len() {
                p = self.split_line(p);
            }
        }
        p
    }

    /// Divide a linha em `p` (Enter). Retorna a posição inicial da nova linha.
    pub fn split_line(&mut self, p: Pos) -> Pos {
        let p = self.clamp(p);
        let tail: Vec<char> = self.lines[p.line].split_off(p.col);
        self.lines.insert(p.line + 1, tail);
        Pos::new(p.line + 1, 0)
    }

    /// Junta a linha `i` com a anterior (Backspace no início da linha).
    pub fn join_with_prev(&mut self, line: usize) -> Option<Pos> {
        if line == 0 || line >= self.lines.len() {
            return None;
        }
        let col = self.lines[line - 1].len();
        let mut tail = self.lines.remove(line);
        self.lines[line - 1].append(&mut tail);
        Some(Pos::new(line - 1, col))
    }

    pub fn delete_char(&mut self, p: Pos, forward: bool) -> Option<Pos> {
        let p = self.clamp(p);
        if forward {
            if p.col < self.line_len(p.line) {
                self.lines[p.line].remove(p.col);
                Some(p)
            } else if p.line + 1 < self.lines.len() {
                self.join_with_prev(p.line + 1)
            } else {
                None
            }
        } else if p.col > 0 {
            self.lines[p.line].remove(p.col - 1);
            Some(Pos::new(p.line, p.col - 1))
        } else {
            self.join_with_prev(p.line)
        }
    }

    /// Apaga o intervalo [a,b] (ordenado internamente), devolve o texto
    /// removido e a posição resultante.
    #[allow(clippy::needless_range_loop)] // índices preservam a referência à linha final antes do truncate
    pub fn delete_range(&mut self, a: Pos, b: Pos) -> (String, Pos) {
        let (start, end) = Pos::min_max(a, b);
        let start = self.clamp(start);
        let end = self.clamp(end);
        if start == end {
            return (String::new(), start);
        }
        let mut removed = String::new();
        if start.line == end.line {
            let line = &mut self.lines[start.line];
            for c in line.drain(start.col..end.col) {
                removed.push(c);
            }
        } else {
            // primeira linha: do start.col ao fim
            let first: Vec<char> = self.lines[start.line].split_off(start.col);
            removed.extend(first.iter());
            // linhas intermediárias + última
            for l in start.line + 1..=end.line {
                removed.push('\n');
                let take = if l == end.line {
                    end.col
                } else {
                    self.lines[l].len()
                };
                let chars: Vec<char> = self.lines[l].drain(..take).collect();
                removed.extend(chars.iter());
            }
            // colapsa: remove linhas start+1..=end, mantém conteúdo restante
            // da última na primeira
            let rest: Vec<char> = self.lines[end.line].clone();
            self.lines.truncate(start.line + 1);
            self.lines[start.line].extend(rest);
        }
        (removed, start)
    }

    /// Extrai (sem apagar) o texto de um intervalo.
    pub fn slice(&self, a: Pos, b: Pos) -> String {
        let (start, end) = Pos::min_max(a, b);
        let start = self.clamp(start);
        let end = self.clamp(end);
        let mut out = String::new();
        for line in start.line..=end.line {
            let from = if line == start.line { start.col } else { 0 };
            let to = if line == end.line {
                end.col
            } else {
                self.line_len(line)
            };
            out.extend(self.lines[line][from.min(to)..to].iter());
            if line != end.line {
                out.push('\n');
            }
        }
        out
    }

    pub fn char_at(&self, p: Pos) -> Option<char> {
        self.lines.get(p.line)?.get(p.col).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditKind {
    InsertChar,
    Delete,
    Other,
}

#[derive(Debug, Clone)]
struct Snapshot {
    buf: TextBuffer,
    cursor: Pos,
}

#[derive(Debug, Default, Clone)]
pub struct SearchState {
    pub query: String,
    pub editing: bool,
    pub matches: Vec<Pos>,
    pub idx: usize,
    pub message: Option<String>,
}

/// Estado de edição de UM arquivo.
#[derive(Debug, Clone)]
pub struct Editor {
    pub buf: TextBuffer,
    pub cursor: Pos,
    pub anchor: Option<Pos>,
    pub dirty: bool,
    pub path: Option<PathBuf>,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    last_kind: Option<EditKind>,
    clipboard: String,
    pub search: Option<SearchState>,
    pub scroll_row: usize,
    pub scroll_col: usize,
}

impl Editor {
    pub fn new(text: &str, path: Option<PathBuf>) -> Self {
        Self {
            buf: TextBuffer::from_text(text),
            cursor: Pos::new(0, 0),
            anchor: None,
            dirty: false,
            path,
            undo: Vec::new(),
            redo: Vec::new(),
            last_kind: None,
            clipboard: String::new(),
            search: None,
            scroll_row: 0,
            scroll_col: 0,
        }
    }

    // ------------------------------------------------------------- snapshot
    fn snapshot(&mut self, kind: EditKind) {
        // coalescência: inserções consecutivas viram UMA entrada de undo
        if self.last_kind == Some(EditKind::InsertChar) && kind == EditKind::InsertChar {
            return;
        }
        self.last_kind = Some(kind);
        self.undo.push(Snapshot {
            buf: self.buf.clone(),
            cursor: self.cursor,
        });
        if self.undo.len() > 200 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn undo(&mut self) {
        self.last_kind = None;
        if let Some(s) = self.undo.pop() {
            self.redo.push(Snapshot {
                buf: self.buf.clone(),
                cursor: self.cursor,
            });
            self.buf = s.buf;
            self.cursor = s.cursor;
            self.anchor = None;
            self.dirty = true;
        }
    }

    pub fn redo(&mut self) {
        self.last_kind = None;
        if let Some(s) = self.redo.pop() {
            self.undo.push(Snapshot {
                buf: self.buf.clone(),
                cursor: self.cursor,
            });
            self.buf = s.buf;
            self.cursor = s.cursor;
            self.anchor = None;
            self.dirty = true;
        }
    }

    // ------------------------------------------------------------- seleção
    pub fn selection(&self) -> Option<(Pos, Pos)> {
        let a = self.anchor?;
        if a == self.cursor {
            None
        } else {
            Some(Pos::min_max(a, self.cursor))
        }
    }

    fn delete_selection(&mut self) -> String {
        if let Some((a, b)) = self.selection() {
            let (removed, pos) = self.buf.delete_range(a, b);
            self.cursor = pos;
            self.anchor = None;
            removed
        } else {
            String::new()
        }
    }

    // ------------------------------------------------------------- edição
    pub fn insert_char(&mut self, c: char) {
        self.snapshot(EditKind::InsertChar);
        if c == '\n' {
            self.newline();
            return;
        }
        self.delete_selection();
        self.buf.insert_char(self.cursor, c);
        self.cursor.col += 1;
        self.dirty = true;
    }

    pub fn insert_text(&mut self, text: &str) {
        self.snapshot(EditKind::Other);
        self.delete_selection();
        self.cursor = self.buf.insert_str(self.cursor, text);
        self.dirty = true;
        self.last_kind = None;
    }

    /// Enter com auto-indentação: herda o espaço da linha atual e soma 4
    /// se a linha anterior termina com `{`.
    pub fn newline(&mut self) {
        self.snapshot(EditKind::Other);
        self.delete_selection();
        let line = self.buf.line_str(self.cursor.line);
        let indent: String = line
            .chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .collect();
        let before = line.chars().take(self.cursor.col).collect::<String>();
        let extra = if before.trim_end().ends_with('{') {
            "    "
        } else {
            ""
        };
        let p = self.buf.split_line(self.cursor);
        let text = format!("{indent}{extra}");
        self.cursor = self.buf.insert_str(p, &text);
        self.dirty = true;
        self.last_kind = None;
    }

    pub fn backspace(&mut self) {
        self.snapshot(EditKind::Delete);
        if !self.delete_selection().is_empty() {
            self.dirty = true;
            return;
        }
        if let Some(p) = self.buf.delete_char(self.cursor, false) {
            self.cursor = p;
            self.dirty = true;
        }
        self.last_kind = None;
    }

    pub fn delete(&mut self) {
        self.snapshot(EditKind::Delete);
        if !self.delete_selection().is_empty() {
            self.dirty = true;
            return;
        }
        if let Some(p) = self.buf.delete_char(self.cursor, true) {
            self.cursor = p;
            self.dirty = true;
        }
        self.last_kind = None;
    }

    pub fn cut(&mut self) -> bool {
        if self.selection().is_none() {
            // sem seleção: recorta a linha inteira (comportamento comum)
            self.snapshot(EditKind::Other);
            let text = self.buf.line_str(self.cursor.line);
            self.clipboard = format!("{text}\n");
            let end_of_line = Pos::new(self.cursor.line, self.buf.line_len(self.cursor.line));
            let next_line_start = Pos::new(self.cursor.line + 1, 0);
            let (_, pos) = if self.cursor.line + 1 < self.buf.line_count() {
                self.buf
                    .delete_range(Pos::new(self.cursor.line, 0), next_line_start)
            } else {
                self.buf
                    .delete_range(Pos::new(self.cursor.line, 0), end_of_line)
            };
            self.cursor = pos;
            self.dirty = true;
            return true;
        }
        self.snapshot(EditKind::Other);
        let removed = self.delete_selection();
        if !removed.is_empty() {
            self.clipboard = removed;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn copy(&mut self) -> bool {
        match self.selection() {
            Some((a, b)) => {
                self.clipboard = self.buf.slice(a, b);
                true
            }
            None => false,
        }
    }

    pub fn paste(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }
        let text = self.clipboard.clone();
        self.insert_text(&text);
    }

    pub fn select_all(&mut self) {
        self.anchor = Some(Pos::new(0, 0));
        let last = self.buf.line_count() - 1;
        self.cursor = Pos::new(last, self.buf.line_len(last));
    }

    // ------------------------------------------------------------- navegação
    fn word_boundary(&self, from: Pos, forward: bool) -> Pos {
        let is_word = |c: char| c.is_alphanumeric() || c == '_';
        let mut p = from;
        let step = |p: &mut Pos| -> bool {
            if forward {
                if p.col < self.buf.line_len(p.line) && p.col < self.buf.line_len(p.line) {
                    p.col += 1;
                    true
                } else if p.line + 1 < self.buf.line_count() {
                    p.line += 1;
                    p.col = 0;
                    true
                } else {
                    false
                }
            } else if p.col > 0 {
                p.col -= 1;
                true
            } else if p.line > 0 {
                p.line -= 1;
                p.col = self.buf.line_len(p.line);
                true
            } else {
                false
            }
        };
        // pula não-palavra
        loop {
            let c = self.buf.char_at(if forward {
                p
            } else {
                Pos::new(p.line, p.col.saturating_sub(1))
            });
            match c {
                Some(c) if !is_word(c) => {
                    if !step(&mut p) {
                        break;
                    }
                }
                _ => break,
            }
        }
        // consome palavra
        loop {
            let c = self.buf.char_at(if forward {
                p
            } else {
                Pos::new(p.line, p.col.saturating_sub(1))
            });
            match c {
                Some(c) if is_word(c) => {
                    if !step(&mut p) {
                        break;
                    }
                }
                _ => break,
            }
        }
        p
    }

    pub fn move_cursor(&mut self, dx: i32, dy: i32, shift: bool, word: bool) {
        if shift && self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }
        let mut p = self.cursor;
        if dy != 0 {
            let target =
                (p.line as i64 + dy as i64).clamp(0, self.buf.line_count() as i64 - 1) as usize;
            p = Pos::new(target, p.col);
        }
        if dx != 0 {
            if word {
                p = self.word_boundary(p, dx > 0);
            } else {
                let len = self.buf.line_len(p.line) as i64;
                let col = (p.col as i64 + dx as i64).clamp(0, len) as usize;
                p = Pos::new(p.line, col);
            }
        }
        self.cursor = self.buf.clamp(p);
        if !shift {
            self.anchor = None;
        }
        self.last_kind = None;
    }

    pub fn move_home(&mut self, shift: bool) {
        if shift && self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }
        self.cursor.col = 0;
        if !shift {
            self.anchor = None;
        }
    }

    pub fn move_end(&mut self, shift: bool) {
        if shift && self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }
        self.cursor.col = self.buf.line_len(self.cursor.line);
        if !shift {
            self.anchor = None;
        }
    }

    pub fn move_doc(&mut self, to_end: bool, shift: bool) {
        if shift && self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }
        self.cursor = if to_end {
            let last = self.buf.line_count() - 1;
            Pos::new(last, self.buf.line_len(last))
        } else {
            Pos::new(0, 0)
        };
        if !shift {
            self.anchor = None;
        }
    }

    /// Garante que o cursor esteja visível numa viewport de `rows`x`cols`
    /// (sem contar o gutter de números de linha, tratado pelo chamador).
    pub fn ensure_visible(&mut self, rows: usize, cols: usize) {
        let rows = rows.max(1);
        let cols = cols.max(1);
        if self.cursor.line < self.scroll_row {
            self.scroll_row = self.cursor.line;
        }
        if self.cursor.line >= self.scroll_row + rows {
            self.scroll_row = self.cursor.line - rows + 1;
        }
        if self.cursor.col < self.scroll_col {
            self.scroll_col = self.cursor.col;
        }
        if self.cursor.col >= self.scroll_col + cols {
            self.scroll_col = self.cursor.col - cols + 1;
        }
    }

    // ------------------------------------------------------------- busca
    pub fn begin_search(&mut self) {
        let mut st = self.search.take().unwrap_or_default();
        st.editing = true;
        st.message = None;
        self.search = Some(st);
    }

    pub fn search_input(&mut self, c: char) {
        if let Some(st) = self.search.as_mut()
            && st.editing
        {
            st.query.push(c);
        }
    }

    pub fn search_backspace(&mut self) {
        if let Some(st) = self.search.as_mut()
            && st.editing
        {
            st.query.pop();
        }
    }

    /// Conclui a edição da busca e salta para a próxima ocorrência.
    pub fn search_commit(&mut self) {
        let Some(st) = self.search.as_mut() else {
            return;
        };
        st.editing = false;
        self.search_next();
    }

    pub fn search_next(&mut self) {
        let Some(st) = self.search.clone() else {
            return;
        };
        if st.query.trim().is_empty() {
            if let Some(s) = self.search.as_mut() {
                s.matches.clear();
                s.message = Some("busca vazia".into());
            }
            return;
        }
        // coleta ocorrências (case-insensitive, sem alocar por linha inteira)
        let needle: String = st.query.chars().flat_map(|c| c.to_lowercase()).collect();
        let mut matches: Vec<Pos> = Vec::new();
        for li in 0..self.buf.line_count() {
            let line: String = self
                .buf
                .line_str(li)
                .chars()
                .flat_map(|c| c.to_lowercase())
                .collect();
            let chars: Vec<char> = line.chars().collect();
            let nchars: Vec<char> = needle.chars().collect();
            if nchars.is_empty() {
                continue;
            }
            let mut start = 0usize;
            while start + nchars.len() <= chars.len() {
                if chars[start..start + nchars.len()] == nchars[..] {
                    matches.push(Pos::new(li, start));
                    start += nchars.len().max(1);
                } else {
                    start += 1;
                }
            }
        }
        let Some(s) = self.search.as_mut() else {
            return;
        };
        let had_matches = !s.matches.is_empty();
        s.matches = matches;
        if s.matches.is_empty() {
            s.idx = 0;
            s.message = Some("nenhuma ocorrência".into());
            return;
        }
        // próxima ocorrência APÓS o cursor (com wrap)
        let after = if had_matches {
            (s.idx + 1) % s.matches.len()
        } else {
            s.matches
                .iter()
                .position(|p| *p >= self.cursor)
                .unwrap_or(0)
        };
        s.idx = after;
        self.cursor = s.matches[after];
        self.anchor = None;
        s.message = Some(format!("{} de {}", after + 1, s.matches.len()));
    }

    pub fn search_cancel(&mut self) {
        self.search = None;
    }

    // ------------------------------------------------------------- estado
    pub fn mark_saved(&mut self) {
        self.dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ed(text: &str) -> Editor {
        Editor::new(text, None)
    }

    #[test]
    fn insere_e_apaga_caracteres() {
        let mut e = ed("fn main() {}");
        e.cursor = Pos::new(0, 3);
        e.insert_char('X');
        assert_eq!(e.buf.line_str(0), "fn Xmain() {}");
        e.backspace();
        assert_eq!(e.buf.line_str(0), "fn main() {}");
        assert!(e.dirty, "edição deve marcar arquivo como sujo");
    }

    #[test]
    fn enter_auto_indenta_e_abre_chave() {
        let mut e = ed("fn main() {");
        e.cursor = Pos::new(0, 11);
        e.newline();
        assert_eq!(e.buf.line_count(), 2);
        assert_eq!(e.buf.line_str(1), "    ");
        assert_eq!(e.cursor, Pos::new(1, 4));
    }

    #[test]
    fn enter_herda_indentacao() {
        let mut e = ed("    let x = 1;");
        e.cursor = Pos::new(0, 14);
        e.newline();
        assert_eq!(e.buf.line_str(1), "    ");
    }

    #[test]
    fn undo_redo_com_coalescencia() {
        let mut e = ed("abc");
        e.cursor = Pos::new(0, 3);
        for c in "def".chars() {
            e.insert_char(c);
        }
        assert_eq!(e.buf.line_str(0), "abcdef");
        e.undo(); // as 3 letras colapsam em UMA operação
        assert_eq!(e.buf.line_str(0), "abc");
        e.redo();
        assert_eq!(e.buf.line_str(0), "abcdef");
    }

    #[test]
    fn selecao_copiar_recortar_colar() {
        let mut e = ed("hello world");
        e.cursor = Pos::new(0, 0);
        e.anchor = Some(Pos::new(0, 5));
        assert!(e.copy());
        assert_eq!(e.clipboard, "hello");
        e.cut();
        assert_eq!(e.buf.line_str(0), " world");
        e.cursor = Pos::new(0, 6);
        e.paste();
        assert_eq!(e.buf.line_str(0), " worldhello");
    }

    #[test]
    fn recorte_sem_selecao_corta_linha() {
        let mut e = ed("a\nb\nc");
        e.cursor = Pos::new(1, 0);
        e.cut();
        assert_eq!(e.buf.to_string_lossy(), "a\nc");
        assert_eq!(e.clipboard, "b\n");
    }

    #[test]
    fn delete_range_multilinha() {
        let mut b = TextBuffer::from_text("aaa\nbbb\nccc");
        let (removed, pos) = b.delete_range(Pos::new(0, 1), Pos::new(2, 1));
        assert_eq!(removed, "aa\nbbb\nc");
        assert_eq!(pos, Pos::new(0, 1));
        assert_eq!(b.to_string_lossy(), "acc");
    }

    #[test]
    fn slice_preserva_texto() {
        let b = TextBuffer::from_text("alpha\nbeta");
        assert_eq!(b.slice(Pos::new(0, 0), Pos::new(1, 4)), "alpha\nbeta");
        assert_eq!(b.slice(Pos::new(0, 2), Pos::new(0, 5)), "pha");
    }

    #[test]
    fn busca_case_insensitive_com_wrap() {
        let mut e = ed("Foo bar foo BAZ foo");
        e.cursor = Pos::new(0, 0);
        e.begin_search();
        for c in "foo".chars() {
            e.search_input(c);
        }
        e.search_commit();
        {
            let st = e.search.as_ref().unwrap();
            assert_eq!(st.matches.len(), 3);
        }
        assert_eq!(e.cursor, Pos::new(0, 0)); // primeira após o cursor
        e.search_next();
        assert_eq!(e.cursor, Pos::new(0, 8));
        e.search_next();
        assert_eq!(e.cursor, Pos::new(0, 16));
        e.search_next(); // wrap
        assert_eq!(e.cursor, Pos::new(0, 0));
    }

    #[test]
    fn unicode_multibyte_seguro() {
        let mut e = ed("café ☕");
        e.cursor = Pos::new(0, 6); // fim da linha
        e.insert_char('!');
        assert_eq!(e.buf.line_str(0), "café ☕!");
        e.move_cursor(-1, 0, false, false);
        assert_eq!(e.cursor.col, 6);
        e.backspace();
        assert_eq!(e.buf.line_str(0), "café !");
    }

    #[test]
    fn movimento_palavra() {
        let mut e = ed("let nome_composto = 42;");
        e.cursor = Pos::new(0, 0);
        e.move_cursor(1, 0, false, true);
        assert_eq!(e.cursor.col, 3); // fim de "let"
        e.move_cursor(1, 0, false, true);
        assert_eq!(e.cursor.col, 17); // fim de "nome_composto"
    }

    #[test]
    fn select_all_e_substituicao() {
        let mut e = ed("linha1\nlinha2");
        e.select_all();
        e.insert_char('X');
        assert_eq!(e.buf.to_string_lossy(), "X");
    }

    #[test]
    fn ensure_visible_rola() {
        let mut e = ed(&"x\n".repeat(50));
        e.cursor = Pos::new(40, 0);
        e.ensure_visible(10, 20);
        assert_eq!(e.scroll_row, 31);
        e.cursor = Pos::new(2, 0);
        e.ensure_visible(10, 20);
        assert_eq!(e.scroll_row, 2);
    }
}
