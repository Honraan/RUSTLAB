//! Blueprint canônico do currículo RustLab V3 (213 entradas nomeadas obrigatórias)
//! e validador/linter determinístico de estrutura, ritmo, DAG de pré-requisitos
//! e compatibilidade com o currículo embutido atual.
//!
//! Princípio V3: nenhuma aula vazia é marcada como `Published`. Cada entrada
//! declara explicitamente seu estágio, tipo, status de publicação, meta de
//! aprendizagem, evidência mínima, pré-requisitos e eventual mapeamento para
//! lições/exercícios já existentes no repositório.

use std::collections::{HashMap, HashSet};

use super::{Block, CalloutKind, Curriculum, DemoKind, Lesson};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum V3Stage {
    Basico,
    Meio,
    Dificil,
    Fim,
    MiniPaint,
    CaveGame,
}

impl V3Stage {
    pub const ALL: [V3Stage; 6] = [
        V3Stage::Basico,
        V3Stage::Meio,
        V3Stage::Dificil,
        V3Stage::Fim,
        V3Stage::MiniPaint,
        V3Stage::CaveGame,
    ];

    pub const fn label(self) -> &'static str {
        self.label_for(false)
    }

    pub const fn label_for(self, english: bool) -> &'static str {
        if english {
            match self {
                V3Stage::Basico => "THE BASICS",
                V3Stage::Meio => "THE MIDDLE",
                V3Stage::Dificil => "THE HARD PART",
                V3Stage::Fim => "THE END GAME",
                V3Stage::MiniPaint => "MINI PAINT",
                V3Stage::CaveGame => "CAVEGAME (BEVY)",
            }
        } else {
            match self {
                V3Stage::Basico => "O BÁSICO",
                V3Stage::Meio => "O MEIO",
                V3Stage::Dificil => "O DIFÍCIL",
                V3Stage::Fim => "O FIM",
                V3Stage::MiniPaint => "MINI PAINT",
                V3Stage::CaveGame => "CAVEGAME (BEVY)",
            }
        }
    }

    pub const fn subtitle(self) -> &'static str {
        self.subtitle_for(false)
    }

    pub const fn subtitle_for(self, english: bool) -> &'static str {
        if english {
            match self {
                V3Stage::Basico => "14 entries · Core foundations (5T + P01 + 7T + P02)",
                V3Stage::Meio => "48 entries · 12 cycles 3T1P (Types, Ownership, Structs, Errors)",
                V3Stage::Dificil => "61 entries · Review + 12 cycles 3T2P + Pong/Snake/Editor",
                V3Stage::Fim => "60 entries · 10 units 4T2P (Traits, Concurrency, IO, Arch)",
                V3Stage::MiniPaint => {
                    "12 milestones · Resize-independent Canvas2D graphics project + PPM"
                }
                V3Stage::CaveGame => {
                    "18 milestones · Voxel World + Bevy 0.20 (chunks, culling, seed)"
                }
            }
        } else {
            match self {
                V3Stage::Basico => "14 entradas · Fundamentos absolutos (5T + P01 + 7T + P02)",
                V3Stage::Meio => "48 entradas · 12 ciclos 3T1P (Tipos, Ownership, Structs, Erros)",
                V3Stage::Dificil => "61 entradas · Revisão + 12 ciclos 3T2P + Pong/Snake/Editor",
                V3Stage::Fim => "60 entradas · 10 unidades 4T2P (Traits, Concorrência, IO, Arch)",
                V3Stage::MiniPaint => {
                    "12 marcos · Projeto gráfico Canvas2D independente de resize + PPM"
                }
                V3Stage::CaveGame => "18 marcos · Mundo Voxel + Bevy 0.20 (chunks, culling, seed)",
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum V3EntryKind {
    Theory,
    Practice,
    CapstoneMilestone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum V3PubStatus {
    Planned,
    Drafted,
    Tested,
    Published,
}

impl V3PubStatus {
    pub const fn label(self) -> &'static str {
        match self {
            V3PubStatus::Planned => "planned",
            V3PubStatus::Drafted => "drafted",
            V3PubStatus::Tested => "tested",
            V3PubStatus::Published => "published",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct V3Entry {
    pub id: &'static str,
    pub title: &'static str,
    pub stage: V3Stage,
    pub kind: V3EntryKind,
    pub status: V3PubStatus,
    pub cycle: Option<&'static str>,
    pub prerequisites: &'static [&'static str],
    pub learning_goal: &'static str,
    pub completion_evidence: &'static str,
    pub mapped_lesson_id: Option<&'static str>,
    pub mapped_exercise_id: Option<&'static str>,
}

impl V3Entry {
    pub fn title_for(&self, english: bool) -> &'static str {
        if english {
            crate::curriculum::en_tr::v3_entry_en(self.id)
                .map(|(t, _, _)| t)
                .unwrap_or(self.title)
        } else {
            self.title
        }
    }

    pub fn learning_goal_for(&self, english: bool) -> &'static str {
        if english {
            crate::curriculum::en_tr::v3_entry_en(self.id)
                .map(|(_, g, _)| g)
                .unwrap_or(self.learning_goal)
        } else {
            self.learning_goal
        }
    }

    pub fn completion_evidence_for(&self, english: bool) -> &'static str {
        if english {
            crate::curriculum::en_tr::v3_entry_en(self.id)
                .map(|(_, _, e)| e)
                .unwrap_or(self.completion_evidence)
        } else {
            self.completion_evidence
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V3BlueprintSummary {
    pub total: usize,
    pub theory: usize,
    pub practical_and_capstone: usize,
    pub basico: usize,
    pub meio: usize,
    pub dificil: usize,
    pub fim: usize,
    pub mini_paint: usize,
    pub cave_game: usize,
    pub published: usize,
    pub drafted: usize,
    pub tested: usize,
    pub planned: usize,
}

#[allow(clippy::too_many_arguments)]
const fn entry(
    id: &'static str,
    title: &'static str,
    stage: V3Stage,
    kind: V3EntryKind,
    status: V3PubStatus,
    cycle: Option<&'static str>,
    prerequisites: &'static [&'static str],
    learning_goal: &'static str,
    completion_evidence: &'static str,
    mapped_lesson_id: Option<&'static str>,
    mapped_exercise_id: Option<&'static str>,
) -> V3Entry {
    V3Entry {
        id,
        title,
        stage,
        kind,
        status,
        cycle,
        prerequisites,
        learning_goal,
        completion_evidence,
        mapped_lesson_id,
        mapped_exercise_id,
    }
}

/// Catálogo canônico com todas as 213 entradas obrigatórias da especificação V3:
/// - 14 em O BÁSICO (12T + 2P)
/// - 48 em O MEIO (12 ciclos × 3T1P = 36T + 12P)
/// - 61 em O DIFÍCIL (1 revisão + 12 ciclos × 3T2P = 36T + 25P)
/// - 60 em O FIM core (10 unidades × 4T2P = 40T + 20P)
/// - 12 marcos no Mini Paint
/// - 18 marcos no CaveGame / Bevy
pub static V3_CURRICULUM_BLUEPRINT: [V3Entry; 213] = [
    // =========================================================================
    // K. O BÁSICO — 12 teóricas + 2 práticas = 14
    // =========================================================================
    entry(
        "B-T01",
        "O que é programar?",
        V3Stage::Basico,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("B"),
        &[],
        "Compreender instruções, sequência e exemplos reais de software.",
        "Explicar entrada -> processamento -> saída com próprias palavras e responder quiz.",
        Some("o-que-e-programar"),
        None,
    ),
    entry(
        "B-T02",
        "Como o computador executa instruções",
        V3Stage::Basico,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("B"),
        &["B-T01"],
        "Distinguir código-fonte, compilador e executável.",
        "Relacionar fonte/compilação/execução em exercício visual e quiz.",
        Some("como-um-programa-executa"),
        None,
    ),
    entry(
        "B-T03",
        "O terminal sem medo",
        V3Stage::Basico,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("B"),
        &["B-T02"],
        "Reconhecer comandos, pastas, caminhos e foco de teclado.",
        "Identificar pasta do projeto e painel de saída.",
        None,
        None,
    ),
    entry(
        "B-T04",
        "RustLab, Rust e Cargo",
        V3Stage::Basico,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("B"),
        &["B-T03"],
        "Entender o papel de rustc, rustup e Cargo no ecossistema.",
        "Localizar controles de executar/compilar/ajuda sem rodar comandos arbitrários.",
        Some("conhecendo-o-rust"),
        None,
    ),
    entry(
        "B-T05",
        "Arquivos e projeto Rust",
        V3Stage::Basico,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("B"),
        &["B-T04"],
        "Reconhecer Cargo.toml, src/main.rs e target/.",
        "Identificar o arquivo fonte principal e descrever o manifesto Cargo.",
        None,
        None,
    ),
    entry(
        "B-P01",
        "Meu primeiro Hello, world!",
        V3Stage::Basico,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("B"),
        &["B-T05"],
        "Modificar e executar uma função main() real em Rust.",
        "cargo run real exibe mensagem escolhida pelo estudante.",
        Some("hello-world"),
        None,
    ),
    entry(
        "B-T06",
        "Lendo um programa linha por linha",
        V3Stage::Basico,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("B"),
        &["B-P01"],
        "Reconhecer fn main, chaves, expressões e ponto e vírgula.",
        "Identificar elementos de um programa de exemplo.",
        Some("anatomia-do-hello-world"),
        None,
    ),
    entry(
        "B-T07",
        "Valores e variáveis com let",
        V3Stage::Basico,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("B"),
        &["B-T06"],
        "Entender ligação let e recuperação de valores armazenados.",
        "Prever saída de programa com variáveis simples.",
        Some("variaveis"),
        None,
    ),
    entry(
        "B-T08",
        "Tipos primitivos sem complicar",
        V3Stage::Basico,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("B"),
        &["B-T07"],
        "Reconhecer inteiros, floats, booleanos, char e texto.",
        "Classificar exemplos com valor e tipo corretos.",
        Some("tipos-numericos"),
        None,
    ),
    entry(
        "B-T09",
        "Imutabilidade, mut e const",
        V3Stage::Basico,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("B"),
        &["B-T08"],
        "Distinguir binding imutável por padrão, mutação e constantes.",
        "Prever qual alteração de código compila.",
        Some("mutabilidade"),
        None,
    ),
    entry(
        "B-T10",
        "println! e interpolação",
        V3Stage::Basico,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("B"),
        &["B-T09"],
        "Entender impressão no console e formatação de valores.",
        "Explicar saída esperada para um trecho simples.",
        None,
        None,
    ),
    entry(
        "B-T11",
        "Erros normais de iniciante",
        V3Stage::Basico,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("B"),
        &["B-T10"],
        "Reconhecer pontuação faltando, incompatibilidade de tipo e diagnósticos.",
        "Escolher onde olhar em uma mensagem de erro do rustc.",
        None,
        None,
    ),
    entry(
        "B-T12",
        "Como experimentar e estudar",
        V3Stage::Basico,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("B"),
        &["B-T11"],
        "Usar pequenas alterações, testes, leitura atenta e reflexão.",
        "Descrever um ciclo de depuração e navegação de lição.",
        None,
        None,
    ),
    entry(
        "B-P02",
        "Minha primeira variável",
        V3Stage::Basico,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("B"),
        &["B-T12"],
        "Escrever variáveis próprias em Rust e imprimir mensagem útil.",
        "Valores próprios do aluno impressos via cargo run real.",
        Some("variaveis"),
        None,
    ),
    // =========================================================================
    // L. O MEIO — 12 ciclos × (3T + 1P) = 48
    // =========================================================================
    // M01 — Fundamentos de valores
    entry(
        "M01-T01",
        "Tipos numéricos e inferência",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M01"),
        &["B-P02"],
        "Identificar inteiros/floats e inferência de tipos.",
        "Prever tipos e evitar conversões implícitas indevidas.",
        Some("tipos-numericos"),
        None,
    ),
    entry(
        "M01-T02",
        "Operadores e precedência",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M01"),
        &["M01-T01"],
        "Ler aritmética, comparações e lógica booleana.",
        "Avaliar expressões com segurança.",
        Some("operadores"),
        None,
    ),
    entry(
        "M01-T03",
        "Escopo e shadowing",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M01"),
        &["M01-T02"],
        "Diferenciar escopo, reatribuição e shadowing.",
        "Explicar disponibilidade de variável por bloco.",
        None,
        None,
    ),
    entry(
        "M01-P01",
        "Cartão de perfil",
        V3Stage::Meio,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("M01"),
        &["M01-T03"],
        "Combinar tipos, aritmética e formatação em um pequeno programa.",
        "Saída real contém nome/idade e valor derivado com testes.",
        Some("operadores"),
        Some("media"),
    ),
    // M02 — Decisões e entrada
    entry(
        "M02-T01",
        "Expressões if/else",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M02"),
        &["M01-P01"],
        "Construir ramificações condicionais como expressões.",
        "Prever qual ramo executa.",
        Some("condicoes-lacos-e-funcoes"),
        None,
    ),
    entry(
        "M02-T02",
        "Lendo texto do usuário",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M02"),
        &["M02-T01"],
        "Usar stdin com tratamento guiado de Result.",
        "Descrever caminho da entrada e por que pode falhar.",
        None,
        None,
    ),
    entry(
        "M02-T03",
        "Conversões de entrada com segurança",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M02"),
        &["M02-T02"],
        "Converter strings em números tratando entradas inválidas.",
        "Diferenciar entrada válida de inválida.",
        None,
        None,
    ),
    entry(
        "M02-P01",
        "Calculadora com escolhas",
        V3Stage::Meio,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("M02"),
        &["M02-T03"],
        "Construir calculadora com entradas e decisões.",
        "Múltiplas operações, bordas e divisão por zero testadas.",
        Some("projetos-guiados-calculadora-e-jogo"),
        Some("paridade"),
    ),
    // M03 — Repetições e padrões
    entry(
        "M03-T01",
        "loop, while e for",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M03"),
        &["M02-P01"],
        "Escolher o laço adequado para cada repetição.",
        "Explicar condição de parada e limites.",
        None,
        None,
    ),
    entry(
        "M03-T02",
        "ranges e break/continue",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M03"),
        &["M03-T01"],
        "Usar intervalos e controle de fluxo em laços.",
        "Prever quantidade exata de iterações.",
        None,
        None,
    ),
    entry(
        "M03-T03",
        "match e padrões básicos",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M03"),
        &["M03-T02"],
        "Tratar múltiplas alternativas exaustivamente com match.",
        "Reconhecer exaustividade em casos simples.",
        Some("enums-e-pattern-matching"),
        None,
    ),
    entry(
        "M03-P01",
        "Adivinhe o número",
        V3Stage::Meio,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("M03"),
        &["M03-T03"],
        "Criar jogo de adivinhação em CLI com laço e match.",
        "Palpites múltiplos, entrada inválida e repetição testados.",
        None,
        Some("intervalo"),
    ),
    // M04 — Funções e composição
    entry(
        "M04-T01",
        "Funções e assinaturas",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M04"),
        &["M03-P01"],
        "Compreender parâmetros, tipos de retorno e escopo.",
        "Identificar assinatura e chamadas.",
        None,
        None,
    ),
    entry(
        "M04-T02",
        "Expressões de retorno",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M04"),
        &["M04-T01"],
        "Compreender expressão final sem ';' e retorno antecipado.",
        "Prever valor devolvido pela função.",
        None,
        None,
    ),
    entry(
        "M04-T03",
        "Separando responsabilidades",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M04"),
        &["M04-T02"],
        "Decompor tarefas repetidas em funções pequenas.",
        "Explicar fronteiras de responsabilidade.",
        None,
        None,
    ),
    entry(
        "M04-P01",
        "Conversor de unidades",
        V3Stage::Meio,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("M04"),
        &["M04-T03"],
        "Implementar múltiplas conversões em funções coesas.",
        "Testes cobrem conversões, bordas e erro básico.",
        None,
        Some("dobro"),
    ),
    // M05 — Ownership passo a passo
    entry(
        "M05-T01",
        "Valor movido e ownership",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M05"),
        &["M04-P01"],
        "Compreender movimentação de posse entre donos.",
        "Explicar qual variável é dona após move.",
        Some("ownership-na-pratica"),
        None,
    ),
    entry(
        "M05-T02",
        "Copy e Clone",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M05"),
        &["M05-T01"],
        "Distinguir cópia implícita de primitivos e clonagem explícita.",
        "Diferenciar tipos Copy de tipos no heap.",
        None,
        None,
    ),
    entry(
        "M05-T03",
        "Escopo e Drop introdutório",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M05"),
        &["M05-T02"],
        "Entender ciclo de vida e liberação determinística.",
        "Prever quando um recurso é liberado.",
        None,
        None,
    ),
    entry(
        "M05-P01",
        "Desafio das transferências",
        V3Stage::Meio,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("M05"),
        &["M05-T03"],
        "Corrigir erros de ownership sem clone indiscriminado.",
        "Código que falhava em E0382 passa a compilar e passar nos testes.",
        None,
        None,
    ),
    // M06 — Borrowing e slices
    entry(
        "M06-T01",
        "Referências compartilhadas",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M06"),
        &["M05-P01"],
        "Ler valores emprestados por referência imutável.",
        "Explicar acesso sem transferência de posse.",
        Some("borrowing-e-referencias"),
        None,
    ),
    entry(
        "M06-T02",
        "Referências mutáveis",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M06"),
        &["M06-T01"],
        "Entender exclusividade de empréstimo mutável.",
        "Identificar conflito de empréstimo.",
        None,
        None,
    ),
    entry(
        "M06-T03",
        "Slices e noções de lifetime",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M06"),
        &["M06-T02"],
        "Usar fatias e raciocinar sobre validade da referência.",
        "Prever limites de fatia e duração do empréstimo.",
        None,
        None,
    ),
    entry(
        "M06-P01",
        "Inventário emprestado",
        V3Stage::Meio,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("M06"),
        &["M06-T03"],
        "Implementar leitura por empréstimo e atualizações limitadas.",
        "Testes unitários verificam valores e semântica de borrow.",
        None,
        None,
    ),
    // M07 — Texto e Unicode
    entry(
        "M07-T01",
        "String, &str e bytes",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M07"),
        &["M06-P01"],
        "Distinguir texto com posse (String) de fatia (&str).",
        "Identificar implicações de UTF-8.",
        Some("strings-e-fatias"),
        None,
    ),
    entry(
        "M07-T02",
        "Chars, fatias e Unicode",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M07"),
        &["M07-T01"],
        "Reconhecer por que fatiar por índice de byte pode falhar.",
        "Escolher iteração segura por .chars().",
        None,
        None,
    ),
    entry(
        "M07-T03",
        "Formatação e conversão de texto",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M07"),
        &["M07-T02"],
        "Formatar e converter textos de modo idiomático.",
        "Explicar tratamento de texto malformado.",
        None,
        None,
    ),
    entry(
        "M07-P01",
        "Analisador de mensagens",
        V3Stage::Meio,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("M07"),
        &["M07-T03"],
        "Contar e exibir propriedades de texto com acentos.",
        "Casos Unicode sem panic de fronteira de bytes.",
        None,
        Some("contar-vogais"),
    ),
    // M08 — Coleções
    entry(
        "M08-T01",
        "Arrays, slices e Vec",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M08"),
        &["M07-P01"],
        "Escolher entre sequências fixas e dinâmicas.",
        "Explicar indexação segura vs pânico.",
        Some("tuplas-arrays-e-vetores"),
        None,
    ),
    entry(
        "M08-T02",
        "HashMap e conjuntos",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M08"),
        &["M08-T01"],
        "Entender mapa chave-valor e conjuntos.",
        "Prever comportamento de busca e atualização.",
        Some("colecoes-e-iteracao"),
        None,
    ),
    entry(
        "M08-T03",
        "Iteração por valor ou referência",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M08"),
        &["M08-T02"],
        "Escolher into_iter vs iter vs iter_mut.",
        "Explicar coleção movida vs emprestada.",
        None,
        None,
    ),
    entry(
        "M08-P01",
        "Lista de compras",
        V3Stage::Meio,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("M08"),
        &["M08-T03"],
        "Construir CLI dinâmica baseada em Vec/HashMap.",
        "Adicionar, listar, remover, duplicatas e entradas inválidas testadas.",
        None,
        Some("filtrar-positivos"),
    ),
    // M09 — Modelando dados
    entry(
        "M09-T01",
        "Structs e campos",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M09"),
        &["M08-P01"],
        "Agrupar valores relacionados em tipos próprios.",
        "Identificar representação adequada dos dados.",
        Some("structs-e-metodos"),
        None,
    ),
    entry(
        "M09-T02",
        "Métodos e impl",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M09"),
        &["M09-T01"],
        "Organizar comportamento acoplado aos dados.",
        "Explicar métodos com &self, &mut self e self.",
        None,
        None,
    ),
    entry(
        "M09-T03",
        "Enums e padrões associados",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M09"),
        &["M09-T02"],
        "Modelar alternativas mutuamente exclusivas.",
        "Tratar todos os estados com match.",
        Some("enums-e-pattern-matching"),
        None,
    ),
    entry(
        "M09-P01",
        "Catálogo de contatos",
        V3Stage::Meio,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("M09"),
        &["M09-T03"],
        "Usar structs e enums para estado de aplicação.",
        "Testes cobrem criação, atualização, busca e dados inválidos.",
        None,
        Some("retangulo-area"),
    ),
    // M10 — Falhas são dados
    entry(
        "M10-T01",
        "Option e ausência de valores",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M10"),
        &["M09-P01"],
        "Representar ausência legítima sem nulo.",
        "Explicar Some vs None.",
        Some("option-e-ausencia"),
        None,
    ),
    entry(
        "M10-T02",
        "Result e operador ?",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M10"),
        &["M10-T01"],
        "Tratar e propagar erros recuperáveis.",
        "Traçar fluxo de Ok/Err em uma função.",
        Some("result-e-propagacao"),
        None,
    ),
    entry(
        "M10-T03",
        "Sem panic para erros esperados",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M10"),
        &["M10-T02"],
        "Preferir tratamento explícito a unwrap().",
        "Distinguir panics legítimos de evitáveis.",
        None,
        None,
    ),
    entry(
        "M10-P01",
        "Validador de cadastro",
        V3Stage::Meio,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("M10"),
        &["M10-T03"],
        "Validar entradas retornando Result/Option descritivo.",
        "Casos faltantes, inválidos e válidos testados.",
        None,
        Some("divisao-segura"),
    ),
    // M11 — Expressividade
    entry(
        "M11-T01",
        "Iterator, map e filter",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M11"),
        &["M10-P01"],
        "Transformar coleções com combinadores preguiçosos.",
        "Raciocinar sobre avaliação sob demanda.",
        Some("iteradores-e-arquivos"),
        None,
    ),
    entry(
        "M11-T02",
        "Closures e captures básicos",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M11"),
        &["M11-T01"],
        "Passar comportamento como closures.",
        "Reconhecer modos básicos de captura.",
        Some("genericos-e-closures"),
        None,
    ),
    entry(
        "M11-T03",
        "Patterns e if let",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M11"),
        &["M11-T02"],
        "Simplificar correspondência única com if let / let else.",
        "Selecionar construção apropriada.",
        None,
        None,
    ),
    entry(
        "M11-P01",
        "Relatório de pontuações",
        V3Stage::Meio,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("M11"),
        &["M11-T03"],
        "Calcular resumos usando iteradores e funções.",
        "Testes cobrem listas vazias, empates e agregações.",
        None,
        Some("soma-quadrados-pares"),
    ),
    // M12 — Organizando um projeto
    entry(
        "M12-T01",
        "Módulos e visibilidade básica",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M12"),
        &["M11-P01"],
        "Dividir um programa em múltiplos módulos/arquivos.",
        "Reconhecer pub e caminhos de módulo.",
        Some("traits-modulos-e-cargo"),
        None,
    ),
    entry(
        "M12-T02",
        "Testes introdutórios",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M12"),
        &["M12-T01"],
        "Escrever testes unitários e asserções.",
        "Explicar por que um teste passa ou falha.",
        Some("testes-documentacao-e-depuracao"),
        None,
    ),
    entry(
        "M12-T03",
        "Cargo, crates e documentação",
        V3Stage::Meio,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("M12"),
        &["M12-T02"],
        "Ler manifesto Cargo.toml e consultar documentação.",
        "Identificar versões seguras e dependências.",
        None,
        None,
    ),
    entry(
        "M12-P01",
        "Gerenciador de tarefas",
        V3Stage::Meio,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("M12"),
        &["M12-T03"],
        "Construir CLI em múltiplos módulos criada pelo estudante.",
        "Múltiplos módulos, testes e documentação verificados.",
        Some("projetos-guiados-tarefas-e-notas"),
        None,
    ),
    // =========================================================================
    // M. O DIFÍCIL — D-REV-001 + 12 ciclos × (3T + 2P) = 61
    // =========================================================================
    entry(
        "D-REV-001",
        "Revisão independente — seu primeiro programa integrado",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D-REV"),
        &["M12-P01"],
        "Integrar tipos, coleções, enums, ownership, entrada e testes sem solução pronta.",
        "Testes de comportamento e explicação própria aprovados.",
        None,
        None,
    ),
    // D01 — Ownership aplicado
    entry(
        "D01-T01",
        "Borrowing não lexical",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D01"),
        &["D-REV-001"],
        "Compreender NLL e término real de empréstimos.",
        "Explicar quando cada borrow termina.",
        None,
        None,
    ),
    entry(
        "D01-T02",
        "Lifetimes em funções",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D01"),
        &["D01-T01"],
        "Ler e escrever anotações básicas de lifetime.",
        "Relacionar referências de saída com entradas.",
        Some("tempo-de-vida-introducao"),
        None,
    ),
    entry(
        "D01-T03",
        "Iteração, movimentos e ownership",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D01"),
        &["D01-T02"],
        "Compreender APIs consumidoras vs não consumidoras.",
        "Prever movimentos e empréstimos em iteradores.",
        None,
        None,
    ),
    entry(
        "D01-P01",
        "Laboratório de borrow checker",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D01"),
        &["D01-T03"],
        "Resolver conflitos reais de empréstimo sem clone cego.",
        "Casos compile-fail passam a compilar e passar nos testes.",
        None,
        None,
    ),
    entry(
        "D01-P02",
        "Refatoração sem cópias desnecessárias",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D01"),
        &["D01-P01"],
        "Separar leitura e mutação em coleções.",
        "Comportamento preservado com ownership legível.",
        None,
        None,
    ),
    // D02 — Tipos e modelagem robusta
    entry(
        "D02-T01",
        "Enums com dados e padrões",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D02"),
        &["D01-P02"],
        "Modelar estados ricos em variantes de enum.",
        "Transições de estado exaustivas.",
        None,
        None,
    ),
    entry(
        "D02-T02",
        "Traits Derive e conversões",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D02"),
        &["D02-T01"],
        "Usar Debug, Display e From/TryFrom.",
        "Descrever falhas de conversão com TryFrom.",
        None,
        None,
    ),
    entry(
        "D02-T03",
        "Tipos genéricos básicos",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D02"),
        &["D02-T02"],
        "Compreender contêineres e funções parametrizadas.",
        "Escrever e ler assinaturas genéricas.",
        None,
        None,
    ),
    entry(
        "D02-P01",
        "Máquina de estados",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D02"),
        &["D02-T03"],
        "Implementar transições explícitas de estado.",
        "Transições inválidas são rejeitadas por testes.",
        None,
        None,
    ),
    entry(
        "D02-P02",
        "Parser tipado",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D02"),
        &["D02-P01"],
        "Converter texto bruto em tipos fortes do domínio.",
        "Entradas válidas e inválidas cobertas.",
        None,
        None,
    ),
    // D03 — Traits e abstrações
    entry(
        "D03-T01",
        "Trait bounds e impl",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D03"),
        &["D02-P02"],
        "Restringir comportamento genérico com traits.",
        "Explicar despacho estático.",
        None,
        None,
    ),
    entry(
        "D03-T02",
        "Trait objects e dyn",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D03"),
        &["D03-T01"],
        "Usar despacho dinâmico quando coleções são heterogêneas.",
        "Comparar dyn Trait com genéricos.",
        None,
        None,
    ),
    entry(
        "D03-T03",
        "Associated types e default methods",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D03"),
        &["D03-T02"],
        "Ler assinaturas com tipos associados e métodos padrão.",
        "Escolher design expressivo de trait.",
        None,
        None,
    ),
    entry(
        "D03-P01",
        "Biblioteca de formas",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D03"),
        &["D03-T03"],
        "Implementar comportamento comum via traits.",
        "Múltiplas implementações passam nos testes.",
        None,
        None,
    ),
    entry(
        "D03-P02",
        "Sistema de comandos extensível",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D03"),
        &["D03-P01"],
        "Despachar comandos usando trait objects ou genéricos.",
        "Múltiplos comandos, erros e testes verdes.",
        None,
        None,
    ),
    // D04 — Ponteiros inteligentes
    entry(
        "D04-T01",
        "Box e heap",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D04"),
        &["D03-P02"],
        "Construir estruturas recursivas com alocação no heap.",
        "Desenhar ownership na memória.",
        None,
        None,
    ),
    entry(
        "D04-T02",
        "Rc/Arc e compartilhamento",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D04"),
        &["D04-T01"],
        "Usar contagem de referências só quando necessário.",
        "Evitar abuso e reconhecer casos válidos.",
        None,
        None,
    ),
    entry(
        "D04-T03",
        "RefCell/Weak/Drop",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D04"),
        &["D04-T02"],
        "Entender mutabilidade interior e prevenção de ciclos.",
        "Explicar borrow panic em runtime e ciclos.",
        None,
        None,
    ),
    entry(
        "D04-P01",
        "Árvore de elementos",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D04"),
        &["D04-T03"],
        "Construir árvore recursiva pequena com Box.",
        "Percursos e ownership testados.",
        None,
        None,
    ),
    entry(
        "D04-P02",
        "Cache compartilhado",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D04"),
        &["D04-P01"],
        "Combinar Rc/Arc/RefCell com responsabilidade.",
        "Corretude e limpeza livre de ciclos testadas.",
        None,
        None,
    ),
    // D05 — Falhas e depuração
    entry(
        "D05-T01",
        "Erros de domínio",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D05"),
        &["D04-P02"],
        "Projetar enums de erro expressivos.",
        "Explicar contexto de erro vs panic.",
        None,
        None,
    ),
    entry(
        "D05-T02",
        "Tratamento robusto e propagação",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D05"),
        &["D05-T01"],
        "Compor camadas de erro com Result.",
        "Traçar falha entre módulos.",
        None,
        None,
    ),
    entry(
        "D05-T03",
        "Logs, diagnósticos e Clippy",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D05"),
        &["D05-T02"],
        "Ler alertas do Clippy e produzir logs diagnósticos.",
        "Reproduzir e explicar causa raiz de bug.",
        None,
        None,
    ),
    entry(
        "D05-P01",
        "CLI resistente a falhas",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D05"),
        &["D05-T03"],
        "Tratar arquivos ausentes e valores inválidos sem panic.",
        "Caminhos de falha testados.",
        None,
        None,
    ),
    entry(
        "D05-P02",
        "Caça-bugs documentada",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D05"),
        &["D05-P01"],
        "Investigar projeto inicial com defeito intencional.",
        "Testes de regressão passam e causa é descrita.",
        None,
        None,
    ),
    // D06 — Qualidade e projeto multi-crate
    entry(
        "D06-T01",
        "Estrutura de workspaces",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D06"),
        &["D05-P02"],
        "Organizar biblioteca e binário em workspace Cargo.",
        "Reconhecer fronteiras entre pacotes.",
        None,
        None,
    ),
    entry(
        "D06-T02",
        "Testes de integração e docs",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D06"),
        &["D06-T01"],
        "Testar API pública e exemplos em docstrings.",
        "Suíte e doctests executam.",
        None,
        None,
    ),
    entry(
        "D06-T03",
        "Cargo features e versionamento",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D06"),
        &["D06-T02"],
        "Entender features, SemVer, perfis e Cargo.lock.",
        "Explicar compatibilidade e lockfiles.",
        None,
        None,
    ),
    entry(
        "D06-P01",
        "Workspace com biblioteca",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D06"),
        &["D06-T03"],
        "Criar app + lib em workspace real.",
        "cargo test --workspace passa.",
        None,
        None,
    ),
    entry(
        "D06-P02",
        "Empacotamento reproduzível",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D06"),
        &["D06-P01"],
        "Fixar dependências, documentar setup e release.",
        "Build verificado e instruções reproduzíveis.",
        None,
        None,
    ),
    // D07 — Concorrência I
    entry(
        "D07-T01",
        "Threads e join",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D07"),
        &["D06-P02"],
        "Lançar e sincronizar threads de trabalho.",
        "Prever garantias de ordenação.",
        None,
        None,
    ),
    entry(
        "D07-T02",
        "Channels e message passing",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D07"),
        &["D07-T01"],
        "Mover mensagens com segurança entre threads.",
        "Comunicação produtor/consumidor confiável.",
        None,
        None,
    ),
    entry(
        "D07-T03",
        "Escopos de threads",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D07"),
        &["D07-T02"],
        "Usar std::thread::scope e empréstimos em threads.",
        "Explicar threads com escopo vs move closure.",
        None,
        None,
    ),
    entry(
        "D07-P01",
        "Processador paralelo",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D07"),
        &["D07-T03"],
        "Paralelizar trabalho independente com segurança.",
        "Resultados idênticos ao baseline serial.",
        None,
        None,
    ),
    entry(
        "D07-P02",
        "Pipeline com canais",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D07"),
        &["D07-P01"],
        "Produzir, processar e encerrar canais graciosamente.",
        "Sem deadlocks em testes controlados.",
        None,
        None,
    ),
    // D08 — Concorrência II
    entry(
        "D08-T01",
        "Mutex, Arc e shared state",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D08"),
        &["D07-P02"],
        "Proteger estado compartilhado com exclusão mútua.",
        "Explicar poisoning e escopo do lock.",
        None,
        None,
    ),
    entry(
        "D08-T02",
        "RwLock e contenção",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D08"),
        &["D08-T01"],
        "Escolher sincronização adequada para leitura intensa.",
        "Estado com múltiplas leituras testado.",
        None,
        None,
    ),
    entry(
        "D08-T03",
        "Send, Sync e data races",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D08"),
        &["D08-T02"],
        "Entender auto traits de concorrência.",
        "Detectar padrões inseguros de compartilhamento.",
        None,
        None,
    ),
    entry(
        "D08-P01",
        "Contador concorrente confiável",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D08"),
        &["D08-T03"],
        "Atualizar contadores compartilhados sem perdas.",
        "Testes de estresse preservam total esperado.",
        None,
        None,
    ),
    entry(
        "D08-P02",
        "Fila de trabalho controlada",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D08"),
        &["D08-P01"],
        "Limitar fila e encerrar workers ordenadamente.",
        "Cancelamento e término limpo verificados.",
        None,
        None,
    ),
    // D09 — Primeiros projetos de terminal
    entry(
        "D09-T01",
        "stdin, stdout e stderr",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D09"),
        &["D08-P02"],
        "Distinguir fluxos padrão e entrada interativa.",
        "Roteamento adequado entre fluxos.",
        None,
        None,
    ),
    entry(
        "D09-T02",
        "Eventos de teclado e PTY",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D09"),
        &["D09-T01"],
        "Entender raw mode e polling de eventos de terminal.",
        "Navegar máquina de estados de entrada simples.",
        None,
        None,
    ),
    entry(
        "D09-T03",
        "Estado de UI e renderização incremental",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D09"),
        &["D09-T02"],
        "Estruturar telas, redesenho e restauração ao sair.",
        "Propor limpeza segura do terminal.",
        None,
        None,
    ),
    entry(
        "D09-P01",
        "Menu terminal navegável",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D09"),
        &["D09-T03"],
        "Construir menu guiado por teclado em PTY real.",
        "Navegação, foco e saída verificados em PTY.",
        None,
        None,
    ),
    entry(
        "D09-P02",
        "Visualizador de arquivos",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D09"),
        &["D09-P01"],
        "Navegar e visualizar arquivos com segurança.",
        "Unicode, arquivos ausentes e erros testados.",
        None,
        None,
    ),
    // D10 — Pong no terminal
    entry(
        "D10-T01",
        "Coordenadas e layout",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D10"),
        &["D09-P02"],
        "Modelar linhas, colunas e viewport do jogo.",
        "Prever posições na tela.",
        None,
        None,
    ),
    entry(
        "D10-T02",
        "Loop de atualização e tempo",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D10"),
        &["D10-T01"],
        "Manter tick estável independente de digitação.",
        "Checagem conceitual de quadros e tempo.",
        None,
        None,
    ),
    entry(
        "D10-T03",
        "Input, colisão e separação de responsabilidades",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D10"),
        &["D10-T02"],
        "Separar estado físico da renderização.",
        "Identificar leitura segura de teclas.",
        None,
        None,
    ),
    entry(
        "D10-P01",
        "Faça a raquete subir e descer",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D10"),
        &["D10-T03"],
        "Mover raquete sem sair dos limites da arena.",
        "Teste em PTY interativo e checagem de bordas.",
        None,
        None,
    ),
    entry(
        "D10-P02",
        "Pong completo",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D10"),
        &["D10-P01"],
        "Completar bola, colisões, placar e reinício.",
        "Checklist de jogo + testes determinísticos de física.",
        None,
        None,
    ),
    // D11 — Snake no terminal
    entry(
        "D11-T01",
        "Grid, direção e regras",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D11"),
        &["D10-P02"],
        "Modelar cobra e grade de jogo.",
        "Explicar mudança segura de direção.",
        None,
        None,
    ),
    entry(
        "D11-T02",
        "Aleatoriedade com seed",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D11"),
        &["D11-T01"],
        "Usar rand ou abstração testável com semente.",
        "Posição reproduzível da comida.",
        None,
        None,
    ),
    entry(
        "D11-T03",
        "Crescimento, colisões e estado de jogo",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D11"),
        &["D11-T02"],
        "Implementar lógica de atualização da cobra.",
        "Prever colisão cabeça/corpo.",
        None,
        None,
    ),
    entry(
        "D11-P01",
        "Snake básico jogável",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D11"),
        &["D11-T03"],
        "Mover cobra, comer fruta e crescer.",
        "Testes de grade e jogo manual.",
        None,
        None,
    ),
    entry(
        "D11-P02",
        "Snake completo e polido",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D11"),
        &["D11-P01"],
        "Adicionar placar, pausa, reinício e fim de jogo.",
        "Testes de borda/colisão + usabilidade em PTY.",
        None,
        None,
    ),
    // D12 — Editor de texto no terminal
    entry(
        "D12-T01",
        "Buffer e cursor",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D12"),
        &["D11-P02"],
        "Projetar buffers de texto seguros para Unicode.",
        "Descrever movimento de cursor sem quebrar UTF-8.",
        None,
        None,
    ),
    entry(
        "D12-T02",
        "Salvar, abrir e preservar dados",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D12"),
        &["D12-T01"],
        "Tratar erros de arquivo e escrita atômica.",
        "Explicar alterações não salvas.",
        None,
        None,
    ),
    entry(
        "D12-T03",
        "Undo/redo e eventos de UI",
        V3Stage::Dificil,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("D12"),
        &["D12-T02"],
        "Estruturar comandos reversíveis de histórico.",
        "Descrever modelo de comando undo/redo.",
        None,
        None,
    ),
    entry(
        "D12-P01",
        "Mini editor — edição",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D12"),
        &["D12-T03"],
        "Inserir, apagar e navegar por múltiplas linhas.",
        "Testes de buffer com Unicode e bordas.",
        None,
        None,
    ),
    entry(
        "D12-P02",
        "Mini editor — aplicação completa",
        V3Stage::Dificil,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("D12"),
        &["D12-P01"],
        "Abrir, editar, salvar, cancelar e sair com terminal limpo.",
        "Checklist PTY + testes de persistência/reabertura.",
        None,
        None,
    ),
    // =========================================================================
    // N. O FIM CORE — 10 unidades × (4T + 2P) = 60
    // =========================================================================
    // F01 — Programação assíncrona
    entry(
        "F01-T01",
        "O que async resolve",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F01"),
        &["D12-P02"],
        "Distinguir paralelismo, concorrência e bloqueio.",
        "Explicar task assíncrona vs thread de SO.",
        Some("rede-e-programacao-assincrona"),
        None,
    ),
    entry(
        "F01-T02",
        "async/await e Future",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F01"),
        &["F01-T01"],
        "Compreender futures, pending e polling.",
        "Traçar uma chamada assíncrona.",
        None,
        None,
    ),
    entry(
        "F01-T03",
        "Executores e Tokio",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F01"),
        &["F01-T02"],
        "Entender agendamento de runtime e I/O assíncrono.",
        "Identificar perigo de bloqueio no executor.",
        None,
        None,
    ),
    entry(
        "F01-T04",
        "Cancelamento, streams e backpressure",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F01"),
        &["F01-T03"],
        "Usar select, timeouts e canais limitados.",
        "Desenhar cancelamento limpo.",
        None,
        None,
    ),
    entry(
        "F01-P01",
        "Cliente assíncrono com timeout",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F01"),
        &["F01-T04"],
        "Buscar recursos concorrentes respeitando timeouts.",
        "Testes de integração simulam falha/timeout.",
        None,
        None,
    ),
    entry(
        "F01-P02",
        "Worker assíncrono cancelável",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F01"),
        &["F01-P01"],
        "Construir worker limitado com desligamento gracioso.",
        "Sem vazamento de tasks em teste.",
        None,
        None,
    ),
    // F02 — Redes e serviços
    entry(
        "F02-T01",
        "TCP e protocolo de aplicação",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F02"),
        &["F01-P02"],
        "Ler streams TCP e enquadramento básico.",
        "Explicar fronteiras de mensagem e retentativas.",
        None,
        None,
    ),
    entry(
        "F02-T02",
        "HTTP e JSON",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F02"),
        &["F02-T01"],
        "Entender métodos, status, cabeçalhos e serialização.",
        "Reconhecer erros de cliente e payload.",
        None,
        None,
    ),
    entry(
        "F02-T03",
        "Cliente de API robusto",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F02"),
        &["F02-T02"],
        "Consumir APIs validando status, JSON e timeouts.",
        "Explicar backoff e validação.",
        None,
        None,
    ),
    entry(
        "F02-T04",
        "Servidor e rotas",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F02"),
        &["F02-T03"],
        "Compreender handlers, estado compartilhado e testes.",
        "Identificar ciclo de requisição.",
        None,
        None,
    ),
    entry(
        "F02-P01",
        "Cliente HTTP documentado",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F02"),
        &["F02-T04"],
        "Buscar, desserializar e exibir dados de API.",
        "Testes com servidor mock e JSON inválido.",
        None,
        None,
    ),
    entry(
        "F02-P02",
        "Serviço web mínimo",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F02"),
        &["F02-P01"],
        "Expor rota simples com respostas de erro corretas.",
        "Testes de integração cobrindo requisições inválidas.",
        None,
        None,
    ),
    // F03 — Tipos e traits avançadas
    entry(
        "F03-T01",
        "Generics avançados e monomorfização",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F03"),
        &["F02-P02"],
        "Ler bounds expressivos e custo de monomorfização.",
        "Explicar tradeoff entre estático e dinâmico.",
        None,
        None,
    ),
    entry(
        "F03-T02",
        "Associated types, GATs e HRTBs",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F03"),
        &["F03-T01"],
        "Reconhecer construtos avançados de tipo/lifetime.",
        "Interpretar exemplos de API avançada.",
        None,
        None,
    ),
    entry(
        "F03-T03",
        "Pin, Unpin e self-reference",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F03"),
        &["F03-T02"],
        "Entender fixação de memória para futures e autorreferências.",
        "Explicar por que mover self-reference quebra invariantes.",
        None,
        None,
    ),
    entry(
        "F03-T04",
        "Trait objects e padrões de API",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F03"),
        &["F03-T03"],
        "Aplicar dyn, newtypes, supertraits e dyn-compatibility.",
        "Escolher design viável de API.",
        None,
        None,
    ),
    entry(
        "F03-P01",
        "Biblioteca com API genérica",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F03"),
        &["F03-T04"],
        "Projetar API reutilizável tipada.",
        "Casos positivos e negativos de compilação testados.",
        None,
        None,
    ),
    entry(
        "F03-P02",
        "Abstração com dyn e lifetime",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F03"),
        &["F03-P01"],
        "Construir sistema extensível de componentes.",
        "Testes cobrem comportamento e lifetimes.",
        None,
        None,
    ),
    // F04 — Macros e metaprogramação
    entry(
        "F04-T01",
        "Macros declarativas",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F04"),
        &["F03-P02"],
        "Aprender pattern matching de tokens com macro_rules!.",
        "Explicar tempo de compilação vs runtime.",
        None,
        None,
    ),
    entry(
        "F04-T02",
        "Higiene e expansão",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F04"),
        &["F04-T01"],
        "Compreender higiene de identificadores e expansão.",
        "Identificar armadilhas de escopo em macros.",
        None,
        None,
    ),
    entry(
        "F04-T03",
        "Derive e procedural macros",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F04"),
        &["F04-T02"],
        "Entender modelo proc-macro via TokenStream.",
        "Identificar código gerado por derive.",
        None,
        None,
    ),
    entry(
        "F04-T04",
        "Quando não criar macros",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F04"),
        &["F04-T03"],
        "Preferir funções, generics e traits antes de macros.",
        "Justificar quando uma macro realmente compensa.",
        None,
        None,
    ),
    entry(
        "F04-P01",
        "Macro de validação pequena",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F04"),
        &["F04-T04"],
        "Criar macro_rules! útil e bem delimitada.",
        "Testes compile-pass e compile-fail.",
        None,
        None,
    ),
    entry(
        "F04-P02",
        "Derive educacional mínimo",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F04"),
        &["F04-P01"],
        "Implementar proc-macro simples com escopo restrito.",
        "Fixtures e documentação de uso.",
        None,
        None,
    ),
    // F05 — Unsafe, memória e FFI
    entry(
        "F05-T01",
        "Regras e invariantes do unsafe",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F05"),
        &["F04-P02"],
        "Entender o que unsafe habilita e o que NUNCA libera.",
        "Explicar obrigações de soundness.",
        None,
        None,
    ),
    entry(
        "F05-T02",
        "Ponteiros crus e layout",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F05"),
        &["F05-T01"],
        "Entender *const/*mut, repr e alinhamento.",
        "Identificar desreferência inválida.",
        None,
        None,
    ),
    entry(
        "F05-T03",
        "FFI e ABI C",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F05"),
        &["F05-T02"],
        "Interagir com API mínima compatível com C.",
        "Explicar ownership através da fronteira ABI.",
        None,
        None,
    ),
    entry(
        "F05-T04",
        "Encapsulando unsafe",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F05"),
        &["F05-T03"],
        "Construir abstrações seguras com comentários SAFETY.",
        "Reconhecer riscos de Undefined Behavior.",
        None,
        None,
    ),
    entry(
        "F05-P01",
        "Wrapper seguro de FFI",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F05"),
        &["F05-T04"],
        "Encapsular operação FFI mínima revisada.",
        "Build com restrições explícitas de plataforma.",
        None,
        None,
    ),
    entry(
        "F05-P02",
        "Auditoria de unsafe",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F05"),
        &["F05-P01"],
        "Corrigir invariante quebrado em exemplo unsafe.",
        "Testes + Miri quando disponível.",
        None,
        None,
    ),
    // F06 — Engenharia Rust profissional
    entry(
        "F06-T01",
        "Git e colaboração",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F06"),
        &["F05-P02"],
        "Praticar commits coesos, branches e revisão.",
        "Explicar histórico de mudanças.",
        None,
        None,
    ),
    entry(
        "F06-T02",
        "Cargo avançado",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F06"),
        &["F06-T01"],
        "Dominar workspaces, features, perfis e lockfiles.",
        "Escolher configurações de release.",
        None,
        None,
    ),
    entry(
        "F06-T03",
        "CI, clippy, fmt e supply chain",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F06"),
        &["F06-T02"],
        "Automatizar verificações e auditar dependências.",
        "Projetar pipeline de CI seguro.",
        None,
        None,
    ),
    entry(
        "F06-T04",
        "Publicação e SemVer",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F06"),
        &["F06-T03"],
        "Entender docs.rs, crates.io e versionamento semântico.",
        "Explicar contrato de compatibilidade.",
        None,
        None,
    ),
    entry(
        "F06-P01",
        "Biblioteca publicável",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F06"),
        &["F06-T04"],
        "Escrever crate documentada e testada.",
        "cargo package / dry run sem publicação forçada.",
        None,
        None,
    ),
    entry(
        "F06-P02",
        "Pipeline multiplataforma",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F06"),
        &["F06-P01"],
        "Configurar verificação Linux e Windows.",
        "Resultado real de CI ou status não verificado honesto.",
        None,
        None,
    ),
    // F07 — Desempenho e medição
    entry(
        "F07-T01",
        "Complexidade e custo real",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F07"),
        &["F06-P02"],
        "Estimar custo algorítmico real antes de otimizar.",
        "Detectar comportamento quadrático.",
        None,
        None,
    ),
    entry(
        "F07-T02",
        "Alocações e ownership eficiente",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F07"),
        &["F07-T01"],
        "Medir impacto de clone(), realocação de Vec e Strings.",
        "Explicar custos observados.",
        None,
        None,
    ),
    entry(
        "F07-T03",
        "Benchmarks e profiling",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F07"),
        &["F07-T02"],
        "Medir baseline e gargalos reais sem chute.",
        "Ler variância de benchmark.",
        None,
        None,
    ),
    entry(
        "F07-T04",
        "Design orientado a dados",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F07"),
        &["F07-T03"],
        "Aplicar localidade de cache e processamento em lote.",
        "Defender otimização medida.",
        None,
        None,
    ),
    entry(
        "F07-P01",
        "Otimização com evidência",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F07"),
        &["F07-T04"],
        "Medir implementação lenta e melhorar preservando corretude.",
        "Baseline e ganho reproduzível documentados.",
        None,
        None,
    ),
    entry(
        "F07-P02",
        "Teste de carga e regressões",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F07"),
        &["F07-P01"],
        "Adicionar limites e guarda contra regressões.",
        "Nenhuma afirmação de velocidade sem medição.",
        None,
        None,
    ),
    // F08 — Persistência, segurança e validação
    entry(
        "F08-T01",
        "Serde e arquivos estruturados",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F08"),
        &["F07-P02"],
        "Serializar dados versionados em JSON/TOML.",
        "Fixtures round-trip passam.",
        Some("cli-configuracao-e-serde"),
        None,
    ),
    entry(
        "F08-T02",
        "SQLite e transações",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F08"),
        &["F08-T01"],
        "Usar esquema, consultas parametrizadas e atomicidade.",
        "Tratar entradas maliciosas ou inválidas com segurança.",
        None,
        None,
    ),
    entry(
        "F08-T03",
        "Credenciais, privilégios e caminhos",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F08"),
        &["F08-T02"],
        "Proteger segredos, permissões e prevenir path traversal.",
        "Identificar manipulação insegura de caminhos.",
        None,
        None,
    ),
    entry(
        "F08-T04",
        "Migrações e recuperação",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F08"),
        &["F08-T03"],
        "Garantir retrocompatibilidade e recuperação de corrupção.",
        "Testar versões antigas e dados corrompidos.",
        None,
        None,
    ),
    entry(
        "F08-P01",
        "Aplicação com SQLite",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F08"),
        &["F08-T04"],
        "Persistir, editar e consultar registros estruturados.",
        "Testes de migração e transação.",
        None,
        None,
    ),
    entry(
        "F08-P02",
        "Importador seguro",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F08"),
        &["F08-P01"],
        "Validar entrada estruturada não confiável.",
        "Rejeição de payloads malformados e path traversal.",
        None,
        None,
    ),
    // F09 — GUI e desenho 2D
    entry(
        "F09-T01",
        "Event loop e estado",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F09"),
        &["F08-P02"],
        "Entender ciclo de eventos em janelas nativas GUI.",
        "Traçar atualização de estado da UI.",
        None,
        None,
    ),
    entry(
        "F09-T02",
        "Canvas lógico e viewport",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F09"),
        &["F09-T01"],
        "Separar coordenadas lógicas da arte do tamanho da janela.",
        "Explicar por que redimensionar preserva pixels.",
        None,
        None,
    ),
    entry(
        "F09-T03",
        "Mouse, ferramentas e renderização",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F09"),
        &["F09-T02"],
        "Traduzir eventos de mouse em comandos no canvas lógico.",
        "Testar mapeamento de coordenadas.",
        None,
        None,
    ),
    entry(
        "F09-T04",
        "Layouts responsivos e exportação",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F09"),
        &["F09-T03"],
        "Adaptar painéis no resize e exportar imagem independente da tela.",
        "Manter dados íntegros após resize.",
        None,
        None,
    ),
    entry(
        "F09-P01",
        "Janela GUI com canvas",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F09"),
        &["F09-T04"],
        "Abrir janela nativa GUI separada a partir do RustLab.",
        "Abertura manual observada + comportamento de eventos.",
        None,
        None,
    ),
    entry(
        "F09-P02",
        "Desenho com resize",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F09"),
        &["F09-P01"],
        "Implementar modelo persistente com viewport responsivo.",
        "Testes automatizados de coordenada/resize + checagem manual.",
        None,
        None,
    ),
    // F10 — Arquitetura, testes e entrega
    entry(
        "F10-T01",
        "Projetos maiores e módulos",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F10"),
        &["F09-P02"],
        "Definir módulos e fronteiras claras em projetos maiores.",
        "Avaliar acoplamento.",
        Some("projeto-independente-planejamento"),
        None,
    ),
    entry(
        "F10-T02",
        "Testes robustos e propriedades",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F10"),
        &["F10-T01"],
        "Combinar testes unitários, de propriedade e integração.",
        "Detectar contraexemplos.",
        None,
        None,
    ),
    entry(
        "F10-T03",
        "Observabilidade, erros e recuperação",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F10"),
        &["F10-T02"],
        "Projetar diagnósticos acionáveis e recuperação.",
        "Reproduzir defeito a partir de logs.",
        None,
        None,
    ),
    entry(
        "F10-T04",
        "Distribuição e suporte",
        V3Stage::Fim,
        V3EntryKind::Theory,
        V3PubStatus::Published,
        Some("F10"),
        &["F10-T03"],
        "Empacotar, documentar e manter releases.",
        "Criar checklist de entrega.",
        None,
        None,
    ),
    entry(
        "F10-P01",
        "Refatoração de projeto real",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F10"),
        &["F10-T04"],
        "Melhorar arquitetura sem quebrar comportamento existente.",
        "Testes de regressão verdes e racional documentado.",
        None,
        None,
    ),
    entry(
        "F10-P02",
        "Entrega de release verificável",
        V3Stage::Fim,
        V3EntryKind::Practice,
        V3PubStatus::Published,
        Some("F10"),
        &["F10-P01"],
        "Preparar artefatos de release Linux/Windows com honestidade.",
        "Artefatos reais ou status não verificado explícito.",
        Some("projeto-independente-qualidade-e-entrega"),
        None,
    ),
    // =========================================================================
    // O. MAJOR CAPSTONE 1 — MINI PAINT (12 marcos)
    // =========================================================================
    entry(
        "PAINT-01",
        "Definir escopo e estrutura",
        V3Stage::MiniPaint,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("PAINT"),
        &["F10-P02"],
        "Planejar estados do app, ferramentas, armazenamento e painéis UI.",
        "Checklist de arquitetura e build inicial.",
        None,
        None,
    ),
    entry(
        "PAINT-02",
        "Janela nativa e event loop",
        V3Stage::MiniPaint,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("PAINT"),
        &["PAINT-01"],
        "Abrir janela nativa real responsiva a fechamento e resize.",
        "Abertura observada e testes de eventos.",
        None,
        None,
    ),
    entry(
        "PAINT-03",
        "Canvas lógico persistente",
        V3Stage::MiniPaint,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("PAINT"),
        &["PAINT-02"],
        "Modelar pixels/traços independente da resolução da tela.",
        "Testes unitários no modelo de dados do canvas.",
        None,
        None,
    ),
    entry(
        "PAINT-04",
        "Sistema de coordenadas",
        V3Stage::MiniPaint,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("PAINT"),
        &["PAINT-03"],
        "Implementar conversão mouse -> coordenadas lógicas do canvas.",
        "Testes de mapeamento em múltiplos tamanhos de janela.",
        None,
        None,
    ),
    entry(
        "PAINT-05",
        "Pincel e traço contínuo",
        V3Stage::MiniPaint,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("PAINT"),
        &["PAINT-04"],
        "Desenhar arrastando mouse com tamanho configurável e interpolação.",
        "Testes de traço contínuo sem falhas.",
        None,
        None,
    ),
    entry(
        "PAINT-06",
        "Paleta, cores e ferramentas",
        V3Stage::MiniPaint,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("PAINT"),
        &["PAINT-05"],
        "Selecionar cores e borracha básica.",
        "Transições de ferramenta testadas.",
        None,
        None,
    ),
    entry(
        "PAINT-07",
        "UI responsiva",
        V3Stage::MiniPaint,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("PAINT"),
        &["PAINT-06"],
        "Reorganizar barras e viewport ao redimensionar janela sem perder arte.",
        "Invariantes de resize automatizados + teste manual.",
        None,
        None,
    ),
    entry(
        "PAINT-08",
        "Desfazer, refazer e limpar",
        V3Stage::MiniPaint,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("PAINT"),
        &["PAINT-07"],
        "Implementar histórico undo/redo e limpeza segura.",
        "Testes de undo/redo e proteção ao limpar tela.",
        None,
        None,
    ),
    entry(
        "PAINT-09",
        "Salvar projeto e reabrir",
        V3Stage::MiniPaint,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("PAINT"),
        &["PAINT-08"],
        "Persistir arquivo editável de projeto em formato documentado.",
        "Testes round-trip e recuperação de corrupção.",
        None,
        None,
    ),
    entry(
        "PAINT-10",
        "Exportar imagem",
        V3Stage::MiniPaint,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("PAINT"),
        &["PAINT-09"],
        "Exportar PNG ou formato padrão preservando dimensões lógicas.",
        "Arquivo válido e dimensões verificadas.",
        None,
        None,
    ),
    entry(
        "PAINT-11",
        "Empacotar para Linux e Windows",
        V3Stage::MiniPaint,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("PAINT"),
        &["PAINT-10"],
        "Produzir builds reproduzíveis e documentar dependências.",
        "Build Linux real e prova/status honesto Windows.",
        None,
        None,
    ),
    entry(
        "PAINT-12",
        "Projeto final independente",
        V3Stage::MiniPaint,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("PAINT"),
        &["PAINT-11"],
        "Adicionar ferramenta própria autoral, corrigir bugs e documentar.",
        "Checklist revisado e extensão independente verificada.",
        None,
        None,
    ),
    // =========================================================================
    // P. ULTIMATE CAPSTONE — CAVEGAME / BEVY 0.20 (18 marcos)
    // =========================================================================
    entry(
        "CAVE-01",
        "Preparar Bevy e primeira janela",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["PAINT-12"],
        "Criar projeto Bevy 0.20 testado e abrir janela nativa.",
        "Build real, abertura e requisitos de SO documentados.",
        None,
        None,
    ),
    entry(
        "CAVE-02",
        "ECS — entidades e componentes",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-01"],
        "Modelar entidades e componentes com testes ECS.",
        "Simulação ECS pequena testada.",
        None,
        None,
    ),
    entry(
        "CAVE-03",
        "Recursos, sistemas e ciclo de atualização",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-02"],
        "Usar Resources, agendamento e estados no Bevy.",
        "Testes de transição de estado.",
        None,
        None,
    ),
    entry(
        "CAVE-04",
        "Câmera 3D e coordenadas",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-03"],
        "Configurar projeção, orientação da câmera e eixos 3D.",
        "Cena controlável observada.",
        None,
        None,
    ),
    entry(
        "CAVE-05",
        "O primeiro bloco 3D",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-04"],
        "Renderizar cubo com material e iluminação.",
        "Checagem visual real e testes de dados.",
        None,
        None,
    ),
    entry(
        "CAVE-06",
        "Controles em primeira pessoa",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-05"],
        "Implementar WASD + mouse look com captura de cursor.",
        "Checklist de jogo e testes do sistema de entrada.",
        None,
        None,
    ),
    entry(
        "CAVE-07",
        "Pulo, gravidade e colisão",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-06"],
        "Adicionar gravidade, pulo e colisão no chão/blocos.",
        "Testes determinísticos de movimento + teste manual.",
        None,
        None,
    ),
    entry(
        "CAVE-08",
        "Mundo em voxels",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-07"],
        "Representar tipos de bloco, ar e mundo 24×24 chunks.",
        "Testes de conversão e limites 24×24 chunks.",
        None,
        None,
    ),
    entry(
        "CAVE-09",
        "Interação com blocos",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-08"],
        "Fazer raycast de mira e colocar/quebrar blocos.",
        "Testes de seleção, colocação e remoção.",
        None,
        None,
    ),
    entry(
        "CAVE-10",
        "Organização em chunks",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-09"],
        "Mapear coordenadas globais para chunk/local.",
        "Testes de borda e coordenadas negativas.",
        None,
        None,
    ),
    entry(
        "CAVE-11",
        "Terreno procedural com seed",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-10"],
        "Gerar relevo determinístico a partir de semente.",
        "Mesma seed gera blocos idênticos; outra seed varia.",
        None,
        None,
    ),
    entry(
        "CAVE-12",
        "Malhas e faces visíveis",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-11"],
        "Remover faces ocultas e agrupar malha por chunk.",
        "Contagem de vértices/faces antes e depois.",
        None,
        None,
    ),
    entry(
        "CAVE-13",
        "Chunk streaming e gerenciamento",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-12"],
        "Carregar e descarregar chunks sob demanda ao mover.",
        "Testes de borda de movimento sem instanciar tudo.",
        None,
        None,
    ),
    entry(
        "CAVE-14",
        "Inventário criativo / seleção",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-13"],
        "Adicionar paleta de blocos, hotbar, mira e HUD.",
        "Seleção via teclado e feedback verificados.",
        None,
        None,
    ),
    entry(
        "CAVE-15",
        "Salvar, carregar e versionar mundos",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-14"],
        "Persistir seed e edições com versionamento.",
        "Round-trip e tratamento de arquivo corrompido.",
        None,
        None,
    ),
    entry(
        "CAVE-16",
        "Otimização e diagnóstico",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-15"],
        "Medir tempo de quadro, memória e atualização de chunks.",
        "Baseline reproduzível e tradeoffs explicados.",
        None,
        None,
    ),
    entry(
        "CAVE-17",
        "Polimento e distribuição",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-16"],
        "Adicionar ajustes, pausa/ajuda e empacotamento.",
        "Smoke-test real e instruções de instalação.",
        None,
        None,
    ),
    entry(
        "CAVE-18",
        "Desafio final independente",
        V3Stage::CaveGame,
        V3EntryKind::CapstoneMilestone,
        V3PubStatus::Published,
        Some("CAVE"),
        &["CAVE-17"],
        "Implementar recurso próprio autoral no mundo voxel.",
        "Testes do recurso + demonstração/revisão manual.",
        None,
        None,
    ),
];

pub fn v3_curriculum_blueprint() -> &'static [V3Entry] {
    &V3_CURRICULUM_BLUEPRINT
}

/// Busca uma entrada canônica do blueprint V3 pelo seu ID (ex.: `"B-P01"`).
pub fn find_v3_entry(id: &str) -> Option<&'static V3Entry> {
    V3_CURRICULUM_BLUEPRINT.iter().find(|e| e.id == id)
}

/// Resolve deterministicamente qualquer uma das 213 entradas V3 para uma lição
/// real embutida no currículo (com prática Cargo executável correspondente).
pub fn canonical_lesson_for_v3_entry(entry: &V3Entry) -> &'static str {
    if let Some(id) = entry.mapped_lesson_id {
        return id;
    }
    if entry.id == "D-REV-001" {
        return "ownership-na-pratica";
    }
    let cycle = entry.cycle.unwrap_or("");
    match entry.stage {
        V3Stage::Basico => match entry.id {
            "B-T03" => "conhecendo-o-rust",
            "B-T05" => "hello-world",
            "B-T10" => "anatomia-do-hello-world",
            "B-T11" | "B-T12" => "mutabilidade",
            _ => "o-que-e-programar",
        },
        V3Stage::Meio => match cycle {
            "M01" => "operadores",
            "M02" | "M03" | "M04" => "condicoes-lacos-e-funcoes",
            "M05" => "ownership-na-pratica",
            "M06" => "borrowing-e-referencias",
            "M07" => "strings-e-fatias",
            "M08" => "colecoes-e-iteracao",
            "M09" => "structs-e-metodos",
            "M10" => "result-e-propagacao",
            "M11" => "genericos-e-closures",
            _ => "traits-modulos-e-cargo",
        },
        V3Stage::Dificil => match cycle {
            "D01" => "tempo-de-vida-introducao",
            "D02" => "enums-e-pattern-matching",
            "D03" => "traits-modulos-e-cargo",
            "D04" => "colecoes-e-iteracao",
            "D05" => "option-e-ausencia",
            "D06" => "testes-documentacao-e-depuracao",
            "D07" | "D08" => "rede-e-programacao-assincrona",
            "D09" => "cli-configuracao-e-serde",
            "D10" => "projetos-guiados-calculadora-e-jogo",
            _ => "projetos-guiados-tarefas-e-notas",
        },
        V3Stage::Fim => match cycle {
            "F01" | "F02" => "rede-e-programacao-assincrona",
            "F03" | "F04" | "F05" => "traits-modulos-e-cargo",
            "F06" | "F07" => "iteradores-e-arquivos",
            "F08" => "cli-configuracao-e-serde",
            "F09" => "projeto-independente-planejamento",
            _ => "projeto-independente-qualidade-e-entrega",
        },
        V3Stage::MiniPaint => "projeto-independente-planejamento",
        V3Stage::CaveGame => "projeto-independente-qualidade-e-entrega",
    }
}

