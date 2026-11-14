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
    MapaLifetimes,
    LayoutMemoria,
    DesacucararMacros,
    PoliglotaIntuicao,
}

impl LabToolTab {
    pub const ALL: [Self; 10] = [
        Self::RaioXMemoria,
        Self::LenteCompilador,
        Self::DebuggerVisual,
        Self::ConserteOCodigo,
        Self::PlaygroundComunidade,
        Self::EcossistemasModernos,
        Self::MapaLifetimes,
        Self::LayoutMemoria,
        Self::DesacucararMacros,
        Self::PoliglotaIntuicao,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::RaioXMemoria => 0,
            Self::LenteCompilador => 1,
            Self::DebuggerVisual => 2,
            Self::ConserteOCodigo => 3,
            Self::PlaygroundComunidade => 4,
            Self::EcossistemasModernos => 5,
            Self::MapaLifetimes => 6,
            Self::LayoutMemoria => 7,
            Self::DesacucararMacros => 8,
            Self::PoliglotaIntuicao => 9,
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
            '7' => Some(Self::MapaLifetimes),
            '8' => Some(Self::LayoutMemoria),
            '9' => Some(Self::DesacucararMacros),
            '0' => Some(Self::PoliglotaIntuicao),
            _ => None,
        }
    }

    pub const fn short_title(self) -> &'static str {
        match self {
            Self::RaioXMemoria => "1:RaioX",
            Self::LenteCompilador => "2:Lente",
            Self::DebuggerVisual => "3:Debug",
            Self::ConserteOCodigo => "4:Fix",
            Self::PlaygroundComunidade => "5:Play",
            Self::EcossistemasModernos => "6:Eco",
            Self::MapaLifetimes => "7:Life",
            Self::LayoutMemoria => "8:Size",
            Self::DesacucararMacros => "9:Macro",
            Self::PoliglotaIntuicao => "0:Polig",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::RaioXMemoria => "Raio-X de Ownership, Stack/Heap & Alocações",
            Self::LenteCompilador => "Lente Socrática do Compilador & Modo 10 Anos (ELI10)",
            Self::DebuggerVisual => "Debugger Visual & Nativo (rust-gdb / lldb)",
            Self::ConserteOCodigo => "Desafio Gamificado: Conserte o Código & Golfing",
            Self::PlaygroundComunidade => "Rust Playground, Exportação & Soluções da Comunidade",
            Self::EcossistemasModernos => {
                "Trilhas Modernas: WebAssembly · CLI Tokio · Backend Axum"
            }
            Self::MapaLifetimes => "Mapa Visual de Lifetimes ('a, 'b) & Escopos",
            Self::LayoutMemoria => "Raio-X de Tamanho (size_of / align_of), Padding & NPO",
            Self::DesacucararMacros => "Desaçucarador de Macros & Syntax Sugar",
            Self::PoliglotaIntuicao => "Dicionário Poliglota (Python/JS/C++) & Flashcards SRS",
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
    pub lifetime_gutter_enabled: bool,
    pub last_lifetimes: Option<LifetimeVisualizerReport>,
    pub last_layout: Option<MemoryLayoutReport>,
    pub last_desugar: Option<DesugarReport>,
    pub desugar_sel: usize,
    pub polyglot_mode_flashcards: bool,
    pub polyglot_sel: usize,
    pub polyglot_lang_filter: usize,
    pub flashcard_sel: usize,
    pub flashcard_feedback: Option<(bool, String)>,
    pub completed_flashcards: HashSet<&'static str>,
    pub native_debugger_status: Option<String>,
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
        self.last_lifetimes = Some(LifetimeVisualizerReport::analyze(code));
        self.last_layout = Some(MemoryLayoutReport::analyze(code));
        self.last_desugar = Some(DesugarReport::analyze(code));
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

// =========================================================================
// 7. MAPA VISUAL DE LIFETIMES ('a, 'b) COLORIDO & DETECÇÃO DE ESCOPOS
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifetimeKind {
    Owner,
    SharedBorrow,
    MutBorrow,
    ExplicitAnnotation,
}

impl LifetimeKind {
    pub const fn badge(self) -> &'static str {
        match self {
            Self::Owner => "DONO",
            Self::SharedBorrow => "&'a IMUT",
            Self::MutBorrow => "&'b mut",
            Self::ExplicitAnnotation => "ANOTAÇÃO 'a",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifetimeSpan {
    pub name: String,
    pub label: String,
    pub kind: LifetimeKind,
    pub target: Option<String>,
    pub birth_line: usize,
    pub end_line: usize,
    pub depth: usize,
    pub timeline_bar: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifetimeConflict {
    pub line: usize,
    pub error_code: &'static str,
    pub title: String,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LifetimeVisualizerReport {
    pub total_lines: usize,
    pub explicit_annotations: Vec<(usize, String)>,
    pub spans: Vec<LifetimeSpan>,
    pub conflicts: Vec<LifetimeConflict>,
}

impl LifetimeVisualizerReport {
    /// Analisa declarações, blocos `{ ... }`, empréstimos `&` / `&mut` e lifetimes explícitos.
    pub fn analyze(code: &str) -> Self {
        let raw_lines: Vec<&str> = code.lines().collect();
        let total_lines = raw_lines.len().max(1);
        let mut explicit_annotations = Vec::new();
        let mut spans: Vec<LifetimeSpan> = Vec::new();
        let mut conflicts: Vec<LifetimeConflict> = Vec::new();
        let mut depth = 0usize;
        let mut fn_return_ref_depth: Option<usize> = None;

        for (idx, raw) in raw_lines.iter().enumerate() {
            let line_no = idx + 1;
            let code_only = strip_line_comment(raw).trim();
            if code_only.is_empty() {
                continue;
            }

            for lt in extract_lifetime_tokens(code_only) {
                if !explicit_annotations
                    .iter()
                    .any(|(l, t)| *l == line_no && t == &lt)
                {
                    explicit_annotations.push((line_no, lt));
                }
            }

            if code_only.starts_with("fn ") && code_only.contains("->") && code_only.contains('&') {
                fn_return_ref_depth = Some(depth + usize::from(code_only.contains('{')));
            }

            // Fecha escopos que terminam nesta linha antes de abrir novos.
            let close_count = code_only.matches('}').count();
            for _ in 0..close_count {
                let closing_depth = depth;
                for sp in &mut spans {
                    if sp.depth >= closing_depth && sp.end_line == 0 {
                        sp.end_line = line_no;
                    }
                }
                if fn_return_ref_depth == Some(closing_depth) {
                    fn_return_ref_depth = None;
                }
                depth = depth.saturating_sub(1);
            }

            // Retorno de referência local na função atual.
            if (fn_return_ref_depth.is_some()
                || code_only.starts_with("return &")
                || code_only.starts_with('&'))
                && let Some(ret_target) = extract_returned_borrow_target(code_only)
                && spans.iter().any(|sp| {
                    sp.name == ret_target
                        && sp.kind == LifetimeKind::Owner
                        && sp.depth >= fn_return_ref_depth.unwrap_or(1)
                })
            {
                conflicts.push(LifetimeConflict {
                    line: line_no,
                    error_code: "E0515",
                    title: format!("Retorno de referência a valor local `{ret_target}`"),
                    explanation: format!(
                        "`{ret_target}` nasce dentro da função e é destruído ao fechar `}}` na saída; retorne o valor com posse (`String` / `T`) ou receba `&'a` por parâmetro."
                    ),
                });
            }

            // Reatribuição de referência já existente apontando para variável de bloco interno (`E0597`).
            if !code_only.starts_with("let ")
                && let Some((lhs, rhs)) = code_only.split_once('=')
                && !lhs.contains('!')
                && !rhs.starts_with('=')
            {
                let lhs_ident = extract_leading_ident(lhs.trim());
                let rhs_clean = rhs.trim().trim_end_matches(';');
                if !lhs_ident.is_empty()
                    && let Some(target_ident) = parse_borrow_target(rhs_clean)
                    && let Some(outer_ref) = spans.iter().rfind(|sp| sp.name == lhs_ident).cloned()
                    && let Some(inner_owner) = spans.iter().rfind(|sp| {
                        sp.name == target_ident
                            && sp.kind == LifetimeKind::Owner
                            && sp.depth > outer_ref.depth
                    })
                {
                    conflicts.push(LifetimeConflict {
                        line: line_no,
                        error_code: "E0597",
                        title: format!(
                            "`{target_ident}` (bloco L{}) vive menos que `{lhs_ident}` (escopo L{})",
                            inner_owner.birth_line, outer_ref.birth_line
                        ),
                        explanation: format!(
                            "`{target_ident}` será descartado no `}}` interno enquanto `{lhs_ident}` continua vivo no escopo externo."
                        ),
                    });
                }
            }

            // Declaração `let [mut] nome [: tipo] = rhs;`
            if let Some(after_let) = code_only.strip_prefix("let ") {
                let (lhs, rhs) = match after_let.split_once('=') {
                    Some((l, r)) => (l.trim(), r.trim().trim_end_matches(';').trim()),
                    None => (after_let.trim_end_matches(';').trim(), ""),
                };
                let (ident, _mutable, ty_opt) = parse_let_lhs(lhs);
                if !ident.is_empty() {
                    let is_mut_borrow = rhs.starts_with("&mut ");
                    let is_shared_borrow = !is_mut_borrow && rhs.starts_with('&');
                    let target = parse_borrow_target(rhs);
                    let last_use = find_last_mention_line(&raw_lines, idx, &ident, depth);

                    if let Some(tgt) = target.as_deref() {
                        check_active_borrow_overlap(
                            &spans,
                            &raw_lines,
                            line_no,
                            tgt,
                            is_mut_borrow,
                            &mut conflicts,
                        );
                    }

                    let kind = if is_mut_borrow {
                        LifetimeKind::MutBorrow
                    } else if is_shared_borrow
                        || ty_opt.as_deref().is_some_and(|t| t.contains('\''))
                    {
                        LifetimeKind::SharedBorrow
                    } else {
                        LifetimeKind::Owner
                    };
                    let label = match (kind, target.as_deref()) {
                        (LifetimeKind::MutBorrow, Some(t)) => {
                            format!("&mut de `{t}` (exclusivo)")
                        }
                        (LifetimeKind::SharedBorrow, Some(t)) => {
                            format!("& de `{t}` (compartilhado)")
                        }
                        _ => format!("Posse `{ident}` (stack/escopo)"),
                    };
                    spans.push(LifetimeSpan {
                        name: ident,
                        label,
                        kind,
                        target,
                        birth_line: line_no,
                        end_line: last_use.max(line_no),
                        depth,
                        timeline_bar: String::new(),
                    });
                }
            }

            // Verifica mutação direta enquanto existe borrow ainda usado abaixo.
            if !code_only.starts_with("let ")
                && let Some(mutated) = detect_mutated_target(code_only)
            {
                for b in spans.iter().filter(|sp| {
                    sp.target.as_deref() == Some(mutated.as_str())
                        && matches!(
                            sp.kind,
                            LifetimeKind::SharedBorrow | LifetimeKind::MutBorrow
                        )
                        && sp.birth_line < line_no
                }) {
                    let borrow_used_later = raw_lines[idx..]
                        .iter()
                        .skip(1)
                        .any(|later| contains_ident(strip_line_comment(later), &b.name));
                    if borrow_used_later {
                        conflicts.push(LifetimeConflict {
                            line: line_no,
                            error_code: "E0502/E0506",
                            title: format!(
                                "Mutação em `{mutated}` com empréstimo `{}` ativo até abaixo",
                                b.name
                            ),
                            explanation: format!(
                                "`{}` emprestou `{mutated}` em L{} e ainda será lido depois da linha L{line_no}.",
                                b.name, b.birth_line
                            ),
                        });
                        break;
                    }
                }
            }

            let open_count = code_only.matches('{').count();
            depth = depth.saturating_add(open_count);
        }

        for sp in &mut spans {
            if sp.end_line == 0 || sp.end_line < sp.birth_line {
                sp.end_line = total_lines;
            }
            sp.timeline_bar = render_timeline_bar(sp.birth_line, sp.end_line, total_lines, 18);
        }

        Self {
            total_lines,
            explicit_annotations,
            spans,
            conflicts,
        }
    }

    pub fn gutter_marker_for_line(&self, line_1based: usize) -> Option<(&'static str, bool, bool)> {
        if self.conflicts.iter().any(|c| c.line == line_1based) {
            return Some(("✘", true, false));
        }
        if let Some(sp) = self
            .spans
            .iter()
            .find(|s| s.birth_line == line_1based && s.kind != LifetimeKind::Owner)
        {
            return Some(("┣", false, sp.kind == LifetimeKind::MutBorrow));
        }
        if let Some(sp) = self.spans.iter().find(|s| {
            s.kind != LifetimeKind::Owner && line_1based > s.birth_line && line_1based <= s.end_line
        }) {
            return Some(("┃", false, sp.kind == LifetimeKind::MutBorrow));
        }
        None
    }

    pub fn summary_lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        out.push(format!(
            "Lifetimes explícitos detectados: {}",
            if self.explicit_annotations.is_empty() {
                "nenhum (elision automática do compilador)".to_string()
            } else {
                self.explicit_annotations
                    .iter()
                    .map(|(l, t)| format!("{t} @ L{l}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ));
        if self.conflicts.is_empty() {
            out.push("Status do Borrow Checker: ✓ Nenhum conflito estrutural de tempo de vida detectado.".to_string());
        } else {
            out.push(format!("Conflitos de Lifetime ({}):", self.conflicts.len()));
            for c in &self.conflicts {
                out.push(format!("  ✘ L{} [{}] {}", c.line, c.error_code, c.title));
                out.push(format!("    {}", c.explanation));
            }
        }
        out.push(String::new());
        out.push("Linha do Tempo dos Escopos (Origem ──▶ Fim):".to_string());
        if self.spans.is_empty() {
            out.push("  (nenhum binding/empréstimo mapeado neste arquivo)".to_string());
        } else {
            for sp in &self.spans {
                out.push(format!(
                    "  [{}] {:<10} {}  L{}-L{} · {}",
                    sp.kind.badge(),
                    sp.name,
                    sp.timeline_bar,
                    sp.birth_line,
                    sp.end_line,
                    sp.label
                ));
            }
        }
        out
    }
}

fn extract_lifetime_tokens(line: &str) -> Vec<String> {
    let mut found = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\'' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            let start = i;
            let mut end = i + 2;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            // Ignora literais de char como `'a'`.
            if bytes.get(end) != Some(&b'\'') {
                let tok = line[start..end].to_string();
                if !found.contains(&tok) {
                    found.push(tok);
                }
            }
            i = end;
            continue;
        }
        i += 1;
    }
    found
}

fn extract_returned_borrow_target(code_only: &str) -> Option<String> {
    let rest = if let Some(r) = code_only.strip_prefix("return ") {
        r.trim()
    } else if code_only.starts_with('&') {
        code_only
    } else {
        return None;
    };
    let ident = parse_borrow_target(rest.trim_end_matches(';'))?;
    Some(ident)
}

fn parse_borrow_target(rhs: &str) -> Option<String> {
    let trimmed = rhs.trim();
    let after_amp = trimmed
        .strip_prefix("&mut ")
        .or_else(|| trimmed.strip_prefix('&'))?
        .trim();
    let ident = extract_leading_ident(after_amp);
    if ident.is_empty() || ident == "self" {
        None
    } else {
        Some(ident.to_string())
    }
}

fn find_last_mention_line(
    raw_lines: &[&str],
    start_idx: usize,
    ident: &str,
    decl_depth: usize,
) -> usize {
    let mut depth = decl_depth;
    let mut last = start_idx + 1;
    for (offset, raw) in raw_lines.iter().enumerate().skip(start_idx) {
        let line_no = offset + 1;
        let code = strip_line_comment(raw).trim();
        if offset > start_idx {
            let closes = code.matches('}').count();
            if closes > 0 && depth <= decl_depth {
                return last.max(line_no);
            }
            if contains_ident(code, ident) {
                last = line_no;
            }
            depth = depth
                .saturating_sub(closes)
                .saturating_add(code.matches('{').count());
        } else {
            depth = depth.saturating_add(code.matches('{').count());
        }
    }
    last
}

fn check_active_borrow_overlap(
    spans: &[LifetimeSpan],
    raw_lines: &[&str],
    line_no: usize,
    target: &str,
    new_is_mut: bool,
    conflicts: &mut Vec<LifetimeConflict>,
) {
    let idx = line_no.saturating_sub(1);
    for prev in spans.iter().filter(|s| {
        s.target.as_deref() == Some(target)
            && matches!(s.kind, LifetimeKind::SharedBorrow | LifetimeKind::MutBorrow)
            && s.birth_line < line_no
    }) {
        let prev_still_used = raw_lines[idx..]
            .iter()
            .skip(1)
            .any(|later| contains_ident(strip_line_comment(later), &prev.name));
        if !prev_still_used {
            continue;
        }
        match (prev.kind, new_is_mut) {
            (LifetimeKind::MutBorrow, true) => conflicts.push(LifetimeConflict {
                line: line_no,
                error_code: "E0499",
                title: format!(
                    "Segundo `&mut` em `{target}` enquanto `{}` (L{}) segue ativo",
                    prev.name, prev.birth_line
                ),
                explanation: "Em Rust só pode haver UMA referência `&mut` ativa ao mesmo dado por vez."
                    .to_string(),
            }),
            (LifetimeKind::SharedBorrow, true) | (LifetimeKind::MutBorrow, false) => {
                conflicts.push(LifetimeConflict {
                    line: line_no,
                    error_code: "E0502",
                    title: format!(
                        "Conflito `&` vs `&mut` em `{target}` com `{}` (L{}) ainda em uso",
                        prev.name, prev.birth_line
                    ),
                    explanation: "Não é possível misturar empréstimo compartilhado `&` com mutável `&mut` simultaneamente."
                        .to_string(),
                })
            }
            _ => {}
        }
    }
}

fn contains_ident(haystack: &str, ident: &str) -> bool {
    if ident.is_empty() {
        return false;
    }
    haystack
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .any(|tok| tok == ident)
}

fn detect_mutated_target(code_only: &str) -> Option<String> {
    if let Some((lhs, rhs)) = code_only.split_once('=')
        && !lhs.contains('!')
        && !rhs.starts_with('=')
    {
        let clean = lhs.trim_end_matches(|c: char| "+-*/|&^ ".contains(c));
        let ident = extract_leading_ident(clean);
        if !ident.is_empty() && clean == ident {
            return Some(ident.to_string());
        }
    }
    for method in [
        ".push(",
        ".push_str(",
        ".clear(",
        ".insert(",
        ".pop(",
        ".sort(",
    ] {
        if let Some((recv, _)) = code_only.split_once(method) {
            let ident = extract_leading_ident(recv);
            if !ident.is_empty() {
                return Some(ident.to_string());
            }
        }
    }
    None
}

fn render_timeline_bar(birth: usize, end: usize, total: usize, width: usize) -> String {
    let denom = total.saturating_sub(1).max(1);
    let start_col = (birth.saturating_sub(1) * (width - 1)) / denom;
    let end_col = ((end.saturating_sub(1) * (width - 1)) / denom).max(start_col);
    let mut bar = String::with_capacity(width + 2);
    bar.push('[');
    for col in 0..width {
        if col == start_col && col == end_col {
            bar.push('◆');
        } else if col == start_col {
            bar.push('┣');
        } else if col == end_col {
            bar.push('┫');
        } else if col > start_col && col < end_col {
            bar.push('━');
        } else {
            bar.push('·');
        }
    }
    bar.push(']');
    bar
}

// =========================================================================
// 8. RAIO-X DE TAMANHO E LAYOUT NA STACK (size_of / align_of / NPO)
// =========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldLayoutInfo {
    pub name: String,
    pub ty: String,
    pub size: usize,
    pub align: usize,
    pub offset: usize,
    pub padding_before: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructLayoutAnalysis {
    pub name: String,
    pub is_repr_c: bool,
    pub declared_size: usize,
    pub declared_align: usize,
    pub total_padding: usize,
    pub optimal_size: usize,
    pub bytes_saved: usize,
    pub fields: Vec<FieldLayoutInfo>,
    pub reordered_preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardLayoutFact {
    pub type_expr: &'static str,
    pub size_bytes: usize,
    pub align_bytes: usize,
    pub note: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MemoryLayoutReport {
    pub user_types: Vec<StructLayoutAnalysis>,
    pub npo_facts: Vec<StandardLayoutFact>,
}

impl MemoryLayoutReport {
    pub fn analyze(code: &str) -> Self {
        let mut user_types: Vec<StructLayoutAnalysis> = Vec::new();
        let mut known_structs: HashMap<String, (usize, usize)> = HashMap::new();
        let lines: Vec<&str> = code.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let trimmed = strip_line_comment(lines[i]).trim();
            let is_repr_c = i > 0 && strip_line_comment(lines[i - 1]).contains("repr(C)");
            let after_vis = trimmed.strip_prefix("pub ").unwrap_or(trimmed).trim();
            if let Some(rest) = after_vis.strip_prefix("struct ")
                && let Some((name_raw, tail)) = rest.split_once('{')
            {
                let struct_name = extract_leading_ident(name_raw).to_string();
                let mut body = tail.to_string();
                while !body.contains('}') && i + 1 < lines.len() {
                    i += 1;
                    body.push('\n');
                    body.push_str(strip_line_comment(lines[i]));
                }
                let body_inside = body.split('}').next().unwrap_or("");
                let raw_fields = parse_struct_fields(body_inside);
                if !struct_name.is_empty() && !raw_fields.is_empty() {
                    let analysis =
                        compute_struct_layout(&struct_name, is_repr_c, &raw_fields, &known_structs);
                    let effective_size = if is_repr_c {
                        analysis.declared_size
                    } else {
                        analysis.optimal_size
                    };
                    known_structs.insert(struct_name, (effective_size, analysis.declared_align));
                    user_types.push(analysis);
                }
            }
            i += 1;
        }

        let npo_facts = vec![
            StandardLayoutFact {
                type_expr: "&T  /  Option<&T>",
                size_bytes: 8,
                align_bytes: 8,
                note: "NPO (Null Pointer Optimization): ponteiro não-nulo usa endereço 0 como `None` sem gastar 1 byte extra!",
            },
            StandardLayoutFact {
                type_expr: "Box<T>  /  Option<Box<T>>",
                size_bytes: 8,
                align_bytes: 8,
                note: "Mesmo tamanho (8 bytes): `Box` nunca é nulo em memória válida.",
            },
            StandardLayoutFact {
                type_expr: "u64  (8B)  →  Option<u64>",
                size_bytes: 16,
                align_bytes: 8,
                note: "Precisa de tag discriminante + 7 bytes de padding, dobrando de 8B para 16B.",
            },
            StandardLayoutFact {
                type_expr: "&str  (fat pointer)",
                size_bytes: 16,
                align_bytes: 8,
                note: "Armazena apenas ponteiro (8B) + tamanho `len` (8B) na Stack, zero alocação Heap.",
            },
            StandardLayoutFact {
                type_expr: "String  /  Vec<T>",
                size_bytes: 24,
                align_bytes: 8,
                note: "Cabeçalho na Stack de 24 bytes (ptr + len + capacity) apontando para o buffer na Heap.",
            },
        ];

        Self {
            user_types,
            npo_facts,
        }
    }
}

fn parse_struct_fields(body: &str) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    for entry in body.split(',') {
        let clean = entry.trim();
        if clean.is_empty() {
            continue;
        }
        let no_vis = clean.strip_prefix("pub ").unwrap_or(clean).trim();
        if let Some((name, ty)) = no_vis.split_once(':') {
            let fname = extract_leading_ident(name);
            let fty = ty.trim();
            if !fname.is_empty() && !fty.is_empty() {
                fields.push((fname.to_string(), fty.to_string()));
            }
        }
    }
    fields
}

fn compute_struct_layout(
    name: &str,
    is_repr_c: bool,
    raw_fields: &[(String, String)],
    known: &HashMap<String, (usize, usize)>,
) -> StructLayoutAnalysis {
    let typed: Vec<(String, String, usize, usize)> = raw_fields
        .iter()
        .map(|(n, ty)| {
            let (sz, al) = type_size_align(ty, known);
            (n.clone(), ty.clone(), sz, al)
        })
        .collect();

    let (declared_size, declared_align, total_padding, fields) = layout_sequence(&typed);

    let mut sorted = typed.clone();
    sorted.sort_by(|a, b| b.3.cmp(&a.3).then_with(|| b.2.cmp(&a.2)));
    let (optimal_size, _, _, _) = layout_sequence(&sorted);
    let bytes_saved = declared_size.saturating_sub(optimal_size);

    let mut reordered_preview = format!("struct {name} {{\n");
    for (fnm, fty, _, _) in &sorted {
        reordered_preview.push_str(&format!("    {fnm}: {fty},\n"));
    }
    reordered_preview.push('}');

    StructLayoutAnalysis {
        name: name.to_string(),
        is_repr_c,
        declared_size,
        declared_align,
        total_padding,
        optimal_size,
        bytes_saved,
        fields,
        reordered_preview,
    }
}

fn layout_sequence(
    items: &[(String, String, usize, usize)],
) -> (usize, usize, usize, Vec<FieldLayoutInfo>) {
    let mut offset = 0usize;
    let mut max_align = 1usize;
    let mut padding = 0usize;
    let mut fields = Vec::with_capacity(items.len());
    for (name, ty, size, align) in items {
        let a = (*align).max(1);
        max_align = max_align.max(a);
        let aligned = round_up(offset, a);
        let pad = aligned.saturating_sub(offset);
        padding += pad;
        fields.push(FieldLayoutInfo {
            name: name.clone(),
            ty: ty.clone(),
            size: *size,
            align: a,
            offset: aligned,
            padding_before: pad,
        });
        offset = aligned + size;
    }
    let final_size = round_up(offset, max_align);
    padding += final_size.saturating_sub(offset);
    (final_size, max_align, padding, fields)
}

const fn round_up(value: usize, align: usize) -> usize {
    if align <= 1 {
        return value;
    }
    let rem = value % align;
    if rem == 0 {
        value
    } else {
        value + (align - rem)
    }
}

pub fn type_size_align(ty: &str, known: &HashMap<String, (usize, usize)>) -> (usize, usize) {
    let t = ty.trim();
    if let Some(&pair) = known.get(t) {
        return pair;
    }
    if t == "&str" || t.starts_with("&[") || t.contains(" str") {
        return (16, 8);
    }
    if t.starts_with('&') || t.starts_with("*const ") || t.starts_with("*mut ") {
        return (8, 8);
    }
    if let Some(inner) = t.strip_prefix("Option<").and_then(|s| s.strip_suffix('>')) {
        let inner = inner.trim();
        if inner.starts_with('&')
            || inner.starts_with("Box<")
            || inner.starts_with("NonNull<")
            || inner == "bool"
        {
            return type_size_align(inner, known);
        }
        let (isz, ial) = type_size_align(inner, known);
        let tag_plus_val = round_up(1, ial) + isz;
        return (round_up(tag_plus_val, ial), ial);
    }
    if t.starts_with("Box<")
        || t.starts_with("NonNull<")
        || t.starts_with("Arc<")
        || t.starts_with("Rc<")
    {
        return (8, 8);
    }
    if t == "String" || t.starts_with("Vec<") {
        return (24, 8);
    }
    match t {
        "()" => (0, 1),
        "bool" | "u8" | "i8" => (1, 1),
        "u16" | "i16" => (2, 2),
        "u32" | "i32" | "f32" | "char" => (4, 4),
        "u64" | "i64" | "f64" | "usize" | "isize" => (8, 8),
        "u128" | "i128" => (16, 16),
        _ => (8, 8),
    }
}

// =========================================================================
// 9. DESAÇUCARADOR DE MACROS & SYNTAX SUGAR
// =========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesugarItem {
    pub id: &'static str,
    pub title: &'static str,
    pub line_1based: Option<usize>,
    pub sugar_snippet: String,
    pub expanded_rust: String,
    pub explanation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DesugarReport {
    pub detected_from_editor: bool,
    pub items: Vec<DesugarItem>,
}

impl DesugarReport {
    pub fn analyze(code: &str) -> Self {
        let mut items = Vec::new();
        for (idx, raw) in code.lines().enumerate() {
            let line_no = idx + 1;
            let t = strip_line_comment(raw).trim();
            if t.is_empty() {
                continue;
            }
            if (t.contains("println!(") || t.contains("format!("))
                && !items.iter().any(|i: &DesugarItem| i.id == "macro-print")
            {
                items.push(DesugarItem {
                    id: "macro-print",
                    title: "Macro println! / format!",
                    line_1based: Some(line_no),
                    sugar_snippet: t.to_string(),
                    expanded_rust: "{\n    ::std::io::_print(::core::format_args_nl!(\"...\"));\n}".to_string(),
                    explanation: "Macros terminadas em `!` rodam em tempo de compilação: `println!` valida a string de formato na compilação e gera chamadas diretas a `format_args!` sem alocar `String`!",
                });
            }
            if t.contains("vec![") && !items.iter().any(|i| i.id == "macro-vec") {
                items.push(DesugarItem {
                    id: "macro-vec",
                    title: "Macro vec![...]",
                    line_1based: Some(line_no),
                    sugar_snippet: t.to_string(),
                    expanded_rust: "<[_]>::into_vec(\n    #[rustc_box]\n    ::alloc::boxed::Box::new([...])\n)".to_string(),
                    explanation: "`vec![a, b, c]` não é mágico: aloca um array exato na Heap em um único bloco via `Box<[T]>` e converte para `Vec<T>` sem realocar `push` por `push`.",
                });
            }
            if contains_question_mark_op(t) && !items.iter().any(|i| i.id == "sugar-qmark") {
                items.push(DesugarItem {
                    id: "sugar-qmark",
                    title: "Operador `?` (Try trait)",
                    line_1based: Some(line_no),
                    sugar_snippet: t.to_string(),
                    expanded_rust: "match Try::branch(expr) {\n    ControlFlow::Continue(val) => val,\n    ControlFlow::Break(err) => return FromResidual::from_residual(err),\n}".to_string(),
                    explanation: "`?` desaçucara em um `match`: se deu `Ok(v)`/`Some(v)`, extrai `v`; se falhou, converte o erro com `From::from` e retorna antecipadamente.",
                });
            }
            if t.starts_with("for ")
                && t.contains(" in ")
                && !items.iter().any(|i| i.id == "sugar-for")
            {
                items.push(DesugarItem {
                    id: "sugar-for",
                    title: "Laço `for x in colecao`",
                    line_1based: Some(line_no),
                    sugar_snippet: t.to_string(),
                    expanded_rust: "let mut __iter = ::core::iter::IntoIterator::into_iter(colecao);\nloop {\n    match ::core::iter::Iterator::next(&mut __iter) {\n        Some(x) => { /* corpo */ }\n        None => break,\n    }\n}".to_string(),
                    explanation: "Todo `for` em Rust vira um `loop` chamando `Iterator::next(&mut iter)` sobre qualquer tipo que implemente `IntoIterator`.",
                });
            }
            if t.starts_with("#[derive(") && !items.iter().any(|i| i.id == "attr-derive") {
                items.push(DesugarItem {
                    id: "attr-derive",
                    title: "Macro de Atributo #[derive(...)]",
                    line_1based: Some(line_no),
                    sugar_snippet: t.to_string(),
                    expanded_rust: "impl ::core::fmt::Debug for SuaStruct {\n    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {\n        f.debug_struct(\"SuaStruct\") /* campos... */ .finish()\n    }\n}".to_string(),
                    explanation: "`#[derive(...)]` injeta implementações padrão automáticas dos traits pedidos no mesmo arquivo em tempo de compilação.",
                });
            }
            if t.starts_with("if let ") && !items.iter().any(|i| i.id == "sugar-iflet") {
                items.push(DesugarItem {
                    id: "sugar-iflet",
                    title: "Syntax Sugar `if let`",
                    line_1based: Some(line_no),
                    sugar_snippet: t.to_string(),
                    expanded_rust: "match valor {\n    Padrao(x) => { /* ramo then */ }\n    _ => {}\n}".to_string(),
                    explanation: "`if let` é literalmente um `match` que trata apenas uma variante específica e descarta todo o resto com `_ => {}`.",
                });
            }
            if (t.contains("async fn ") || t.contains(".await"))
                && !items.iter().any(|i| i.id == "sugar-async")
            {
                items.push(DesugarItem {
                    id: "sugar-async",
                    title: "Assíncrono `async fn` / `.await`",
                    line_1based: Some(line_no),
                    sugar_snippet: t.to_string(),
                    expanded_rust: "fn minha_fn() -> impl ::core::future::Future<Output = T> {\n    enum __Estado { Inicio, Aguardando(SubFuture), Concluido }\n    /* máquina de estados poll(&mut self, cx) */\n}".to_string(),
                    explanation: "`async fn` NÃO cria thread: o compilador gera uma máquina de estados compacta na memória que avança quando o runtime chama `.poll()`.",
                });
            }
        }

        let detected_from_editor = !items.is_empty();
        if items.len() < 2 {
            for fallback in classic_desugar_examples() {
                if !items.iter().any(|i| i.id == fallback.id) {
                    items.push(fallback);
                }
            }
        }

        Self {
            detected_from_editor,
            items,
        }
    }
}

fn contains_question_mark_op(line: &str) -> bool {
    let mut in_quotes = false;
    let mut prev = '\0';
    for ch in line.chars() {
        if ch == '"' && prev != '\\' {
            in_quotes = !in_quotes;
        } else if ch == '?' && !in_quotes {
            return true;
        }
        prev = ch;
    }
    false
}

fn classic_desugar_examples() -> Vec<DesugarItem> {
    vec![
        DesugarItem {
            id: "macro-vec",
            title: "Macro vec![1, 2, 3] (Exemplo clássico)",
            line_1based: None,
            sugar_snippet: "let v = vec![1, 2, 3];".to_string(),
            expanded_rust: "let v = <[_]>::into_vec(::alloc::boxed::Box::new([1, 2, 3]));".to_string(),
            explanation: "Cria um array na Heap com tamanho exato e entrega um `Vec` pronto sem realocações.",
        },
        DesugarItem {
            id: "sugar-for",
            title: "Laço `for item in lista` (Exemplo clássico)",
            line_1based: None,
            sugar_snippet: "for item in &lista { println!(\"{item}\"); }".to_string(),
            expanded_rust: "let mut __it = (&lista).into_iter();\nwhile let Some(item) = __it.next() {\n    ::std::io::_print(format_args_nl!(\"{item}\"));\n}".to_string(),
            explanation: "Açúcar sintático sobre o trait `Iterator::next` — por isso funciona igualmente para arrays, ranges `0..10` e coleções.",
        },
        DesugarItem {
            id: "sugar-qmark",
            title: "Operador `?` (Exemplo clássico)",
            line_1based: None,
            sugar_snippet: "let n: i32 = texto.parse()?;".to_string(),
            expanded_rust: "let n: i32 = match texto.parse() {\n    Ok(v) => v,\n    Err(e) => return Err(From::from(e)),\n};".to_string(),
            explanation: "Substitui blocos repetitivos de tratamento de erro mantendo propagação totalmente explícita no tipo de retorno.",
        },
    ]
}

// =========================================================================
// 10. DICIONÁRIO POLIGLOTA (PYTHON/JS/C++) & FLASHCARDS DE INTUIÇÃO SRS
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PolyglotEntry {
    pub id: &'static str,
    pub concept: &'static str,
    pub from_python: &'static str,
    pub from_js: &'static str,
    pub from_cpp: &'static str,
    pub idiomatic_rust: &'static str,
    pub why_rust_differs: &'static str,
    pub rust_code: &'static str,
}

pub const POLYGLOT_DICTIONARY: [PolyglotEntry; 6] = [
    PolyglotEntry {
        id: "POLY-01",
        concept: "Ausência de Valor (Null / None / undefined)",
        from_python: "val = None  # só falha em runtime com AttributeError",
        from_js: "const val = null ?? 'default';  // undefined vs null",
        from_cpp: "std::optional<T> / nullptr  // desreferenciar nulo é UB",
        idiomatic_rust: "Option<T> (Some(x) | None) — NullPointer não compila!",
        why_rust_differs: "Em Rust não existe `null`: você é obrigado pelo tipo `Option<T>` a tratar `None` com `match`, `if let`, `.unwrap_or()` ou `?`.",
        rust_code: "pub fn encontrar_porta(env: Option<&str>) -> u16 {\n    env.and_then(|s| s.parse().ok()).unwrap_or(8080)\n}\n\nfn main() {\n    println!(\"Porta default: {}\", encontrar_porta(None));\n    println!(\"Porta config : {}\", encontrar_porta(Some(\"3000\")));\n}\n\n#[cfg(test)]\nmod tests {\n    use super::encontrar_porta;\n    #[test]\n    fn trata_ausencia_sem_null() {\n        assert_eq!(encontrar_porta(None), 8080);\n        assert_eq!(encontrar_porta(Some(\"9000\")), 9000);\n    }\n}\n",
    },
    PolyglotEntry {
        id: "POLY-02",
        concept: "Tratamento de Erros (try/catch -> Result + ?)",
        from_python: "try: x = int(s)\nexcept ValueError as e: ...",
        from_js: "try { JSON.parse(s) } catch (err) { ... }",
        from_cpp: "throw std::runtime_error(...) / códigos negativos",
        idiomatic_rust: "Result<T, E> + operador `?` — erros visíveis na assinatura!",
        why_rust_differs: "Exceções invisíveis pulam fluxos escondidos; `Result` força caminhos de falha explícitos sem custo de stack unwinding.",
        rust_code: "pub fn dividir_seguro(a: i32, b: i32) -> Result<i32, &'static str> {\n    if b == 0 { return Err(\"divisão por zero\"); }\n    Ok(a / b)\n}\n\npub fn calcular(x: i32) -> Result<i32, &'static str> {\n    let metade = dividir_seguro(x, 2)?;\n    Ok(metade + 1)\n}\n\nfn main() {\n    println!(\"{:?}\", calcular(10));\n}\n\n#[cfg(test)]\nmod tests {\n    use super::{calcular, dividir_seguro};\n    #[test]\n    fn propaga_erro_sem_excecao() {\n        assert_eq!(calcular(10), Ok(6));\n        assert!(dividir_seguro(5, 0).is_err());\n    }\n}\n",
    },
    PolyglotEntry {
        id: "POLY-03",
        concept: "Transformação de Listas (map / filter / comprehension)",
        from_python: "[x * 2 for x in nums if x > 0]  # aloca nova lista",
        from_js: "nums.filter(x => x > 0).map(x => x * 2)  // cria 2 arrays!",
        from_cpp: "std::ranges::views::filter | views::transform",
        idiomatic_rust: "slice.iter().copied().filter(...).map(...).sum()",
        why_rust_differs: "Iterators em Rust são lazy e monomorfizados: `.filter().map().sum()` compila em um único loop de máquina com ZERO alocação intermediária.",
        rust_code: "pub fn soma_positivos_dobrados(nums: &[i32]) -> i32 {\n    nums.iter().copied().filter(|&x| x > 0).map(|x| x * 2).sum()\n}\n\nfn main() {\n    println!(\"Soma: {}\", soma_positivos_dobrados(&[-3, 2, 5]));\n}\n\n#[cfg(test)]\nmod tests {\n    use super::soma_positivos_dobrados;\n    #[test]\n    fn zero_alloc_iterator() {\n        assert_eq!(soma_positivos_dobrados(&[-3, 2, 5]), 14);\n    }\n}\n",
    },
    PolyglotEntry {
        id: "POLY-04",
        concept: "Classes & Herança -> Struct + Trait + Composição",
        from_python: "class Bot(Entidade): def agir(self): ...",
        from_js: "class Bot extends Entidade { agir() {} }",
        from_cpp: "class Bot : public Entidade { virtual void agir(); };",
        idiomatic_rust: "struct Dados + trait Comportamento + impl Trait for Dados",
        why_rust_differs: "Sem herança de implementação profunda; composição e traits dão despacho estático (`impl Trait`) zero-cost ou dinâmico explícito (`&dyn Trait`).",
        rust_code: "pub trait Desenhavel { fn nome(&self) -> &'static str; }\npub struct Jogador;\nimpl Desenhavel for Jogador { fn nome(&self) -> &'static str { \"Heroi\" } }\n\npub fn renderizar(item: &impl Desenhavel) -> String {\n    format!(\"[Sprite: {}]\", item.nome())\n}\n\nfn main() { println!(\"{}\", renderizar(&Jogador)); }\n\n#[cfg(test)]\nmod tests {\n    use super::{Jogador, renderizar};\n    #[test]\n    fn despacho_por_trait() { assert_eq!(renderizar(&Jogador), \"[Sprite: Heroi]\"); }\n}\n",
    },
    PolyglotEntry {
        id: "POLY-05",
        concept: "Memória & Ponteiros (GC / Smart Pointers -> Ownership)",
        from_python: "GC + refcount automático em tudo (GIL)",
        from_js: "Garbage Collector gerenciado pela engine V8",
        from_cpp: "std::unique_ptr (move) / std::shared_ptr / raw ptrs",
        idiomatic_rust: "Posse única (T), empréstimo (&T / &mut T) e Box/Arc",
        why_rust_differs: "Sem pausa de Garbage Collector e sem dangling pointers: o compilador insere `drop()` deterministicamente onde o dono sai de escopo.",
        rust_code: "pub fn contar_bytes(texto: &str) -> usize {\n    texto.len()\n}\n\nfn main() {\n    let dono = String::from(\"Ownership\");\n    let tamanho = contar_bytes(&dono);\n    println!(\"{dono} tem {tamanho} bytes e continua válido!\");\n}\n\n#[cfg(test)]\nmod tests {\n    use super::contar_bytes;\n    #[test]\n    fn empresta_sem_clonar() { assert_eq!(contar_bytes(\"Rust\"), 4); }\n}\n",
    },
    PolyglotEntry {
        id: "POLY-06",
        concept: "Concorrência Segura (Threads / Promises -> Send + Sync)",
        from_python: "threading limitado por GIL / asyncio",
        from_js: "Event loop single-thread com Promise.all",
        from_cpp: "std::thread + std::mutex (data race possível!)",
        idiomatic_rust: "Mutex<T> envolve o dado + Send/Sync impedem data races!",
        why_rust_differs: "Em C++ o mutex fica ao lado do dado; em Rust o `Mutex<T>` GUARDA o dado dentro dele — é impossível acessar `T` sem trancar antes.",
        rust_code: "use std::sync::{Arc, Mutex};\nuse std::thread;\n\npub fn somar_em_threads() -> i32 {\n    let total = Arc::new(Mutex::new(0));\n    let mut handles = Vec::new();\n    for v in [10, 20, 12] {\n        let t = Arc::clone(&total);\n        handles.push(thread::spawn(move || { *t.lock().unwrap() += v; }));\n    }\n    for h in handles { h.join().unwrap(); }\n    let res = *total.lock().unwrap(); res\n}\n\nfn main() { println!(\"Total concorrente: {}\", somar_em_threads()); }\n\n#[cfg(test)]\nmod tests {\n    use super::somar_em_threads;\n    #[test]\n    fn sem_data_race() { assert_eq!(somar_em_threads(), 42); }\n}\n",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntuitionFlashcard {
    pub id: &'static str,
    pub title: &'static str,
    pub topic: &'static str,
    pub snippet: &'static str,
    pub compiles: bool,
    pub error_code: &'static str,
    pub explanation: &'static str,
}

pub const INTUITION_FLASHCARDS: [IntuitionFlashcard; 8] = [
    IntuitionFlashcard {
        id: "FLASH-01",
        title: "Move de String seguido de leitura",
        topic: "Ownership",
        snippet: "let a = String::from(\"oi\");\nlet b = a;\nprintln!(\"{a}\");",
        compiles: false,
        error_code: "E0382",
        explanation: "`String` não implementa `Copy`; `let b = a` transfere a posse para `b`, invalidando `a`.",
    },
    IntuitionFlashcard {
        id: "FLASH-02",
        title: "Copy de i32 após atribuição",
        topic: "Stack & Copy",
        snippet: "let x = 42;\nlet y = x;\nprintln!(\"{x} + {y}\");",
        compiles: true,
        error_code: "COMPILA",
        explanation: "`i32` vive inteiramente na Stack e implementa `Copy`; ambos `x` e `y` permanecem válidos.",
    },
    IntuitionFlashcard {
        id: "FLASH-03",
        title: "NLL: último uso de `&v` antes de `v.push`",
        topic: "Non-Lexical Lifetimes",
        snippet: "let mut v = vec![1, 2];\nlet primeiro = v[0];\nprintln!(\"{primeiro}\");\nv.push(3);",
        compiles: true,
        error_code: "COMPILA",
        explanation: "`v[0]` copia um `i32` e, mesmo se fosse `let r = &v[0]`, quando `r` não é mais usado após o `println!`, o empréstimo termina antes de `v.push(3)` (NLL)!",
    },
    IntuitionFlashcard {
        id: "FLASH-04",
        title: "Dois `&mut` ativos ao mesmo tempo",
        topic: "Exclusividade",
        snippet: "let mut x = 5;\nlet r1 = &mut x;\nlet r2 = &mut x;\nprintln!(\"{r1} {r2}\");",
        compiles: false,
        error_code: "E0499",
        explanation: "Ambos `r1` e `r2` continuam vivos até o `println!`: não é permitido ter dois empréstimos mutáveis simultâneos.",
    },
    IntuitionFlashcard {
        id: "FLASH-05",
        title: "Retornar `&String` criado dentro da função",
        topic: "Dangling Reference",
        snippet: "fn criar() -> &String {\n    let s = String::from(\"tmp\");\n    &s\n}",
        compiles: false,
        error_code: "E0106/E0515",
        explanation: "`s` é destruído ao final de `criar()`, deixando `&s` apontando para memória inválida. Retorne `String` direto!",
    },
    IntuitionFlashcard {
        id: "FLASH-06",
        title: "Deref Coercion: passar `&String` para `&str`",
        topic: "Slices & Deref",
        snippet: "fn saudar(nome: &str) { println!(\"{nome}\"); }\nlet dono = String::from(\"Ferris\");\nsaudar(&dono);",
        compiles: true,
        error_code: "COMPILA",
        explanation: "`String` implementa `Deref<Target = str>`, permitindo passar `&String` onde `&str` é esperado sem clonar!",
    },
    IntuitionFlashcard {
        id: "FLASH-07",
        title: "Modificar variável enquanto slice é lido depois",
        topic: "Borrow Conflict",
        snippet: "let mut s = String::from(\"rust\");\nlet slice = &s[..2];\ns.push('!');\nprintln!(\"{slice}\");",
        compiles: false,
        error_code: "E0502",
        explanation: "`s.push('!')` pode realocar o buffer da Heap, invalidando `slice` que ainda é usado na última linha.",
    },
    IntuitionFlashcard {
        id: "FLASH-08",
        title: "Multiple `&T` imutáveis simultâneos",
        topic: "Shared Borrowing",
        snippet: "let s = String::from(\"seguro\");\nlet r1 = &s;\nlet r2 = &s;\nprintln!(\"{r1} e {r2}\");",
        compiles: true,
        error_code: "COMPILA",
        explanation: "Várias referências compartilhadas `&T` simultâneas são 100% seguras porque ninguém modifica os dados.",
    },
];

pub fn evaluate_flashcard_answer(
    card: &IntuitionFlashcard,
    user_says_compiles: bool,
) -> (bool, String) {
    let correct = card.compiles == user_says_compiles;
    let verdict = if card.compiles {
        "COMPILA ✔"
    } else {
        card.error_code
    };
    let msg = if correct {
        format!(
            "✔ Acertou! ({verdict}) — {} (+25 XP se inédito)",
            card.explanation
        )
    } else {
        format!(
            "✘ Quase! O resultado real é {verdict} — {}",
            card.explanation
        )
    };
    (correct, msg)
}

// =========================================================================
// 11. INTEGRAÇÃO OPCIONAL COM DEBUGGER NATIVO (rust-gdb / lldb)
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeDebuggerKind {
    RustGdb,
    Gdb,
    RustLldb,
    Lldb,
    SimulatedOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeDebuggerProbe {
    pub kind: NativeDebuggerKind,
    pub binary_name: &'static str,
    pub available: bool,
    pub install_hint: &'static str,
}

pub fn detect_native_debugger() -> NativeDebuggerProbe {
    if let Ok(forced) = std::env::var("RUSTLAB_FAKE_DEBUGGER") {
        return match forced.trim() {
            "rust-gdb" => NativeDebuggerProbe {
                kind: NativeDebuggerKind::RustGdb,
                binary_name: "rust-gdb",
                available: true,
                install_hint: "",
            },
            "rust-lldb" => NativeDebuggerProbe {
                kind: NativeDebuggerKind::RustLldb,
                binary_name: "rust-lldb",
                available: true,
                install_hint: "",
            },
            _ => NativeDebuggerProbe {
                kind: NativeDebuggerKind::SimulatedOnly,
                binary_name: "simulado",
                available: false,
                install_hint: "Instale gdb ou lldb no sistema para depuração nativa.",
            },
        };
    }

    for (kind, bin) in [
        (NativeDebuggerKind::RustGdb, "rust-gdb"),
        (NativeDebuggerKind::RustLldb, "rust-lldb"),
        (NativeDebuggerKind::Gdb, "gdb"),
        (NativeDebuggerKind::Lldb, "lldb"),
    ] {
        if find_executable_in_path(bin) {
            return NativeDebuggerProbe {
                kind,
                binary_name: bin,
                available: true,
                install_hint: "",
            };
        }
    }

    NativeDebuggerProbe {
        kind: NativeDebuggerKind::SimulatedOnly,
        binary_name: "simulado",
        available: false,
        install_hint: "Instale `gdb`/`lldb` (e `rust-gdb`) para anexar depuração nativa; enquanto isso, o Debugger Visual didático funciona 100% offline.",
    }
}

fn find_executable_in_path(name: &str) -> bool {
    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path_var).any(|dir| dir.join(name).is_file())
}

pub fn build_native_debugger_args(
    probe: &NativeDebuggerProbe,
    binary_path: &Path,
    source_file: &str,
    breakpoints: &BTreeSet<usize>,
) -> Vec<String> {
    let mut args = Vec::new();
    match probe.kind {
        NativeDebuggerKind::RustGdb | NativeDebuggerKind::Gdb => {
            args.push("-q".to_string());
            for line in breakpoints {
                args.push("-ex".to_string());
                args.push(format!("break {source_file}:{line}"));
            }
            args.push(binary_path.display().to_string());
        }
        NativeDebuggerKind::RustLldb | NativeDebuggerKind::Lldb => {
            for line in breakpoints {
                args.push("-o".to_string());
                args.push(format!("breakpoint set --file {source_file} --line {line}"));
            }
            args.push(binary_path.display().to_string());
        }
        NativeDebuggerKind::SimulatedOnly => {
            args.push("rust-gdb".to_string());
            args.push("-q".to_string());
            for line in breakpoints {
                args.push("-ex".to_string());
                args.push(format!("break {source_file}:{line}"));
            }
            args.push(binary_path.display().to_string());
        }
    }
    args
}

pub fn format_native_debugger_command(
    probe: &NativeDebuggerProbe,
    binary_path: &Path,
    source_file: &str,
    breakpoints: &BTreeSet<usize>,
) -> String {
    let args = build_native_debugger_args(probe, binary_path, source_file, breakpoints);
    if probe.available {
        format!("{} {}", probe.binary_name, args.join(" "))
    } else {
        args.join(" ")
    }
}

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

    #[test]
    fn mapa_lifetimes_detecta_escopos_e_conflitos() {
        let code = r#"
fn retornar_dangling<'a>(ok: &'a str) -> &str {
    let local = String::from("ops");
    let r = &local;
    &local
}

fn conflito_bloco() {
    let r;
    {
        let x = String::from("curto");
        r = &x;
    }
    println!("{r}");
}
"#;
        let rep = LifetimeVisualizerReport::analyze(code);
        assert!(rep.explicit_annotations.iter().any(|(_, t)| t == "'a"));
        assert!(rep.spans.iter().any(|s| s.name == "local"));
        assert!(
            rep.conflicts.iter().any(|c| c.error_code == "E0515"),
            "deveria detectar retorno de referência local: {:?}",
            rep.conflicts
        );
        assert!(
            rep.conflicts.iter().any(|c| c.error_code == "E0597"),
            "deveria detectar vida curta de bloco interno: {:?}",
            rep.conflicts
        );
        assert!(rep.gutter_marker_for_line(4).is_some());
    }

    #[test]
    fn layout_memoria_calcula_padding_reordenacao_e_npo() {
        let code = r#"
#[repr(C)]
pub struct Desalinhada {
    flag: bool,
    grande: u64,
    pequeno: u8,
}
"#;
        let report = MemoryLayoutReport::analyze(code);
        assert_eq!(report.user_types.len(), 1);
        let st = &report.user_types[0];
        assert!(st.is_repr_c);
        assert_eq!(st.declared_size, 24);
        assert_eq!(st.optimal_size, 16);
        assert_eq!(st.bytes_saved, 8);
        assert!(st.total_padding >= 14);

        let mut known = HashMap::new();
        assert_eq!(type_size_align("Option<&str>", &known), (16, 8));
        assert_eq!(type_size_align("Option<Box<u64>>", &known), (8, 8));
        known.insert("Desalinhada".to_string(), (24, 8));
        assert_eq!(type_size_align("Desalinhada", &known), (24, 8));
        assert!(
            report
                .npo_facts
                .iter()
                .any(|f| f.type_expr.contains("Option<&T>"))
        );
    }

    #[test]
    fn desacucarador_expande_macros_e_syntax_sugar() {
        let code = r#"
#[derive(Debug, Clone)]
struct Ponto { x: i32 }

fn ler(s: &str) -> Result<i32, std::num::ParseIntError> {
    let itens = vec![1, 2, 3];
    for item in &itens {
        println!("{item}");
    }
    let n = s.parse()?;
    Ok(n)
}
"#;
        let rep = DesugarReport::analyze(code);
        assert!(rep.detected_from_editor);
        assert!(rep.items.iter().any(|i| i.id == "attr-derive"));
        assert!(rep.items.iter().any(|i| i.id == "macro-vec"));
        assert!(rep.items.iter().any(|i| i.id == "sugar-for"));
        assert!(rep.items.iter().any(|i| i.id == "macro-print"));
        assert!(rep.items.iter().any(|i| i.id == "sugar-qmark"));
    }

    #[test]
    fn dicionario_poliglota_e_flashcards_intuicao_funcionam() {
        assert_eq!(POLYGLOT_DICTIONARY.len(), 6);
        for entry in POLYGLOT_DICTIONARY {
            assert!(entry.rust_code.contains("fn main()"));
            assert!(entry.rust_code.contains("#[cfg(test)]"));
        }
        assert_eq!(INTUITION_FLASHCARDS.len(), 8);
        let c1 = &INTUITION_FLASHCARDS[0]; // E0382 (não compila)
        assert!(!c1.compiles);
        let (ok, msg) = evaluate_flashcard_answer(c1, false);
        assert!(ok);
        assert!(msg.contains("E0382"));
        let (bad, _) = evaluate_flashcard_answer(c1, true);
        assert!(!bad);
    }

    #[test]
    fn debugger_nativo_gera_comandos_gdb_e_lldb_com_breakpoints() {
        let mut bps = BTreeSet::new();
        bps.insert(5);
        bps.insert(12);

        let gdb = NativeDebuggerProbe {
            kind: NativeDebuggerKind::RustGdb,
            binary_name: "rust-gdb",
            available: true,
            install_hint: "",
        };
        let cmd_gdb = format_native_debugger_command(
            &gdb,
            Path::new("target/debug/app"),
            "src/main.rs",
            &bps,
        );
        assert!(cmd_gdb.starts_with("rust-gdb -q"));
        assert!(cmd_gdb.contains("break src/main.rs:5"));
        assert!(cmd_gdb.contains("break src/main.rs:12"));

        let lldb = NativeDebuggerProbe {
            kind: NativeDebuggerKind::RustLldb,
            binary_name: "rust-lldb",
            available: true,
            install_hint: "",
        };
        let cmd_lldb = format_native_debugger_command(
            &lldb,
            Path::new("target/debug/app"),
            "src/main.rs",
            &bps,
        );
        assert!(cmd_lldb.starts_with("rust-lldb"));
        assert!(cmd_lldb.contains("breakpoint set --file src/main.rs --line 5"));
    }
}
