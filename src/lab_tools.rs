//! Ferramentas pedagógicas avançadas do RustLab (`v0.3.2`):
//! 1. Visualizador Interativo de Ownership, Stack/Heap & Medidor de Alocações (`MemoryXRayReport`)
//! 2. Lente Socrática do Compilador, Modo ELI10 & Soluções Antes/Depois (`SocraticErrorLens`)
//! 3. Debugger Visual Passo a Passo com Breakpoints & Variáveis Locais (`VisualDebuggerState`)
//! 4. Modo Desafio Gamificado "Conserte o Código" & Golfing de Alocações (`TroubleshootingQuest`)
//! 5. Exportação para Rust Playground, Markdown & Soluções da Comunidade
//! 6. Trilhas e Templates de Ecossistemas Modernos (WebAssembly, CLI `clap`+`tokio`, Backend `axum`)

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Abas do Hub Laboratório RustLab (`Alt+X`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabToolTab {
    #[default]
    RaioXMemoria,
    LenteCompilador,
    DebuggerVisual,
    ConserteOCodigo,
    PlaygroundComunidade,
    EcossistemasModernos,
}

impl LabToolTab {
    pub const ALL: [Self; 6] = [
        Self::RaioXMemoria,
        Self::LenteCompilador,
        Self::DebuggerVisual,
        Self::ConserteOCodigo,
        Self::PlaygroundComunidade,
        Self::EcossistemasModernos,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::RaioXMemoria => 0,
            Self::LenteCompilador => 1,
            Self::DebuggerVisual => 2,
            Self::ConserteOCodigo => 3,
            Self::PlaygroundComunidade => 4,
            Self::EcossistemasModernos => 5,
        }
    }

    pub const fn from_digit(ch: char) -> Option<Self> {
        match ch {
            '1' => Some(Self::RaioXMemoria),
            '2' => Some(Self::LenteCompilador),
            '3' => Some(Self::DebuggerVisual),
            '4' => Some(Self::ConserteOCodigo),
            '5' => Some(Self::PlaygroundComunidade),
            '6' => Some(Self::EcossistemasModernos),
            _ => None,
        }
    }

    pub const fn short_title(self) -> &'static str {
        match self {
            Self::RaioXMemoria => "1 Raio-X",
            Self::LenteCompilador => "2 Lente/ELI10",
            Self::DebuggerVisual => "3 Debugger",
            Self::ConserteOCodigo => "4 Conserte!",
            Self::PlaygroundComunidade => "5 Playground",
            Self::EcossistemasModernos => "6 Ecossistemas",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::RaioXMemoria => "Raio-X de Ownership, Stack/Heap & Alocações",
            Self::LenteCompilador => "Lente Socrática do Compilador & Modo 10 Anos (ELI10)",
            Self::DebuggerVisual => "Debugger Visual Passo a Passo, Breakpoints & Locais",
            Self::ConserteOCodigo => "Desafio Gamificado: Conserte o Código & Golfing",
            Self::PlaygroundComunidade => "Rust Playground, Exportação & Soluções da Comunidade",
            Self::EcossistemasModernos => {
                "Trilhas Modernas: WebAssembly · CLI Tokio · Backend Axum"
            }
        }
    }

    pub const fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }

    pub const fn prev(self) -> Self {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

/// Estado agregado do Hub Laboratório no Modo Criar.
#[derive(Debug, Clone, Default)]
pub struct LabState {
    pub open: bool,
    pub tab: LabToolTab,
    pub scroll: u16,
    pub eli10_mode: bool,
    pub selected_fix: usize,
    pub quest_sel: usize,
    pub active_quest_id: Option<&'static str>,
    pub completed_quests: HashSet<&'static str>,
    pub community_sel: usize,
    pub ecosystem_sel: usize,
    pub last_xray: Option<MemoryXRayReport>,
    pub last_lens: Option<SocraticErrorLens>,
    pub last_export_path: Option<PathBuf>,
    pub last_quest_feedback: Option<String>,
    pub debugger: VisualDebuggerState,
}

impl LabState {
    pub fn open_tab(&mut self, tab: LabToolTab, code: &str, diags_text: &str) {
        self.open = true;
        self.tab = tab;
        self.scroll = 0;
        self.refresh(code, diags_text);
        if tab == LabToolTab::DebuggerVisual
            && (!self.debugger.active || self.debugger.steps.is_empty())
        {
            self.debugger.start_or_rebuild(code);
        }
    }

    pub fn refresh(&mut self, code: &str, diags_text: &str) {
        self.last_xray = Some(MemoryXRayReport::analyze(code));
        self.last_lens = Some(SocraticErrorLens::analyze(diags_text, code));
    }
}

// =========================================================================
// 1. VISUALIZADOR INTERATIVO DE OWNERSHIP & MEDIDOR DE ALOCAÇÕES
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageRegion {
    Stack,
    Heap,
}

impl StorageRegion {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Stack => "STACK",
            Self::Heap => "HEAP",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarOwnershipState {
    Ativo,
    Movido { destino: String, linha: usize },
    EmprestadoImutavel(usize),
    EmprestadoMutavel(usize),
    ConflitoBorrow,
}

impl VarOwnershipState {
    pub fn tag(&self) -> String {
        match self {
            Self::Ativo => "[✓ ativo]".to_string(),
            Self::Movido { destino, linha } => {
                format!("[✗ movido -> {destino} @ L{linha}]")
            }
            Self::EmprestadoImutavel(linha) => format!("[& emprestado @ L{linha}]"),
            Self::EmprestadoMutavel(linha) => format!("[&mut emprestado @ L{linha}]"),
            Self::ConflitoBorrow => "[! CONFLITO DE BORROW]".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarSlot {
    pub name: String,
    pub mutable: bool,
    pub ty_hint: String,
    pub region: StorageRegion,
    pub state: VarOwnershipState,
    pub declared_line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AllocationMetrics {
    pub stack_count: usize,
    pub heap_allocations: usize,
    pub clones_count: usize,
    pub zero_cost_chains: usize,
}

impl AllocationMetrics {
    pub const fn badge(self) -> &'static str {
        if self.heap_allocations == 0 && self.clones_count == 0 {
            "[Zero-Cost ★ 0 Alocações Heap]"
        } else if self.clones_count == 0 {
            "[Alocação Controlada · Sem Clones Extras]"
        } else {
            "[Atenção: Clones/Heap Evitáveis]"
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MemoryXRayReport {
    pub vars: Vec<VarSlot>,
    pub metrics: AllocationMetrics,
    pub notes: Vec<String>,
}

impl MemoryXRayReport {
    /// Analisa estaticamente o código Rust para mapear Stack vs Heap, Moves, Borrows e Alocações.
    pub fn analyze(code: &str) -> Self {
        let mut vars: Vec<VarSlot> = Vec::new();
        let mut index_by_name: HashMap<String, usize> = HashMap::new();
        let mut metrics = AllocationMetrics::default();
        let mut notes = Vec::new();

        for (idx, raw_line) in code.lines().enumerate() {
            let line_no = idx + 1;
            let trimmed = strip_line_comment(raw_line).trim();
            if trimmed.is_empty() {
                continue;
            }

            metrics.clones_count += trimmed.matches(".clone()").count();
            metrics.heap_allocations += count_heap_markers(trimmed);
            metrics.zero_cost_chains += count_zero_cost_markers(trimmed);

            if let Some(rest) = trimmed.strip_prefix("let ") {
                if let Some((lhs, rhs)) = rest.split_once('=') {
                    let (name, mutable, ty_hint) = parse_let_lhs(lhs);
                    let rhs_clean = rhs.trim().trim_end_matches(';').trim();
                    let (region, inferred_ty) = classify_rhs(rhs_clean, &vars, &index_by_name);
                    let final_ty = ty_hint.unwrap_or(inferred_ty);

                    // Verifica borrow criado no RHS (&x ou &mut x).
                    if let Some(target) = rhs_clean.strip_prefix("&mut ") {
                        let base = extract_leading_ident(target);
                        mark_borrow(&mut vars, &index_by_name, base, true, line_no, &mut notes);
                    } else if let Some(target) = rhs_clean.strip_prefix('&') {
                        let base = extract_leading_ident(target);
                        mark_borrow(&mut vars, &index_by_name, base, false, line_no, &mut notes);
                    } else if !rhs_clean.contains(".clone()") {
                        let src_ident = extract_leading_ident(rhs_clean);
                        if rhs_clean == src_ident
                            && let Some(&src_idx) = index_by_name.get(src_ident)
                            && vars[src_idx].region == StorageRegion::Heap
                            && !name.is_empty()
                        {
                            vars[src_idx].state = VarOwnershipState::Movido {
                                destino: name.clone(),
                                linha: line_no,
                            };
                            notes.push(format!(
                                "L{line_no}: Posse de `{src_ident}` foi movida para `{name}` (move semantics)."
                            ));
                        }
                    }

                    if !name.is_empty() && name != "_" {
                        if region == StorageRegion::Stack {
                            metrics.stack_count += 1;
                        }
                        let slot = VarSlot {
                            name: name.clone(),
                            mutable,
                            ty_hint: final_ty,
                            region,
                            state: VarOwnershipState::Ativo,
                            declared_line: line_no,
                        };
                        let slot_idx = vars.len();
                        vars.push(slot);
                        index_by_name.insert(name, slot_idx);
                    }
                }
                continue;
            }

            // Detecta mutações em linha avulsa (`s.push(...)`, `s.push_str(...)`, `s = ...`) sobre var emprestada.
            for (var_name, &var_idx) in &index_by_name {
                let push_call = format!("{var_name}.push");
                let mut_borrow = format!("&mut {var_name}");
                let imm_borrow = format!("&{var_name}");
                if trimmed.contains(&push_call) || trimmed.contains(&mut_borrow) {
                    if matches!(
                        vars[var_idx].state,
                        VarOwnershipState::EmprestadoImutavel(_)
                    ) {
                        vars[var_idx].state = VarOwnershipState::ConflitoBorrow;
                        notes.push(format!(
                            "L{line_no}: Conflito de borrow em `{var_name}` — mutação enquanto existe `&` ativo!"
                        ));
                    } else if matches!(vars[var_idx].state, VarOwnershipState::Ativo)
                        && trimmed.contains(&mut_borrow)
                    {
                        vars[var_idx].state = VarOwnershipState::EmprestadoMutavel(line_no);
                    }
                } else if trimmed.contains(&imm_borrow)
                    && matches!(vars[var_idx].state, VarOwnershipState::EmprestadoMutavel(_))
                {
                    vars[var_idx].state = VarOwnershipState::ConflitoBorrow;
                    notes.push(format!(
                        "L{line_no}: Conflito de borrow em `{var_name}` — `&` enquanto existe `&mut` ativo!"
                    ));
                }

                // Detecta passagem por valor em chamada simples `fn_name(var)` para variáveis Heap.
                if vars[var_idx].region == StorageRegion::Heap
                    && !trimmed.starts_with("println!")
                    && !trimmed.starts_with("eprintln!")
                    && !trimmed.starts_with("format!")
                    && !trimmed.contains(&format!("{var_name}."))
                    && trimmed.contains(&format!("({var_name})"))
                    && !trimmed.contains(&format!("(&{var_name})"))
                    && !trimmed.contains(&format!("(&mut {var_name})"))
                {
                    vars[var_idx].state = VarOwnershipState::Movido {
                        destino: "chamada()".to_string(),
                        linha: line_no,
                    };
                    notes.push(format!(
                        "L{line_no}: `{var_name}` foi movido para dentro da função chamada (considere passar `&{var_name}`)."
                    ));
                }
            }
        }

        Self {
            vars,
            metrics,
            notes,
        }
    }

    pub fn summary_lines(&self) -> Vec<String> {
        let mut out = vec![
            format!(
                "Medidor de Alocações: {} · Stack: {} · Heap: {} · Clones: {} · Iteradores Zero-Cost: {}",
                self.metrics.badge(),
                self.metrics.stack_count,
                self.metrics.heap_allocations,
                self.metrics.clones_count,
                self.metrics.zero_cost_chains
            ),
            String::new(),
            "Mapa Visual STACK ──ptr──▶ HEAP:".to_string(),
        ];
        if self.vars.is_empty() {
            out.push("  (Nenhuma variável `let` detectada no arquivo atual)".to_string());
        } else {
            for slot in &self.vars {
                let mut_tag = if slot.mutable { "mut " } else { "" };
                let state_tag = slot.state.tag();
                if slot.region == StorageRegion::Heap {
                    out.push(format!(
                        "  [STACK L{:02}] {mut_tag}{}: {} (ptr,len,cap) ──▶ [HEAP: buffer]  {state_tag}",
                        slot.declared_line, slot.name, slot.ty_hint
                    ));
                } else {
                    out.push(format!(
                        "  [STACK L{:02}] {mut_tag}{}: {} (valor direto)                {state_tag}",
                        slot.declared_line, slot.name, slot.ty_hint
                    ));
                }
            }
        }
        if !self.notes.is_empty() {
            out.push(String::new());
            out.push("Eventos de Ownership & Borrow Checker:".to_string());
            for note in &self.notes {
                out.push(format!("  • {note}"));
            }
        }
        out
    }
}

fn mark_borrow(
    vars: &mut [VarSlot],
    index_by_name: &HashMap<String, usize>,
    target: &str,
    mutable: bool,
    line_no: usize,
    notes: &mut Vec<String>,
) {
    let Some(&idx) = index_by_name.get(target) else {
        return;
    };
    match vars[idx].state.clone() {
        VarOwnershipState::Movido { destino, .. } => {
            vars[idx].state = VarOwnershipState::ConflitoBorrow;
            notes.push(format!(
                "L{line_no}: Tentou emprestar `{target}` após move para `{destino}`!"
            ));
        }
        VarOwnershipState::EmprestadoMutavel(_) => {
            vars[idx].state = VarOwnershipState::ConflitoBorrow;
            notes.push(format!(
                "L{line_no}: `{target}` já possui `&mut` ativo — não pode coexistir outro empréstimo!"
            ));
        }
        VarOwnershipState::EmprestadoImutavel(_) if mutable => {
            vars[idx].state = VarOwnershipState::ConflitoBorrow;
            notes.push(format!(
                "L{line_no}: `{target}` já tem `&` imutável ativo — `&mut` causaria corrida de dados!"
            ));
        }
        VarOwnershipState::Ativo | VarOwnershipState::EmprestadoImutavel(_) => {
            vars[idx].state = if mutable {
                VarOwnershipState::EmprestadoMutavel(line_no)
            } else {
                VarOwnershipState::EmprestadoImutavel(line_no)
            };
        }
        VarOwnershipState::ConflitoBorrow => {}
    }
}

fn strip_line_comment(line: &str) -> &str {
    line.split("//").next().unwrap_or("")
}

fn count_heap_markers(line: &str) -> usize {
    const MARKERS: &[&str] = &[
        "String::from",
        "String::new",
        ".to_string(",
        ".to_owned(",
        "format!(",
        "Vec::new",
        "Vec::with_capacity",
        "vec![",
        "Box::new",
        "HashMap::new",
        "BTreeMap::new",
        ".collect::",
        ".collect()",
    ];
    MARKERS.iter().map(|m| line.matches(m).count()).sum()
}

fn count_zero_cost_markers(line: &str) -> usize {
    const ZERO_COST: &[&str] = &[
        ".iter()",
        ".into_iter()",
        ".copied()",
        ".filter(",
        ".map(",
        ".sum::<",
        ".sum()",
        ".fold(",
        ".enumerate()",
    ];
    ZERO_COST.iter().map(|m| line.matches(m).count()).sum()
}

fn parse_let_lhs(lhs: &str) -> (String, bool, Option<String>) {
    let mut work = lhs.trim();
    let mutable = if let Some(after) = work.strip_prefix("mut ") {
        work = after.trim();
        true
    } else {
        false
    };
    let (name_part, ty_part) = match work.split_once(':') {
        Some((n, t)) => (n.trim(), Some(t.trim().to_string())),
        None => (work, None),
    };
    let ident = extract_leading_ident(name_part).to_string();
    (ident, mutable, ty_part)
}

fn extract_leading_ident(s: &str) -> &str {
    let s = s.trim();
    let end = s
        .char_indices()
        .take_while(|(i, c)| {
            if *i == 0 {
                c.is_ascii_alphabetic() || *c == '_'
            } else {
                c.is_ascii_alphanumeric() || *c == '_'
            }
        })
        .map(|(i, c)| i + c.len_utf8())
        .last()
        .unwrap_or(0);
    &s[..end]
}

fn classify_rhs(
    rhs: &str,
    vars: &[VarSlot],
    index_by_name: &HashMap<String, usize>,
) -> (StorageRegion, String) {
    if rhs.starts_with("String::from")
        || rhs.contains(".to_string()")
        || rhs.contains("format!(")
        || rhs.contains("String::new")
    {
        return (StorageRegion::Heap, "String".to_string());
    }
    if rhs.starts_with("vec![") || rhs.starts_with("Vec::") {
        return (StorageRegion::Heap, "Vec<_>".to_string());
    }
    if rhs.starts_with("Box::new") {
        return (StorageRegion::Heap, "Box<_>".to_string());
    }
    if rhs.starts_with("HashMap::") {
        return (StorageRegion::Heap, "HashMap<_, _>".to_string());
    }
    if rhs.starts_with("&mut ") {
        return (StorageRegion::Stack, "&mut T".to_string());
    }
    if rhs.starts_with('&') {
        return (StorageRegion::Stack, "&T".to_string());
    }
    if rhs.starts_with('"') {
        return (StorageRegion::Stack, "&'static str".to_string());
    }
    if rhs == "true" || rhs == "false" {
        return (StorageRegion::Stack, "bool".to_string());
    }
    if rhs.parse::<i64>().is_ok() {
        return (StorageRegion::Stack, "i32".to_string());
    }
    if rhs.parse::<f64>().is_ok() {
        return (StorageRegion::Stack, "f64".to_string());
    }
    let ident = extract_leading_ident(rhs);
    if !ident.is_empty()
        && let Some(&idx) = index_by_name.get(ident)
    {
        return (vars[idx].region, vars[idx].ty_hint.clone());
    }
    if rhs.contains(".clone()") {
        return (StorageRegion::Heap, "Clone(Heap)".to_string());
    }
    (StorageRegion::Stack, "valor".to_string())
}

// =========================================================================
// 2. LENTE DE DIAGNÓSTICO SOCRÁTICA + MODO ELI10 + ANTES/DEPOIS
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdiomaticFix {
    pub label: &'static str,
    pub before: &'static str,
    pub after: &'static str,
    pub explanation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocraticErrorLens {
    pub code: &'static str,
    pub title: &'static str,
    pub socratic_question: &'static str,
    pub eli10_metaphor: &'static str,
    pub fixes: [IdiomaticFix; 3],
}

impl SocraticErrorLens {
    pub fn analyze(diagnostics_text: &str, source_code: &str) -> Self {
        let low = diagnostics_text.to_ascii_lowercase();
        if diagnostics_text.contains("E0382") || low.contains("borrow of moved value") {
            return Self {
                code: "E0382",
                title: "Valor usado após transferência de posse (Move)",
                socratic_question: "Quem precisa ser o DONO definitivo desse dado no final, e quem precisa apenas LER temporariamente?",
                eli10_metaphor: "Imagine um único caderno físico: se você entregou o caderno para um amigo (move), não pode mais riscar nele — a menos que apenas mostre a página (&) ou tire xerox (.clone()).",
                fixes: [
                    IdiomaticFix {
                        label: "Emprestar (&) para leitura",
                        before: "imprimir(texto);\nprintln!(\"{texto}\");",
                        after: "imprimir(&texto);\nprintln!(\"{texto}\");",
                        explanation: "Passar `&texto` empresta sem transferir posse; custo zero.",
                    },
                    IdiomaticFix {
                        label: "Reordenar último uso antes do move",
                        before: "consumir(texto);\nlet tam = texto.len();",
                        after: "let tam = texto.len();\nconsumir(texto);",
                        explanation: "Leia propriedades enquanto você ainda tem a posse.",
                    },
                    IdiomaticFix {
                        label: "Clonar apenas quando precisar de 2 donos",
                        before: "enviar(texto);\narquivar(texto);",
                        after: "enviar(texto.clone());\narquivar(texto);",
                        explanation: "Cria cópia independente na Heap quando ambos exigem ownership.",
                    },
                ],
            };
        }
        if diagnostics_text.contains("E0502")
            || diagnostics_text.contains("E0499")
            || low.contains("cannot borrow")
        {
            return Self {
                code: "E0502/E0499",
                title: "Conflito entre empréstimo imutável (&) e mutável (&mut)",
                socratic_question: "Você ainda usa a referência `&` depois da linha que tenta modificar o dado com `&mut`?",
                eli10_metaphor: "Pense numa receita no quadro: várias pessoas podem olhar juntas (&). Mas se alguém for apagar e reescrever (&mut), ninguém pode estar lendo no mesmo instante para não ler algo pela metade!",
                fixes: [
                    IdiomaticFix {
                        label: "Encurtar escopo da leitura (NLL)",
                        before: "let r = &v[0];\nv.push(99);\nprintln!(\"{r}\");",
                        after: "let primeiro = v[0];\nv.push(99);\nprintln!(\"{primeiro}\");",
                        explanation: "Copie valores pequenos (`i32`) ou termine o uso de `r` antes de `v.push`.",
                    },
                    IdiomaticFix {
                        label: "Agrupar mutações antes de emprestar",
                        before: "let r = &v;\nv.push(10);\nprintln!(\"{r:?}\");",
                        after: "v.push(10);\nlet r = &v;\nprintln!(\"{r:?}\");",
                        explanation: "Faça todas as alterações primeiro e crie referências só depois.",
                    },
                    IdiomaticFix {
                        label: "Usar índice em vez de referência viva",
                        before: "let melhor = &itens[0];\nitens.push(novo);",
                        after: "let idx = 0;\nitens.push(novo);\nlet melhor = &itens[idx];",
                        explanation: "`usize` não prende o vetor no Borrow Checker.",
                    },
                ],
            };
        }
        if diagnostics_text.contains("E0384") || low.contains("cannot assign twice to immutable") {
            return Self {
                code: "E0384",
                title: "Tentativa de alterar variável imutável",
                socratic_question: "Essa variável realmente precisa mudar de valor ao longo do tempo, ou outro `let` (shadowing) deixaria o fluxo mais claro?",
                eli10_metaphor: "Em Rust toda caixa começa lacrada com cadeado (`let`). Para abrir e trocar o conteúdo depois, coloque a etiqueta `let mut` na hora de criar!",
                fixes: [
                    IdiomaticFix {
                        label: "Adicionar `mut` na declaração",
                        before: "let total = 0;\ntotal += 5;",
                        after: "let mut total = 0;\ntotal += 5;",
                        explanation: "Padrão para acumuladores, contadores e estados mutáveis.",
                    },
                    IdiomaticFix {
                        label: "Sombreamento (Shadowing) com `let`",
                        before: "let texto = \" 42 \";\ntexto = texto.trim();",
                        after: "let texto = \" 42 \";\nlet texto = texto.trim();",
                        explanation: "Permite transformar ou mudar o tipo sem tornar a variável mutável.",
                    },
                    IdiomaticFix {
                        label: "Expressão direta sem mutabilidade",
                        before: "let msg;\nif ok { msg = \"sim\"; } else { msg = \"não\"; }",
                        after: "let msg = if ok { \"sim\" } else { \"não\" };",
                        explanation: "Em Rust `if/else` retorna valor diretamente.",
                    },
                ],
            };
        }
        if diagnostics_text.contains("E0597")
            || diagnostics_text.contains("E0106")
            || low.contains("does not live long enough")
        {
            return Self {
                code: "E0597/E0106",
                title: "Referência vive mais que o valor dono (Lifetime)",
                socratic_question: "Onde o dono real dos bytes nasce e quando as chaves `{}` dele terminam?",
                eli10_metaphor: "Um ingresso (`&`) só vale enquanto o parque de diversões (dono) estiver aberto. Se o parque fecha ao sair das chaves `{}`, você precisa levar o brinquedo próprio (`String`) para casa!",
                fixes: [
                    IdiomaticFix {
                        label: "Retornar valor com posse (`String` em vez de `&str`)",
                        before: "fn nome() -> &str { let s = String::from(\"Ana\"); &s }",
                        after: "fn nome() -> String { String::from(\"Ana\") }",
                        explanation: "Não dá para retornar referência de variável criada dentro da função.",
                    },
                    IdiomaticFix {
                        label: "Conectar entradas e saída com `'a`",
                        before: "fn maior(a: &str, b: &str) -> &str { ... }",
                        after: "fn maior<'a>(a: &'a str, b: &'a str) -> &'a str { ... }",
                        explanation: "Explica ao compilador que o retorno vem de `a` ou `b`.",
                    },
                    IdiomaticFix {
                        label: "Mover criação para fora do bloco menor",
                        before: "let r; { let x = 5; r = &x; }\nprintln!(\"{r}\");",
                        after: "let x = 5;\nlet r = &x;\nprintln!(\"{r}\");",
                        explanation: "Garante que `x` vive tanto quanto `r`.",
                    },
                ],
            };
        }
        if diagnostics_text.contains("E0308") || low.contains("mismatched types") {
            return Self {
                code: "E0308",
                title: "Tipos incompatíveis (ou `;` sobrando no retorno)",
                socratic_question: "Há um ponto e vírgula `;` no fim da última linha da função transformando o retorno em `()`?",
                eli10_metaphor: "É como tentar encaixar um bloco quadrado (`String`) num buraco redondo (`&str`) — ou fechar a caixa vazia `;` quando o destinatário esperava um presente lá dentro.",
                fixes: [
                    IdiomaticFix {
                        label: "Remover `;` da última expressão de retorno",
                        before: "fn dobro(x: i32) -> i32 {\n    x * 2;\n}",
                        after: "fn dobro(x: i32) -> i32 {\n    x * 2\n}",
                        explanation: "Sem `;`, a expressão vira o retorno da função.",
                    },
                    IdiomaticFix {
                        label: "Converter `String` ↔ `&str` sem alocar à toa",
                        before: "let s: String = \"oi\";",
                        after: "let s: String = \"oi\".to_string(); // ou use &str",
                        explanation: "`&str` aceita `&meu_string`; `String` pede `.to_string()`/`.into()`.",
                    },
                    IdiomaticFix {
                        label: "Anotar tipo explícito no `collect` / `parse`",
                        before: "let n = \"42\".parse();",
                        after: "let n: i32 = \"42\".parse().unwrap_or(0);",
                        explanation: "Informa o tipo numérico exato ao compilador.",
                    },
                ],
            };
        }

        // Lente preventiva baseada no código-fonte quando não há erro ativo.
        if source_code.contains(".clone()") {
            return Self {
                code: "LENTE-CLONE",
                title: "Lente Preventiva: Otimize cópias na Heap (.clone)",
                socratic_question: "Esse `.clone()` é realmente necessário ou a função aceita uma fatia emprestada (`&str` / `&[T]`)?",
                eli10_metaphor: "Clonar é tirar uma fotocópia inteira de um livro só para ler uma frase. Muitas vezes basta emprestar o livro rapidinho com `&`!",
                fixes: [
                    IdiomaticFix {
                        label: "Trocar `String` por `&str` em parâmetros de leitura",
                        before: "fn contar(s: String) -> usize { s.len() }\ncontar(nome.clone());",
                        after: "fn contar(s: &str) -> usize { s.len() }\ncontar(&nome);",
                        explanation: "Elimina 100% da alocação Heap na chamada.",
                    },
                    IdiomaticFix {
                        label: "Usar `.iter()` em vez de `.clone()` em loops",
                        before: "for item in lista.clone() { ... }",
                        after: "for item in &lista { ... }",
                        explanation: "Iterar por referência preserva a posse do vetor original.",
                    },
                    IdiomaticFix {
                        label: "Usar `std::mem::take` ao extrair de struct",
                        before: "let antigo = self.buf.clone();\nself.buf.clear();",
                        after: "let antigo = std::mem::take(&mut self.buf);",
                        explanation: "Move a posse sem copiar nenhum byte!",
                    },
                ],
            };
        }

        Self {
            code: "LENTE-OK",
            title: "Lente Socrática: Boas Práticas & Sem Erros Ativos",
            socratic_question: "Todas as suas funções tratam falhas sem `unwrap()` e recebem `&str`/`&[T]` quando apenas leem dados?",
            eli10_metaphor: "Seu código está como uma oficina organizada: cada variável sabe quem é seu dono e não há ferramentas perdidas pelo chão!",
            fixes: [
                IdiomaticFix {
                    label: "Preferir `Option`/`Result` + `?` ao invés de `.unwrap()`",
                    before: "let n: i32 = txt.trim().parse().unwrap();",
                    after: "let n: i32 = txt.trim().parse().ok()?;",
                    explanation: "Evita panics em produção mantendo clareza.",
                },
                IdiomaticFix {
                    label: "Usar slices (`&str`, `&[T]`) em assinaturas",
                    before: "fn soma(v: Vec<i32>) -> i32",
                    after: "fn soma(v: &[i32]) -> i32",
                    explanation: "Aceita arrays, `Vec` e fatias com zero alocações.",
                },
                IdiomaticFix {
                    label: "Pipeline funcional zero-cost",
                    before: "let mut acc = 0; for x in v { if *x > 0 { acc += x; } }",
                    after: "let acc: i32 = v.iter().filter(|&&x| x > 0).sum();",
                    explanation: "Compila para o mesmo loop ótimo de máquina sem vetor extra.",
                },
            ],
        }
    }
}

// =========================================================================
// 3. DEBUGGER VISUAL PASSO A PASSO NO TERMINAL
// =========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugLocal {
    pub name: String,
    pub ty: String,
    pub value: String,
    pub region: StorageRegion,
    pub status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugStep {
    pub step_index: usize,
    pub line_1based: usize,
    pub depth: usize,
    pub func: String,
    pub code_line: String,
    pub locals: Vec<DebugLocal>,
    pub stdout: Vec<String>,
    pub call_stack: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Default)]
pub struct VisualDebuggerState {
    pub breakpoints: BTreeSet<usize>,
    pub active: bool,
    pub current_step: usize,
    pub steps: Vec<DebugStep>,
}

impl VisualDebuggerState {
    /// Liga/desliga breakpoint na linha 1-based e retorna `true` se ficou ativo.
    pub fn toggle_breakpoint(&mut self, line_1based: usize) -> bool {
        if self.breakpoints.contains(&line_1based) {
            self.breakpoints.remove(&line_1based);
            false
        } else {
            self.breakpoints.insert(line_1based);
            true
        }
    }

    pub fn active_line(&self) -> Option<usize> {
        if !self.active {
            return None;
        }
        self.steps.get(self.current_step).map(|s| s.line_1based)
    }

    pub fn current(&self) -> Option<&DebugStep> {
        if !self.active {
            return None;
        }
        self.steps.get(self.current_step)
    }

    pub fn start_or_rebuild(&mut self, code: &str) {
        self.steps = build_debug_trace(code);
        self.active = !self.steps.is_empty();
        self.current_step = 0;
    }

    /// Avança mantendo-se no mesmo nível de chamada (`Step Over`).
    pub fn step_over(&mut self) -> Option<&DebugStep> {
        if !self.active || self.steps.is_empty() {
            return None;
        }
        let cur_depth = self.steps[self.current_step].depth;
        if self.current_step + 1 < self.steps.len() {
            let mut idx = self.current_step + 1;
            while idx + 1 < self.steps.len() && self.steps[idx].depth > cur_depth {
                idx += 1;
            }
            self.current_step = idx;
        }
        self.steps.get(self.current_step)
    }

    /// Entra em funções chamadas passo a passo (`Step Into`).
    pub fn step_into(&mut self) -> Option<&DebugStep> {
        if !self.active || self.steps.is_empty() {
            return None;
        }
        if self.current_step + 1 < self.steps.len() {
            self.current_step += 1;
        }
        self.steps.get(self.current_step)
    }

    /// Continua até o próximo breakpoint marcado (ou fim do programa).
    pub fn continue_to_breakpoint(&mut self) -> Option<&DebugStep> {
        if !self.active || self.steps.is_empty() {
            return None;
        }
        let mut idx = self.current_step + 1;
        while idx < self.steps.len() {
            if self.breakpoints.contains(&self.steps[idx].line_1based) {
                self.current_step = idx;
                return self.steps.get(idx);
            }
            idx += 1;
        }
        self.current_step = self.steps.len() - 1;
        self.steps.get(self.current_step)
    }

    pub fn restart(&mut self) {
        if !self.steps.is_empty() {
            self.active = true;
            self.current_step = 0;
        }
    }

    pub fn stop(&mut self) {
        self.active = false;
        self.current_step = 0;
    }
}

#[derive(Debug, Clone)]
struct ParsedFunction {
    name: String,
    params: Vec<String>,
    body_lines: Vec<(usize, String)>,
}

fn parse_functions(code: &str) -> BTreeMap<String, ParsedFunction> {
    let mut map = BTreeMap::new();
    let lines: Vec<&str> = code.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = strip_line_comment(lines[i]).trim();
        let after_pub = trimmed.strip_prefix("pub ").unwrap_or(trimmed).trim();
        if let Some(after_fn) = after_pub.strip_prefix("fn ")
            && let Some(open_paren) = after_fn.find('(')
        {
            let fn_name = after_fn[..open_paren].trim().to_string();
            let params_str = after_fn[open_paren + 1..].split(')').next().unwrap_or("");
            let params: Vec<String> = params_str
                .split(',')
                .filter_map(|p| {
                    let ident = extract_leading_ident(p.trim());
                    if ident.is_empty() || ident == "self" {
                        None
                    } else {
                        Some(ident.to_string())
                    }
                })
                .collect();

            let mut depth: i32 = 0;
            let mut started = false;
            let mut body_lines = Vec::new();
            let mut j = i;
            while j < lines.len() {
                let raw = lines[j];
                for ch in strip_line_comment(raw).chars() {
                    if ch == '{' {
                        depth += 1;
                        started = true;
                    } else if ch == '}' {
                        depth -= 1;
                    }
                }
                if j > i {
                    let clean = strip_line_comment(raw).trim();
                    if !clean.is_empty() && clean != "{" && clean != "}" {
                        body_lines.push((j + 1, clean.to_string()));
                    }
                }
                if started && depth <= 0 {
                    break;
                }
                j += 1;
            }
            map.insert(
                fn_name.clone(),
                ParsedFunction {
                    name: fn_name,
                    params,
                    body_lines,
                },
            );
            i = j + 1;
            continue;
        }
        i += 1;
    }
    map
}

fn build_debug_trace(code: &str) -> Vec<DebugStep> {
    let funcs = parse_functions(code);
    let mut steps = Vec::new();
    let mut stdout = Vec::new();
    let mut stack = vec!["main()".to_string()];

    if let Some(main_fn) = funcs.get("main") {
        let mut locals: Vec<(String, DebugLocal)> = Vec::new();
        execute_fn_trace(
            main_fn,
            &funcs,
            0,
            &mut locals,
            &mut stdout,
            &mut stack,
            &mut steps,
        );
    } else if let Some(first_fn) = funcs.values().next() {
        let mut locals: Vec<(String, DebugLocal)> = first_fn
            .params
            .iter()
            .map(|p| {
                (
                    p.clone(),
                    DebugLocal {
                        name: p.clone(),
                        ty: "param".to_string(),
                        value: "<arg>".to_string(),
                        region: StorageRegion::Stack,
                        status: "ativo",
                    },
                )
            })
            .collect();
        stack[0] = format!("{}()", first_fn.name);
        execute_fn_trace(
            first_fn,
            &funcs,
            0,
            &mut locals,
            &mut stdout,
            &mut stack,
            &mut steps,
        );
    } else {
        for (idx, raw) in code.lines().enumerate() {
            let clean = strip_line_comment(raw).trim();
            if clean.is_empty() {
                continue;
            }
            steps.push(DebugStep {
                step_index: steps.len(),
                line_1based: idx + 1,
                depth: 0,
                func: "<arquivo>".to_string(),
                code_line: clean.to_string(),
                locals: Vec::new(),
                stdout: stdout.clone(),
                call_stack: vec!["<top-level>".to_string()],
                note: "Linha de código inspecionada.".to_string(),
            });
            if steps.len() >= 200 {
                break;
            }
        }
    }
    steps
}

fn execute_fn_trace(
    func: &ParsedFunction,
    funcs: &BTreeMap<String, ParsedFunction>,
    depth: usize,
    locals: &mut Vec<(String, DebugLocal)>,
    stdout: &mut Vec<String>,
    stack: &mut Vec<String>,
    steps: &mut Vec<DebugStep>,
) {
    for (line_no, stmt) in &func.body_lines {
        if steps.len() >= 200 {
            return;
        }
        let mut note = String::from("Instrução executada.");
        let clean = stmt.trim().trim_end_matches(';').trim();

        if clean.starts_with("let ") {
            if let Some(after_let) = clean.strip_prefix("let ")
                && let Some((lhs, rhs)) = after_let.split_once('=')
            {
                let (name, _, ty_opt) = parse_let_lhs(lhs);
                let rhs = rhs.trim();
                let (val, region, ty) = eval_expr_preview(rhs, locals);
                // Detecta move de variável Heap.
                let src = extract_leading_ident(rhs);
                if rhs == src
                    && let Some((_, orig)) = locals.iter_mut().find(|(n, _)| n == src)
                    && orig.region == StorageRegion::Heap
                {
                    orig.status = "✗ movido";
                    note = format!("Move: posse de `{src}` transferida para `{name}`.");
                } else {
                    note = format!("Variável `{name}` criada em {}.", region.label());
                }
                let final_ty = ty_opt.unwrap_or(ty);
                upsert_local(
                    locals,
                    DebugLocal {
                        name: name.clone(),
                        ty: final_ty,
                        value: val,
                        region,
                        status: "ativo",
                    },
                );
                // Se o RHS chamar outra função conhecida no arquivo, registra step into!
                if let Some(call_name) = extract_fn_call_name(rhs)
                    && let Some(callee) = funcs.get(call_name)
                    && depth < 4
                {
                    steps.push(DebugStep {
                        step_index: steps.len(),
                        line_1based: *line_no,
                        depth,
                        func: func.name.clone(),
                        code_line: stmt.clone(),
                        locals: locals.iter().map(|(_, v)| v.clone()).collect(),
                        stdout: stdout.clone(),
                        call_stack: stack.clone(),
                        note: format!("Chamando `{call_name}(...)`"),
                    });
                    let mut callee_locals = build_callee_locals(callee, rhs, locals);
                    stack.push(format!("{call_name}()"));
                    execute_fn_trace(
                        callee,
                        funcs,
                        depth + 1,
                        &mut callee_locals,
                        stdout,
                        stack,
                        steps,
                    );
                    stack.pop();
                    continue;
                }
            }
        } else if let Some((var_name, rhs)) = parse_compound_or_assign(clean) {
            let (new_val, _, _) = eval_compound(var_name, clean, rhs, locals);
            if let Some((_, slot)) = locals.iter_mut().find(|(n, _)| n == var_name) {
                slot.value = new_val.clone();
                note = format!("`{var_name}` atualizado para `{new_val}`.");
            }
        } else if clean.starts_with("println!(") || clean.starts_with("eprintln!(") {
            let rendered = eval_println_preview(clean, locals);
            stdout.push(rendered.clone());
            note = format!("Saída impressa: {rendered}");
        } else if let Some(call_name) = extract_fn_call_name(clean)
            && let Some(callee) = funcs.get(call_name)
            && depth < 4
        {
            steps.push(DebugStep {
                step_index: steps.len(),
                line_1based: *line_no,
                depth,
                func: func.name.clone(),
                code_line: stmt.clone(),
                locals: locals.iter().map(|(_, v)| v.clone()).collect(),
                stdout: stdout.clone(),
                call_stack: stack.clone(),
                note: format!("Entrando em `{call_name}(...)`"),
            });
            let mut callee_locals = build_callee_locals(callee, clean, locals);
            stack.push(format!("{call_name}()"));
            execute_fn_trace(
                callee,
                funcs,
                depth + 1,
                &mut callee_locals,
                stdout,
                stack,
                steps,
            );
            stack.pop();
            continue;
        }

        steps.push(DebugStep {
            step_index: steps.len(),
            line_1based: *line_no,
            depth,
            func: func.name.clone(),
            code_line: stmt.clone(),
            locals: locals.iter().map(|(_, v)| v.clone()).collect(),
            stdout: stdout.clone(),
            call_stack: stack.clone(),
            note,
        });
    }
}

fn build_callee_locals(
    callee: &ParsedFunction,
    call_expr: &str,
    caller_locals: &[(String, DebugLocal)],
) -> Vec<(String, DebugLocal)> {
    let args_raw = call_expr
        .split_once('(')
        .and_then(|(_, rest)| rest.rsplit_once(')').map(|(inner, _)| inner))
        .unwrap_or("");
    let args: Vec<&str> = args_raw.split(',').map(str::trim).collect();
    callee
        .params
        .iter()
        .enumerate()
        .map(|(idx, pname)| {
            let raw_arg = args.get(idx).copied().unwrap_or("");
            let (val, region, ty) = eval_expr_preview(raw_arg, caller_locals);
            (
                pname.clone(),
                DebugLocal {
                    name: pname.clone(),
                    ty,
                    value: val,
                    region,
                    status: "ativo",
                },
            )
        })
        .collect()
}

fn upsert_local(locals: &mut Vec<(String, DebugLocal)>, item: DebugLocal) {
    if let Some((_, existing)) = locals.iter_mut().find(|(n, _)| n == &item.name) {
        *existing = item;
    } else {
        locals.push((item.name.clone(), item));
    }
}

fn extract_fn_call_name(expr: &str) -> Option<&str> {
    if expr.contains('!') {
        return None;
    }
    let open = expr.find('(')?;
    let before = expr[..open].trim();
    if before.contains('.') || before.contains(' ') {
        return None;
    }
    let name = extract_leading_ident(before);
    if name.is_empty() || matches!(name, "if" | "while" | "for" | "match" | "return") {
        None
    } else {
        Some(name)
    }
}

fn parse_compound_or_assign(stmt: &str) -> Option<(&str, &str)> {
    for op in ["+=", "-=", "*=", "="] {
        if let Some((lhs, rhs)) = stmt.split_once(op)
            && !stmt.contains("==")
            && !stmt.contains("!=")
            && !stmt.contains("<=")
            && !stmt.contains(">=")
        {
            let name = extract_leading_ident(lhs.trim());
            if !name.is_empty() {
                return Some((name, rhs.trim()));
            }
        }
    }
    None
}

fn eval_compound(
    var_name: &str,
    full_stmt: &str,
    rhs: &str,
    locals: &[(String, DebugLocal)],
) -> (String, StorageRegion, String) {
    let cur_i64 = locals
        .iter()
        .find(|(n, _)| n == var_name)
        .and_then(|(_, slot)| slot.value.parse::<i64>().ok());
    let rhs_i64 = eval_int_expr(rhs, locals);
    if let (Some(cur), Some(rv)) = (cur_i64, rhs_i64) {
        let next = if full_stmt.contains("+=") {
            cur.saturating_add(rv)
        } else if full_stmt.contains("-=") {
            cur.saturating_sub(rv)
        } else if full_stmt.contains("*=") {
            cur.saturating_mul(rv)
        } else {
            rv
        };
        return (next.to_string(), StorageRegion::Stack, "i32".to_string());
    }
    eval_expr_preview(rhs, locals)
}

fn eval_int_expr(expr: &str, locals: &[(String, DebugLocal)]) -> Option<i64> {
    let expr = expr.trim();
    if let Ok(v) = expr.parse::<i64>() {
        return Some(v);
    }
    if let Some((_, slot)) = locals.iter().find(|(n, _)| n == expr)
        && let Ok(v) = slot.value.parse::<i64>()
    {
        return Some(v);
    }
    for (op_ch, f) in [
        ('+', i64::saturating_add as fn(i64, i64) -> i64),
        ('-', i64::saturating_sub as fn(i64, i64) -> i64),
        ('*', i64::saturating_mul as fn(i64, i64) -> i64),
    ] {
        if let Some((a, b)) = expr.split_once(op_ch)
            && let (Some(va), Some(vb)) = (eval_int_expr(a, locals), eval_int_expr(b, locals))
        {
            return Some(f(va, vb));
        }
    }
    None
}

fn eval_expr_preview(
    rhs: &str,
    locals: &[(String, DebugLocal)],
) -> (String, StorageRegion, String) {
    let rhs = rhs.trim();
    if let Some(v) = eval_int_expr(rhs, locals) {
        return (v.to_string(), StorageRegion::Stack, "i32".to_string());
    }
    if let Some(inner) = rhs
        .strip_prefix("String::from(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return (
            inner.trim().to_string(),
            StorageRegion::Heap,
            "String".to_string(),
        );
    }
    if let Some(prefix) = rhs.strip_suffix(".to_string()") {
        return (
            prefix.trim().to_string(),
            StorageRegion::Heap,
            "String".to_string(),
        );
    }
    if rhs.starts_with("vec![") {
        return (rhs.to_string(), StorageRegion::Heap, "Vec<_>".to_string());
    }
    if let Some(target) = rhs.strip_prefix('&') {
        let name = extract_leading_ident(target.trim_start_matches("mut "));
        let target_val = locals
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, l)| l.value.as_str())
            .unwrap_or(name);
        return (
            format!("-> {target_val}"),
            StorageRegion::Stack,
            if rhs.starts_with("&mut ") {
                "&mut"
            } else {
                "&"
            }
            .to_string(),
        );
    }
    if let Some((_, slot)) = locals.iter().find(|(n, _)| n == rhs) {
        return (slot.value.clone(), slot.region, slot.ty.clone());
    }
    let region = if rhs.contains("String") || rhs.contains("Vec") || rhs.contains("format!") {
        StorageRegion::Heap
    } else {
        StorageRegion::Stack
    };
    (rhs.to_string(), region, "auto".to_string())
}

fn eval_println_preview(stmt: &str, locals: &[(String, DebugLocal)]) -> String {
    let Some(start) = stmt.find('(') else {
        return stmt.to_string();
    };
    let Some(end) = stmt.rfind(')') else {
        return stmt.to_string();
    };
    let inside = stmt[start + 1..end].trim();
    let mut out = inside.trim_matches('"').to_string();
    if let Some(first_quote) = inside.find('"')
        && let Some(rel_end) = inside[first_quote + 1..].find('"')
    {
        let fmt = &inside[first_quote + 1..first_quote + 1 + rel_end];
        let rest_args = inside[first_quote + 1 + rel_end + 1..]
            .trim()
            .strip_prefix(',')
            .unwrap_or("")
            .trim();
        out = fmt.to_string();
        for (name, slot) in locals {
            let needle = format!("{{{name}}}");
            if out.contains(&needle) {
                out = out.replace(&needle, slot.value.trim_matches('"'));
            }
        }
        if !rest_args.is_empty() {
            for arg in rest_args.split(',').map(str::trim) {
                let (val, _, _) = eval_expr_preview(arg, locals);
                out = out.replacen("{}", val.trim_matches('"'), 1);
            }
        }
    }
    out
}

// =========================================================================
// 4. MODO DESAFIO GAMIFICADO: "CONSERTE O CÓDIGO" & GOLFING DE ALOCAÇÕES
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TroubleshootingQuest {
    pub id: &'static str,
    pub title: &'static str,
    pub error_tag: &'static str,
    pub description: &'static str,
    pub broken_code: &'static str,
    pub fixed_reference: &'static str,
    pub par_lines: usize,
    pub max_heap_allocs: usize,
    pub hint: &'static str,
    pub required_tokens: &'static [&'static str],
    pub forbidden_tokens: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestEvaluation {
    pub modified_lines: usize,
    pub par_lines: usize,
    pub heap_allocations: usize,
    pub clones_count: usize,
    pub golf_bonus: bool,
    pub xp_awarded: u32,
}

pub const TROUBLESHOOTING_QUESTS: [TroubleshootingQuest; 8] = [
    TroubleshootingQuest {
        id: "FIX-01",
        title: "FIX-01 · Conflito de Borrow (& e &mut)",
        error_tag: "E0502",
        description: "O código mantém uma referência `&itens[0]` viva enquanto chama `itens.push(4)`. Faça compilar sem clonar o vetor!",
        broken_code: "pub fn proximo_valor() -> i32 {\n    let mut itens = vec![10, 20, 30];\n    let primeiro = &itens[0];\n    itens.push(40);\n    *primeiro + itens.len() as i32\n}\n\nfn main() {\n    println!(\"{}\", proximo_valor());\n}\n",
        fixed_reference: "pub fn proximo_valor() -> i32 {\n    let mut itens = vec![10, 20, 30];\n    let primeiro = itens[0];\n    itens.push(40);\n    primeiro + itens.len() as i32\n}\n\nfn main() {\n    println!(\"{}\", proximo_valor());\n}\n",
        par_lines: 2,
        max_heap_allocs: 1,
        hint: "Como `i32` implementa `Copy`, armazene `let primeiro = itens[0];` por valor (sem `&`) e some `primeiro` sem `*`!",
        required_tokens: &["let primeiro = itens[0]", "itens.push(40)"],
        forbidden_tokens: &["&itens[0]", ".clone()"],
    },
    TroubleshootingQuest {
        id: "FIX-02",
        title: "FIX-02 · Valor String Movido na Função",
        error_tag: "E0382",
        description: "`saudar(nome)` consome a `String` e o `println!` seguinte falha. Corrija emprestando `&str` (0 clones!).",
        broken_code: "pub fn saudar(pessoa: String) -> String {\n    format!(\"Olá, {pessoa}\")\n}\n\nfn main() {\n    let nome = String::from(\"Ferris\");\n    let msg = saudar(nome);\n    println!(\"{msg} - {nome}\");\n}\n",
        fixed_reference: "pub fn saudar(pessoa: &str) -> String {\n    format!(\"Olá, {pessoa}\")\n}\n\nfn main() {\n    let nome = String::from(\"Ferris\");\n    let msg = saudar(&nome);\n    println!(\"{msg} - {nome}\");\n}\n",
        par_lines: 2,
        max_heap_allocs: 2,
        hint: "Altere o parâmetro `pessoa: String` para `pessoa: &str` e chame `saudar(&nome)`.",
        required_tokens: &["pessoa: &str", "saudar(&nome)"],
        forbidden_tokens: &[".clone()"],
    },
    TroubleshootingQuest {
        id: "FIX-03",
        title: "FIX-03 · Mutabilidade Esquecida no Acumulador",
        error_tag: "E0384",
        description: "O laço tenta somar em `soma`, mas falta mutabilidade. Conserte mexendo em apenas 1 linha.",
        broken_code: "pub fn somar_ate(n: i32) -> i32 {\n    let soma = 0;\n    for i in 1..=n {\n        soma += i;\n    }\n    soma\n}\n\nfn main() {\n    println!(\"{}\", somar_ate(5));\n}\n",
        fixed_reference: "pub fn somar_ate(n: i32) -> i32 {\n    let mut soma = 0;\n    for i in 1..=n {\n        soma += i;\n    }\n    soma\n}\n\nfn main() {\n    println!(\"{}\", somar_ate(5));\n}\n",
        par_lines: 1,
        max_heap_allocs: 0,
        hint: "Adicione `mut` na linha `let soma = 0;`.",
        required_tokens: &["let mut soma = 0"],
        forbidden_tokens: &[],
    },
    TroubleshootingQuest {
        id: "FIX-04",
        title: "FIX-04 · Ponto e Vírgula no Retorno da Função",
        error_tag: "E0308",
        description: "`quadrado` retorna `()` em vez de `i32` por causa de um `;` acidental.",
        broken_code: "pub fn quadrado(x: i32) -> i32 {\n    x * x;\n}\n\nfn main() {\n    println!(\"{}\", quadrado(9));\n}\n",
        fixed_reference: "pub fn quadrado(x: i32) -> i32 {\n    x * x\n}\n\nfn main() {\n    println!(\"{}\", quadrado(9));\n}\n",
        par_lines: 1,
        max_heap_allocs: 0,
        hint: "Remova o `;` de `x * x;` para torná-lo a expressão de retorno.",
        required_tokens: &["x * x"],
        forbidden_tokens: &["x * x;"],
    },
    TroubleshootingQuest {
        id: "FIX-05",
        title: "FIX-05 · Referência Morre Antes do Uso",
        error_tag: "E0597",
        description: "`etiqueta` retorna `&str` apontando para uma `String` local que morre ao fim da função. Transfira a posse!",
        broken_code: "pub fn etiqueta(cod: u32) -> &str {\n    let s = format!(\"ID-{cod}\");\n    &s\n}\n\nfn main() {\n    println!(\"{}\", etiqueta(7));\n}\n",
        fixed_reference: "pub fn etiqueta(cod: u32) -> String {\n    format!(\"ID-{cod}\")\n}\n\nfn main() {\n    println!(\"{}\", etiqueta(7));\n}\n",
        par_lines: 3,
        max_heap_allocs: 1,
        hint: "Troque o tipo de retorno para `-> String` e retorne `format!(\"ID-{cod}\")` diretamente.",
        required_tokens: &["-> String"],
        forbidden_tokens: &["-> &str", "&s"],
    },
    TroubleshootingQuest {
        id: "FIX-06",
        title: "FIX-06 · Golfing de Alocações: Zero-Cost Pipeline",
        error_tag: "ZERO-COST",
        description: "O código aloca um `Vec` extra com `vec![]` e `.push()` só para somar os pares. Reescreva com 0 alocações Heap!",
        broken_code: "pub fn somar_pares(nums: &[i32]) -> i32 {\n    let mut pares = Vec::new();\n    for &n in nums {\n        if n % 2 == 0 {\n            pares.push(n);\n        }\n    }\n    pares.iter().sum()\n}\n\nfn main() {\n    println!(\"{}\", somar_pares(&[1, 2, 3, 4, 6]));\n}\n",
        fixed_reference: "pub fn somar_pares(nums: &[i32]) -> i32 {\n    nums.iter().copied().filter(|n| n % 2 == 0).sum()\n}\n\nfn main() {\n    println!(\"{}\", somar_pares(&[1, 2, 3, 4, 6]));\n}\n",
        par_lines: 7,
        max_heap_allocs: 0,
        hint: "Use `nums.iter().copied().filter(|n| n % 2 == 0).sum()` — nenhuma alocação na Heap!",
        required_tokens: &[".iter()", ".filter(", ".sum()"],
        forbidden_tokens: &["Vec::new", "vec![", ".collect"],
    },
    TroubleshootingQuest {
        id: "FIX-07",
        title: "FIX-07 · Trait Bound em Genéricos",
        error_tag: "E0277",
        description: "`maior<T>` compara `a >= b` sem exigir que `T` implemente `PartialOrd`. Adicione o trait bound necessário.",
        broken_code: "pub fn maior<T>(a: T, b: T) -> T {\n    if a >= b { a } else { b }\n}\n\nfn main() {\n    println!(\"{}\", maior(10, 25));\n}\n",
        fixed_reference: "pub fn maior<T: PartialOrd>(a: T, b: T) -> T {\n    if a >= b { a } else { b }\n}\n\nfn main() {\n    println!(\"{}\", maior(10, 25));\n}\n",
        par_lines: 1,
        max_heap_allocs: 0,
        hint: "Declare `<T: PartialOrd>` na assinatura de `maior`.",
        required_tokens: &["PartialOrd"],
        forbidden_tokens: &["fn maior<T>("],
    },
    TroubleshootingQuest {
        id: "FIX-08",
        title: "FIX-08 · Segurança: Elimine unwrap() no Parser",
        error_tag: "SAFETY",
        description: "Substitua `.unwrap()` por propagação segura retornando `Option<i32>`.",
        broken_code: "pub fn dobrar_texto(txt: &str) -> Option<i32> {\n    let n: i32 = txt.trim().parse().unwrap();\n    Some(n * 2)\n}\n\nfn main() {\n    println!(\"{:?}\", dobrar_texto(\"abc\"));\n}\n",
        fixed_reference: "pub fn dobrar_texto(txt: &str) -> Option<i32> {\n    let n: i32 = txt.trim().parse().ok()?;\n    Some(n * 2)\n}\n\nfn main() {\n    println!(\"{:?}\", dobrar_texto(\"abc\"));\n}\n",
        par_lines: 1,
        max_heap_allocs: 0,
        hint: "Use `.ok()?` após `.parse()` para retornar `None` em entradas inválidas sem panic.",
        required_tokens: &[".ok()?"],
        forbidden_tokens: &[".unwrap()"],
    },
];

pub fn count_modified_lines(original: &str, current: &str) -> usize {
    let a: Vec<&str> = original.lines().map(str::trim).collect();
    let b: Vec<&str> = current.lines().map(str::trim).collect();
    let (m, n) = (a.len(), b.len());
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in (0..m).rev() {
        for j in (0..n).rev() {
            dp[i][j] = if a[i] == b[j] {
                1 + dp[i + 1][j + 1]
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    m.max(n).saturating_sub(dp[0][0])
}

pub fn evaluate_quest(
    quest: &TroubleshootingQuest,
    current_code: &str,
) -> Result<QuestEvaluation, String> {
    if current_code.trim() == quest.broken_code.trim() {
        return Err("Você ainda não editou o código quebrado da missão.".to_string());
    }
    for token in quest.forbidden_tokens {
        if current_code.contains(token) {
            return Err(format!(
                "O padrão problemático `{token}` ainda está presente no código."
            ));
        }
    }
    for token in quest.required_tokens {
        if !current_code.contains(token) {
            return Err(format!(
                "Solução incompleta: esperava encontrar `{token}` na correção idiomática."
            ));
        }
    }
    let xray = MemoryXRayReport::analyze(current_code);
    if xray.metrics.heap_allocations > quest.max_heap_allocs {
        return Err(format!(
            "Orçamento de Heap excedido: usou {} alocação(ões), máximo permitido é {}.",
            xray.metrics.heap_allocations, quest.max_heap_allocs
        ));
    }
    let modified_lines = count_modified_lines(quest.broken_code, current_code);
    let golf_bonus = modified_lines <= quest.par_lines && xray.metrics.clones_count == 0;
    let xp_awarded = if golf_bonus { 120 } else { 80 };
    Ok(QuestEvaluation {
        modified_lines,
        par_lines: quest.par_lines,
        heap_allocations: xray.metrics.heap_allocations,
        clones_count: xray.metrics.clones_count,
        golf_bonus,
        xp_awarded,
    })
}

// =========================================================================
// 5. RUST PLAYGROUND, EXPORTAÇÃO & SOLUÇÕES DA COMUNIDADE
// =========================================================================

pub fn percent_encode_uri_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2);
    for byte in input.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(*byte as char);
        } else {
            out.push_str(&format!("%{:02X}", byte));
        }
    }
    out
}

pub fn build_playground_url(code: &str) -> String {
    let encoded = percent_encode_uri_component(code);
    format!("https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&code={encoded}")
}

pub fn export_project_share_card(root: &Path, code: &str) -> std::io::Result<PathBuf> {
    let url = build_playground_url(code);
    let xray = MemoryXRayReport::analyze(code);
    let summary = xray.summary_lines().join("\n");
    let content = format!(
        "# Projeto RustLab — Pacote de Compartilhamento\n\n\
        ## Link direto para o Rust Playground\n\
        <{url}>\n\n\
        ## Raio-X de Memória & Alocações\n\
        ```text\n{summary}\n```\n\n\
        ## Código-fonte\n\
        ```rust\n{code}\n```\n"
    );
    let path = root.join("COMPARTILHAR_RUSTLAB.md");
    std::fs::write(&path, content)?;
    Ok(path)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommunityApproach {
    pub name: &'static str,
    pub style_badge: &'static str,
    pub pros: &'static str,
    pub code: &'static str,
}

pub fn community_solutions_for(context: &str) -> [CommunityApproach; 3] {
    let low = context.to_ascii_lowercase();
    if low.contains("soma") || low.contains("iter") || low.contains("par") || low.contains("vec") {
        return [
            CommunityApproach {
                name: "1. Pipeline Funcional Zero-Cost (Idiomático Rust)",
                style_badge: "Zero Heap · Conciso",
                pros: "Usa `.iter().filter().map().sum()` compilando sem alocar vetor intermediário.",
                code: "pub fn processar(valores: &[i32]) -> i32 {\n    valores\n        .iter()\n        .copied()\n        .filter(|&v| v > 0)\n        .map(|v| v * 2)\n        .sum()\n}\n\nfn main() {\n    println!(\"Resultado: {}\", processar(&[-2, 3, 5]));\n}\n",
            },
            CommunityApproach {
                name: "2. Imperativo Clássico Explícito (Legível p/ Iniciantes)",
                style_badge: "Fácil Depurar em Breakpoints",
                pros: "Ideal para acompanhar variável por variável no Debugger Visual (`Alt+J`).",
                code: "pub fn processar(valores: &[i32]) -> i32 {\n    let mut total = 0;\n    for &v in valores {\n        if v > 0 {\n            total += v * 2;\n        }\n    }\n    total\n}\n\nfn main() {\n    println!(\"Resultado: {}\", processar(&[-2, 3, 5]));\n}\n",
            },
            CommunityApproach {
                name: "3. Orientado a Tipos & `fold` com Acumulador Rico",
                style_badge: "Imutável · Testável",
                pros: "Agrupa estatísticas de forma pura sem estado mutável espalhado.",
                code: "#[derive(Debug, Default, PartialEq, Eq)]\npub struct Resumo { pub total: i32, pub validos: usize }\n\npub fn resumir(valores: &[i32]) -> Resumo {\n    valores.iter().copied().filter(|&v| v > 0).fold(Resumo::default(), |acc, v| Resumo {\n        total: acc.total + v * 2,\n        validos: acc.validos + 1,\n    })\n}\n\nfn main() {\n    println!(\"{:?}\", resumir(&[-2, 3, 5]));\n}\n",
            },
        ];
    }

    [
        CommunityApproach {
            name: "1. Slices Emprestados (`&str` / `&[T]`) + Zero Clones",
            style_badge: "Zero-Cost Idiomático",
            pros: "Evita exigir `String` em funções que apenas leem texto.",
            code: "pub fn formatar_titulo(nome: &str, nivel: u32) -> String {\n    format!(\"[Nv.{nivel}] {nome}\")\n}\n\nfn main() {\n    println!(\"{}\", formatar_titulo(\"Ferris\", 3));\n}\n",
        },
        CommunityApproach {
            name: "2. Enum + `match` Exaustivo com Propagação `?`",
            style_badge: "Segurança de Tipos",
            pros: "Transforma regras de negócio em variantes verificadas em compilação.",
            code: "pub enum Acao { Somar(i32), Dobrar }\n\npub fn aplicar(valor: i32, acao: Acao) -> Option<i32> {\n    match acao {\n        Acao::Somar(x) => valor.checked_add(x),\n        Acao::Dobrar => valor.checked_mul(2),\n    }\n}\n\nfn main() {\n    println!(\"{:?}\", aplicar(21, Acao::Dobrar));\n}\n",
        },
        CommunityApproach {
            name: "3. Novo-Tipo (`Newtype`) com Validação no Construtor",
            style_badge: "Arquitetura Limpa",
            pros: "Impossibilita estados inválidos depois da criação.",
            code: "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct Pontuacao(u32);\n\nimpl Pontuacao {\n    pub fn nova(v: u32) -> Option<Self> { (v <= 1000).then_some(Self(v)) }\n    pub fn valor(self) -> u32 { self.0 }\n}\n\nfn main() {\n    println!(\"{:?}\", Pontuacao::nova(100));\n}\n",
        },
    ]
}

// =========================================================================
// 6. TRILHAS MODERNAS: WEBASSEMBLY, CLI PRODUÇÃO & BACKEND AXUM
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModernEcosystemTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub ecosystem: &'static str,
    pub dep_name: &'static str,
    pub dep_version: &'static str,
    pub summary: &'static str,
    pub architecture_notes: &'static str,
    pub run_instructions: &'static str,
    pub template_code: &'static str,
}

pub const MODERN_ECOSYSTEMS: [ModernEcosystemTemplate; 3] = [
    ModernEcosystemTemplate {
        id: "wasm-web",
        name: "WebAssembly / Navegador (wasm-bindgen)",
        ecosystem: "Web & Frontend",
        dep_name: "wasm-bindgen",
        dep_version: "0.2",
        summary: "Núcleo gráfico e lógica em Rust prontos para exportar ao navegador via `#[wasm_bindgen]` e `wasm-pack`.",
        architecture_notes: "Separe o estado puro (física, partículas, grid RGBA) de `#[wasm_bindgen]` para rodar testes unitários com `F6` instantaneamente no terminal antes de empacotar com `wasm-pack build --target web`.",
        run_instructions: "F6 roda testes offline · `d` adiciona `wasm-bindgen = \"0.2\"` ao Cargo.toml · Build web: `cargo build --target wasm32-unknown-unknown`.",
        template_code: "/// Motor de partículas 2D pronto para exportar via `#[wasm_bindgen]` para Canvas Web.\npub struct CanvasParticula {\n    pub x: f32,\n    pub y: f32,\n    pub vx: f32,\n    pub vy: f32,\n}\n\nimpl CanvasParticula {\n    pub fn nova(x: f32, y: f32, vx: f32, vy: f32) -> Self {\n        Self { x, y, vx, vy }\n    }\n\n    /// Atualiza posição respeitando paredes do viewport Web (`#[wasm_bindgen]` compatível).\n    pub fn avancar_frame(&mut self, dt: f32, largura: f32, altura: f32) {\n        self.x += self.vx * dt;\n        self.y += self.vy * dt;\n        if self.x <= 0.0 || self.x >= largura {\n            self.vx = -self.vx;\n            self.x = self.x.clamp(0.0, largura);\n        }\n        if self.y <= 0.0 || self.y >= altura {\n            self.vy = -self.vy;\n            self.y = self.y.clamp(0.0, altura);\n        }\n    }\n}\n\nfn main() {\n    let mut p = CanvasParticula::nova(10.0, 20.0, 60.0, 30.0);\n    for frame in 1..=3 {\n        p.avancar_frame(0.5, 100.0, 100.0);\n        println!(\"Frame WASM {frame}: ({:.1}, {:.1})\", p.x, p.y);\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::CanvasParticula;\n\n    #[test]\n    fn particula_ricocheteia_na_borda() {\n        let mut p = CanvasParticula::nova(95.0, 50.0, 20.0, 0.0);\n        p.avancar_frame(0.5, 100.0, 100.0);\n        assert_eq!(p.x, 100.0);\n        assert!(p.vx < 0.0);\n    }\n}\n",
    },
    ModernEcosystemTemplate {
        id: "cli-clap-tokio",
        name: "Sistemas / CLI de Produção (clap + tokio)",
        ecosystem: "Ferramentas CLI & Async",
        dep_name: "clap",
        dep_version: "4",
        summary: "Parser de comandos estilo `clap` + processamento assíncrono em lote com código de saída estruturado.",
        architecture_notes: "Modelar subcomandos como `enum ComandoCli` torna o parser testável com slices de argumentos sem invocar processo externo. Adicione `clap` (feature `derive`) e `tokio` (`full`) ao escalar para I/O real.",
        run_instructions: "F5 executa · F6 testa o parser · `d` adiciona `clap = \"4\"` ao Cargo.toml.",
        template_code: "#[derive(Debug, PartialEq, Eq)]\npub enum ComandoCli {\n    Status { verboso: bool },\n    Executar { alvo: String, jobs: usize },\n}\n\npub fn parse_args(args: &[&str]) -> Result<ComandoCli, String> {\n    match args {\n        [\"status\"] => Ok(ComandoCli::Status { verboso: false }),\n        [\"status\", \"--verboso\" | \"-v\"] => Ok(ComandoCli::Status { verboso: true }),\n        [\"run\", alvo] => Ok(ComandoCli::Executar {\n            alvo: (*alvo).to_string(),\n            jobs: 1,\n        }),\n        [\"run\", alvo, \"--jobs\", j] => {\n            let jobs = j.parse::<usize>().map_err(|_| \"--jobs exige inteiro\".to_string())?;\n            Ok(ComandoCli::Executar {\n                alvo: (*alvo).to_string(),\n                jobs: jobs.max(1),\n            })\n        }\n        _ => Err(\"Uso: app status [-v] | run <alvo> [--jobs N]\".to_string()),\n    }\n}\n\nfn main() {\n    let cli = parse_args(&[\"run\", \"api\", \"--jobs\", \"4\"]).expect(\"args válidos\");\n    println!(\"CLI pronta: {cli:?}\");\n}\n\n#[cfg(test)]\nmod tests {\n    use super::{ComandoCli, parse_args};\n\n    #[test]\n    fn parse_subcomando_run_com_jobs() {\n        assert_eq!(\n            parse_args(&[\"run\", \"worker\", \"--jobs\", \"8\"]),\n            Ok(ComandoCli::Executar {\n                alvo: \"worker\".to_string(),\n                jobs: 8,\n            })\n        );\n        assert!(parse_args(&[\"invalido\"]).is_err());\n    }\n}\n",
    },
    ModernEcosystemTemplate {
        id: "backend-axum",
        name: "Backend & APIs REST (axum + serde)",
        ecosystem: "Web Backend & Microsserviços",
        dep_name: "axum",
        dep_version: "0.8",
        summary: "Roteador HTTP tipado com handlers de saúde (`GET /health`), CRUD em memória e serialização JSON.",
        architecture_notes: "Extrair handlers puros (`Request -> HttpResposta`) permite validar status codes e payloads JSON via `cargo test` em milissegundos antes de subir o servidor Tokio/Axum na porta 3000.",
        run_instructions: "F5 executa simulação · F6 testa rotas HTTP · `d` adiciona `axum = \"0.8\"` ao Cargo.toml.",
        template_code: "#[derive(Debug, Clone, PartialEq, Eq)]\npub struct HttpResposta {\n    pub status: u16,\n    pub body_json: String,\n}\n\npub fn rotear_requisicao(metodo: &str, caminho: &str) -> HttpResposta {\n    match (metodo, caminho) {\n        (\"GET\", \"/health\") => HttpResposta {\n            status: 200,\n            body_json: \"{\\\"status\\\":\\\"ok\\\"}\".to_string(),\n        },\n        (\"GET\", \"/api/v1/versao\") => HttpResposta {\n            status: 200,\n            body_json: \"{\\\"servico\\\":\\\"rustlab-api\\\",\\\"versao\\\":\\\"0.3.2\\\"}\".to_string(),\n        },\n        _ => HttpResposta {\n            status: 404,\n            body_json: \"{\\\"erro\\\":\\\"rota não encontrada\\\"}\".to_string(),\n        },\n    }\n}\n\nfn main() {\n    let resp = rotear_requisicao(\"GET\", \"/health\");\n    println!(\"HTTP {} -> {}\", resp.status, resp.body_json);\n}\n\n#[cfg(test)]\nmod tests {\n    use super::rotear_requisicao;\n\n    #[test]\n    fn rota_health_e_404() {\n        let ok = rotear_requisicao(\"GET\", \"/health\");\n        assert_eq!(ok.status, 200);\n        assert!(ok.body_json.contains(\"ok\"));\n        assert_eq!(rotear_requisicao(\"POST\", \"/desconhecida\").status, 404);\n    }\n}\n",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raiox_detecta_stack_heap_move_conflito_e_zero_cost() {
        let code = r#"
fn main() {
    let x = 42;
    let s1 = String::from("rust");
    let s2 = s1;
    let mut v = vec![1, 2];
    let r = &v;
    v.push(3);
    println!("{x} {s2} {:?}", r);
}
"#;
        let report = MemoryXRayReport::analyze(code);
        assert!(report.metrics.stack_count >= 2);
        assert!(report.metrics.heap_allocations >= 2);
        assert!(report.vars.iter().any(|v| {
            v.name == "s1"
                && matches!(
                    &v.state,
                    VarOwnershipState::Movido { destino, .. } if destino == "s2"
                )
        }));
        assert!(
            report
                .vars
                .iter()
                .any(|v| v.name == "v" && v.state == VarOwnershipState::ConflitoBorrow)
        );

        let zero_cost = "let total: i32 = [1, 2, 3].iter().copied().filter(|n| n % 2 == 1).sum();";
        let zreport = MemoryXRayReport::analyze(zero_cost);
        assert_eq!(zreport.metrics.badge(), "[Zero-Cost ★ 0 Alocações Heap]");
    }

    #[test]
    fn lente_socratica_cobre_erros_e_modo_eli10() {
        for (diag, code) in [
            ("error[E0382]: borrow of moved value", "E0382"),
            ("error[E0502]: cannot borrow `v` as mutable", "E0502/E0499"),
            ("error[E0384]: cannot assign twice to immutable", "E0384"),
            ("error[E0597]: `s` does not live long enough", "E0597/E0106"),
            ("error[E0308]: mismatched types", "E0308"),
        ] {
            let lens = SocraticErrorLens::analyze(diag, "");
            assert_eq!(lens.code, code);
            assert!(!lens.socratic_question.is_empty());
            assert!(!lens.eli10_metaphor.is_empty());
            assert_eq!(lens.fixes.len(), 3);
        }
    }

    #[test]
    fn debugger_visual_passa_por_breakpoints_step_over_step_into_e_locais() {
        let code = r#"
fn somar(a: i32, b: i32) -> i32 {
    let r = a + b;
    r
}

fn main() {
    let mut x = 10;
    let y = somar(x, 5);
    x += 2;
    println!("x={x} y={y}");
}
"#;
        let mut dbg = VisualDebuggerState::default();
        assert!(dbg.toggle_breakpoint(10));
        dbg.start_or_rebuild(code);
        assert!(dbg.active);
        let first = dbg.current().unwrap();
        assert_eq!(first.line_1based, 8);
        assert_eq!(first.locals[0].value, "10");

        // Step Into entra em `somar`
        let step_in = dbg.step_into().unwrap();
        assert_eq!(step_in.line_1based, 9);
        let inside_somar = dbg.step_into().unwrap();
        assert_eq!(inside_somar.func, "somar");
        assert!(inside_somar.call_stack.iter().any(|c| c.contains("somar")));

        // Reinicia e Step Over pula o interior de `somar` direto para a linha seguinte em `main`
        dbg.restart();
        let after_over = dbg.step_over().unwrap();
        assert_eq!(after_over.func, "main");

        // Continue vai até o breakpoint na linha 10 (`x += 2`)
        let at_bp = dbg.continue_to_breakpoint().unwrap();
        assert_eq!(at_bp.line_1based, 10);
        assert_eq!(
            at_bp.locals.iter().find(|l| l.name == "x").unwrap().value,
            "12"
        );
    }

    #[test]
    fn missoes_conserte_o_codigo_validam_solucoes_de_referencia() {
        assert_eq!(TROUBLESHOOTING_QUESTS.len(), 8);
        for q in TROUBLESHOOTING_QUESTS {
            assert!(evaluate_quest(&q, q.broken_code).is_err());
            let eval = evaluate_quest(&q, q.fixed_reference)
                .unwrap_or_else(|e| panic!("referência de {} falhou: {e}", q.id));
            assert!(eval.golf_bonus, "referência de {} deveria bater par", q.id);
        }
    }

    #[test]
    fn playground_export_e_ecossistemas_modernos_funcionam() {
        let code = "fn main() { println!(\"Olá RustLab!\"); }";
        let url = build_playground_url(code);
        assert!(url.starts_with(
            "https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&code="
        ));
        assert!(url.contains("%20"));

        let sols = community_solutions_for("soma iter vec");
        assert_eq!(sols.len(), 3);
        assert_eq!(MODERN_ECOSYSTEMS.len(), 3);
        for eco in MODERN_ECOSYSTEMS {
            assert!(eco.template_code.contains("fn main()"));
            assert!(eco.template_code.contains("#[cfg(test)]"));
        }

        let dir = std::env::temp_dir().join(format!("rustlab-share-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let exported = export_project_share_card(&dir, code).unwrap();
        let text = std::fs::read_to_string(&exported).unwrap();
        assert!(text.contains("https://play.rust-lang.org/"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