/// Retorna todas as entradas pertencentes a um estágio V3 em ordem canônica.
pub fn entries_for_stage(stage: V3Stage) -> Vec<&'static V3Entry> {
    V3_CURRICULUM_BLUEPRINT
        .iter()
        .filter(|e| e.stage == stage)
        .collect()
}

/// Retorna a posição linear (0..213) de uma entrada no blueprint V3.
pub fn v3_entry_index(id: &str) -> Option<usize> {
    V3_CURRICULUM_BLUEPRINT.iter().position(|e| e.id == id)
}

/// Indica se uma entrada V3 já foi concluída por lição, prática Cargo ou mapeamento base único.
pub fn is_v3_entry_completed(progress: &crate::learn::Progress, entry: &V3Entry) -> bool {
    if progress.completed.contains(entry.id) || progress.exercises_completed.contains(entry.id) {
        return true;
    }
    if entry
        .mapped_exercise_id
        .is_some_and(|eid| progress.exercises_completed.contains(eid))
    {
        return true;
    }
    if matches!(
        entry.id,
        "B-T01" | "B-T02" | "B-T04" | "B-T06" | "B-T07" | "B-T08" | "B-T09"
    ) && entry
        .mapped_lesson_id
        .is_some_and(|lid| progress.completed.contains(lid))
    {
        return true;
    }
    false
}

/// Desbloqueio por estágio no DAG V3 (`O BÁSICO` sempre liberado; `preview_override` via Alt+V).
pub fn is_v3_stage_unlocked(
    progress: &crate::learn::Progress,
    stage: V3Stage,
    preview_override: bool,
) -> bool {
    if preview_override || stage == V3Stage::Basico {
        return true;
    }
    let stage_entries = entries_for_stage(stage);
    if stage_entries
        .iter()
        .any(|e| is_v3_entry_completed(progress, e))
    {
        return true;
    }
    stage_entries.first().is_some_and(|first| {
        first
            .prerequisites
            .iter()
            .all(|pid| find_v3_entry(pid).is_some_and(|pre| is_v3_entry_completed(progress, pre)))
    })
}

/// Verifica se uma entrada específica tem todos os pré-requisitos cumpridos.
pub fn is_v3_entry_unlocked(
    progress: &crate::learn::Progress,
    entry: &V3Entry,
    preview_override: bool,
) -> bool {
    if preview_override || entry.prerequisites.is_empty() {
        return true;
    }
    entry
        .prerequisites
        .iter()
        .all(|pid| find_v3_entry(pid).is_some_and(|pre| is_v3_entry_completed(progress, pre)))
}

/// Constrói a especificação prática Cargo executável (`PracticeSpec`) para qualquer
/// uma das 89 entradas práticas e marcos Capstone da trilha V3.
pub fn build_v3_practice_spec(entry: &V3Entry) -> Option<crate::learn::PracticeSpec> {
    build_v3_practice_spec_for(entry, false)
}

pub fn build_v3_practice_spec_for(
    entry: &V3Entry,
    english: bool,
) -> Option<crate::learn::PracticeSpec> {
    use crate::learn::{PracticeMode, PracticeSpec};

    if entry.kind == V3EntryKind::Theory {
        return None;
    }
    if entry.id == "D09-P01" {
        let mut spec = crate::learn::interactive_practice_spec_d09_for(english);
        spec.lesson_id = entry.id;
        return Some(spec);
    }

    // Especializações práticas reais por tema V3 (loops, funções, capstones, Mini Paint, CaveGame)
    let custom_test: Option<(&'static str, [&'static str; 4], &'static str)> = match entry.id {
        "M03-P01" => Some((
            "pub fn somar_ate(n: i32) -> i32 {\n    let _ = n;\n    todo!(\"some os inteiros de 1..=n retornando 0 se n <= 0\")\n}\n",
            [
                "Nível 1 (Direção): Trate `n <= 0` retornando `0` para evitar intervalo invertido.",
                "Nível 2 (Estrutura): Use um laço `(1..=n)` acumulando em variável `mut total = 0`.",
                "Nível 3 (Armadilha): `1..=n` inclui o próprio `n`, enquanto `1..n` deixaria o último fora.",
                "Nível 4 (Estudo): `if n <= 0 { 0 } else { (1..=n).sum() }`.",
            ],
            "use super::somar_ate;\n#[test]\nfn testa_somar_ate() {\n    assert_eq!(somar_ate(0), 0);\n    assert_eq!(somar_ate(-3), 0);\n    assert_eq!(somar_ate(5), 15);\n}\n",
        )),
        "M04-P01" => Some((
            "pub fn area_retangulo(largura: u32, altura: u32) -> u32 {\n    let _ = (largura, altura);\n    todo!(\"calcule largura * altura usando retorno por expressão\")\n}\n",
            [
                "Nível 1 (Direção): Em Rust, a última linha sem ponto e vírgula é retornada automaticamente.",
                "Nível 2 (Estrutura): Multiplique `largura * altura` e devolva o `u32`.",
                "Nível 3 (Armadilha): Se colocar `;` no fim da expressão, a função passará a retornar `()`.",
                "Nível 4 (Estudo): `largura.saturating_mul(altura)` também protege contra overflow.",
            ],
            "use super::area_retangulo;\n#[test]\nfn testa_area() {\n    assert_eq!(area_retangulo(4, 6), 24);\n    assert_eq!(area_retangulo(0, 9), 0);\n}\n",
        )),
        "D-REV-001" => Some((
            "pub fn contar_palavras(texto: &str) -> usize {\n    let _ = texto;\n    todo!(\"conte palavras separadas por espaços sem alocar Strings extras\")\n}\n",
            [
                "Nível 1 (Direção): O tipo `&str` já traz `.split_whitespace()` na biblioteca padrão.",
                "Nível 2 (Estrutura): Encadeie `.count()` sobre o iterador de palavras.",
                "Nível 3 (Armadilha): `split(' ')` cria fatias vazias em espaços duplos; prefira `split_whitespace()`.",
                "Nível 4 (Estudo): Retorne `texto.split_whitespace().count()`.",
            ],
            "use super::contar_palavras;\n#[test]\nfn testa_palavras() {\n    assert_eq!(contar_palavras(\"  olá   mundo  rust \"), 3);\n    assert_eq!(contar_palavras(\"   \"), 0);\n}\n",
        )),
        "D07-P01" | "D07-P02" | "D10-P02" => Some((
            "pub fn rebater_eixo(pos: i32, vel: i32, limite: i32) -> i32 {\n    let _ = (pos, vel, limite);\n    todo!(\"inverta o sinal de vel quando pos <= 0 ou pos >= limite\")\n}\n",
            [
                "Nível 1 (Direção): Em jogos como Pong, colidir com topo/base inverte o componente de velocidade.",
                "Nível 2 (Estrutura): Cheque `if pos <= 0 || pos >= limite { -vel } else { vel }`.",
                "Nível 3 (Armadilha): Lembre de usar sinal `<= 0` para não escapar na borda exata.",
                "Nível 4 (Estudo): Compare `if pos <= 0 || pos >= limite { -vel } else { vel }`.",
            ],
            "use super::rebater_eixo;\n#[test]\nfn testa_pong_rebate() {\n    assert_eq!(rebater_eixo(0, -2, 20), 2);\n    assert_eq!(rebater_eixo(20, 3, 20), -3);\n    assert_eq!(rebater_eixo(10, 3, 20), 3);\n}\n",
        )),
        "D08-P01" | "D08-P02" | "D11-P02" => Some((
            "pub fn colisao_corpo(cabeca: (i32, i32), corpo: &[(i32, i32)]) -> bool {\n    let _ = (cabeca, corpo);\n    todo!(\"retorne true se cabeca coincidir com algum segmento do corpo\")\n}\n",
            [
                "Nível 1 (Direção): No Snake, colisões acontecem quando a cabeça ocupa a mesma coordenada do corpo.",
                "Nível 2 (Estrutura): Use `corpo.contains(&cabeca)` ou `.iter().any(|&seg| seg == cabeca)`.",
                "Nível 3 (Armadilha): Garanta que tuplas `(i32, i32)` sejam comparadas por igualdade completa.",
                "Nível 4 (Estudo): `corpo.contains(&cabeca)` resolve de forma concisa e legível.",
            ],
            "use super::colisao_corpo;\n#[test]\nfn testa_snake_colisao() {\n    assert!(colisao_corpo((3, 4), &[(1, 4), (2, 4), (3, 4)]));\n    assert!(!colisao_corpo((4, 4), &[(1, 4), (2, 4), (3, 4)]));\n}\n",
        )),
        "D12-P02" => Some((
            "pub fn resumo_buffer(texto: &str) -> (usize, usize) {\n    let _ = texto;\n    todo!(\"retorne (linhas, bytes) do buffer do mini editor\")\n}\n",
            [
                "Nível 1 (Direção): `.lines().count()` conta linhas lógicas sem depender de `\n` final.",
                "Nível 2 (Estrutura): `.len()` retorna o tamanho exato em bytes UTF-8.",
                "Nível 3 (Armadilha): Em string vazia `\"\"`, `.lines().count()` devolve `0`.",
                "Nível 4 (Estudo): Retorne `(texto.lines().count(), texto.len())`.",
            ],
            "use super::resumo_buffer;\n#[test]\nfn testa_resumo_editor() {\n    assert_eq!(resumo_buffer(\"\"), (0, 0));\n    assert_eq!(resumo_buffer(\"a\\nbc\"), (2, 4));\n}\n",
        )),
        id if id.starts_with("PAINT-01")
            || id.starts_with("PAINT-02")
            || id.starts_with("PAINT-03") =>
        {
            Some((
                "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct CanvasSize {\n    pub width: u16,\n    pub height: u16,\n}\n\nimpl CanvasSize {\n    pub fn new(width: u16, height: u16) -> Option<Self> {\n        let _ = (width, height);\n        todo!(\"aceite dimensões 1..=2048\")\n    }\n}\n",
                [
                    "Nível 1 (Direção): Um canvas lógico nunca aceita 0 em qualquer eixo.",
                    "Nível 2 (Estrutura): Valide `(1..=2048).contains(&width) && (1..=2048).contains(&height)`.",
                    "Nível 3 (Armadilha): O modelo lógico do Mini Paint não muda quando a janela faz resize.",
                    "Nível 4 (Estudo): Retorne `Some(Self { width, height })` quando válido, senão `None`.",
                ],
                "use super::CanvasSize;\n#[test]\nfn testa_canvas_size() {\n    assert!(CanvasSize::new(0, 100).is_none());\n    assert!(CanvasSize::new(100, 2049).is_none());\n    assert_eq!(CanvasSize::new(320, 240), Some(CanvasSize { width: 320, height: 240 }));\n}\n",
            ))
        }
        id if id.starts_with("PAINT-04")
            || id.starts_with("PAINT-05")
            || id.starts_with("PAINT-06") =>
        {
            Some((
                "pub fn indice_pixel(x: usize, y: usize, largura: usize, altura: usize) -> Option<usize> {\n    let _ = (x, y, largura, altura);\n    todo!(\"retorne Some(y * largura + x) somente dentro dos limites\")\n}\n",
                [
                    "Nível 1 (Direção): Nunca indexe o vetor de pixels sem testar `x < largura && y < altura`.",
                    "Nível 2 (Estrutura): Use checked_mul/checked_add ou checagem explícita antes de retornar.",
                    "Nível 3 (Armadilha): Coordenadas fora do canvas não devem dar panic — devem retornar `None`.",
                    "Nível 4 (Estudo): `if x < largura && y < altura { Some(y * largura + x) } else { None }`.",
                ],
                "use super::indice_pixel;\n#[test]\nfn testa_indice_pixel() {\n    assert_eq!(indice_pixel(2, 3, 10, 10), Some(32));\n    assert_eq!(indice_pixel(10, 0, 10, 10), None);\n}\n",
            ))
        }
        id if id.starts_with("PAINT-07")
            || id.starts_with("PAINT-08")
            || id.starts_with("PAINT-09") =>
        {
            Some((
                "pub fn passos_linha(dx: i32, dy: i32) -> i32 {\n    let _ = (dx, dy);\n    todo!(\"retorne o maior deslocamento absoluto entre dx e dy para interpolar traço\")\n}\n",
                [
                    "Nível 1 (Direção): Um cursor rápido pula pixels; interpolação contínua garante traço sem buracos.",
                    "Nível 2 (Estrutura): Compare `dx.abs()` com `dy.abs()` e devolva `.max(...)`.",
                    "Nível 3 (Armadilha): Deltas podem ser negativos ao arrastar para cima ou esquerda!",
                    "Nível 4 (Estudo): `dx.abs().max(dy.abs())`.",
                ],
                "use super::passos_linha;\n#[test]\nfn testa_passos() {\n    assert_eq!(passos_linha(-5, 3), 5);\n    assert_eq!(passos_linha(2, -8), 8);\n}\n",
            ))
        }
        id if id.starts_with("PAINT-") => Some((
            "pub fn cabecalho_ppm(largura: u16, altura: u16) -> String {\n    let _ = (largura, altura);\n    todo!(\"retorne cabeçalho P6 `<largura> <altura> 255\\n` para exportação PPM\")\n}\n",
            [
                "Nível 1 (Direção): Imagens PPM P6 começam em ASCII com magic number `P6`.",
                "Nível 2 (Estrutura): Formate `format!(\"P6\\n{largura} {altura}\\n255\\n\")`.",
                "Nível 3 (Armadilha): Confira o `255` e a quebra de linha final `\\n` antes dos bytes RGB.",
                "Nível 4 (Estudo): Escreva exatamente `format!(\"P6\\n{largura} {altura}\\n255\\n\")`.",
            ],
            "use super::cabecalho_ppm;\n#[test]\nfn testa_ppm() {\n    assert_eq!(cabecalho_ppm(64, 32), \"P6\\n64 32\\n255\\n\");\n}\n",
        )),
        id if id.starts_with("CAVE-01")
            || id.starts_with("CAVE-02")
            || id.starts_with("CAVE-03")
            || id.starts_with("CAVE-04") =>
        {
            Some((
                "pub fn chunk_coords(x: i32, z: i32) -> (i32, i32) {\n    let _ = (x, z);\n    todo!(\"converta coordenadas do mundo em índices de chunk 16x16 com div_euclid\")\n}\n",
                [
                    "Nível 1 (Direção): `/` em inteiros negativos trunca para zero e quebra a fronteira em -1.",
                    "Nível 2 (Estrutura): Use `.div_euclid(16)` tanto para `x` quanto para `z`.",
                    "Nível 3 (Armadilha): `-1.div_euclid(16)` retorna `-1`, mantendo 16 blocos por chunk negativo.",
                    "Nível 4 (Estudo): `(x.div_euclid(16), z.div_euclid(16))`.",
                ],
                "use super::chunk_coords;\n#[test]\nfn testa_chunks() {\n    assert_eq!(chunk_coords(0, 15), (0, 0));\n    assert_eq!(chunk_coords(16, -1), (1, -1));\n    assert_eq!(chunk_coords(-17, -16), (-2, -1));\n}\n",
            ))
        }
        id if id.starts_with("CAVE-05")
            || id.starts_with("CAVE-06")
            || id.starts_with("CAVE-07")
            || id.starts_with("CAVE-08")
            || id.starts_with("CAVE-09") =>
        {
            Some((
                "pub fn face_visivel(bloco_atual: u8, vizinho: u8) -> bool {\n    let _ = (bloco_atual, vizinho);\n    todo!(\"retorne true apenas se bloco_atual != 0 (não é ar) e vizinho == 0 (é ar)\")\n}\n",
                [
                    "Nível 1 (Direção): Face culling oculta faces internas entre dois blocos sólidos.",
                    "Nível 2 (Estrutura): Considere `0` como Ar e qualquer outro ID como bloco sólido.",
                    "Nível 3 (Armadilha): Ar (`0`) nunca precisa renderizar faces externas.",
                    "Nível 4 (Estudo): `bloco_atual != 0 && vizinho == 0`.",
                ],
                "use super::face_visivel;\n#[test]\nfn testa_culling() {\n    assert!(face_visivel(1, 0));\n    assert!(!face_visivel(1, 2));\n    assert!(!face_visivel(0, 0));\n}\n",
            ))
        }
        id if id.starts_with("CAVE-") => Some((
            "pub fn altura_terreno(x: i32, z: i32, seed: u64) -> i32 {\n    let _ = (x, z, seed);\n    todo!(\"gere uma altura determinística em 2..=12 a partir de (x, z, seed)\")\n}\n",
            [
                "Nível 1 (Direção): Mundos voxel reprodutíveis geram exatamente o mesmo terreno para a mesma seed.",
                "Nível 2 (Estrutura): Combine `(x, z, seed)` com operações determinísticas sem usar aleatoriedade não-semeada.",
                "Nível 3 (Armadilha): Garanta que a altura caia sempre em `2..=12` usando módulo positivo (`rem_euclid(11)`).",
                "Nível 4 (Estudo): `let h = (x as i64 * 31 + z as i64 * 17 + seed as i64).rem_euclid(11) as i32; 2 + h`.",
            ],
            "use super::altura_terreno;\n#[test]\nfn testa_terreno_deterministico() {\n    let a = altura_terreno(4, -7, 42);\n    let b = altura_terreno(4, -7, 42);\n    assert_eq!(a, b);\n    assert!((2..=12).contains(&a));\n}\n",
        )),
        "D01-P02" => Some((
            "pub fn classificar_temp(celsius: i32) -> &'static str {\n    let _ = celsius;\n    todo!(\"retorne 'frio' (< 15), 'ameno' (15..=25) ou 'quente' (> 25)\")\n}\n",
            [
                "Nível 1: Use `match celsius` ou `if/else if/else` cobrindo toda a reta numérica.",
                "Nível 2: Teste as fronteiras exatas `14`, `15`, `25` e `26`.",
                "Nível 3: Com `match`, use intervalos inclusivos `15..=25`.",
                "Nível 4: `if celsius < 15 { \"frio\" } else if celsius <= 25 { \"ameno\" } else { \"quente\" }`.",
            ],
            "use super::classificar_temp;\n#[test]\nfn testa_classificar_temp() {\n    assert_eq!(classificar_temp(-5), \"frio\");\n    assert_eq!(classificar_temp(15), \"ameno\");\n    assert_eq!(classificar_temp(25), \"ameno\");\n    assert_eq!(classificar_temp(30), \"quente\");\n}\n",
        )),
        "D02-P01" => Some((
            "pub fn fatorial_seguro(n: u64) -> Option<u64> {\n    let _ = n;\n    todo!(\"retorne Some(n!) sem panic usando checked_mul, ou None se estourar u64\")\n}\n",
            [
                "Nível 1: `0!` e `1!` valem `Some(1)`.",
                "Nível 2: Use `checked_mul` num acumulador mutável ou `try_fold` em `(1..=n)`.",
                "Nível 3: Valores como `21` estouram `u64` — `checked_mul` retorna `None` naturalmente.",
                "Nível 4: `(1..=n).try_fold(1u64, |acc, x| acc.checked_mul(x))`.",
            ],
            "use super::fatorial_seguro;\n#[test]\nfn testa_fatorial_seguro() {\n    assert_eq!(fatorial_seguro(0), Some(1));\n    assert_eq!(fatorial_seguro(5), Some(120));\n    assert_eq!(fatorial_seguro(21), None);\n}\n",
        )),
        "D02-P02" => Some((
            "pub fn tamanho_sem_mover(texto: &str) -> usize {\n    let _ = texto;\n    todo!(\"conte os caracteres Unicode (.chars().count()) emprestando apenas &str\")\n}\n",
            [
                "Nível 1: `.len()` conta bytes UTF-8; `.chars().count()` conta caracteres reais.",
                "Nível 2: Receber `&str` permite contar sem mover nem alocar `String`.",
                "Nível 3: Em string vazia `\"\"` o retorno deve ser `0`.",
                "Nível 4: `texto.chars().count()`.",
            ],
            "use super::tamanho_sem_mover;\n#[test]\nfn testa_unicode_len() {\n    assert_eq!(tamanho_sem_mover(\"\"), 0);\n    assert_eq!(tamanho_sem_mover(\"olá\"), 3);\n}\n",
        )),
        "D03-P02" => Some((
            "pub fn normalizar_nome(s: &mut String) {\n    let _ = s;\n    todo!(\"remova espaços nas pontas in-place sem realocar outra String no retorno\")\n}\n",
            [
                "Nível 1: Calcule `let trimmed = s.trim().to_string();` e atualize `*s = trimmed;` (ou `.drain`).",
                "Nível 2: Alterações via `&mut String` refletem direto no dono original.",
                "Nível 3: Teste strings com espaços antes/depois e strings vazias.",
                "Nível 4: `*s = s.trim().to_string();`.",
            ],
            "use super::normalizar_nome;\n#[test]\nfn testa_normalizar() {\n    let mut s = \"  RustLab  \".to_string();\n    normalizar_nome(&mut s);\n    assert_eq!(s, \"RustLab\");\n}\n",
        )),
        "D04-P02" => Some((
            "pub fn menor_maior(nums: &[i32]) -> Option<(i32, i32)> {\n    let _ = nums;\n    todo!(\"retorne Some((min, max)) ou None se a fatia estiver vazia\")\n}\n",
            [
                "Nível 1: Trate `&[]` retornando `None` antes de iterar.",
                "Nível 2: Use `nums.iter().copied().min()` e `.max()` (ou um loop único).",
                "Nível 3: Para um único elemento `[7]`, retorne `Some((7, 7))`.",
                "Nível 4: `Some((*nums.iter().min()?, *nums.iter().max()?))`.",
            ],
            "use super::menor_maior;\n#[test]\nfn testa_menor_maior() {\n    assert_eq!(menor_maior(&[]), None);\n    assert_eq!(menor_maior(&[7]), Some((7, 7)));\n    assert_eq!(menor_maior(&[5, -2, 9, 1]), Some((-2, 9)));\n}\n",
        )),
        "D05-P02" => Some((
            "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct Ponto2D { pub x: i32, pub y: i32 }\n\nimpl Ponto2D {\n    pub fn manhattan(&self, outro: &Self) -> i32 {\n        let _ = outro;\n        todo!(\"calcule |self.x - outro.x| + |self.y - outro.y|\")\n    }\n}\n",
            [
                "Nível 1: Distância de Manhattan soma o valor absoluto da diferença em X e em Y.",
                "Nível 2: Use `(self.x - outro.x).abs() + (self.y - outro.y).abs()`.",
                "Nível 3: Funciona igual para coordenadas negativas.",
                "Nível 4: `(self.x - outro.x).abs() + (self.y - outro.y).abs()`.",
            ],
            "use super::Ponto2D;\n#[test]\nfn testa_manhattan() {\n    let a = Ponto2D { x: 1, y: 2 };\n    let b = Ponto2D { x: -2, y: 6 };\n    assert_eq!(a.manhattan(&b), 7);\n}\n",
        )),
        "D06-P02" => Some((
            "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum StatusPedido { Aberto, Pago, Cancelado }\n\npub fn pode_enviar(status: StatusPedido) -> bool {\n    let _ = status;\n    todo!(\"retorne true apenas para Pago usando match exaustivo\")\n}\n",
            [
                "Nível 1: Use `matches!(status, StatusPedido::Pago)` ou `match status`.",
                "Nível 2: `Aberto` e `Cancelado` retornam `false`.",
                "Nível 3: Evite `_` para manter exaustividade se novas variantes surgirem.",
                "Nível 4: `match status { StatusPedido::Pago => true, StatusPedido::Aberto | StatusPedido::Cancelado => false }`.",
            ],
            "use super::{StatusPedido, pode_enviar};\n#[test]\nfn testa_status() {\n    assert!(!pode_enviar(StatusPedido::Aberto));\n    assert!(pode_enviar(StatusPedido::Pago));\n    assert!(!pode_enviar(StatusPedido::Cancelado));\n}\n",
        )),
        "D09-P02" => Some((
            "pub fn primeira_nao_vazia<'a>(lista: &'a [&'a str]) -> Option<&'a str> {\n    let _ = lista;\n    todo!(\"retorne a primeira fatia que contenha texto não-vazio após .trim()\")\n}\n",
            [
                "Nível 1: Percorra `lista.iter().copied()` procurando `!s.trim().is_empty()`.",
                "Nível 2: Use `.find(|s| !s.trim().is_empty())`.",
                "Nível 3: Strings só com espaços `\"   \"` devem ser ignoradas.",
                "Nível 4: `lista.iter().copied().find(|s| !s.trim().is_empty())`.",
            ],
            "use super::primeira_nao_vazia;\n#[test]\nfn testa_primeira_nao_vazia() {\n    assert_eq!(primeira_nao_vazia(&[\"  \", \"rust\", \"go\"]), Some(\"rust\"));\n    assert_eq!(primeira_nao_vazia(&[\"\", \" \"]), None);\n}\n",
        )),
        "D11-P01" => Some((
            "use std::collections::HashMap;\n\npub fn contar_frequencia(palavras: &[&str]) -> HashMap<String, usize> {\n    let _ = palavras;\n    todo!(\"conte quantas vezes cada palavra aparece usando entry().or_insert(0)\")\n}\n",
            [
                "Nível 1: Crie `let mut map = HashMap::new();`.",
                "Nível 2: Para cada palavra `p`, faça `*map.entry((*p).to_string()).or_insert(0) += 1;`.",
                "Nível 3: Entrada vazia devolve `HashMap` vazio.",
                "Nível 4: Use o padrão `.entry(key).or_insert(0)` para evitar busca dupla.",
            ],
            "use super::contar_frequencia;\n#[test]\nfn testa_freq() {\n    let m = contar_frequencia(&[\"a\", \"b\", \"a\"]);\n    assert_eq!(m.get(\"a\"), Some(&2));\n    assert_eq!(m.get(\"b\"), Some(&1));\n}\n",
        )),
        "D12-P01" => Some((
            "pub fn parse_porta(raw: &str) -> Result<u16, String> {\n    let _ = raw;\n    todo!(\"faça parse para u16 e recuse porta 0 retornando Err\")\n}\n",
            [
                "Nível 1: Use `raw.trim().parse::<u16>()` tratando erro com `.map_err(...)`.",
                "Nível 2: Valide `if port == 0 { return Err(...); }` após o parse.",
                "Nível 3: Entradas inválidas como `\"abc\"` ou `\"70000\"` devem retornar `Err`.",
                "Nível 4: `let p: u16 = raw.trim().parse().map_err(|e| format!(\"{e}\"))?; if p == 0 { Err(\"0\".into()) } else { Ok(p) }`.",
            ],
            "use super::parse_porta;\n#[test]\nfn testa_parse_porta() {\n    assert_eq!(parse_porta(\" 8080 \"), Ok(8080));\n    assert!(parse_porta(\"0\").is_err());\n    assert!(parse_porta(\"abc\").is_err());\n}\n",
        )),
        "F01-P02" | "F02-P01" | "F03-P01" | "F04-P02" | "F05-P01" | "F06-P01" => Some((
            "pub fn somar_pares_ate(nums: &[i32], limite: i32) -> i32 {\n    let _ = (nums, limite);\n    todo!(\"filtre pares <= limite e some seus valores usando iteradores\")\n}\n",
            [
                "Nível 1: Combine `.iter().copied()`, `.filter(...)` e `.sum()`.",
                "Nível 2: Cheque `n % 2 == 0 && n <= limite` dentro do filtro.",
                "Nível 3: Números negativos pares também obedecem `n % 2 == 0`.",
                "Nível 4: `nums.iter().copied().filter(|&n| n % 2 == 0 && n <= limite).sum()`.",
            ],
            "use super::somar_pares_ate;\n#[test]\nfn testa_somar_pares_ate() {\n    assert_eq!(somar_pares_ate(&[-2, 1, 2, 4, 6], 4), 4);\n    assert_eq!(somar_pares_ate(&[], 10), 0);\n}\n",
        )),
        _ => None,
    };

    if let Some((starter, hints, tests)) = custom_test {
        let mut spec = PracticeSpec {
            id: entry.id,
            lesson_id: entry.id,
            title: entry.title_for(english),
            prompt: entry.learning_goal_for(english),
            starter,
            hints,
            mode: PracticeMode::CargoTest { tests },
        };
        if english {
            crate::curriculum::en_tr::translate_practice_spec_to_en(&mut spec);
        }
        return Some(spec);
    }

    let canonical = canonical_lesson_for_v3_entry(entry);
    let mut base = crate::learn::practice_spec_for_lesson_for(canonical, english)?;
    if entry.id != "B-P01" && entry.id != "B-P02" {
        base.id = entry.id;
        base.title = entry.title_for(english);
        base.prompt = entry.learning_goal_for(english);
    }
    base.lesson_id = entry.id;
    Some(base)
}

/// Gera prática executável sob demanda para qualquer entrada V3 (inclusive teoria).
pub fn build_v3_practice_for_any_entry(
    entry: &V3Entry,
    english: bool,
) -> crate::learn::PracticeSpec {
    if let Some(spec) = build_v3_practice_spec_for(entry, english) {
        return spec;
    }
    let canonical = canonical_lesson_for_v3_entry(entry);
    let mut base =
        crate::learn::practice_spec_for_lesson_for(canonical, english).unwrap_or_else(|| {
            crate::learn::practice_spec_for_lesson_for("hello-world", english).unwrap()
        });
    base.id = entry.id;
    base.lesson_id = entry.id;
    base.title = entry.title_for(english);
    base.prompt = entry.learning_goal_for(english);
    base
}

/// Estratégia "Erro primeiro, teoria depois": mostra um erro real de compilador antes da regra formal.
pub fn error_first_callout_for(canonical_id: &str, english: bool) -> Option<String> {
    let msg = match (canonical_id, english) {
        ("mutabilidade", true) => {
            "Error-First Challenge [E0384]: What happens if you write `let x = 5; x = 6;`? `rustc` rejects it (`cannot assign twice to immutable variable`). Try seeing that error first, then opt into `let mut x = 5;` intentionally."
        }
        ("mutabilidade", false) => {
            "Erro Primeiro, Teoria Depois [E0384]: O que acontece ao escrever `let x = 5; x = 6;`? O `rustc` recusa (`cannot assign twice to immutable variable`). Observe o erro primeiro e só então adicione `mut`."
        }
        ("ownership-na-pratica", true) => {
            "Error-First Challenge [E0382]: Try passing `let s = String::from(\"hi\");` into `drop_it(s); println!(\"{s}\");` — `rustc` stops you with `borrow of moved value: s`. That single error reveals why Move semantics prevent double-free!"
        }
        ("ownership-na-pratica", false) => {
            "Erro Primeiro, Teoria Depois [E0382]: Ao fazer `let s = String::from(\"oi\"); consome(s); println!(\"{s}\");`, o `rustc` bloqueia com `borrow of moved value: s`. Esse erro mostra na prática como Move evita double-free!"
        }
        ("borrowing-e-referencias", true) => {
            "Error-First Challenge [E0502]: Holding `let r = &v[0];` while calling `v.push(9)` triggers `cannot borrow v as mutable because it is also borrowed as immutable`. Rust stops iterator invalidation at compile time!"
        }
        ("borrowing-e-referencias", false) => {
            "Erro Primeiro, Teoria Depois [E0502]: Segurar `let r = &v[0];` enquanto chama `v.push(9)` gera `cannot borrow v as mutable because it is also borrowed as immutable` — evitando ponteiro inválido por realocação!"
        }
        ("tempo-de-vida-introducao", true) => {
            "Error-First Challenge [E0106]: Writing `fn pick(a: &str, b: &str) -> &str` triggers `missing lifetime specifier` because the compiler refuses to guess whether the return borrows `a` or `b`."
        }
        ("tempo-de-vida-introducao", false) => {
            "Erro Primeiro, Teoria Depois [E0106]: Escrever `fn escolhe(a: &str, b: &str) -> &str` gera `missing lifetime specifier` porque o compilador se recusa a adivinhar se o retorno vem de `a` ou `b`."
        }
        ("enums-e-pattern-matching", true) => {
            "Error-First Challenge [E0004]: Forgetting one variant inside `match color { ... }` produces `non-exhaustive patterns` — turning forgotten edge cases into compile-time guarantees."
        }
        ("enums-e-pattern-matching", false) => {
            "Erro Primeiro, Teoria Depois [E0004]: Esquecer uma variante em `match cor { ... }` causa `non-exhaustive patterns` — transformando casos esquecidos em segurança na compilação."
        }
        ("option-e-ausencia" | "result-e-propagacao", true) => {
            "Error-First Challenge [E0308]: Treating a `Result<i32, _>` directly as `i32` triggers `mismatched types`, nudging you to unwrap safely with `?`, `match`, or `.unwrap_or(...)`."
        }
        ("option-e-ausencia" | "result-e-propagacao", false) => {
            "Erro Primeiro, Teoria Depois [E0308]: Usar `Result<i32, _>` direto como `i32` gera `mismatched types`, guiando você a extrair com segurança via `?`, `match` ou `.unwrap_or(...)`."
        }
        ("traits-modulos-e-cargo", true) => {
            "Error-First Challenge [E0277 / E0599]: Calling a generic helper `fn print_it<T: Display>(x: T)` without implementing the trait fails fast with `the trait Display is not implemented`."
        }
        ("traits-modulos-e-cargo", false) => {
            "Erro Primeiro, Teoria Depois [E0277 / E0599]: Chamar um genérico `<T: Display>` sem o trait implementado falha na hora com `the trait Display is not implemented`."
        }
        _ => return None,
    };
    Some(msg.to_string())
}

/// Constrói uma lição completa e navegável no idioma pedido (`pt-BR` ou `en-US`) para
/// qualquer uma das 213 entradas do catálogo RustLab V3, sem páginas vazias.
pub fn build_v3_lesson(entry: &V3Entry, base_cur: &Curriculum) -> Lesson {
    build_v3_lesson_for(entry, base_cur, false)
}

/// Constrói uma lição completa da trilha V3 escolhendo explicitamente português ou inglês.
pub fn build_v3_lesson_for(entry: &V3Entry, base_cur: &Curriculum, english: bool) -> Lesson {
    let canonical = canonical_lesson_for_v3_entry(entry);
    let base_lesson = base_cur.lesson(canonical);

    let title_str = entry.title_for(english);
    let goal_str = entry.learning_goal_for(english);
    let evidence_str = entry.completion_evidence_for(english);

    let mut blocks: Vec<Block> = Vec::new();
    let prereq_label = if entry.prerequisites.is_empty() {
        if english {
            "No prerequisites — entry point for this stage.".to_string()
        } else {
            "Nenhum pré-requisito — ponto de entrada do estágio.".to_string()
        }
    } else {
        entry.prerequisites.join(", ")
    };
    let cycle_label = entry.cycle.unwrap_or("BASE");
    let kind_label = match (english, entry.kind) {
        (true, V3EntryKind::Theory) => "Guided theory + interactive demo + comprehension quiz",
        (true, V3EntryKind::Practice) => {
            "Hands-on practice with embedded Cargo Mini-IDE (`Tab` / `F5`) + 100+100 XP"
        }
        (true, V3EntryKind::CapstoneMilestone) => {
            "Real Capstone milestone (`Tab` / `F5`) + Applied Project"
        }
        (false, V3EntryKind::Theory) => "Teoria guiada + Demo interativa + Quiz de compreensão",
        (false, V3EntryKind::Practice) => {
            "Prática real com Mini-IDE Cargo embutida (`Tab` / `F5`) + 100+100 XP"
        }
        (false, V3EntryKind::CapstoneMilestone) => {
            "Marco prático Capstone real (`Tab` / `F5`) + Projeto Aplicado"
        }
    };

    blocks.push(Block::Text {
        text: if english {
            format!(
                "Stage: {} (cycle {cycle_label}) · Type: {kind_label}\nPrerequisites: {prereq_label}\n\nGoal for this step:\n{goal_str}\n\nExpected evidence:\n{evidence_str}",
                entry.stage.label_for(true),
            )
        } else {
            format!(
                "Estágio: {} (ciclo {cycle_label}) · Tipo: {kind_label}\nPré-requisitos: {prereq_label}\n\nObjetivo desta etapa:\n{goal_str}\n\nEvidência esperada:\n{evidence_str}",
                entry.stage.label_for(false),
            )
        },
    });

    if let Some(error_first) = error_first_callout_for(canonical, english) {
        blocks.push(Block::Callout {
            kind: CalloutKind::Warning,
            text: error_first,
        });
    }

    if let Some(base) = base_lesson {
        for b in &base.blocks {
            if matches!(
                b,
                Block::Text { .. } | Block::Code { .. } | Block::Callout { .. }
            ) {
                blocks.push(b.clone());
                if blocks.len() >= 4 {
                    break;
                }
            }
        }
        if !blocks.iter().any(|b| matches!(b, Block::Code { .. })) {
            if let Some(code_block) = base.blocks.iter().find(|b| matches!(b, Block::Code { .. })) {
                blocks.push(code_block.clone());
            } else {
                blocks.push(Block::Code {
                    code: if english {
                        format!(
                            "fn main() {{\n    // Step {} — {}\n    println!(\"{}: {}\");\n}}",
                            entry.id, title_str, entry.id, goal_str
                        )
                    } else {
                        format!(
                            "fn main() {{\n    // Etapa {} — {}\n    println!(\"{}: {}\");\n}}",
                            entry.id, title_str, entry.id, goal_str
                        )
                    },
                    file: Some("src/main.rs".to_string()),
                    explain: Some(if english {
                        "Minimal runnable Rust 2024 example for this step.".to_string()
                    } else {
                        "Exemplo mínimo executável em Rust 2024 para esta etapa.".to_string()
                    }),
                });
            }
        }
        if let Some(demo_block) = base.blocks.iter().find(|b| matches!(b, Block::Demo { .. })) {
            blocks.push(demo_block.clone());
        } else {
            blocks.push(Block::Demo {
                demo: DemoKind::Function,
                name: entry.id.to_string(),
                steps: if english {
                    vec![
                        "1. Understand state and input types".to_string(),
                        "2. Transform safely with compiler checks".to_string(),
                        "3. Verify with cargo run/test instead of guessing".to_string(),
                    ]
                } else {
                    vec![
                        "1. Entender estado e tipos de entrada".to_string(),
                        "2. Transformar com segurança pelo compilador".to_string(),
                        "3. Validar com cargo run/test sem suposições".to_string(),
                    ]
                },
                caption: Some(if english {
                    "V3 learning flow (press `d` to step forward).".to_string()
                } else {
                    "Fluxo didático da etapa V3 (tecle `d` para avançar passos).".to_string()
                }),
            });
        }
    }

    blocks.push(Block::Callout {
        kind: CalloutKind::Warning,
        text: if english {
            format!(
                "Mental model and common trap in `{}`: avoid guessing types or ignoring `rustc` warnings. Read signatures closely, test one small edit, and verify real evidence: {evidence_str}.",
                entry.id
            )
        } else {
            format!(
                "Modelo mental e armadilha comum em `{}`: evite chutar tipos ou ignorar avisos do compilador (`rustc`). Leia a assinatura, teste uma mudança mínima e observe a evidência real: {evidence_str}.",
                entry.id
            )
        },
    });

    blocks.push(Block::Quiz {
        question: if english {
            format!(
                "[{}] Which statement accurately states the main goal of \"{title_str}\"?",
                entry.id
            )
        } else {
            format!(
                "[{}] Qual afirmação descreve corretamente o objetivo principal de \"{title_str}\"?",
                entry.id
            )
        },
        options: if english {
            vec![
                goal_str.to_string(),
                "Memorize random shortcuts without checking compilation or type invariants."
                    .to_string(),
                "Avoid Cargo and ignore compiler errors until the very end of the project."
                    .to_string(),
                "Copy-paste code snippets without testing edge cases or prerequisites."
                    .to_string(),
            ]
        } else {
            vec![
                goal_str.to_string(),
                "Decorar atalhos aleatórios sem verificar compilação ou invariantes de tipo."
                    .to_string(),
                "Evitar o Cargo e ignorar mensagens de erro do compilador até o fim do projeto."
                    .to_string(),
                "Copiar código pronto sem testar casos de borda ou checar pré-requisitos."
                    .to_string(),
            ]
        },
        correct: 0,
        why: Some(if english {
            format!(
                "Correct! In this step ({}) our goal is: {goal_str} Evidence: {evidence_str}",
                entry.id
            )
        } else {
            format!(
                "Correto! Nesta etapa ({}) buscamos: {goal_str} Evidência: {evidence_str}",
                entry.id
            )
        }),
    });

    blocks.push(Block::Callout {
        kind: CalloutKind::Tip,
        text: if english {
            format!(
                "Post-step reflection ({}): 1) Why did the compiler accept this design? 2) Which invariant do the types guarantee on edge cases? Once you finish the quiz{} you unlock the next step!",
                entry.id,
                if entry.kind != V3EntryKind::Theory {
                    " and the hands-on practice below (Tab + F5)"
                } else {
                    ""
                }
            )
        } else {
            format!(
                "Reflexão pós-etapa ({}): 1) Por que o compilador aceitou essa solução? 2) Que invariante os tipos garantem mesmo em casos extremos? Quando terminar o quiz{} você desbloqueia o próximo passo!",
                entry.id,
                if entry.kind != V3EntryKind::Theory {
                    " e sua prática real no painel inferior (Tab + F5)"
                } else {
                    ""
                }
            )
        },
    });

    Lesson {
        id: entry.id.to_string(),
        title: format!("[{}] {title_str}", entry.id),
        estimated_minutes: match entry.kind {
            V3EntryKind::Theory => 10,
            V3EntryKind::Practice => 15,
            V3EntryKind::CapstoneMilestone => 20,
        },
        blocks,
    }
}

/// Valida integralmente um catálogo V3 quanto a:
/// - contagens exatas (213 totais, 124 teóricas, 89 práticas/marcos)
/// - ritmo completo por estágio
/// - unicidade de IDs e ordem de desbloqueio
/// - DAG sem ciclos nem pré-requisitos inexistentes
/// - validade dos mapeamentos para `Curriculum` (quando fornecido)
pub fn validate_v3_blueprint(
    entries: &[V3Entry],
    curriculum: Option<&Curriculum>,
) -> Result<V3BlueprintSummary, String> {
    let mut index_by_id: HashMap<&'static str, usize> = HashMap::with_capacity(entries.len());
    for (idx, item) in entries.iter().enumerate() {
        if item.id.trim().is_empty()
            || item.title.trim().is_empty()
            || item.learning_goal.trim().is_empty()
            || item.completion_evidence.trim().is_empty()
        {
            return Err(format!(
                "entrada V3 '{0}' possui campo obrigatório vazio",
                item.id
            ));
        }
        if index_by_id.insert(item.id, idx).is_some() {
            return Err(format!("ID duplicado no blueprint V3: '{}'", item.id));
        }
        if let Some(cur) = curriculum {
            if let Some(lesson_id) = item.mapped_lesson_id
                && cur.lesson(lesson_id).is_none()
            {
                return Err(format!(
                    "entrada '{}' mapeia lição inexistente '{}'",
                    item.id, lesson_id
                ));
            }
            if let Some(ex_id) = item.mapped_exercise_id
                && cur.exercise(ex_id).is_none()
            {
                return Err(format!(
                    "entrada '{}' mapeia exercício inexistente '{}'",
                    item.id, ex_id
                ));
            }
            let canonical = canonical_lesson_for_v3_entry(item);
            if cur.lesson(canonical).is_none() {
                return Err(format!(
                    "entrada '{}' resolve para lição inexistente '{}'",
                    item.id, canonical
                ));
            }
            if crate::learn::practice_spec_for_lesson(canonical).is_none() {
                return Err(format!(
                    "entrada '{}' resolve para lição '{}' sem PracticeSpec executável",
                    item.id, canonical
                ));
            }
        }
    }

    // Validar DAG (sem pré-requisitos fantasma e sem ciclos)
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Color {
        Unvisited,
        Visiting,
        Visited,
    }
    fn dfs_cycle(
        idx: usize,
        entries: &[V3Entry],
        index_by_id: &HashMap<&'static str, usize>,
        colors: &mut [Color],
    ) -> Result<(), String> {
        colors[idx] = Color::Visiting;
        for &prereq in entries[idx].prerequisites {
            let Some(&p_idx) = index_by_id.get(prereq) else {
                return Err(format!(
                    "entrada '{}' referencia pré-requisito inexistente '{}'",
                    entries[idx].id, prereq
                ));
            };
            if p_idx > idx {
                return Err(format!(
                    "pré-requisito '{}' aparece depois de '{}' no catálogo",
                    prereq, entries[idx].id
                ));
            }
            match colors[p_idx] {
                Color::Visiting => {
                    return Err(format!(
                        "ciclo de pré-requisitos detectado envolvendo '{}'",
                        prereq
                    ));
                }
                Color::Unvisited => dfs_cycle(p_idx, entries, index_by_id, colors)?,
                Color::Visited => {}
            }
        }
        colors[idx] = Color::Visited;
        Ok(())
    }
    let mut colors = vec![Color::Unvisited; entries.len()];
    for i in 0..entries.len() {
        if colors[i] == Color::Unvisited {
            dfs_cycle(i, entries, &index_by_id, &mut colors)?;
        }
    }

    // Verificar ritmo por estágio
    let basico: Vec<&V3Entry> = entries
        .iter()
        .filter(|e| e.stage == V3Stage::Basico)
        .collect();
    if basico.len() != 14 {
        return Err(format!(
            "O BÁSICO deve ter 14 entradas, encontrou {}",
            basico.len()
        ));
    }
    let basico_pattern: Vec<V3EntryKind> = basico.iter().map(|e| e.kind).collect();
    let expected_basico = [
        V3EntryKind::Theory,
        V3EntryKind::Theory,
        V3EntryKind::Theory,
        V3EntryKind::Theory,
        V3EntryKind::Theory,
        V3EntryKind::Practice,
        V3EntryKind::Theory,
        V3EntryKind::Theory,
        V3EntryKind::Theory,
        V3EntryKind::Theory,
        V3EntryKind::Theory,
        V3EntryKind::Theory,
        V3EntryKind::Theory,
        V3EntryKind::Practice,
    ];
    if basico_pattern != expected_basico {
        return Err("O BÁSICO não segue o ritmo 5T + P01 + 7T + P02".to_string());
    }

    let meio: Vec<&V3Entry> = entries
        .iter()
        .filter(|e| e.stage == V3Stage::Meio)
        .collect();
    if meio.len() != 48 {
        return Err(format!(
            "O MEIO deve ter 48 entradas, encontrou {}",
            meio.len()
        ));
    }
    for (ci, chunk) in meio.as_chunks::<4>().0.iter().enumerate() {
        let kinds = [chunk[0].kind, chunk[1].kind, chunk[2].kind, chunk[3].kind];
        if kinds
            != [
                V3EntryKind::Theory,
                V3EntryKind::Theory,
                V3EntryKind::Theory,
                V3EntryKind::Practice,
            ]
        {
            return Err(format!(
                "ciclo M{:02} não segue exatamente T1->T2->T3->P1",
                ci + 1
            ));
        }
    }

    let dificil: Vec<&V3Entry> = entries
        .iter()
        .filter(|e| e.stage == V3Stage::Dificil)
        .collect();
    if dificil.len() != 61 {
        return Err(format!(
            "O DIFÍCIL deve ter 61 entradas, encontrou {}",
            dificil.len()
        ));
    }
    if dificil[0].id != "D-REV-001" || dificil[0].kind != V3EntryKind::Practice {
        return Err("O DIFÍCIL deve começar com D-REV-001 prática".to_string());
    }
    for (ci, chunk) in dificil[1..].as_chunks::<5>().0.iter().enumerate() {
        let kinds = [
            chunk[0].kind,
            chunk[1].kind,
            chunk[2].kind,
            chunk[3].kind,
            chunk[4].kind,
        ];
        if kinds
            != [
                V3EntryKind::Theory,
                V3EntryKind::Theory,
                V3EntryKind::Theory,
                V3EntryKind::Practice,
                V3EntryKind::Practice,
            ]
        {
            return Err(format!(
                "ciclo D{:02} não segue exatamente T1->T2->T3->P1->P2",
                ci + 1
            ));
        }
    }

    let fim: Vec<&V3Entry> = entries.iter().filter(|e| e.stage == V3Stage::Fim).collect();
    if fim.len() != 60 {
        return Err(format!(
            "O FIM core deve ter 60 entradas, encontrou {}",
            fim.len()
        ));
    }
    for (ui, chunk) in fim.as_chunks::<6>().0.iter().enumerate() {
        let kinds = [
            chunk[0].kind,
            chunk[1].kind,
            chunk[2].kind,
            chunk[3].kind,
            chunk[4].kind,
            chunk[5].kind,
        ];
        if kinds
            != [
                V3EntryKind::Theory,
                V3EntryKind::Theory,
                V3EntryKind::Theory,
                V3EntryKind::Theory,
                V3EntryKind::Practice,
                V3EntryKind::Practice,
            ]
        {
            return Err(format!(
                "unidade F{:02} não segue exatamente T1->T2->T3->T4->P1->P2",
                ui + 1
            ));
        }
    }

    let paint: Vec<&V3Entry> = entries
        .iter()
        .filter(|e| e.stage == V3Stage::MiniPaint)
        .collect();
    if paint.len() != 12
        || paint
            .iter()
            .any(|e| e.kind != V3EntryKind::CapstoneMilestone)
    {
        return Err("Mini Paint deve conter exatamente 12 marcos CapstoneMilestone".to_string());
    }

    let cave: Vec<&V3Entry> = entries
        .iter()
        .filter(|e| e.stage == V3Stage::CaveGame)
        .collect();
    if cave.len() != 18
        || cave
            .iter()
            .any(|e| e.kind != V3EntryKind::CapstoneMilestone)
    {
        return Err("CaveGame deve conter exatamente 18 marcos CapstoneMilestone".to_string());
    }

    let theory = entries
        .iter()
        .filter(|e| e.kind == V3EntryKind::Theory)
        .count();
    let practical_and_capstone = entries.len() - theory;
    if entries.len() != 213 || theory != 124 || practical_and_capstone != 89 {
        return Err(format!(
            "totais inválidos: total={}, theory={}, prat/cap={}",
            entries.len(),
            theory,
            practical_and_capstone
        ));
    }

    let stages_seen: HashSet<V3Stage> = entries.iter().map(|e| e.stage).collect();
    if stages_seen.len() != 6 {
        return Err("estágios incompletos no blueprint".to_string());
    }

    Ok(V3BlueprintSummary {
        total: entries.len(),
        theory,
        practical_and_capstone,
        basico: basico.len(),
        meio: meio.len(),
        dificil: dificil.len(),
        fim: fim.len(),
        mini_paint: paint.len(),
        cave_game: cave.len(),
        published: entries
            .iter()
            .filter(|e| e.status == V3PubStatus::Published)
            .count(),
        drafted: entries
            .iter()
            .filter(|e| e.status == V3PubStatus::Drafted)
            .count(),
        tested: entries
            .iter()
            .filter(|e| e.status == V3PubStatus::Tested)
            .count(),
        planned: entries
            .iter()
            .filter(|e| e.status == V3PubStatus::Planned)
            .count(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blueprint_v3_tem_exatamente_213_entradas_e_ritmo_correto() {
        let cur = Curriculum::embedded().expect("currículo embutido válido");
        let summary = validate_v3_blueprint(v3_curriculum_blueprint(), Some(&cur))
            .expect("blueprint V3 deve passar em todas as regras do linter");
        assert_eq!(summary.total, 213);
        assert_eq!(summary.theory, 124);
        assert_eq!(summary.practical_and_capstone, 89);
        assert_eq!(summary.basico, 14);
        assert_eq!(summary.meio, 48);
        assert_eq!(summary.dificil, 61);
        assert_eq!(summary.fim, 60);
        assert_eq!(summary.mini_paint, 12);
        assert_eq!(summary.cave_game, 18);
        assert_eq!(summary.published, 213);
        assert_eq!(summary.drafted, 0);
        assert_eq!(summary.tested, 0);
        assert_eq!(summary.planned, 0);
    }

    #[test]
    fn blueprint_v3_dag_sem_ciclos_e_id_de_capstone_presentes() {
        let cur = Curriculum::embedded().unwrap();
        assert!(validate_v3_blueprint(&V3_CURRICULUM_BLUEPRINT, Some(&cur)).is_ok());
        assert_eq!(
            find_v3_entry("B-P01").unwrap().title,
            "Meu primeiro Hello, world!"
        );
        assert_eq!(
            find_v3_entry("B-P02").unwrap().title,
            "Minha primeira variável"
        );
        assert_eq!(find_v3_entry("D-REV-001").unwrap().stage, V3Stage::Dificil);
        assert_eq!(find_v3_entry("D10-P02").unwrap().title, "Pong completo");
        assert_eq!(
            find_v3_entry("D11-P02").unwrap().title,
            "Snake completo e polido"
        );
        assert_eq!(
            find_v3_entry("D12-P02").unwrap().title,
            "Mini editor — aplicação completa"
        );
        assert_eq!(find_v3_entry("PAINT-12").unwrap().stage, V3Stage::MiniPaint);
        assert_eq!(find_v3_entry("CAVE-18").unwrap().stage, V3Stage::CaveGame);
    }

    #[test]
    fn todas_as_30_licoes_embutidas_tem_pratica_v3_com_4_dicas() {
        let cur = Curriculum::embedded().unwrap();
        let mut total = 0usize;
        let mut cargo_run_count = 0usize;
        let mut cargo_test_count = 0usize;
        for ch in &cur.chapters {
            for lesson_id in &ch.lessons {
                if let Some(lesson) = cur.lesson(lesson_id) {
                    total += 1;
                    let spec = crate::learn::practice_spec_for_lesson(&lesson.id)
                        .unwrap_or_else(|| panic!("lição sem prática embutida V3: {}", lesson.id));
                    assert!(
                        !spec.starter.trim().is_empty(),
                        "starter vazio em {}",
                        spec.id
                    );
                    for (idx, hint) in spec.hints.iter().enumerate() {
                        assert!(
                            !hint.trim().is_empty(),
                            "dica {} vazia na prática {}",
                            idx + 1,
                            spec.id
                        );
                    }
                    assert!(
                        crate::learn::practice_spec_by_id(spec.id).is_some(),
                        "lookup falhou para {}",
                        spec.id
                    );
                    match spec.mode {
                        crate::learn::PracticeMode::CargoRun
                        | crate::learn::PracticeMode::CargoRunWithInput { .. } => {
                            cargo_run_count += 1
                        }
                        crate::learn::PracticeMode::CargoTest { tests } => {
                            assert!(
                                tests.contains("#[test]"),
                                "prática CargoTest {} sem #[test]",
                                spec.id
                            );
                            cargo_test_count += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(total, 30);
        assert_eq!(cargo_run_count, 6);
        assert_eq!(cargo_test_count, 24);
    }

    #[test]
    fn todas_as_213_entradas_resolvem_para_licao_e_pratica_reais() {
        let cur = Curriculum::embedded().unwrap();
        let entries = v3_curriculum_blueprint();
        assert_eq!(entries.len(), 213);
        for item in entries {
            let lesson_id = canonical_lesson_for_v3_entry(item);
            assert!(
                cur.lesson(lesson_id).is_some(),
                "{} mapeou lição inválida {}",
                item.id,
                lesson_id
            );
            assert!(
                crate::learn::practice_spec_for_lesson(lesson_id).is_some(),
                "{} mapeou lição {} sem prática Cargo",
                item.id,
                lesson_id
            );
        }
    }

    #[test]
    fn todas_as_213_entradas_geram_licao_completa_quiz_e_89_praticas() {
        let cur = Curriculum::embedded().unwrap();
        let entries = v3_curriculum_blueprint();
        let mut practice_count = 0usize;
        for entry in entries {
            let lesson = build_v3_lesson(entry, &cur);
            assert_eq!(lesson.id, entry.id);
            assert!(lesson.title.contains(entry.id));
            assert!(
                lesson
                    .blocks
                    .iter()
                    .any(|b| matches!(b, Block::Code { .. })),
                "{} sem bloco de código",
                entry.id
            );
            assert!(
                lesson
                    .blocks
                    .iter()
                    .any(|b| matches!(b, Block::Demo { .. })),
                "{} sem demo visual",
                entry.id
            );
            assert!(
                lesson.blocks.iter().any(|b| matches!(
                    b,
                    Block::Quiz {
                        options,
                        correct,
                        ..
                    } if options.len() == 4 && *correct < options.len()
                )),
                "{} sem quiz válido de 4 opções",
                entry.id
            );
            match entry.kind {
                V3EntryKind::Theory => assert!(build_v3_practice_spec(entry).is_none()),
                V3EntryKind::Practice | V3EntryKind::CapstoneMilestone => {
                    practice_count += 1;
                    let spec = build_v3_practice_spec(entry)
                        .unwrap_or_else(|| panic!("{} sem PracticeSpec V3", entry.id));
                    assert!(
                        !spec.starter.trim().is_empty(),
                        "{} starter vazio",
                        entry.id
                    );
                    for h in &spec.hints {
                        assert!(!h.trim().is_empty(), "{} dica vazia", entry.id);
                    }
                }
            }
        }
        assert_eq!(practice_count, 89);
    }

    #[test]
    fn gating_e_preview_do_curriculo_v3_respeitam_prerequisitos() {
        let mut prog = crate::learn::Progress::default();
        let b_t01 = find_v3_entry("B-T01").unwrap();
        let b_t02 = find_v3_entry("B-T02").unwrap();
        let m01_t01 = find_v3_entry("M01-T01").unwrap();

        assert!(is_v3_stage_unlocked(&prog, V3Stage::Basico, false));
        assert!(!is_v3_stage_unlocked(&prog, V3Stage::Meio, false));
        assert!(is_v3_stage_unlocked(&prog, V3Stage::Meio, true));

        assert!(is_v3_entry_unlocked(&prog, b_t01, false));
        assert!(!is_v3_entry_unlocked(&prog, b_t02, false));
        assert!(!is_v3_entry_unlocked(&prog, m01_t01, false));
        assert!(is_v3_entry_unlocked(&prog, m01_t01, true));

        prog.completed.insert("B-T01".to_string());
        assert!(is_v3_entry_unlocked(&prog, b_t02, false));
        prog.exercises_completed.insert("B-P02".to_string());
        assert!(is_v3_stage_unlocked(&prog, V3Stage::Meio, false));
        assert!(is_v3_entry_unlocked(&prog, m01_t01, false));
    }

    #[test]
    fn blueprint_v3_em_ingles_cobre_estagios_entradas_licoes_e_praticas() {
        let cur_en = crate::curriculum::Curriculum::embedded_for(true).unwrap();

        for stage in V3Stage::ALL {
            assert!(!stage.label_for(true).trim().is_empty());
            assert!(!stage.subtitle_for(true).trim().is_empty());
        }
        assert_eq!(V3Stage::Basico.label_for(true), "THE BASICS");

        let entries = v3_curriculum_blueprint();
        assert_eq!(entries.len(), 213);

        let mut practice_count = 0usize;
        for entry in entries {
            assert!(!entry.title_for(true).trim().is_empty(), "{}", entry.id);
            assert!(
                !entry.learning_goal_for(true).trim().is_empty(),
                "{}",
                entry.id
            );
            assert!(
                !entry.completion_evidence_for(true).trim().is_empty(),
                "{}",
                entry.id
            );
            let lesson = build_v3_lesson_for(entry, &cur_en, true);
            assert_eq!(lesson.id, entry.id);
            assert!(!lesson.blocks.is_empty(), "{}", entry.id);
            if let Some(spec) = build_v3_practice_spec_for(entry, true) {
                practice_count += 1;
                assert!(!spec.title.trim().is_empty(), "{}", entry.id);
                assert!(!spec.prompt.trim().is_empty(), "{}", entry.id);
                assert!(!spec.hints.is_empty(), "{}", entry.id);
                assert!(!spec.starter.trim().is_empty(), "{}", entry.id);
            }
        }
        assert_eq!(practice_count, 89);
    }

    #[test]
    fn erro_primeiro_e_praticas_expandidas_v3_funcionam() {
        assert!(
            error_first_callout_for("ownership-na-pratica", true)
                .unwrap()
                .contains("E0382")
        );
        assert!(
            error_first_callout_for("ownership-na-pratica", false)
                .unwrap()
                .contains("E0382")
        );

        let cur = Curriculum::embedded_for(true).unwrap();
        let entry = find_v3_entry("M05-T01").unwrap(); // ownership theory -> error-first callout + on-demand practice
        let lesson = build_v3_lesson_for(entry, &cur, true);
        assert!(lesson.blocks.iter().any(|b| matches!(
            b,
            Block::Callout { text, .. } if text.contains("E0382")
        )));
        let any_spec = build_v3_practice_for_any_entry(entry, true);
        assert_eq!(any_spec.id, "M05-T01");
        assert!(!any_spec.starter.trim().is_empty());

        for id in [
            "D01-P02", "D02-P01", "D03-P02", "D12-P01", "F01-P02", "F06-P01",
        ] {
            let e = find_v3_entry(id).unwrap();
            let spec_pt = build_v3_practice_spec_for(e, false).unwrap();
            let spec_en = build_v3_practice_spec_for(e, true).unwrap();
            assert_ne!(spec_pt.prompt, spec_en.prompt, "{id}");
        }
    }
}
