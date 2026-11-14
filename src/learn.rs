//! Estado do modo Aprender: progresso do aluno, visualização da lição,
//! quizzes, demos e prática embutida com Cargo real. Toda a lógica aqui é pura
//! (sem terminal) para ser unit-testada — o desenho fica em `ui/browser.rs` e
//! `ui/lesson.rs`.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Screen, StatusKind};
use crate::config::ExecConsent;
use crate::curriculum::{Block, ChapterStatus, Lesson};
use crate::editor::Editor;
use crate::execution::{ExecEvent, StreamKind, ToolCommand};
use crate::i18n::t;

const MAX_PRACTICE_OUTPUT: usize = 300;
static PRACTICE_TEMP_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Estados emocionais e expressões canônicas do Ferris no Modo Aprender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FerrisMood {
    #[default]
    Neutral,
    Happy,
    Encouraging,
    Confused,
}

impl FerrisMood {
    /// Expressão ASCII exata do mascote para o estado atual.
    pub fn ascii(self) -> &'static str {
        match self {
            Self::Neutral => "c(o.o)c",
            Self::Happy => "c(ovo)c",
            Self::Encouraging => "c(o,x,o)c",
            Self::Confused => "c(o,0)c",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Neutral => "Ferris observando",
            Self::Happy => "Boa!",
            Self::Encouraging => "Quase lá!",
            Self::Confused => "Hmm?",
        }
    }

    pub fn caption(self) -> &'static str {
        match self {
            Self::Neutral => "Leia a lição e pratique abaixo.",
            Self::Happy => "Excelente! Você resolveu de verdade.",
            Self::Encouraging => "Todo erro ajuda a entender. Tente ajustar e rodar.",
            Self::Confused => "Isso parece incompleto ou inesperado. Confira o código.",
        }
    }
}

/// Cálculo canônico de XP da especificação V3 para desafios práticos:
/// 100 XP base + 100 XP bônus quando concluído sem dicas e sem auxílio de IA.
pub fn calculate_v3_practice_xp(hints_used: usize, ai_used: bool) -> (u32, u32) {
    if hints_used == 0 && !ai_used {
        (100, 100)
    } else {
        (100, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PracticeOutcome {
    NotRun,
    Running,
    Passed,
    CompileFailed,
    RunFailed,
    ValidationFailed,
    TimedOut,
    Cancelled,
    StartFailed,
}

impl PracticeOutcome {
    /// Rótulo compacto em português para cabeçalhos e leitura acessível (V3).
    pub const fn status_badge(self) -> &'static str {
        match self {
            Self::NotRun => "[Não verificado]",
            Self::Running => "[Executando]",
            Self::Passed => "[Aprovado]",
            Self::CompileFailed => "[Erro de compilação]",
            Self::RunFailed => "[Falha na execução]",
            Self::ValidationFailed => "[Validação pendente]",
            Self::TimedOut => "[Tempo esgotado]",
            Self::Cancelled => "[Cancelado]",
            Self::StartFailed => "[Falha ao iniciar]",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PracticeMode {
    CargoRun,
    CargoRunWithInput {
        stdin_lines: &'static [&'static str],
    },
    CargoTest {
        tests: &'static str,
    },
}

#[derive(Debug, Clone)]
pub struct PracticeSpec {
    pub id: &'static str,
    pub lesson_id: &'static str,
    pub title: &'static str,
    pub prompt: &'static str,
    pub starter: &'static str,
    pub hints: [&'static str; 4],
    pub mode: PracticeMode,
}

#[derive(Debug, Clone)]
pub struct LessonPractice {
    pub spec: PracticeSpec,
    pub editor: Editor,
    pub shown_hints: usize,
    pub hint_index: usize,
    pub hint_scroll: u16,
    pub ai_used: bool,
    pub outcome: PracticeOutcome,
    pub output: Vec<String>,
    pub stdout_lines: Vec<String>,
    pub output_scroll: usize,
    pub compile_error_seen: bool,
    pub consent_pending: bool,
    pub pending_dir: Option<PathBuf>,
}

impl LessonPractice {
    pub fn for_lesson(lesson_id: &str) -> Option<Self> {
        let spec = practice_spec_for_lesson(lesson_id)?;
        let editor = Editor::new(spec.starter, None);
        Some(Self {
            spec,
            editor,
            shown_hints: 0,
            hint_index: 0,
            hint_scroll: 0,
            ai_used: false,
            outcome: PracticeOutcome::NotRun,
            output: Vec::new(),
            stdout_lines: Vec::new(),
            output_scroll: 0,
            compile_error_seen: false,
            consent_pending: false,
            pending_dir: None,
        })
    }

    fn push_output(&mut self, line: String) {
        self.output.push(line);
        while self.output.len() > MAX_PRACTICE_OUTPUT {
            self.output.remove(0);
        }
    }
}

/// Explica erros comuns do `rustc` em português claro sem consumir dicas nem XP independente.
pub fn explain_compiler_error_pt_br(output: &str) -> Option<String> {
    const PATTERNS: &[(&[&str], &str)] = &[
        (
            &["E0384"],
            "Variável imutável alterada (`E0384`): em Rust bindings `let` são fixos por padrão; use `let mut` quando precisar reatribuir.",
        ),
        (
            &["E0308"],
            "Tipos incompatíveis (`E0308`): confira o tipo esperado e verifique se há um `;` sobrando na expressão de retorno.",
        ),
        (
            &["E0382"],
            "Valor usado após move (`E0382`): a posse foi transferida antes; empreste com `&` ou reordene os usos.",
        ),
        (
            &["E0502", "E0499"],
            "Conflito de empréstimo (`E0499`/`E0502`): não é possível manter `&mut` ativo junto de outras referências ao mesmo dado.",
        ),
        (
            &["E0597"],
            "Referência não vive o suficiente (`E0597`): o dado emprestado sai de escopo antes da referência parar de ser usada.",
        ),
        (
            &["E0425"],
            "Identificador não encontrado (`E0425`): confira ortografia, maiúsculas/minúsculas ou escopo da variável/função.",
        ),
        (
            &["E0106"],
            "Tempo de vida ausente (`E0106`): quando referências entram e retornam de uma função, conecte-as com `'a`.",
        ),
        (
            &["E0277"],
            "Trait não implementada (`E0277`): o tipo não implementa o comportamento exigido (como `Debug`, `Display` ou `Copy`).",
        ),
        (
            &["E0061"],
            "Quantidade incorreta de argumentos (`E0061`): confira quantos parâmetros a assinatura da função exige.",
        ),
        (
            &["unclosed delimiter"],
            "Delimitador aberto: alguma chave `{` ou parêntese `(` ficou sem fechar no arquivo.",
        ),
        (
            &["cannot find macro"],
            "Macro não encontrada: confira se escreveu `println!` com ponto de exclamação (`!`).",
        ),
        (
            &["expected `;`", "expected one of"],
            "Ponto e vírgula ou sintaxe incompleta: verifique `;`, aspas equilibradas ou parênteses faltando.",
        ),
    ];

    PATTERNS
        .iter()
        .find(|(keys, _)| keys.iter().any(|k| output.contains(k)))
        .map(|(_, msg)| (*msg).to_string())
}

/// Lente socrática e modo "Explicar como se eu tivesse 10 anos" (ELI10) para erros `rustc`.
pub fn explain_compiler_error_socratic_pt_br(output: &str, eli10: bool) -> Option<String> {
    if explain_compiler_error_pt_br(output).is_none() && !output.contains("error[E") {
        return None;
    }
    let lens = crate::lab_tools::SocraticErrorLens::analyze(output, "");
    let fix = lens.fixes[0];
    let body = if eli10 {
        format!(
            "[{}] Modo 10 Anos: {}\nExemplo idiomático ({}): Antes `{}` -> Depois `{}`",
            lens.code,
            lens.eli10_metaphor,
            fix.label,
            fix.before.replace('\n', " "),
            fix.after.replace('\n', " "),
        )
    } else {
        format!(
            "[{}] {} — Pergunta: {} (Alt+E alterna Modo 10 Anos)",
            lens.code, lens.title, lens.socratic_question
        )
    };
    Some(body)
}

pub fn practice_spec_by_id(spec_id: &str) -> Option<PracticeSpec> {
    if spec_id == "D09-P01" {
        return Some(interactive_practice_spec_d09());
    }
    ALL_PRACTICE_SPECS
        .iter()
        .find(|s| s.id == spec_id)
        .cloned()
        .or_else(|| {
            crate::curriculum::v3_blueprint::find_v3_entry(spec_id)
                .and_then(crate::curriculum::v3_blueprint::build_v3_practice_spec)
        })
}

pub fn practice_spec_for_lesson(lesson_id: &str) -> Option<PracticeSpec> {
    ALL_PRACTICE_SPECS
        .iter()
        .find(|s| s.lesson_id == lesson_id)
        .cloned()
        .or_else(|| {
            crate::curriculum::v3_blueprint::find_v3_entry(lesson_id)
                .and_then(crate::curriculum::v3_blueprint::build_v3_practice_spec)
        })
}

/// Especificação prática interativa V3 (`D09-P01 · Menu terminal navegável` via `stdin`).
pub fn interactive_practice_spec_d09() -> PracticeSpec {
    PracticeSpec {
        id: "D09-P01",
        lesson_id: "cli-configuracao-e-serde",
        title: "Prática D09-P01 · Menu terminal interativo (stdin)",
        prompt: "Leia opções de `stdin` em loop: `\"1\"` imprime `Ajuda`, `\"2\"` imprime `Pontos: 10` e `\"0\"` imprime `Saindo` e encerra (`F5`).",
        starter: "use std::io::{self, BufRead};\n\nfn main() {\n    for linha in io::stdin().lock().lines() {\n        let op = linha.unwrap_or_default();\n        match op.trim() {\n            // Complete os ramos \"1\", \"2\" e \"0\"\n            _ => break,\n        }\n    }\n}\n",
        hints: [
            "Nível 1: Use `match op.trim()` dentro do laço sobre `io::stdin().lock().lines()`.",
            "Nível 2: Para `\"1\"`, imprima `println!(\"Ajuda\");`; para `\"2\"`, imprima `println!(\"Pontos: 10\");`.",
            "Nível 3: Para `\"0\"`, imprima `println!(\"Saindo\");` e execute `break;`.",
            "Nível 4: Ao rodar com `F5`, o RustLab envia as linhas de teste pelo `stdin` e fecha com EOF.",
        ],
        mode: PracticeMode::CargoRunWithInput {
            stdin_lines: &["2", "0"],
        },
    }
}

const ALL_PRACTICE_SPECS: &[PracticeSpec] = &[
    PracticeSpec {
        id: "B-T01-P",
        lesson_id: "o-que-e-programar",
        title: "Prática B-T01-P · Passos precisos em Rust",
        prompt: "Troque `\"...\"` por uma frase descrevendo entrada, processamento e saída no `println!` e rode com `F5`.",
        starter: "fn main() {\n    // Exemplo: Entrada (nome) -> Processamento (saudação) -> Saída\n    println!(\"...\");\n}\n",
        hints: [
            "Nível 1: Programas transformam entradas em saídas seguindo passos exatos.",
            "Nível 2: Substitua os três pontinhos dentro de `println!(\"...\");` por uma frase real.",
            "Nível 3: Mantenha as aspas duplas e o `;` no final da linha.",
            "Nível 4: Escreva ao menos 8 caracteres reais descrevendo seu fluxo e pressione `F5`.",
        ],
        mode: PracticeMode::CargoRun,
    },
    PracticeSpec {
        id: "B-T02-P",
        lesson_id: "como-um-programa-executa",
        title: "Prática B-T02-P · Do fonte ao executável",
        prompt: "Imprima uma linha citando como o compilador gera o executável e rode com `F5`.",
        starter: "fn main() {\n    println!(\"...\");\n}\n",
        hints: [
            "Nível 1: Lembre das etapas: código-fonte, compilador e executável.",
            "Nível 2: Use `println!` para escrever uma frase mencionando `compilador` ou `executável`.",
            "Nível 3: Não deixe apenas `...` dentro das aspas.",
            "Nível 4: Escreva uma frase explicando o papel do compilador e rode com `F5`.",
        ],
        mode: PracticeMode::CargoRun,
    },
    PracticeSpec {
        id: "B-T04-P",
        lesson_id: "conhecendo-o-rust",
        title: "Prática B-T04-P · Ferramentas rustc e Cargo",
        prompt: "Escreva um `println!` citando o `cargo` ou o `rustc` e rode com `F5`.",
        starter: "fn main() {\n    println!(\"...\");\n}\n",
        hints: [
            "Nível 1: O compilador do Rust é `rustc`, e o gerenciador é `cargo`.",
            "Nível 2: Coloque uma frase curta dentro de `println!` contendo `cargo` ou `rustc`.",
            "Nível 3: Confira que as aspas e o `;` estão corretos.",
            "Nível 4: Substitua `...` por uma frase citando Cargo e pressione `F5`.",
        ],
        mode: PracticeMode::CargoRun,
    },
    PracticeSpec {
        id: "B-P01",
        lesson_id: "hello-world",
        title: "Prática B-P01 · Primeiro println! com Cargo",
        prompt: "Substitua o placeholder `\"...\"` por uma mensagem sua em português usando `println!`, salve e rode com `F5`.",
        starter: "fn main() {\n    // Escreva sua mensagem abaixo e aperte F5 para compilar e rodar\n    println!(\"...\");\n}\n",
        hints: [
            "Nível 1 (Direção): A macro `println!(\"...\");` imprime uma linha inteira no terminal.",
            "Nível 2 (Estrutura): Mantenha `fn main() { ... }` e altere apenas o texto dentro das aspas de `println!`.",
            "Nível 3 (Armadilha): Confira as aspas duplas fechadas e o ponto e vírgula `;` ao final da linha.",
            "Nível 4 (Esboço): Coloque uma saudação real diferente de `...` dentro de `println!`, depois pressione `Ctrl+S` e `F5`.",
        ],
        mode: PracticeMode::CargoRun,
    },
    PracticeSpec {
        id: "B-T06-P",
        lesson_id: "anatomia-do-hello-world",
        title: "Prática B-T06-P · Duas instruções em main",
        prompt: "Adicione uma segunda chamada `println!` dentro de `fn main()` para imprimir ao menos duas linhas (`F5`).",
        starter: "fn main() {\n    println!(\"linha 1\");\n}\n",
        hints: [
            "Nível 1: Cada instrução sequencial em `fn main()` termina com `;`.",
            "Nível 2: Escreva outra linha `println!(\"...\");` logo abaixo da primeira.",
            "Nível 3: Garanta que a segunda chamada está antes da chave `}` de fechamento.",
            "Nível 4: Tenha duas linhas impressas diferentes de vazio ao rodar `F5`.",
        ],
        mode: PracticeMode::CargoRun,
    },
    PracticeSpec {
        id: "B-P02",
        lesson_id: "variaveis",
        title: "Prática B-P02 · Variáveis e mutabilidade com Cargo",
        prompt: "Atualize a variável mutável `pontos` para um valor diferente de 0 antes do `println!` e rode com `F5`.",
        starter: "fn main() {\n    let curso = \"RustLab\";\n    let mut pontos = 0;\n    // Atualize `pontos` antes de imprimir\n    println!(\"{curso}: {pontos}\");\n}\n",
        hints: [
            "Nível 1 (Direção): Em Rust, `let mut` libera reatribuição da variável após a criação.",
            "Nível 2 (Estrutura): Adicione uma linha antes de `println!` para somar ou atribuir um novo valor a `pontos`.",
            "Nível 3 (Armadilha): Não reescreva `let pontos = ...` sem `mut` se quiser apenas atualizar a variável existente.",
            "Nível 4 (Esboço): Entre a declaração e a impressão, atualize `pontos` para sair de `0` e pressione `F5`.",
        ],
        mode: PracticeMode::CargoRun,
    },
    PracticeSpec {
        id: "B-T09-P",
        lesson_id: "mutabilidade",
        title: "Prática B-T09-P · Mutabilidade controlada (cargo test)",
        prompt: "Implemente `incrementar(mut valor: i32, passo: i32) -> i32` somando `passo` em `valor` e retornando o total (`F5`).",
        starter: "pub fn incrementar(mut valor: i32, passo: i32) -> i32 {\n    let _ = (&mut valor, passo);\n    todo!(\"atualize valor com passo e retorne\")\n}\n",
        hints: [
            "Nível 1: Parâmetros marcados com `mut` podem receber novos valores dentro da função.",
            "Nível 2: Some `passo` a `valor` com `valor += passo;` e devolva `valor`.",
            "Nível 3: Não termine a última expressão de retorno com `;`.",
            "Nível 4: Remova `todo!` e retorne `valor + passo` após atualizar `valor`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::incrementar;\n#[test]\nfn testa_incrementar() {\n    assert_eq!(incrementar(10, 5), 15);\n    assert_eq!(incrementar(-3, 3), 0);\n    assert_eq!(incrementar(0, -4), -4);\n}\n",
        },
    },
    PracticeSpec {
        id: "M01-T01-P",
        lesson_id: "tipos-numericos",
        title: "Prática M01-T01-P · Ponto flutuante f64",
        prompt: "Implemente `dobro_decimal(x: f64) -> f64` devolvendo o dobro de `x` (`F5`).",
        starter: "pub fn dobro_decimal(x: f64) -> f64 {\n    let _ = x;\n    todo!(\"multiplique x por 2.0\")\n}\n",
        hints: [
            "Nível 1: Em Rust não há conversão implícita entre `i32` e `f64`.",
            "Nível 2: Use literal decimal `2.0` ao operar com `f64`.",
            "Nível 3: Evite `2` inteiro; escreva `x * 2.0` sem `;`.",
            "Nível 4: A função pode ser resolvida em uma expressão devolvendo o produto por `2.0`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::dobro_decimal;\n#[test]\nfn testa_dobro_decimal() {\n    assert!((dobro_decimal(2.5) - 5.0).abs() < 1e-9);\n    assert!((dobro_decimal(-1.5) + 3.0).abs() < 1e-9);\n}\n",
        },
    },
    PracticeSpec {
        id: "M01-P01",
        lesson_id: "operadores",
        title: "Prática M01-P01 · Precedência de operadores",
        prompt: "Implemente `cartao_pontos(base: i32, bonus: i32) -> i32` calculando `(base + bonus) * 2` (`F5`).",
        starter: "pub fn cartao_pontos(base: i32, bonus: i32) -> i32 {\n    let _ = (base, bonus);\n    todo!(\"retorne (base + bonus) * 2\")\n}\n",
        hints: [
            "Nível 1: Multiplicação tem precedência sobre adição.",
            "Nível 2: Parênteses explícitos forçam a soma antes da multiplicação.",
            "Nível 3: Sem parênteses, `base + bonus * 2` produz resultado errado.",
            "Nível 4: Escreva `(base + bonus) * 2` como expressão de retorno.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::cartao_pontos;\n#[test]\nfn testa_cartao_pontos() {\n    assert_eq!(cartao_pontos(10, 5), 30);\n    assert_eq!(cartao_pontos(0, 0), 0);\n    assert_eq!(cartao_pontos(-2, 7), 10);\n}\n",
        },
    },
    PracticeSpec {
        id: "M02-T01-P",
        lesson_id: "condicoes-lacos-e-funcoes",
        title: "Prática M02-T01-P · Expressões if/else",
        prompt: "Implemente `classificar_temperatura(graus: i32) -> &'static str`: `< 15 => \"frio\"`, `15..=25 => \"ameno\"`, `> 25 => \"quente\"` (`F5`).",
        starter: "pub fn classificar_temperatura(graus: i32) -> &'static str {\n    let _ = graus;\n    todo!(\"retorne frio, ameno ou quente\")\n}\n",
        hints: [
            "Nível 1: `if / else if / else` em Rust é uma expressão que devolve valor.",
            "Nível 2: Teste primeiro `< 15`, depois `<= 25`, e deixe o `else` para `\"quente\"`.",
            "Nível 3: Todos os ramos precisam retornar o mesmo tipo `&'static str`.",
            "Nível 4: Confira limites exatos de 15 e 25 para cair em `\"ameno\"`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::classificar_temperatura;\n#[test]\nfn testa_temperatura() {\n    assert_eq!(classificar_temperatura(10), \"frio\");\n    assert_eq!(classificar_temperatura(15), \"ameno\");\n    assert_eq!(classificar_temperatura(25), \"ameno\");\n    assert_eq!(classificar_temperatura(30), \"quente\");\n}\n",
        },
    },
    PracticeSpec {
        id: "M07-P01",
        lesson_id: "strings-e-fatias",
        title: "Prática M07-P01 · Strings e fatias UTF-8",
        prompt: "Implemente `saudar_usuario(nome: &str) -> String`: remova espaços com `.trim()`; se vazio devolva `\"Olá, visitante!\"`, senão `\"Olá, <nome>!\"` (`F5`).",
        starter: "pub fn saudar_usuario(nome: &str) -> String {\n    let _ = nome;\n    todo!(\"trate nome vazio e formate saudação\")\n}\n",
        hints: [
            "Nível 1: Use `.trim()` em `&str` para limpar espaços nas pontas sem alocar.",
            "Nível 2: Verifique `limpo.is_empty()` antes de formatar.",
            "Nível 3: A função retorna `String` com posse (`format!(...)` ou `to_string()`).",
            "Nível 4: Retorne `\"Olá, visitante!\".to_string()` quando vazio ou `format!(\"Olá, {limpo}!\")`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::saudar_usuario;\n#[test]\nfn testa_saudacao() {\n    assert_eq!(saudar_usuario(\"  Ana  \"), \"Olá, Ana!\");\n    assert_eq!(saudar_usuario(\"   \"), \"Olá, visitante!\");\n}\n",
        },
    },
    PracticeSpec {
        id: "M08-P01",
        lesson_id: "tuplas-arrays-e-vetores",
        title: "Prática M08-P01 · Slice para tupla (min, max)",
        prompt: "Implemente `extremos_array(valores: &[i32]) -> Option<(i32, i32)>` retornando `None` se vazio ou `Some((menor, maior))` (`F5`).",
        starter: "pub fn extremos_array(valores: &[i32]) -> Option<(i32, i32)> {\n    let _ = valores;\n    todo!(\"retorne Some((min, max)) ou None\")\n}\n",
        hints: [
            "Nível 1: Nunca indexe `valores[0]` sem garantir que não está vazio.",
            "Nível 2: Desestruture `let (&primeiro, resto) = valores.split_first()?;` ou cheque `is_empty()`.",
            "Nível 3: Atualize `min = min.min(v)` e `max = max.max(v)` no laço.",
            "Nível 4: Envolva `(min, max)` em `Some((min, max))` ao final.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::extremos_array;\n#[test]\nfn testa_extremos() {\n    assert_eq!(extremos_array(&[]), None);\n    assert_eq!(extremos_array(&[4]), Some((4, 4)));\n    assert_eq!(extremos_array(&[7, -2, 9, 1]), Some((-2, 9)));\n}\n",
        },
    },
    PracticeSpec {
        id: "M09-P01",
        lesson_id: "colecoes-e-iteracao",
        title: "Prática M09-P01 · Soma seletiva em iteração",
        prompt: "Implemente `somar_positivos(lista: &[i32]) -> i32` somando apenas números estritamente maiores que zero (`F5`).",
        starter: "pub fn somar_positivos(lista: &[i32]) -> i32 {\n    let _ = lista;\n    todo!(\"filtre positivos e some\")\n}\n",
        hints: [
            "Nível 1: Você pode iterar por `lista.iter().copied()` ou com `for &n in lista`.",
            "Nível 2: Considere `n > 0`; zero e negativos devem ser ignorados.",
            "Nível 3: Usando iteradores: `lista.iter().copied().filter(|&n| n > 0).sum()`.",
            "Nível 4: Se a lista só tiver negativos ou for vazia, retorne `0`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::somar_positivos;\n#[test]\nfn testa_positivos() {\n    assert_eq!(somar_positivos(&[-5, 0, 3, 7, -1]), 10);\n    assert_eq!(somar_positivos(&[-2, 0]), 0);\n}\n",
        },
    },
    PracticeSpec {
        id: "M05-P01",
        lesson_id: "ownership-na-pratica",
        title: "Prática M05-P01 · Transferência e retorno de String",
        prompt: "Implemente `anexar_sufixo(mut texto: String, sufixo: &str) -> String` anexando o sufixo e devolvendo a posse da `String` (`F5`).",
        starter: "pub fn anexar_sufixo(mut texto: String, sufixo: &str) -> String {\n    let _ = (&mut texto, sufixo);\n    todo!(\"anexe sufixo a texto e retorne texto\")\n}\n",
        hints: [
            "Nível 1: A função recebeu posse de `texto` e referência emprestada de `sufixo`.",
            "Nível 2: O método `texto.push_str(sufixo);` modifica a `String` no lugar sem clonar.",
            "Nível 3: Ao final, devolva `texto` sem `;` para transferir posse de volta.",
            "Nível 4: Remova `todo!` após `texto.push_str(sufixo);` e escreva `texto`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::anexar_sufixo;\n#[test]\nfn testa_anexar() {\n    assert_eq!(anexar_sufixo(String::from(\"Rust\"), \"Lab\"), \"RustLab\");\n}\n",
        },
    },
    PracticeSpec {
        id: "M06-P01",
        lesson_id: "borrowing-e-referencias",
        title: "Prática M06-P01 · Leitura emprestada Unicode",
        prompt: "Implemente `contar_caracteres(texto: &str) -> usize` contando caracteres Unicode (não bytes!) via `&str` (`F5`).",
        starter: "pub fn contar_caracteres(texto: &str) -> usize {\n    let _ = texto;\n    todo!(\"conte os chars Unicode de texto\")\n}\n",
        hints: [
            "Nível 1: `texto.len()` devolve bytes UTF-8, não letras/caracteres.",
            "Nível 2: O método `.chars()` itera sobre escalares Unicode de `&str`.",
            "Nível 3: Encadeie `.count()` ao iterador de caracteres.",
            "Nível 4: Retorne `texto.chars().count()` sem mover a string original.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::contar_caracteres;\n#[test]\nfn testa_chars() {\n    assert_eq!(contar_caracteres(\"olá!\"), 4);\n    assert_eq!(contar_caracteres(\"c(ovo)c\"), 7);\n}\n",
        },
    },
    PracticeSpec {
        id: "D01-P01",
        lesson_id: "tempo-de-vida-introducao",
        title: "Prática D01-P01 · Anotação de lifetime 'a",
        prompt: "Implemente `maior_fatia<'a>(a: &'a str, b: &'a str) -> &'a str` devolvendo a fatia com mais bytes (em empate, retorne `a`) (`F5`).",
        starter: "pub fn maior_fatia<'a>(a: &'a str, b: &'a str) -> &'a str {\n    let _ = (a, b);\n    todo!(\"compare a.len() e b.len()\")\n}\n",
        hints: [
            "Nível 1: O lifetime `'a` já garante que o retorno vive no máximo quanto o menor entre `a` e `b`.",
            "Nível 2: Use `if a.len() >= b.len() { a } else { b }`.",
            "Nível 3: Repare no `>=` para respeitar a regra de desempate a favor de `a`.",
            "Nível 4: Não precisa alocar `String`; devolva diretamente `a` ou `b`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::maior_fatia;\n#[test]\nfn testa_lifetime() {\n    assert_eq!(maior_fatia(\"curta\", \"gigante\"), \"gigante\");\n    assert_eq!(maior_fatia(\"ab\", \"xy\"), \"ab\");\n}\n",
        },
    },
    PracticeSpec {
        id: "M10-P01",
        lesson_id: "structs-e-metodos",
        title: "Prática M10-P01 · Struct Contador e métodos",
        prompt: "Implemente em `Contador`: `novo(inicio: i32) -> Self` e `avancar(&mut self) -> i32` que soma 1 e retorna o novo valor (`F5`).",
        starter: "pub struct Contador {\n    atual: i32,\n}\n\nimpl Contador {\n    pub fn novo(inicio: i32) -> Self {\n        let _ = inicio;\n        todo!(\"crie Contador com atual = inicio\")\n    }\n    pub fn avancar(&mut self) -> i32 {\n        todo!(\"incremente self.atual e retorne\")\n    }\n}\n",
        hints: [
            "Nível 1: Métodos construtores retornam `Self { campo }` sem `&self`.",
            "Nível 2: Métodos que alteram campos pedem `&mut self`.",
            "Nível 3: Em `avancar`, faça `self.atual += 1; self.atual`.",
            "Nível 4: Confira a visibilidade `pub` nos dois métodos.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::Contador;\n#[test]\nfn testa_contador() {\n    let mut c = Contador::novo(10);\n    assert_eq!(c.avancar(), 11);\n    assert_eq!(c.avancar(), 12);\n}\n",
        },
    },
    PracticeSpec {
        id: "M11-P01",
        lesson_id: "enums-e-pattern-matching",
        title: "Prática M11-P01 · Enum e match exaustivo",
        prompt: "Implemente `executar(cmd: Comando) -> i32` tratando `Somar(a, b)`, `Negar(x)` e `Zero` via `match` (`F5`).",
        starter: "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum Comando {\n    Somar(i32, i32),\n    Negar(i32),\n    Zero,\n}\n\npub fn executar(cmd: Comando) -> i32 {\n    let _ = cmd;\n    todo!(\"faça match em Comando\")\n}\n",
        hints: [
            "Nível 1: `match` em Rust exige cobrir todas as variantes do enum.",
            "Nível 2: Desestruture `Comando::Somar(a, b) => a + b`.",
            "Nível 3: Em `Comando::Negar(x)` retorne `-x`; em `Comando::Zero` retorne `0`.",
            "Nível 4: Não use curinga `_` desnecessário; nomeie as três variantes.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::{executar, Comando};\n#[test]\nfn testa_comando() {\n    assert_eq!(executar(Comando::Somar(4, 6)), 10);\n    assert_eq!(executar(Comando::Negar(7)), -7);\n    assert_eq!(executar(Comando::Zero), 0);\n}\n",
        },
    },
    PracticeSpec {
        id: "D03-P01",
        lesson_id: "traits-modulos-e-cargo",
        title: "Prática D03-P01 · Trait Resumo",
        prompt: "Implemente `Resumo` para `Artigo` retornando `\"<titulo> — <autor>\"` (`F5`).",
        starter: "pub trait Resumo {\n    fn resumir(&self) -> String;\n}\n\npub struct Artigo {\n    pub titulo: String,\n    pub autor: String,\n}\n\nimpl Resumo for Artigo {\n    fn resumir(&self) -> String {\n        todo!(\"formate titulo e autor separados por ' — '\")\n    }\n}\n",
        hints: [
            "Nível 1: O bloco `impl Trait for Struct` cumpre o contrato da trait.",
            "Nível 2: Use `&self.titulo` e `&self.autor` dentro de `format!`.",
            "Nível 3: Atenção ao travessão ` — ` com espaços ao redor conforme o enunciado.",
            "Nível 4: Escreva `format!(\"{} — {}\", self.titulo, self.autor)`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::{Artigo, Resumo};\n#[test]\nfn testa_resumo() {\n    let a = Artigo { titulo: \"Rust V3\".into(), autor: \"Ferris\".into() };\n    assert_eq!(a.resumir(), \"Rust V3 — Ferris\");\n}\n",
        },
    },
    PracticeSpec {
        id: "M12-T01-P",
        lesson_id: "option-e-ausencia",
        title: "Prática M12-T01-P · Ausência segura com Option",
        prompt: "Implemente `primeiro_par(nums: &[i32]) -> Option<i32>` devolvendo o primeiro número par encontrado ou `None` (`F5`).",
        starter: "pub fn primeiro_par(nums: &[i32]) -> Option<i32> {\n    let _ = nums;\n    todo!(\"encontre o primeiro número par\")\n}\n",
        hints: [
            "Nível 1: Em Rust, `Option<T>` substitui nulos com `Some(valor)` ou `None`.",
            "Nível 2: Um número inteiro é par quando `n % 2 == 0`.",
            "Nível 3: Use `nums.iter().copied().find(|n| n % 2 == 0)`.",
            "Nível 4: Retorne diretamente o resultado de `.find(...)`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::primeiro_par;\n#[test]\nfn testa_primeiro_par() {\n    assert_eq!(primeiro_par(&[1, 3, 8, 10]), Some(8));\n    assert_eq!(primeiro_par(&[1, 3, 5]), None);\n}\n",
        },
    },
    PracticeSpec {
        id: "M12-P01",
        lesson_id: "result-e-propagacao",
        title: "Prática M12-P01 · Erros tipados com Result",
        prompt: "Implemente `parse_positivo(texto: &str) -> Result<i32, String>`: rejeite texto inválido ou números `<= 0` com `Err(...)` (`F5`).",
        starter: "pub fn parse_positivo(texto: &str) -> Result<i32, String> {\n    let _ = texto;\n    todo!(\"faça parse e valide > 0\")\n}\n",
        hints: [
            "Nível 1: Use `.trim().parse::<i32>()` convertendo o erro via `.map_err(...)` e operador `?`.",
            "Nível 2: Após o parse bem-sucedido, cheque se `n > 0`.",
            "Nível 3: Números `0` ou negativos devem devolver `Err(String)`.",
            "Nível 4: Se `n > 0`, devolva `Ok(n)`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::parse_positivo;\n#[test]\nfn testa_parse_positivo() {\n    assert_eq!(parse_positivo(\" 42 \"), Ok(42));\n    assert!(parse_positivo(\"0\").is_err());\n    assert!(parse_positivo(\"-5\").is_err());\n    assert!(parse_positivo(\"abc\").is_err());\n}\n",
        },
    },
    PracticeSpec {
        id: "D04-P01",
        lesson_id: "testes-documentacao-e-depuracao",
        title: "Prática D04-P01 · Função pura testável (palíndromo)",
        prompt: "Implemente `eh_palindromo(s: &str) -> bool` ignorando espaços e diferenças entre maiúsculas e minúsculas ASCII (`F5`).",
        starter: "pub fn eh_palindromo(s: &str) -> bool {\n    let _ = s;\n    todo!(\"verifique se s é palíndromo\")\n}\n",
        hints: [
            "Nível 1: Colete apenas os caracteres diferentes de espaço convertidos para minúsculo.",
            "Nível 2: Compare o `Vec<char>` normalizado com sua versão invertida.",
            "Nível 3: `c.to_ascii_lowercase()` trata letras ASCII sem complicação.",
            "Nível 4: Strings vazias ou só com espaços são palíndromos triviais (`true`).",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::eh_palindromo;\n#[test]\nfn testa_palindromo() {\n    assert!(eh_palindromo(\"Arara\"));\n    assert!(eh_palindromo(\"A sacada da casa\"));\n    assert!(!eh_palindromo(\"RustLab\"));\n}\n",
        },
    },
    PracticeSpec {
        id: "D05-P01",
        lesson_id: "genericos-e-closures",
        title: "Prática D05-P01 · Genéricos e closures Fn",
        prompt: "Implemente `aplicar_duas_vezes<F: Fn(i32) -> i32>(x: i32, f: F) -> i32` aplicando `f` duas vezes em `x` (`F5`).",
        starter: "pub fn aplicar_duas_vezes<F: Fn(i32) -> i32>(x: i32, f: F) -> i32 {\n    let _ = (x, f);\n    todo!(\"aplique f em f(x)\")\n}\n",
        hints: [
            "Nível 1: Um parâmetro genérico com bound `Fn(i32) -> i32` pode ser chamado como função comum.",
            "Nível 2: Calcule o primeiro passo `let primeiro = f(x);`.",
            "Nível 3: Passe `primeiro` para `f` novamente.",
            "Nível 4: Tudo se resume a `f(f(x))`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::aplicar_duas_vezes;\n#[test]\nfn testa_duas_vezes() {\n    assert_eq!(aplicar_duas_vezes(3, |n| n + 2), 7);\n    assert_eq!(aplicar_duas_vezes(2, |n| n * 3), 18);\n}\n",
        },
    },
    PracticeSpec {
        id: "D06-P01",
        lesson_id: "iteradores-e-arquivos",
        title: "Prática D06-P01 · Pipeline de iteradores",
        prompt: "Implemente `quadrados_impares(limite: i32) -> Vec<i32>` retornando os quadrados dos ímpares de `1..=limite` (`F5`).",
        starter: "pub fn quadrados_impares(limite: i32) -> Vec<i32> {\n    let _ = limite;\n    todo!(\"gere quadrados dos ímpares em 1..=limite\")\n}\n",
        hints: [
            "Nível 1: Se `limite < 1`, o intervalo `1..=limite` fica vazio naturalmente.",
            "Nível 2: Encadeie `.filter(|n| n % 2 != 0)` no range `1..=limite`.",
            "Nível 3: Transforme cada elemento com `.map(|n| n * n)`.",
            "Nível 4: Finalize com `.collect()` para produzir `Vec<i32>`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::quadrados_impares;\n#[test]\nfn testa_quadrados() {\n    assert_eq!(quadrados_impares(0), Vec::<i32>::new());\n    assert_eq!(quadrados_impares(6), vec![1, 9, 25]);\n}\n",
        },
    },
    PracticeSpec {
        id: "F04-P01",
        lesson_id: "cli-configuracao-e-serde",
        title: "Prática F04-P01 · Parser de chave=valor CLI",
        prompt: "Implemente `parse_chave_valor(linha: &str) -> Option<(&str, &str)>` dividindo no primeiro `=`, removendo espaços e rejeitando chave vazia (`F5`).",
        starter: "pub fn parse_chave_valor(linha: &str) -> Option<(&str, &str)> {\n    let _ = linha;\n    todo!(\"divida linha em = e limpe espaços\")\n}\n",
        hints: [
            "Nível 1: Use `.split_once('=')` para dividir exatamente no primeiro `=`.",
            "Nível 2: Aplique `.trim()` na chave e no valor resultantes.",
            "Nível 3: Se a chave estiver vazia após `trim()`, devolva `None`.",
            "Nível 4: Valores com `=` internos continuam intactos usando `split_once`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::parse_chave_valor;\n#[test]\nfn testa_kv() {\n    assert_eq!(parse_chave_valor(\" tema = cuia \"), Some((\"tema\", \"cuia\")));\n    assert_eq!(parse_chave_valor(\"url=http://a?x=1\"), Some((\"url\", \"http://a?x=1\")));\n    assert_eq!(parse_chave_valor(\" =123\"), None);\n    assert_eq!(parse_chave_valor(\"sem_igual\"), None);\n}\n",
        },
    },
    PracticeSpec {
        id: "F01-P01",
        lesson_id: "rede-e-programacao-assincrona",
        title: "Prática F01-P01 · Linha de requisição de rede",
        prompt: "Implemente `formatar_requisicao(metodo: &str, rota: &str) -> String` gerando `\"<METODO_MAIUSCULO> <rota> HTTP/1.1\"` (`F5`).",
        starter: "pub fn formatar_requisicao(metodo: &str, rota: &str) -> String {\n    let _ = (metodo, rota);\n    todo!(\"formate requisição HTTP/1.1\")\n}\n",
        hints: [
            "Nível 1: Normalize espaços com `.trim()` antes de montar a string.",
            "Nível 2: Converta o método HTTP para maiúsculas via `.to_uppercase()`.",
            "Nível 3: Monte `format!(\"{metodo} {rota} HTTP/1.1\")`.",
            "Nível 4: Não esqueça do sufixo exato `HTTP/1.1`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::formatar_requisicao;\n#[test]\nfn testa_req() {\n    assert_eq!(formatar_requisicao(\" get \", \" /status \"), \"GET /status HTTP/1.1\");\n}\n",
        },
    },
    PracticeSpec {
        id: "M02-P01",
        lesson_id: "projetos-guiados-calculadora-e-jogo",
        title: "Prática M02-P01 · Núcleo de calculadora segura",
        prompt: "Implemente `calcular(a: i32, op: char, b: i32) -> Option<i32>` para `+`, `-`, `*`, `/`, devolvendo `None` em divisão por 0 ou operador inválido (`F5`).",
        starter: "pub fn calcular(a: i32, op: char, b: i32) -> Option<i32> {\n    let _ = (a, op, b);\n    todo!(\"trate +, -, * e / sem panic\")\n}\n",
        hints: [
            "Nível 1: Use `match op` para tratar os quatro operadores.",
            "Nível 2: Use `checked_div(b)` ou guarde `if b == 0 { return None; }` na divisão.",
            "Nível 3: Operadores desconhecidos devem cair em `_ => None`.",
            "Nível 4: `a.checked_add(b)` / `checked_sub` / `checked_mul` tornam tudo ainda mais seguro.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::calcular;\n#[test]\nfn testa_calc() {\n    assert_eq!(calcular(8, '+', 2), Some(10));\n    assert_eq!(calcular(8, '/', 2), Some(4));\n    assert_eq!(calcular(8, '/', 0), None);\n    assert_eq!(calcular(8, '%', 2), None);\n}\n",
        },
    },
    PracticeSpec {
        id: "D10-P01",
        lesson_id: "projetos-guiados-tarefas-e-notas",
        title: "Prática D10-P01 · Filtro de tarefas concluídas",
        prompt: "Implemente `filtrar_concluidas(tarefas: &[(&str, bool)]) -> Vec<String>` retornando apenas os títulos marcados com `true` (`F5`).",
        starter: "pub fn filtrar_concluidas(tarefas: &[(&str, bool)]) -> Vec<String> {\n    let _ = tarefas;\n    todo!(\"devolva títulos das tarefas concluídas\")\n}\n",
        hints: [
            "Nível 1: Cada item da fatia é uma tupla `(titulo, concluida)`.",
            "Nível 2: Filtre pelos pares em que o booleano seja `true`.",
            "Nível 3: Converta cada título `&str` para `String` com `.to_string()`.",
            "Nível 4: Mantenha a ordem original das tarefas concluídas.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::filtrar_concluidas;\n#[test]\nfn testa_tarefas() {\n    let list = [(\"Ler\", true), (\"Codar\", false), (\"Testar\", true)];\n    assert_eq!(filtrar_concluidas(&list), vec![\"Ler\".to_string(), \"Testar\".to_string()]);\n}\n",
        },
    },
    PracticeSpec {
        id: "PAINT-01-P",
        lesson_id: "projeto-independente-planejamento",
        title: "Prática PAINT-01-P · Dimensão lógica do Mini Paint",
        prompt: "Implemente `CanvasSize::new(width: u16, height: u16) -> Option<Self>` aceitando somente dimensões `1..=2048` (`F5`).",
        starter: "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct CanvasSize {\n    pub width: u16,\n    pub height: u16,\n}\n\nimpl CanvasSize {\n    pub fn new(width: u16, height: u16) -> Option<Self> {\n        let _ = (width, height);\n        todo!(\"valide dimensões > 0 e <= 2048\")\n    }\n}\n",
        hints: [
            "Nível 1: Um canvas lógico nunca pode ter largura ou altura 0 nem estourar memória.",
            "Nível 2: Cheque `(1..=2048).contains(&width) && (1..=2048).contains(&height)`.",
            "Nível 3: Retorne `None` se qualquer eixo estiver fora da faixa.",
            "Nível 4: Quando válido, retorne `Some(Self { width, height })`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::CanvasSize;\n#[test]\nfn testa_canvas_size() {\n    assert!(CanvasSize::new(0, 64).is_none());\n    assert!(CanvasSize::new(64, 2049).is_none());\n    assert_eq!(CanvasSize::new(320, 240), Some(CanvasSize { width: 320, height: 240 }));\n}\n",
        },
    },
    PracticeSpec {
        id: "CAVE-01-P",
        lesson_id: "projeto-independente-qualidade-e-entrega",
        title: "Prática CAVE-01-P · Coordenadas de chunk 16x16 (CaveGame)",
        prompt: "Implemente `chunk_coords(x: i32, z: i32) -> (i32, i32)` mapeando coordenadas globais para chunks `16x16` (incluindo negativos!) (`F5`).",
        starter: "pub fn chunk_coords(x: i32, z: i32) -> (i32, i32) {\n    let _ = (x, z);\n    todo!(\"use divisão euclidiana por 16\")\n}\n",
        hints: [
            "Nível 1: Divisão inteira comum `/` falha perto de zero em coordenadas negativas (`-1 / 16 == 0`).",
            "Nível 2: Em jogos voxel, `-1` pertence ao chunk `-1`.",
            "Nível 3: Rust oferece `i32::div_euclid(16)` diretamente na stdlib!",
            "Nível 4: Retorne `(x.div_euclid(16), z.div_euclid(16))`.",
        ],
        mode: PracticeMode::CargoTest {
            tests: "use super::chunk_coords;\n#[test]\nfn testa_chunk_coords() {\n    assert_eq!(chunk_coords(0, 15), (0, 0));\n    assert_eq!(chunk_coords(16, -1), (1, -1));\n    assert_eq!(chunk_coords(-17, -16), (-2, -1));\n}\n",
        },
    },
];

pub fn validate_practice_run(spec_id: &str, code: &str, stdout: &str) -> Result<(), &'static str> {
    let stdout_trimmed = stdout.trim();
    if let Some(spec) = practice_spec_by_id(spec_id)
        && code.trim() == spec.starter.trim()
    {
        return Err("Você ainda não alterou o código inicial da prática.");
    }
    match spec_id {
        "B-T01-P" => {
            if !code.contains("println!") {
                return Err("Use `println!(...)` dentro de `fn main()` para imprimir seu fluxo.");
            }
            if stdout_trimmed.len() < 8 || stdout_trimmed == "..." {
                return Err(
                    "Escreva uma descrição real com pelo menos 8 caracteres no `println!`.",
                );
            }
            Ok(())
        }
        "B-T02-P" => {
            let low = stdout_trimmed.to_ascii_lowercase();
            if !code.contains("println!")
                || stdout_trimmed == "..."
                || !(low.contains("compil") || low.contains("execut"))
            {
                return Err("Imprima uma frase explicando `compilador` ou `executável`.");
            }
            Ok(())
        }
        "B-T04-P" => {
            let low = stdout_trimmed.to_ascii_lowercase();
            if !code.contains("println!")
                || stdout_trimmed == "..."
                || !(low.contains("cargo") || low.contains("rustc"))
            {
                return Err("Imprima uma frase citando `cargo` ou `rustc`.");
            }
            Ok(())
        }
        "B-P01" => {
            if !code.contains("println!") {
                return Err(
                    "Use a macro `println!(...)` dentro de `fn main()` para imprimir sua saída.",
                );
            }
            if stdout_trimmed.is_empty() || stdout_trimmed == "..." {
                return Err(
                    "A saída ainda está vazia ou com o placeholder `...`. Escreva uma frase sua!",
                );
            }
            Ok(())
        }
        "B-T06-P" => {
            let lines = stdout_trimmed
                .lines()
                .filter(|l| !l.trim().is_empty())
                .count();
            if lines < 2 {
                return Err("Imprima pelo menos duas linhas com `println!` em `fn main()`.");
            }
            Ok(())
        }
        "B-P02" => {
            if !code.contains("let mut") || !code.contains("println!") {
                return Err("Mantenha uma variável `let mut` e exiba o resultado com `println!`.");
            }
            if stdout_trimmed.is_empty() || stdout_trimmed == "RustLab: 0" {
                return Err(
                    "`pontos` continuou em 0 na saída; atualize a variável antes do `println!`.",
                );
            }
            Ok(())
        }
        "D09-P01" => {
            if !stdout_trimmed.contains("Pontos: 10") || !stdout_trimmed.contains("Saindo") {
                return Err(
                    "O menu interativo deve responder `Pontos: 10` para a opção `2` e `Saindo` para `0`.",
                );
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Progresso, respostas e registro idempotente de XP.
#[derive(Debug, Default, Clone)]
pub struct Progress {
    pub read: HashSet<String>,
    pub completed: HashSet<String>,
    pub exercises_completed: HashSet<String>,
    pub hints_used: HashMap<String, u8>,
    pub exercise_answers: HashMap<String, String>,
    pub xp_events: HashMap<String, u32>,
    pub project_milestones: HashSet<String>,
    pub srs_cards: HashMap<String, crate::storage::SrsCard>,
}

impl Progress {
    pub fn is_completed(&self, id: &str) -> bool {
        self.completed.contains(id)
    }

    /// Relógio lógico monotônico derivado das revisões concluídas pelo aluno.
    pub fn current_learning_step(&self) -> u32 {
        self.srs_cards.values().map(|c| c.review_count).sum()
    }

    /// Atualiza maestria (0..=5) e agenda a próxima revisão espaçada determinística.
    pub fn record_srs_mastery(&mut self, item_id: &str, independent: bool) {
        let new_step = self.current_learning_step().saturating_add(1);
        let card = self.srs_cards.entry(item_id.to_string()).or_default();
        card.review_count = card.review_count.saturating_add(1);
        if independent {
            card.independent_passes = card.independent_passes.saturating_add(1);
            card.mastery_level = card.mastery_level.saturating_add(1).min(5);
            card.interval_steps = match card.mastery_level {
                0 | 1 => 2,
                2 => 4,
                3 => 8,
                4 => 14,
                _ => 24,
            };
        } else {
            card.mastery_level = card.mastery_level.saturating_sub(1);
            card.interval_steps = 1;
        }
        card.due_at_step = new_step.saturating_add(card.interval_steps);
    }

    /// Lista IDs com revisão vencida no passo lógico atual, ordenados do mais urgente ao maior nível.
    pub fn due_srs_items(&self) -> Vec<String> {
        let now = self.current_learning_step();
        let mut due: Vec<(&String, &crate::storage::SrsCard)> = self
            .srs_cards
            .iter()
            .filter(|(_, c)| c.due_at_step <= now)
            .collect();
        due.sort_by(|(id_a, a), (id_b, b)| {
            a.due_at_step
                .cmp(&b.due_at_step)
                .then_with(|| a.mastery_level.cmp(&b.mastery_level))
                .then_with(|| id_a.cmp(id_b))
        });
        due.into_iter().map(|(id, _)| id.clone()).collect()
    }

    pub fn from_data(data: crate::storage::ProgressData) -> Self {
        Self {
            read: data.read_lessons.into_iter().collect(),
            completed: data.completed_lessons.into_iter().collect(),
            exercises_completed: data.completed_exercises.into_iter().collect(),
            hints_used: data.hints_used.into_iter().collect(),
            exercise_answers: data.exercise_answers.into_iter().collect(),
            xp_events: data.xp_events.into_iter().collect(),
            project_milestones: data.project_milestones.into_iter().collect(),
            srs_cards: data.srs_cards.into_iter().collect(),
        }
    }

    pub fn to_data(&self) -> crate::storage::ProgressData {
        crate::storage::ProgressData {
            version: 1,
            read_lessons: self.read.iter().cloned().collect(),
            completed_lessons: self.completed.iter().cloned().collect(),
            completed_exercises: self.exercises_completed.iter().cloned().collect(),
            hints_used: self
                .hints_used
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect(),
            exercise_answers: self
                .exercise_answers
                .iter()
                .map(|(id, code)| (id.clone(), code.clone()))
                .collect(),
            xp_events: self
                .xp_events
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect(),
            project_milestones: self.project_milestones.iter().cloned().collect(),
            srs_cards: self
                .srs_cards
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuizState {
    pub solved: Option<usize>,
    pub attempts: usize,
}

/// Tela de uma lição aberta.
#[derive(Debug, Clone)]
pub struct LessonView {
    pub chapter: usize,
    pub lesson: usize,
    pub scroll: usize,
    /// um estado por bloco de quiz, na ordem em que aparecem
    pub quizzes: Vec<QuizState>,
    /// um passo atual por bloco de demo, na ordem em que aparecem
    pub demos: Vec<usize>,
    /// altura real do corpo, medida no último draw (para rolagem/clamps)
    pub last_height: usize,
    pub at_bottom: bool,
    /// Estado emocional atual do mascote Ferris na lição.
    pub ferris: FerrisMood,
    /// Prática embutida opcional com execução real do Cargo (B-P01 / B-P02).
    pub practice: Option<LessonPractice>,
    /// Quando `true`, teclas editam diretamente a mini-IDE embutida da lição.
    pub practice_focused: bool,
    /// ID canônico V3 quando aberta via trilha completa (213 entradas).
    pub v3_entry_id: Option<&'static str>,
    /// Conteúdo da lição gerado dinamicamente pelo catálogo V3 (`None` para capítulos TOML).
    pub custom_lesson: Option<Lesson>,
}

impl LessonView {
    pub fn new(chapter: usize, lesson: usize, lesson_data: &Lesson) -> Self {
        Self {
            chapter,
            lesson,
            scroll: 0,
            quizzes: lesson_data
                .blocks
                .iter()
                .filter(|b| matches!(b, Block::Quiz { .. }))
                .map(|_| QuizState {
                    solved: None,
                    attempts: 0,
                })
                .collect(),
            demos: vec![0; lesson_data.count_demos()],
            last_height: 0,
            at_bottom: false,
            ferris: FerrisMood::Neutral,
            practice: LessonPractice::for_lesson(&lesson_data.id),
            practice_focused: false,
            v3_entry_id: None,
            custom_lesson: None,
        }
    }

    /// Requisito 8 — validação determinística: lição com quiz só
    /// conclui com o acerto; sem quiz, basta ter chegado ao fim.
    pub fn is_finished(&self) -> bool {
        self.quizzes.iter().all(|q| q.solved.is_some())
            && (!self.quizzes.is_empty() || self.at_bottom)
    }

    /// quantas opções ainda podem ser tecladas no primeiro quiz aberto
    pub fn open_quiz_options(&self, lesson: &Lesson) -> Option<(usize, usize)> {
        let mut serial = 0;
        for b in &lesson.blocks {
            if let Block::Quiz { options, .. } = b {
                if self.quizzes.get(serial)?.solved.is_none() {
                    return Some((serial, options.len()));
                }
                serial += 1;
            }
        }
        None
    }
}

/// Navegação do navegador de curso.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserPane {
    Chapters,
    Lessons,
}

#[derive(Debug, Clone, Copy)]
pub struct BrowserState {
    pub focus: BrowserPane,
    pub chapter_idx: usize,
    pub lesson_idx: usize,
    /// Inicia na Trilha V3 (213 lições / 6 estágios, default `true`); `v`/`V` alterna para os 10 capítulos clássicos.
    pub v3_mode: bool,
    /// Acessibilidade/inspeção: libera pré-visualização de estágios bloqueados (`Alt+V`).
    pub v3_preview_unlocked: bool,
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            focus: BrowserPane::Chapters,
            chapter_idx: 0,
            lesson_idx: 0,
            v3_mode: true,
            v3_preview_unlocked: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Teclado — decide a ação, depois aplica (mantém o borrow checker feliz)
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
pub enum LessonAction {
    Scroll(i32),
    Top,
    Bottom,
    CycleDemo,
    Answer(usize),
    Next,
    Prev,
    Close,
    Ignored,
}

pub fn lesson_action(lesson: &Lesson, view: &LessonView, key: &KeyEvent) -> LessonAction {
    if key.modifiers.contains(KeyModifiers::CONTROL) || key.modifiers.contains(KeyModifiers::ALT) {
        return LessonAction::Ignored;
    }
    match key.code {
        KeyCode::Esc => LessonAction::Close,
        KeyCode::Up | KeyCode::Char('k') => LessonAction::Scroll(-1),
        KeyCode::Down | KeyCode::Char('j') => LessonAction::Scroll(1),
        KeyCode::PageUp | KeyCode::Char('u') => LessonAction::Scroll(-10),
        KeyCode::PageDown => LessonAction::Scroll(10),
        KeyCode::Home => LessonAction::Top,
        KeyCode::End => LessonAction::Bottom,
        KeyCode::Char('d') if lesson.count_demos() > 0 => LessonAction::CycleDemo,
        KeyCode::Char(c) if ('1'..='9').contains(&c) => {
            // tecla só responde se houver quiz aberto com essa opção
            match view.open_quiz_options(lesson) {
                Some((_, n)) if ((c as usize) - ('1' as usize)) < n => {
                    LessonAction::Answer(c as usize - '1' as usize)
                }
                _ => LessonAction::Ignored,
            }
        }
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => LessonAction::Next,
        KeyCode::Left | KeyCode::Char('h') => LessonAction::Prev,
        _ => LessonAction::Ignored,
    }
}

pub fn save_active_practice_draft(app: &mut App) {
    let Some(view) = app.lesson_view.as_ref() else {
        return;
    };
    let Some(practice) = view.practice.as_ref() else {
        return;
    };
    let key = practice.spec.lesson_id.to_string();
    let code = practice.editor.buf.to_string_lossy();
    app.lesson_drafts.insert(key, code);
}

pub fn mark_lesson_ai_used(app: &mut App) {
    if let Some(view) = app.lesson_view.as_mut()
        && let Some(practice) = view.practice.as_mut()
    {
        practice.ai_used = true;
    }
    if let Some(state) = app.exercises.as_mut()
        && let Some(session) = state.session.as_mut()
    {
        session.ai_used = true;
    }
}

pub fn on_lesson_paste(app: &mut App, text: &str) {
    let Some(view) = app.lesson_view.as_mut() else {
        return;
    };
    if view.practice_focused
        && let Some(practice) = view.practice.as_mut()
        && !practice.consent_pending
        && practice.outcome != PracticeOutcome::Running
    {
        let before = practice.editor.buf.to_string_lossy();
        practice.editor.insert_text(text);
        if practice.editor.buf.to_string_lossy() != before {
            practice.outcome = PracticeOutcome::NotRun;
            practice.compile_error_seen = false;
            view.ferris = FerrisMood::Neutral;
        }
        let key = practice.spec.lesson_id.to_string();
        let code = practice.editor.buf.to_string_lossy();
        app.lesson_drafts.insert(key, code);
    }
}

pub fn on_lesson_key(app: &mut App, key: KeyEvent) {
    // Alternância explícita entre leitura+quiz (topo/quiz) e mini-IDE embutida (baixo)
    if matches!(key.code, KeyCode::Tab | KeyCode::BackTab) {
        let toggle = app.lesson_view.as_ref().and_then(|v| {
            let p = v.practice.as_ref()?;
            if p.consent_pending || p.outcome == PracticeOutcome::Running {
                return None;
            }
            Some(!v.practice_focused)
        });
        if let Some(next_focus) = toggle {
            save_active_practice_draft(app);
            if let Some(v) = app.lesson_view.as_mut() {
                v.practice_focused = next_focus;
            }
            app.set_status(
                if next_focus {
                    "Mini-IDE embutida focada · digite seu código e rode com F5 (Tab volta)."
                } else {
                    "Foco voltou para a explicação e quiz (Tab foca a prática)."
                },
                StatusKind::Info,
            );
        }
        return;
    }

    if app
        .lesson_view
        .as_ref()
        .is_some_and(|v| v.practice_focused && v.practice.is_some())
    {
        practice_key(app, key);
        return;
    }

    if key.code == KeyCode::Char('e') && key.modifiers == KeyModifiers::NONE {
        let selected = lesson_of(app).and_then(|lesson| {
            app.curriculum
                .as_ref()?
                .exercises()
                .position(|ex| ex.lesson_ids.contains(&lesson.id))
        });
        if let Some(selected) = selected {
            save_active_practice_draft(app);
            let state = app
                .exercises
                .get_or_insert_with(crate::exercises::ExerciseState::default);
            state.selected = selected;
            state.view = crate::exercises::ExerciseView::List;
            state.return_screen = Screen::LessonView;
            app.screen = Screen::Exercises;
        } else {
            let msg = if app
                .lesson_view
                .as_ref()
                .is_some_and(|v| v.practice.is_some())
            {
                "esta lição ainda não tem exercícios no catálogo (use Tab para a prática embutida)"
            } else {
                "esta lição ainda não tem exercícios relacionados"
            };
            app.set_status(msg, StatusKind::Info);
        }
        return;
    }
    let progress_before = (app.progress.read.clone(), app.progress.completed.clone());
    // 1. decidir a ação com leituras emprestadas
    let (action, out_of_range_quiz) = match (app.lesson_view.as_ref(), app.curriculum.as_ref()) {
        (Some(view), Some(cur)) => {
            let id = lesson_id(cur, view);
            match view.custom_lesson.as_ref().or_else(|| cur.lesson(&id)) {
                Some(lesson) => {
                    let act = lesson_action(lesson, view, &key);
                    let oor = act == LessonAction::Ignored
                        && matches!(key.code, KeyCode::Char(c) if ('1'..='9').contains(&c))
                        && view.open_quiz_options(lesson).is_some();
                    (act, oor)
                }
                None => (LessonAction::Ignored, false),
            }
        }
        _ => (LessonAction::Close, false),
    };

    if out_of_range_quiz && let Some(v) = app.lesson_view.as_mut() {
        v.ferris = FerrisMood::Confused;
    }

    // 2. ações puras de scroll/demo/quiz (borrow de lesson_view apenas)
    match action {
        LessonAction::Scroll(delta) => {
            if let Some(v) = app.lesson_view.as_mut() {
                let max = v.last_height;
                let target = (v.scroll as i32 + delta).clamp(0, max as i32) as usize;
                v.scroll = target;
                if delta > 0 && target >= max {
                    v.at_bottom = true;
                }
            }
        }
        LessonAction::Top => {
            if let Some(v) = app.lesson_view.as_mut() {
                v.scroll = 0;
            }
        }
        LessonAction::Bottom => {
            if let Some(v) = app.lesson_view.as_mut() {
                v.scroll = v.last_height;
                v.at_bottom = true;
            }
        }
        LessonAction::CycleDemo => {
            if let Some(v) = app.lesson_view.as_mut() {
                cycle_demo(app.curriculum.as_ref(), v);
            }
        }
        LessonAction::Answer(pick) => {
            if let Some(v) = app.lesson_view.as_mut() {
                answer_quiz(app.curriculum.as_ref(), v, pick);
            }
        }
        _ => {}
    }

    // 3. ações que trocam de tela (nenhum borrow vivo aqui)
    match action {
        LessonAction::Close => {
            if app
                .lesson_view
                .as_ref()
                .and_then(|v| v.practice.as_ref())
                .is_some_and(|p| p.outcome == PracticeOutcome::Running)
            {
                app.set_status(
                    "Cargo ainda está executando; aguarde ou cancele com Ctrl+K.",
                    StatusKind::Warning,
                );
                return;
            }
            save_active_practice_draft(app);
            if let Some(v) = app.lesson_view.as_ref().cloned() {
                let id = v.v3_entry_id.map(str::to_string).unwrap_or_else(|| {
                    app.curriculum
                        .as_ref()
                        .map(|c| lesson_id(c, &v))
                        .unwrap_or_default()
                });
                if !id.is_empty() {
                    app.progress.read.insert(id);
                }
            }
            app.lesson_view = None;
            app.screen = Screen::Browser;
        }
        LessonAction::Next => go_next(app),
        LessonAction::Prev => go_prev(app),
        _ => {}
    }
    if progress_before != (app.progress.read.clone(), app.progress.completed.clone())
        && let Err(err) = app.persist_progress()
    {
        app.set_status(
            format!("progresso não foi salvo: {err}"),
            StatusKind::Warning,
        );
    }
}

fn practice_key(app: &mut App, key: KeyEvent) {
    let consent_pending = app
        .lesson_view
        .as_ref()
        .and_then(|v| v.practice.as_ref())
        .is_some_and(|p| p.consent_pending);
    if consent_pending {
        match key.code {
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('s') => {
                if let Some(p) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut()) {
                    p.consent_pending = false;
                }
                app.config.execution.consent = ExecConsent::Granted;
                if let Err(e) = app.save_config() {
                    app.set_status(
                        format!("não salvei consentimento: {e}"),
                        StatusKind::Warning,
                    );
                }
                run_lesson_practice(app);
            }
            KeyCode::Esc | KeyCode::Char('n') => {
                if let Some(p) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut()) {
                    p.consent_pending = false;
                }
                app.set_status("Nenhum código Cargo foi executado.", StatusKind::Info);
            }
            _ => {}
        }
        return;
    }

    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let running = app
        .lesson_view
        .as_ref()
        .and_then(|v| v.practice.as_ref())
        .is_some_and(|p| p.outcome == PracticeOutcome::Running);

    if key.code == KeyCode::Esc {
        if running {
            app.set_status(
                "Cargo ainda está executando; aguarde ou use Ctrl+K.",
                StatusKind::Warning,
            );
            return;
        }
        save_active_practice_draft(app);
        if let Some(v) = app.lesson_view.as_mut() {
            v.practice_focused = false;
        }
        app.set_status(
            "Foco voltou para a explicação (código preservado).",
            StatusKind::Info,
        );
        return;
    }

    match key.code {
        KeyCode::Char('k') if ctrl => {
            if app.exec.running {
                app.exec.cancel();
            }
        }
        KeyCode::F(5) => {
            if app.config.execution.consent == ExecConsent::Granted {
                run_lesson_practice(app);
            } else if let Some(p) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut()) {
                p.consent_pending = true;
            }
        }
        KeyCode::Char('s') if ctrl => save_lesson_practice_disk(app),
        KeyCode::Char('e') if alt => {
            if let Some(p) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut()) {
                let diag = p.output.join("\n");
                let code = p.editor.buf.to_string_lossy();
                let lens = crate::lab_tools::SocraticErrorLens::analyze(&diag, &code);
                let fix = lens.fixes[0];
                p.push_output(format!("[Lente {}] {}", lens.code, lens.title));
                p.push_output(format!("Modo 10 Anos (ELI10): {}", lens.eli10_metaphor));
                p.push_output(format!("Pergunta Socrática: {}", lens.socratic_question));
                p.push_output(format!(
                    "Solução {}: {} -> {}",
                    fix.label,
                    fix.before.replace('\n', " | "),
                    fix.after.replace('\n', " | ")
                ));
                p.output_scroll = 0;
            }
            app.set_status(
                "Lente Socrática + Modo 10 Anos adicionados na saída.",
                StatusKind::Info,
            );
        }
        KeyCode::Char('c') if alt => {
            if let Some(p) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut()) {
                let code = p.editor.buf.to_string_lossy();
                let sols = crate::lab_tools::community_solutions_for(p.spec.prompt);
                p.push_output("Soluções idiomáticas da comunidade:".to_string());
                for sol in sols {
                    p.push_output(format!(
                        "• {} [{}] — {}",
                        sol.name, sol.style_badge, sol.pros
                    ));
                }
                p.push_output(format!(
                    "Rust Playground: {}",
                    crate::lab_tools::build_playground_url(&code)
                ));
                p.output_scroll = 0;
            }
            app.set_status(
                "Soluções da comunidade + link Playground exibidos.",
                StatusKind::Info,
            );
        }
        KeyCode::Char('l') if alt => {
            if let Some(p) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut()) {
                let code = p.editor.buf.to_string_lossy();
                let lt = crate::lab_tools::LifetimeVisualizerReport::analyze(&code);
                let layout = crate::lab_tools::MemoryLayoutReport::analyze(&code);
                p.push_output("Mapa de Lifetimes & Layout em Memória:".to_string());
                for line in lt.summary_lines().into_iter().take(8) {
                    p.push_output(line);
                }
                if let Some(st) = layout.user_types.first() {
                    p.push_output(format!(
                        "Layout `struct {}`: {}B (align {}B, padding {}B, ótimo {}B)",
                        st.name,
                        st.declared_size,
                        st.declared_align,
                        st.total_padding,
                        st.optimal_size
                    ));
                }
                p.output_scroll = 0;
            }
            app.set_status(
                "Mapa de Lifetimes e Layout exibidos na saída.",
                StatusKind::Info,
            );
        }
        KeyCode::Char('i') if alt => {
            if let Some(p) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut()) {
                let card_idx =
                    app.progress.completed.len() % crate::lab_tools::INTUITION_FLASHCARDS.len();
                let card = crate::lab_tools::INTUITION_FLASHCARDS[card_idx];
                p.push_output(format!(
                    "[Flashcard {}] {} ({})",
                    card.id, card.title, card.topic
                ));
                for l in card.snippet.lines() {
                    p.push_output(format!("  {l}"));
                }
                let ans = if card.compiles {
                    "COMPILA ✔"
                } else {
                    card.error_code
                };
                p.push_output(format!("Gabarito rápido: {ans} — {}", card.explanation));
                p.output_scroll = 0;
            }
            app.set_status("Flashcard de intuição exibido na saída.", StatusKind::Info);
        }
        KeyCode::F(1) => reveal_practice_hint(app),
        KeyCode::F(2) => {
            if let Some(p) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut()) {
                p.hint_index = p.hint_index.saturating_sub(1);
                p.hint_scroll = 0;
            }
        }
        KeyCode::F(3) => {
            if let Some(p) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut()) {
                p.hint_index = (p.hint_index + 1).min(p.shown_hints.saturating_sub(1));
                p.hint_scroll = 0;
            }
        }
        KeyCode::F(4) => {
            if let Some(p) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut()) {
                p.hint_scroll = if shift {
                    p.hint_scroll.saturating_sub(1)
                } else {
                    p.hint_scroll.saturating_add(1)
                };
            }
        }
        KeyCode::PageUp => {
            if let Some(p) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut()) {
                p.output_scroll = p
                    .output_scroll
                    .saturating_add(4)
                    .min(p.output.len().saturating_sub(1));
            }
        }
        KeyCode::PageDown => {
            if let Some(p) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut()) {
                p.output_scroll = p.output_scroll.saturating_sub(4);
            }
        }
        _ if running => {}
        KeyCode::Char(c) if !ctrl && !alt => edit_practice_buffer(app, |e| e.insert_char(c)),
        KeyCode::Enter => edit_practice_buffer(app, Editor::newline),
        KeyCode::Backspace => edit_practice_buffer(app, Editor::backspace),
        KeyCode::Delete => edit_practice_buffer(app, Editor::delete),
        KeyCode::Left => edit_practice_buffer(app, |e| e.move_cursor(-1, 0, shift, ctrl)),
        KeyCode::Right => edit_practice_buffer(app, |e| e.move_cursor(1, 0, shift, ctrl)),
        KeyCode::Up => edit_practice_buffer(app, |e| e.move_cursor(0, -1, shift, false)),
        KeyCode::Down => edit_practice_buffer(app, |e| e.move_cursor(0, 1, shift, false)),
        KeyCode::Home if ctrl => edit_practice_buffer(app, |e| e.move_doc(false, shift)),
        KeyCode::End if ctrl => edit_practice_buffer(app, |e| e.move_doc(true, shift)),
        KeyCode::Home => edit_practice_buffer(app, |e| e.move_home(shift)),
        KeyCode::End => edit_practice_buffer(app, |e| e.move_end(shift)),
        KeyCode::Char('a') if ctrl => edit_practice_buffer(app, Editor::select_all),
        KeyCode::Char('c') if ctrl => edit_practice_buffer(app, |e| {
            e.copy();
        }),
        KeyCode::Char('x') if ctrl => edit_practice_buffer(app, |e| {
            e.cut();
        }),
        KeyCode::Char('v') if ctrl => edit_practice_buffer(app, Editor::paste),
        KeyCode::Char('z') if ctrl && shift => edit_practice_buffer(app, Editor::redo),
        KeyCode::Char('z') if ctrl => edit_practice_buffer(app, Editor::undo),
        KeyCode::Char('y') if ctrl => edit_practice_buffer(app, Editor::redo),
        _ => {}
    }
}

fn edit_practice_buffer(app: &mut App, f: impl FnOnce(&mut Editor)) {
    let Some(view) = app.lesson_view.as_mut() else {
        return;
    };
    let Some(practice) = view.practice.as_mut() else {
        return;
    };
    if practice.outcome == PracticeOutcome::Running {
        return;
    }
    let before = practice.editor.buf.to_string_lossy();
    f(&mut practice.editor);
    let after = practice.editor.buf.to_string_lossy();
    if after != before {
        practice.outcome = PracticeOutcome::NotRun;
        practice.compile_error_seen = false;
        practice.stdout_lines.clear();
        practice.output.clear();
        practice.output_scroll = 0;
        view.ferris = FerrisMood::Neutral;
        let key = practice.spec.lesson_id.to_string();
        app.lesson_drafts.insert(key, after);
    }
}

fn save_lesson_practice_disk(app: &mut App) {
    let Some((lesson_id, code)) = app
        .lesson_view
        .as_ref()
        .and_then(|v| v.practice.as_ref())
        .map(|p| (p.spec.lesson_id.to_string(), p.editor.buf.to_string_lossy()))
    else {
        return;
    };
    app.lesson_drafts.insert(lesson_id.clone(), code.clone());
    app.progress
        .exercise_answers
        .insert(format!("practice:{lesson_id}"), code);
    match app.persist_progress() {
        Ok(()) => {
            if let Some(p) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut()) {
                p.editor.mark_saved();
            }
            app.set_status("Rascunho da lição salvo em disco.", StatusKind::Success);
        }
        Err(e) => {
            app.set_status(format!("Não consegui salvar: {e}"), StatusKind::Warning);
        }
    }
}

fn reveal_practice_hint(app: &mut App) {
    let Some((spec_id, max_hints)) = app
        .lesson_view
        .as_ref()
        .and_then(|v| v.practice.as_ref())
        .map(|p| (p.spec.id, p.spec.hints.len()))
    else {
        return;
    };
    let Some(p) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut()) else {
        return;
    };
    if p.shown_hints >= max_hints {
        app.set_status("Todas as 4 dicas já foram reveladas.", StatusKind::Info);
        return;
    }
    p.shown_hints += 1;
    p.hint_index = p.shown_hints - 1;
    p.hint_scroll = 0;
    let shown = p.shown_hints;
    app.progress.hints_used.insert(
        format!("practice:{spec_id}"),
        shown.min(u8::MAX as usize) as u8,
    );
    let _ = app.persist_progress();
    app.set_status(
        format!("Dica {shown}/{max_hints} revelada (XP independente ajustado)."),
        StatusKind::Info,
    );
}

fn fresh_practice_temp_dir(spec_id: &str) -> std::io::Result<PathBuf> {
    for _ in 0..100 {
        let nonce = PRACTICE_TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "rustlab-lesson-{}-{spec_id}-{nonce}",
            std::process::id()
        ));
        match std::fs::create_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "sem diretório temporário livre",
    ))
}

fn prepare_practice_project(
    dir: &std::path::Path,
    spec: &PracticeSpec,
    code: &str,
) -> std::io::Result<()> {
    std::fs::create_dir_all(dir.join("src"))?;
    let pkg = spec.id.to_lowercase().replace('-', "_");
    match spec.mode {
        PracticeMode::CargoRun | PracticeMode::CargoRunWithInput { .. } => {
            std::fs::write(
                dir.join("Cargo.toml"),
                format!(
                    "[package]\nname = \"rustlab_lesson_{pkg}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n"
                ),
            )?;
            std::fs::write(dir.join("src/main.rs"), code)
        }
        PracticeMode::CargoTest { tests } => {
            std::fs::write(
                dir.join("Cargo.toml"),
                format!(
                    "[package]\nname = \"rustlab_lesson_{pkg}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\ndoctest = false\n\n[workspace]\n"
                ),
            )?;
            std::fs::write(
                dir.join("src/lib.rs"),
                format!("{code}\n\n#[cfg(test)]\nmod rustlab_validation {{\n{tests}\n}}\n"),
            )
        }
    }
}

pub fn run_lesson_practice(app: &mut App) {
    if app.exec.running {
        app.set_status(
            "Já há um comando Cargo rodando; aguarde ou use Ctrl+K.",
            StatusKind::Warning,
        );
        return;
    }
    save_lesson_practice_disk(app);
    let Some((spec, code)) = app
        .lesson_view
        .as_ref()
        .and_then(|v| v.practice.as_ref())
        .map(|p| (p.spec.clone(), p.editor.buf.to_string_lossy()))
    else {
        return;
    };
    let dir = match fresh_practice_temp_dir(spec.id) {
        Ok(dir) => dir,
        Err(e) => {
            if let Some(view) = app.lesson_view.as_mut() {
                view.ferris = FerrisMood::Confused;
                if let Some(p) = view.practice.as_mut() {
                    p.outcome = PracticeOutcome::StartFailed;
                    p.output = vec![format!("Falha ao preparar projeto Cargo: {e}")];
                }
            }
            app.set_status(
                format!("Falha ao criar pasta temporária: {e}"),
                StatusKind::Warning,
            );
            return;
        }
    };
    if let Err(e) = prepare_practice_project(&dir, &spec, &code) {
        let _ = std::fs::remove_dir_all(&dir);
        if let Some(view) = app.lesson_view.as_mut() {
            view.ferris = FerrisMood::Confused;
            if let Some(p) = view.practice.as_mut() {
                p.outcome = PracticeOutcome::StartFailed;
                p.output = vec![format!("Falha ao gravar Cargo.toml/src: {e}")];
            }
        }
        return;
    }
    if let Some(view) = app.lesson_view.as_mut() {
        view.ferris = FerrisMood::Neutral;
        if let Some(p) = view.practice.as_mut() {
            p.pending_dir = Some(dir.clone());
            p.output.clear();
            p.stdout_lines.clear();
            p.output_scroll = 0;
            p.compile_error_seen = false;
            p.outcome = PracticeOutcome::Running;
        }
    }
    let (cmd, status_msg, scripted_stdin) = match spec.mode {
        PracticeMode::CargoRun => (
            ToolCommand::Run,
            "Compilando e executando sua prática com Cargo…",
            &[][..],
        ),
        PracticeMode::CargoRunWithInput { stdin_lines } => (
            ToolCommand::RunInteractive,
            "Compilando e executando sua prática interativa (stdin) com Cargo…",
            stdin_lines,
        ),
        PracticeMode::CargoTest { .. } => (
            ToolCommand::Test,
            "Compilando e rodando os testes da prática com Cargo…",
            &[][..],
        ),
    };
    match app.exec.submit(cmd, dir) {
        Ok(()) => {
            if !scripted_stdin.is_empty() {
                for line in scripted_stdin {
                    let _ = app.exec.send_line(line);
                }
                let _ = app.exec.close_input();
            }
            app.set_status(status_msg, StatusKind::Info);
        }
        Err(e) => {
            if let Some(view) = app.lesson_view.as_mut() {
                view.ferris = FerrisMood::Encouraging;
                if let Some(p) = view.practice.as_mut() {
                    p.outcome = PracticeOutcome::StartFailed;
                    p.output.push(e.to_string());
                    if let Some(d) = p.pending_dir.take() {
                        let _ = std::fs::remove_dir_all(d);
                    }
                }
            }
            app.set_status(e, StatusKind::Warning);
        }
    }
}

pub fn on_lesson_exec_event(app: &mut App, ev: ExecEvent) {
    enum PostAction {
        None,
        Passed {
            spec_id: &'static str,
            lesson_id: &'static str,
            base_xp: u32,
            bonus_xp: u32,
        },
        StatusWarning(String),
        StatusInfo(String),
    }

    let mut post = PostAction::None;
    if let Some(view) = app.lesson_view.as_mut()
        && let Some(p) = view.practice.as_mut()
    {
        match ev {
            ExecEvent::Chunk { kind, text } => {
                if kind == StreamKind::Stdout {
                    p.stdout_lines.push(text.clone());
                } else if is_practice_compile_error(&text) {
                    p.compile_error_seen = true;
                }
                p.push_output(text);
            }
            ExecEvent::Finished { success, .. } => {
                if let Some(dir) = p.pending_dir.take() {
                    let _ = std::fs::remove_dir_all(dir);
                }
                if !success {
                    if p.compile_error_seen {
                        p.outcome = PracticeOutcome::CompileFailed;
                        if let Some(help) = explain_compiler_error_pt_br(&p.output.join("\n")) {
                            p.push_output(format!("Explicação do Ferris: {help}"));
                        }
                        post = PostAction::StatusWarning(
                            "Não compilou ainda — leia a explicação do Ferris ou o erro e tente de novo."
                                .to_string(),
                        );
                    } else {
                        p.outcome = PracticeOutcome::RunFailed;
                        post = PostAction::StatusWarning(
                            "A execução/testes falharam — revise a saída e tente de novo."
                                .to_string(),
                        );
                    }
                    view.ferris = FerrisMood::Encouraging;
                } else {
                    let code = p.editor.buf.to_string_lossy();
                    let stdout_joined = p.stdout_lines.join("\n");
                    match validate_practice_run(p.spec.id, &code, &stdout_joined) {
                        Ok(()) => {
                            p.outcome = PracticeOutcome::Passed;
                            view.ferris = FerrisMood::Happy;
                            let (base_xp, bonus_xp) =
                                calculate_v3_practice_xp(p.shown_hints, p.ai_used);
                            post = PostAction::Passed {
                                spec_id: p.spec.id,
                                lesson_id: p.spec.lesson_id,
                                base_xp,
                                bonus_xp,
                            };
                        }
                        Err(msg) => {
                            p.outcome = PracticeOutcome::ValidationFailed;
                            p.push_output(format!("Validação RustLab: {msg}"));
                            view.ferris = if code.trim() == p.spec.starter.trim()
                                || stdout_joined.trim().is_empty()
                                || stdout_joined.trim() == "..."
                            {
                                FerrisMood::Confused
                            } else {
                                FerrisMood::Encouraging
                            };
                            post = PostAction::StatusWarning(msg.to_string());
                        }
                    }
                }
            }
            ExecEvent::TimedOut { .. } => {
                if let Some(dir) = p.pending_dir.take() {
                    let _ = std::fs::remove_dir_all(dir);
                }
                p.outcome = PracticeOutcome::TimedOut;
                p.push_output("Tempo limite atingido durante `cargo run`.".to_string());
                view.ferris = FerrisMood::Encouraging;
                post = PostAction::StatusWarning("Execução excedeu o limite de tempo.".to_string());
            }
            ExecEvent::Cancelled { .. } => {
                if let Some(dir) = p.pending_dir.take() {
                    let _ = std::fs::remove_dir_all(dir);
                }
                p.outcome = PracticeOutcome::Cancelled;
                p.push_output("Execução cancelada pelo aluno.".to_string());
                view.ferris = FerrisMood::Neutral;
                post = PostAction::StatusInfo("Execução Cargo cancelada.".to_string());
            }
            ExecEvent::SpawnFailed { reason, .. } => {
                if let Some(dir) = p.pending_dir.take() {
                    let _ = std::fs::remove_dir_all(dir);
                }
                p.outcome = PracticeOutcome::StartFailed;
                p.push_output(reason.clone());
                view.ferris = FerrisMood::Confused;
                post = PostAction::StatusWarning(reason);
            }
            ExecEvent::Started { .. } => {}
        }
    }

    match post {
        PostAction::Passed {
            spec_id,
            lesson_id,
            base_xp,
            bonus_xp,
        } => {
            app.progress.read.insert(lesson_id.to_string());
            app.progress.exercises_completed.insert(spec_id.to_string());
            app.progress.record_srs_mastery(spec_id, bonus_xp > 0);
            app.award_xp(format!("practice:{spec_id}"), base_xp, "prática concluída");
            if bonus_xp > 0 {
                app.award_xp(
                    format!("practice:{spec_id}:independent"),
                    bonus_xp,
                    "bônus sem dicas",
                );
            }
            let _ = app.persist_progress();
            app.set_status(
                format!(
                    "✓ Prática {spec_id} aprovada pelo Cargo! (+{} XP)",
                    base_xp + bonus_xp
                ),
                StatusKind::Success,
            );
        }
        PostAction::StatusWarning(msg) => app.set_status(msg, StatusKind::Warning),
        PostAction::StatusInfo(msg) => app.set_status(msg, StatusKind::Info),
        PostAction::None => {}
    }
}

fn is_practice_compile_error(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.contains("could not compile")
        || trimmed.contains("error[E")
        || trimmed.starts_with("error:")
}

fn cycle_demo(cur: Option<&crate::curriculum::Curriculum>, view: &mut LessonView) {
    if view.demos.is_empty() {
        return;
    }
    let borrowed = if let Some(custom) = view.custom_lesson.as_ref() {
        custom
    } else if let Some(c) = cur {
        let id = lesson_id(c, view);
        let Some(l) = c.lesson(&id) else {
            return;
        };
        l
    } else {
        return;
    };
    // comprimentos, por número de série da demo
    let lens: Vec<usize> = borrowed
        .blocks
        .iter()
        .filter_map(|b| match b {
            Block::Demo { steps, .. } => Some(steps.len().max(1)),
            _ => None,
        })
        .collect();
    // avança a primeira demo que ainda pode andar; as já completas dão a volta
    let total_demos = view.demos.len();
    for (i, d) in view.demos.iter_mut().enumerate() {
        let max = *lens.get(i).unwrap_or(&1);
        let next = (*d + 1) % max;
        *d = next;
        if next != 0 || i == total_demos - 1 {
            break;
        }
    }
}

fn answer_quiz(cur: Option<&crate::curriculum::Curriculum>, view: &mut LessonView, pick: usize) {
    let borrowed = if let Some(custom) = view.custom_lesson.as_ref() {
        custom
    } else if let Some(c) = cur {
        let id = lesson_id(c, view);
        let Some(l) = c.lesson(&id) else {
            return;
        };
        l
    } else {
        return;
    };
    let Some((serial, _n)) = view.open_quiz_options(borrowed) else {
        return;
    };
    // encontrar o índice correto deste quiz específico
    let mut s = 0;
    let mut correct = 0;
    for b in &borrowed.blocks {
        if let Block::Quiz { correct: cr, .. } = b {
            if s == serial {
                correct = *cr;
                break;
            }
            s += 1;
        }
    }
    let st = &mut view.quizzes[serial];
    st.attempts += 1;
    if pick == correct {
        st.solved = Some(pick);
        view.ferris = FerrisMood::Happy;
    } else {
        view.ferris = FerrisMood::Encouraging;
    }
}

fn lesson_id(cur: &crate::curriculum::Curriculum, view: &LessonView) -> String {
    if let Some(v3_id) = view.v3_entry_id {
        return v3_id.to_string();
    }
    cur.chapters
        .get(view.chapter)
        .and_then(|ch| ch.lessons.get(view.lesson))
        .cloned()
        .unwrap_or_default()
}

pub fn lesson_of(app: &App) -> Option<&Lesson> {
    let v = app.lesson_view.as_ref()?;
    if let Some(custom) = v.custom_lesson.as_ref() {
        return Some(custom);
    }
    let c = app.curriculum.as_ref()?;
    c.lesson(lesson_id(c, v).as_str())
}

fn go_next(app: &mut App) {
    let view = match app.lesson_view.as_ref() {
        Some(v) => v.clone(),
        None => return,
    };
    let (id, requires_quiz) = if let Some(custom) = view.custom_lesson.as_ref() {
        (
            view.v3_entry_id.unwrap_or(&custom.id).to_string(),
            custom.requires_quiz(),
        )
    } else {
        let Some(c) = app.curriculum.as_ref() else {
            return;
        };
        let id = lesson_id(c, &view);
        let Some(lesson) = c.lesson(&id) else {
            return;
        };
        (id, lesson.requires_quiz())
    };
    if !view.is_finished() {
        if requires_quiz {
            app.set_status(t("lesson.blocked_next"), StatusKind::Info);
        } else {
            app.set_status(t("lesson.scroll_to_end"), StatusKind::Info);
        }
        return;
    }
    let newly_completed = app.progress.completed.insert(id.clone());
    if newly_completed {
        app.award_xp(format!("lesson:{id}"), 20, "lição concluída");
    }
    app.progress.record_srs_mastery(&id, true);
    let _ = app.persist_progress();

    if let Some(v3_id) = view.v3_entry_id {
        let entries = crate::curriculum::v3_blueprint::v3_curriculum_blueprint();
        let next_entry = crate::curriculum::v3_blueprint::v3_entry_index(v3_id)
            .and_then(|idx| entries.get(idx + 1));
        if let Some(next_item) = next_entry {
            open_v3_entry(app, next_item.id);
        } else {
            save_active_practice_draft(app);
            app.lesson_view = None;
            app.screen = Screen::Browser;
            app.set_status(t("lesson.end-of-curriculum"), StatusKind::Success);
            app.anim
                .play(crate::animation::ScreenFx::Enter, app.theme.accent);
        }
        return;
    }

    let next = {
        let c = app.curriculum.as_ref().unwrap();
        next_global(c, view.chapter, view.lesson)
    };
    match next {
        Some((ci, li)) => open_lesson(app, ci, li),
        None => {
            save_active_practice_draft(app);
            app.lesson_view = None;
            app.screen = Screen::Browser;
            app.set_status(t("lesson.end-of-curriculum"), StatusKind::Success);
            app.anim
                .play(crate::animation::ScreenFx::Enter, app.theme.accent);
        }
    }
}

fn go_prev(app: &mut App) {
    let view = match app.lesson_view.as_ref() {
        Some(v) => v.clone(),
        None => return,
    };
    if let Some(v3_id) = view.v3_entry_id {
        let entries = crate::curriculum::v3_blueprint::v3_curriculum_blueprint();
        if let Some(idx) = crate::curriculum::v3_blueprint::v3_entry_index(v3_id)
            && idx > 0
        {
            open_v3_entry(app, entries[idx - 1].id);
        }
        return;
    }
    let Some(c) = app.curriculum.as_ref() else {
        return;
    };
    if let Some((ci, li)) = prev_global(c, view.chapter, view.lesson) {
        open_lesson(app, ci, li);
    }
}

fn next_global(
    c: &crate::curriculum::Curriculum,
    chapter: usize,
    lesson: usize,
) -> Option<(usize, usize)> {
    let ch = c.chapters.get(chapter)?;
    if let Some(next_id) = ch.lessons.get(lesson + 1)
        && c.lesson(next_id).is_some()
    {
        return Some((chapter, lesson + 1));
    }
    for ci in chapter + 1..c.chapters.len() {
        let cand = &c.chapters[ci];
        match cand.status {
            ChapterStatus::Available => {
                if let Some(first) = cand.lessons.first()
                    && c.lesson(first).is_some()
                {
                    return Some((ci, 0));
                }
            }
            ChapterStatus::Planned => break, // fim do conteúdo já escrito
        }
    }
    None
}

fn prev_global(
    c: &crate::curriculum::Curriculum,
    chapter: usize,
    lesson: usize,
) -> Option<(usize, usize)> {
    if lesson > 0 {
        return Some((chapter, lesson - 1));
    }
    for ci in (0..chapter).rev() {
        let cand = &c.chapters[ci];
        if cand.status == ChapterStatus::Available && !cand.lessons.is_empty() {
            return Some((ci, cand.lessons.len() - 1));
        }
    }
    None
}

fn restore_practice_view(app: &App, draft_key: &str, view: &mut LessonView) {
    if let Some(practice) = view.practice.as_mut() {
        let shown = app
            .progress
            .hints_used
            .get(&format!("practice:{}", practice.spec.id))
            .copied()
            .unwrap_or(0) as usize;
        practice.shown_hints = shown.min(practice.spec.hints.len());
        practice.hint_index = practice.shown_hints.saturating_sub(1);
        let saved_code = app.lesson_drafts.get(draft_key).cloned().or_else(|| {
            app.progress
                .exercise_answers
                .get(&format!("practice:{draft_key}"))
                .cloned()
        });
        if let Some(code) = saved_code {
            practice.editor = Editor::new(&code, None);
        }
        if app.progress.exercises_completed.contains(practice.spec.id) {
            practice.outcome = PracticeOutcome::Passed;
            view.ferris = FerrisMood::Happy;
        }
    }
}

pub fn open_lesson(app: &mut App, chapter: usize, lesson: usize) {
    save_active_practice_draft(app);
    let (id, lesson_data) = {
        let Some(c) = app.curriculum.as_ref() else {
            return;
        };
        let Some(ch) = c.chapters.get(chapter) else {
            return;
        };
        let Some(id) = ch.lessons.get(lesson) else {
            return;
        };
        let Some(l) = c.lesson(id) else { return };
        (id.clone(), l.clone())
    };
    let mut view = LessonView::new(chapter, lesson, &lesson_data);
    restore_practice_view(app, &id, &mut view);
    app.lesson_view = Some(view);
    app.screen = Screen::LessonView;
    app.anim
        .play(crate::animation::ScreenFx::Enter, app.theme.accent);
}

/// Abre qualquer uma das 213 entradas da trilha completa V3 no visualizador de lição.
pub fn open_v3_entry(app: &mut App, v3_id: &'static str) {
    use crate::curriculum::v3_blueprint::{
        V3Stage, build_v3_lesson, entries_for_stage, find_v3_entry,
    };

    save_active_practice_draft(app);
    let Some(entry) = find_v3_entry(v3_id) else {
        return;
    };
    let lesson_data = {
        let Some(cur) = app.curriculum.as_ref() else {
            return;
        };
        build_v3_lesson(entry, cur)
    };
    let stage_idx = V3Stage::ALL
        .iter()
        .position(|s| *s == entry.stage)
        .unwrap_or(0);
    let entry_idx = entries_for_stage(entry.stage)
        .iter()
        .position(|e| e.id == entry.id)
        .unwrap_or(0);

    let mut view = LessonView::new(stage_idx, entry_idx, &lesson_data);
    view.v3_entry_id = Some(entry.id);
    view.custom_lesson = Some(lesson_data);
    restore_practice_view(app, entry.id, &mut view);

    app.progress.read.insert(entry.id.to_string());
    let _ = app.persist_progress();
    app.lesson_view = Some(view);
    app.screen = Screen::LessonView;
    app.anim
        .play(crate::animation::ScreenFx::Enter, app.theme.accent);
}

pub fn open_browser(app: &mut App) {
    if app.curriculum.is_some() {
        app.screen = Screen::Browser;
        app.anim
            .play(crate::animation::ScreenFx::Enter, app.theme.accent);
    } else {
        app.set_status(t("learn.empty"), StatusKind::Warning);
    }
}

// ---------------------------------------------------------------------------
// Teclado do navegador de curso
// ---------------------------------------------------------------------------

pub fn on_browser_key(app: &mut App, key: &KeyEvent) {
    use BrowserPane::*;
    if app.curriculum.is_none() {
        app.screen = Screen::Menu;
        return;
    }

    if matches!(key.code, KeyCode::Char('v' | 'V'))
        && key.modifiers.contains(KeyModifiers::ALT)
        && !key.modifiers.contains(KeyModifiers::CONTROL)
    {
        if !app.browser.v3_mode {
            app.browser.v3_mode = true;
            app.browser.chapter_idx = 0;
            app.browser.lesson_idx = 0;
            app.browser.focus = Chapters;
        }
        app.browser.v3_preview_unlocked = !app.browser.v3_preview_unlocked;
        app.set_status(
            if app.browser.v3_preview_unlocked {
                "Pré-visualização V3 ativada: todos os 6 estágios e 213 lições liberados (Alt+V)."
            } else {
                "Gating V3 restaurado: desbloqueio por pré-requisitos ativo (Alt+V)."
            },
            StatusKind::Info,
        );
        return;
    }

    if matches!(key.code, KeyCode::Char('v' | 'V'))
        && !key.modifiers.contains(KeyModifiers::CONTROL)
        && !key.modifiers.contains(KeyModifiers::ALT)
    {
        app.browser.v3_mode = !app.browser.v3_mode;
        app.browser.chapter_idx = 0;
        app.browser.lesson_idx = 0;
        app.browser.focus = Chapters;
        app.set_status(
            if app.browser.v3_mode {
                "Trilha V3 completa (213 lições · 6 estágios) · Tab navega · Alt+V libera prévia."
            } else {
                "Visão clássica por capítulos (30 lições) ativada (V volta para a Trilha V3)."
            },
            StatusKind::Info,
        );
        return;
    }

    if app.browser.v3_mode {
        on_browser_key_v3(app, key);
        return;
    }

    // Colheita de dados (imutável) ANTES de qualquer mutação, para
    // manter o borrow checker e o código ambos felizes.
    let (n_chapters, sel_status, lessons) = {
        let c = app.curriculum.as_ref().unwrap();
        let idx = app
            .browser
            .chapter_idx
            .min(c.chapters.len().saturating_sub(1));
        let ch = &c.chapters[idx];
        (c.chapters.len(), ch.status, ch.lessons.clone())
    };
    let n_lessons = lessons.len();

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = Screen::Menu;
        }
        KeyCode::Char('r') => {
            open_next_due_srs(app);
        }
        KeyCode::Tab | KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
            let focus = match app.browser.focus {
                Chapters => Lessons,
                Lessons => Chapters,
            };
            // painel de lições de capítulo planejado não recebe foco
            app.browser.focus =
                if focus == Lessons && (sel_status != ChapterStatus::Available || n_lessons == 0) {
                    Chapters
                } else {
                    focus
                };
        }
        KeyCode::Up | KeyCode::Char('k') => match app.browser.focus {
            Chapters => app.browser.chapter_idx = app.browser.chapter_idx.saturating_sub(1),
            Lessons => app.browser.lesson_idx = app.browser.lesson_idx.saturating_sub(1),
        },
        KeyCode::Down | KeyCode::Char('j') => match app.browser.focus {
            Chapters => app.browser.chapter_idx = (app.browser.chapter_idx + 1).min(n_chapters - 1),
            Lessons => {
                app.browser.lesson_idx =
                    (app.browser.lesson_idx + 1).min(n_lessons.saturating_sub(1))
            }
        },
        KeyCode::Enter => match app.browser.focus {
            Chapters => {
                if sel_status == ChapterStatus::Available && n_lessons > 0 {
                    app.browser.focus = Lessons;
                    app.browser.lesson_idx = first_unfinished(app, &lessons);
                } else {
                    app.set_status(t("browser.planned-note"), StatusKind::Info);
                }
            }
            Lessons => {
                let (ci, li) = (app.browser.chapter_idx, app.browser.lesson_idx);
                if let Some(id) = lessons.get(li)
                    && app.curriculum.as_ref().and_then(|c| c.lesson(id)).is_some()
                {
                    open_lesson(app, ci, li);
                }
            }
        },
        _ => {}
    }
}

fn on_browser_key_v3(app: &mut App, key: &KeyEvent) {
    use crate::curriculum::v3_blueprint::{
        V3Stage, entries_for_stage, is_v3_entry_completed, is_v3_entry_unlocked,
        is_v3_stage_unlocked,
    };
    use BrowserPane::*;

    let stage_idx = app.browser.chapter_idx.min(V3Stage::ALL.len() - 1);
    let stage = V3Stage::ALL[stage_idx];
    let entries = entries_for_stage(stage);
    let preview = app.browser.v3_preview_unlocked;

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = Screen::Menu;
        }
        KeyCode::Char('r') => {
            open_next_due_srs(app);
        }
        KeyCode::Tab | KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
            app.browser.focus = match app.browser.focus {
                Chapters if !entries.is_empty() => Lessons,
                _ => Chapters,
            };
        }
        KeyCode::Up | KeyCode::Char('k') => match app.browser.focus {
            Chapters => app.browser.chapter_idx = app.browser.chapter_idx.saturating_sub(1),
            Lessons => app.browser.lesson_idx = app.browser.lesson_idx.saturating_sub(1),
        },
        KeyCode::Down | KeyCode::Char('j') => match app.browser.focus {
            Chapters => {
                app.browser.chapter_idx = (app.browser.chapter_idx + 1).min(V3Stage::ALL.len() - 1);
            }
            Lessons => {
                app.browser.lesson_idx =
                    (app.browser.lesson_idx + 1).min(entries.len().saturating_sub(1));
            }
        },
        KeyCode::Enter => match app.browser.focus {
            Chapters => {
                if !is_v3_stage_unlocked(&app.progress, stage, preview) {
                    app.set_status(
                        "Estágio bloqueado pelos pré-requisitos V3 (Alt+V libera prévia livre).",
                        StatusKind::Info,
                    );
                }
                if !entries.is_empty() {
                    app.browser.focus = Lessons;
                    app.browser.lesson_idx = entries
                        .iter()
                        .position(|e| !is_v3_entry_completed(&app.progress, e))
                        .unwrap_or(0);
                }
            }
            Lessons => {
                let li = app.browser.lesson_idx.min(entries.len().saturating_sub(1));
                if let Some(entry) = entries.get(li) {
                    if is_v3_entry_unlocked(&app.progress, entry, preview) {
                        open_v3_entry(app, entry.id);
                    } else {
                        app.set_status(
                            format!(
                                "Lição {} bloqueada — conclua antes: {} (ou Alt+V para pré-visualizar).",
                                entry.id,
                                entry.prerequisites.join(", ")
                            ),
                            StatusKind::Warning,
                        );
                    }
                }
            }
        },
        _ => {}
    }
}

/// Para um capítulo recém-aberto: a lição não concluída mais antiga.
fn first_unfinished(app: &App, lessons: &[String]) -> usize {
    lessons
        .iter()
        .position(|id| !app.progress.completed.contains(id))
        .unwrap_or(0)
}

/// Abre a primeira lição ou prática com revisão espaçada pendente no currículo.
fn open_next_due_srs(app: &mut App) {
    enum SrsTarget {
        Classic(String, usize, usize),
        V3(String, &'static str),
    }

    let due = app.progress.due_srs_items();
    let target = {
        let Some(cur) = app.curriculum.as_ref() else {
            return;
        };
        let resolve_lesson_pos = |lesson_id: &str| -> Option<(usize, usize)> {
            for (ci, ch) in cur.chapters.iter().enumerate() {
                if ch.status != ChapterStatus::Available {
                    continue;
                }
                if let Some(li) = ch.lessons.iter().position(|id| id == lesson_id)
                    && cur.lesson(lesson_id).is_some()
                {
                    return Some((ci, li));
                }
            }
            None
        };
        due.iter().find_map(|item_id| {
            if let Some((ci, li)) = resolve_lesson_pos(item_id) {
                return Some(SrsTarget::Classic(item_id.clone(), ci, li));
            }
            if let Some(spec) = ALL_PRACTICE_SPECS.iter().find(|s| s.id == item_id.as_str())
                && let Some((ci, li)) = resolve_lesson_pos(spec.lesson_id)
            {
                return Some(SrsTarget::Classic(item_id.clone(), ci, li));
            }
            if let Some(pos) = cur
                .exercise(item_id)
                .and_then(|ex| ex.lesson_ids.iter().find_map(|lid| resolve_lesson_pos(lid)))
            {
                return Some(SrsTarget::Classic(item_id.clone(), pos.0, pos.1));
            }
            crate::curriculum::v3_blueprint::find_v3_entry(item_id)
                .map(|entry| SrsTarget::V3(item_id.clone(), entry.id))
        })
    };

    match target {
        Some(SrsTarget::Classic(due_id, ci, li)) => {
            app.browser.chapter_idx = ci;
            app.browser.lesson_idx = li;
            open_lesson(app, ci, li);
            app.set_status(
                format!("Revisão espaçada aberta ({due_id}) — pratique de novo sem dicas!"),
                StatusKind::Info,
            );
        }
        Some(SrsTarget::V3(due_id, v3_id)) => {
            open_v3_entry(app, v3_id);
            app.set_status(
                format!("Revisão espaçada V3 aberta ({due_id}) — pratique de novo sem dicas!"),
                StatusKind::Info,
            );
        }
        None => {
            app.set_status(
                "Nenhuma revisão espaçada pendente no momento — avance nas lições!",
                StatusKind::Info,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{AudioBackend, AudioReport};
    use crate::config::Config;

    fn app_with_curriculum() -> App {
        let mut app = App::from_config(
            Config::default(),
            None,
            true,
            AudioReport {
                backend: AudioBackend::None,
                detail: String::new(),
            },
        );
        app.set_config_path(std::env::temp_dir().join("rustlab-learn-test.toml"));
        app.set_storage_path(std::env::temp_dir().join(format!(
            "rustlab-learn-progress-{}.toml",
            std::process::id()
        )));
        app.curriculum = Some(crate::curriculum::Curriculum::embedded().unwrap());
        app
    }

    fn press(app: &mut App, code: KeyCode) {
        app.handle_input(KeyEvent::new(code, KeyModifiers::NONE));
    }

    #[test]
    fn abrir_licao() {
        let mut app = app_with_curriculum();
        open_lesson(&mut app, 0, 0);
        assert!(matches!(app.screen, Screen::LessonView));
        assert!(app.lesson_view.is_some());
    }

    #[test]
    fn pratica_relacionada_volta_sem_alterar_quiz_ou_concluir_licao() {
        let mut app = app_with_curriculum();
        let (chapter, lesson, _) = app
            .curriculum
            .as_ref()
            .unwrap()
            .lesson_order()
            .into_iter()
            .find(|(_, _, id)| *id == "tipos-numericos")
            .unwrap();
        open_lesson(&mut app, chapter, lesson);
        app.lesson_view.as_mut().unwrap().scroll = 7;
        let before = app
            .lesson_view
            .as_ref()
            .unwrap()
            .quizzes
            .iter()
            .map(|q| q.solved)
            .collect::<Vec<_>>();
        press(&mut app, KeyCode::Char('e'));
        assert_eq!(app.screen, Screen::Exercises);
        let selected = app.exercises.as_ref().unwrap().selected;
        assert_eq!(
            app.curriculum
                .as_ref()
                .unwrap()
                .exercises()
                .nth(selected)
                .unwrap()
                .id,
            "media"
        );
        assert!(!app.exec.running);
        press(&mut app, KeyCode::Esc);
        assert_eq!(app.screen, Screen::LessonView);
        assert_eq!(app.lesson_view.as_ref().unwrap().scroll, 7);
        assert_eq!(
            app.lesson_view
                .as_ref()
                .unwrap()
                .quizzes
                .iter()
                .map(|q| q.solved)
                .collect::<Vec<_>>(),
            before
        );
        assert!(!app.progress.completed.contains("tipos-numericos"));
    }

    #[test]
    fn licao_sem_pratica_relacionada_avisa_sem_sair() {
        let mut app = app_with_curriculum();
        open_lesson(&mut app, 0, 0);
        press(&mut app, KeyCode::Char('e'));
        assert_eq!(app.screen, Screen::LessonView);
        assert!(app.status.as_ref().unwrap().message.contains("ainda não"));
    }

    #[test]
    fn quiz_bloqueia_avanco_ate_acertar() {
        let mut app = app_with_curriculum();
        open_lesson(&mut app, 0, 3); // hello-world (quiz correto = índice 1)
        assert!(!app.lesson_view.as_ref().unwrap().is_finished());
        press(&mut app, KeyCode::Enter);
        assert!(app.lesson_view.is_some());
        assert!(!app.progress.completed.contains("hello-world"));
        press(&mut app, KeyCode::Char('2'));
        assert_eq!(app.lesson_view.as_ref().unwrap().quizzes[0].solved, Some(1));
        press(&mut app, KeyCode::Enter);
        assert!(app.progress.completed.contains("hello-world"));
        let v = app.lesson_view.as_ref().unwrap();
        assert_eq!((v.chapter, v.lesson), (0, 4));
    }

    #[test]
    fn opcao_errada_registra_tentativa() {
        let mut app = app_with_curriculum();
        open_lesson(&mut app, 0, 0); // correta = índice 1
        press(&mut app, KeyCode::Char('1'));
        {
            let v = app.lesson_view.as_ref().unwrap();
            assert!(v.quizzes[0].solved.is_none());
            assert_eq!(v.quizzes[0].attempts, 1);
        }
        press(&mut app, KeyCode::Char('2'));
        assert_eq!(app.lesson_view.as_ref().unwrap().quizzes[0].solved, Some(1));
    }

    #[test]
    fn tecla_fora_do_range_do_quiz_e_ignorada() {
        let mut app = app_with_curriculum();
        open_lesson(&mut app, 0, 0); // quiz com 4 opções
        press(&mut app, KeyCode::Char('9')); // além de 4 → não conta tentativa
        assert_eq!(app.lesson_view.as_ref().unwrap().quizzes[0].attempts, 0);
    }

    #[test]
    fn licao_sem_quiz_exige_chegar_ao_fim() {
        let mut app = app_with_curriculum();
        open_lesson(&mut app, 0, 2); // conhecendo-o-rust, sem quiz
        press(&mut app, KeyCode::Enter); // no topo: ainda não chegou ao fim
        assert!(!app.progress.completed.contains("conhecendo-o-rust"));
        app.lesson_view.as_mut().unwrap().at_bottom = true; // o draw faria isso
        press(&mut app, KeyCode::Enter);
        assert!(app.progress.completed.contains("conhecendo-o-rust"));
    }

    #[test]
    fn esc_registra_leitura_e_volta_ao_browser() {
        let mut app = app_with_curriculum();
        open_lesson(&mut app, 0, 1);
        press(&mut app, KeyCode::Esc);
        assert!(app.progress.read.contains("como-um-programa-executa"));
        assert!(matches!(app.screen, Screen::Browser));
    }

    #[test]
    fn demo_avanca_e_da_volta() {
        let mut app = app_with_curriculum();
        open_lesson(&mut app, 0, 1); // demo de 4 passos
        for _ in 0..3 {
            press(&mut app, KeyCode::Char('d'));
        }
        assert_eq!(app.lesson_view.as_ref().unwrap().demos[0], 3);
        press(&mut app, KeyCode::Char('d'));
        assert_eq!(app.lesson_view.as_ref().unwrap().demos[0], 0);
    }

    #[test]
    fn navegacao_entre_capitulos() {
        let mut app = app_with_curriculum();
        open_lesson(&mut app, 0, 4); // anatomia (quiz correto = índice 2)
        press(&mut app, KeyCode::Char('3'));
        press(&mut app, KeyCode::Enter);
        let v = app.lesson_view.as_ref().unwrap();
        assert_eq!((v.chapter, v.lesson), (1, 0)); // primeira lição da Etapa 2
    }

    #[test]
    fn fim_da_linha_de_lcoes_volta_ao_browser() {
        let mut app = app_with_curriculum();
        open_lesson(&mut app, 9, 1); // projeto-independente-qualidade-e-entrega (correta = índice 0)
        press(&mut app, KeyCode::Char('1'));
        press(&mut app, KeyCode::Enter);
        assert!(matches!(app.screen, Screen::Browser));
        assert!(app.lesson_view.is_none());
    }

    #[test]
    fn progresso_de_sessao_limpo_no_inico() {
        let app = app_with_curriculum();
        assert!(app.progress.read.is_empty());
        assert!(app.progress.completed.is_empty());
    }

    #[test]
    fn ferris_ascii_canonico_v3() {
        assert_eq!(FerrisMood::Neutral.ascii(), "c(o.o)c");
        assert_eq!(FerrisMood::Happy.ascii(), "c(ovo)c");
        assert_eq!(FerrisMood::Encouraging.ascii(), "c(o,x,o)c");
        assert_eq!(FerrisMood::Confused.ascii(), "c(o,0)c");
    }

    #[test]
    fn xp_v3_100_mais_100_sem_dicas() {
        assert_eq!(calculate_v3_practice_xp(0, false), (100, 100));
        assert_eq!(calculate_v3_practice_xp(1, false), (100, 0));
        assert_eq!(calculate_v3_practice_xp(0, true), (100, 0));
    }

    #[test]
    fn validacao_pratica_bp01_rejeita_placeholder_e_aceita_saida_real() {
        let spec = practice_spec_for_lesson("hello-world").unwrap();
        assert!(validate_practice_run("B-P01", spec.starter, "...").is_err());
        assert!(
            validate_practice_run("B-P01", "fn main() {\n    println!(\"...\");\n}\n", "...")
                .is_err()
        );
        assert!(
            validate_practice_run(
                "B-P01",
                "fn main() {\n    println!(\"Olá, RustLab!\");\n}\n",
                "Olá, RustLab!"
            )
            .is_ok()
        );
    }

    #[test]
    fn pratica_embutida_preserva_rascunho_ao_alternar_foco_e_sair() {
        let mut app = app_with_curriculum();
        open_lesson(&mut app, 0, 3); // hello-world -> B-P01
        assert!(!app.lesson_view.as_ref().unwrap().practice_focused);

        press(&mut app, KeyCode::Tab);
        assert!(app.lesson_view.as_ref().unwrap().practice_focused);

        app.lesson_view
            .as_mut()
            .unwrap()
            .practice
            .as_mut()
            .unwrap()
            .editor = Editor::new(
            "fn main() {\n    println!(\"Meu primeiro app!\");\n}\n",
            None,
        );
        save_active_practice_draft(&mut app);

        // Esc sai primeiro da mini-IDE embutida preservando o rascunho
        press(&mut app, KeyCode::Esc);
        assert!(!app.lesson_view.as_ref().unwrap().practice_focused);
        assert_eq!(app.screen, Screen::LessonView);

        // Segundo Esc sai para Browser; reabrir restaura exatamente o rascunho
        press(&mut app, KeyCode::Esc);
        assert_eq!(app.screen, Screen::Browser);
        open_lesson(&mut app, 0, 3);
        let restored = app
            .lesson_view
            .as_ref()
            .unwrap()
            .practice
            .as_ref()
            .unwrap()
            .editor
            .buf
            .to_string_lossy();
        assert!(restored.contains("Meu primeiro app!"));

        // Verifica também premiação real 100 + 100 XP no encerramento do Cargo
        on_lesson_exec_event(
            &mut app,
            ExecEvent::Chunk {
                kind: StreamKind::Stdout,
                text: "Meu primeiro app!".to_string(),
            },
        );
        on_lesson_exec_event(
            &mut app,
            ExecEvent::Finished {
                label: "cargo run".to_string(),
                success: true,
                code: Some(0),
            },
        );
        assert_eq!(app.lesson_view.as_ref().unwrap().ferris, FerrisMood::Happy);
        assert_eq!(app.progress.xp_events.get("practice:B-P01"), Some(&100));
        assert_eq!(
            app.progress.xp_events.get("practice:B-P01:independent"),
            Some(&100)
        );
    }

    #[test]
    fn srs_calcula_vencimento_e_maestria() {
        let mut p = Progress::default();
        p.record_srs_mastery("B-T09-P", true);
        assert_eq!(p.current_learning_step(), 1);
        assert_eq!(p.srs_cards["B-T09-P"].mastery_level, 1);
        assert_eq!(p.srs_cards["B-T09-P"].due_at_step, 3);
        assert!(p.due_srs_items().is_empty());

        // Assistido reduz/agenda cedo (intervalo 1)
        p.record_srs_mastery("B-T10-P", false);
        assert_eq!(p.srs_cards["B-T10-P"].mastery_level, 0);
        assert_eq!(p.srs_cards["B-T10-P"].interval_steps, 1);
        assert!(p.due_srs_items().is_empty());

        // Avançando 1 passo faz ambos B-T09-P (due=3) e B-T10-P (due=3) vencerem juntos
        p.record_srs_mastery("B-P01", true);
        assert_eq!(p.current_learning_step(), 3);
        let due = p.due_srs_items();
        assert_eq!(due, vec!["B-T10-P".to_string(), "B-T09-P".to_string()]);

        // Re-revisar B-T09-P com sucesso sobe maestria para 2 e empurra prazo para frente
        p.record_srs_mastery("B-T09-P", true);
        assert_eq!(p.srs_cards["B-T09-P"].mastery_level, 2);
        assert!(!p.due_srs_items().contains(&"B-T09-P".to_string()));
    }

    #[test]
    fn explicador_offline_de_erros_cobre_codigos_comuns() {
        for (sample, expected) in [
            ("error[E0308]: mismatched types", "E0308"),
            (
                "error[E0384]: cannot assign twice to immutable variable",
                "E0384",
            ),
            ("error[E0382]: borrow of moved value: `s`", "E0382"),
            ("error[E0597]: `x` does not live long enough", "E0597"),
            ("error[E0425]: cannot find value `foo`", "E0425"),
            ("error: expected `;`, found `}`", "Ponto e vírgula"),
        ] {
            let msg = explain_compiler_error_pt_br(sample)
                .unwrap_or_else(|| panic!("esperava explicação para: {sample}"));
            assert!(msg.contains(expected), "mensagem inesperada: {msg}");
        }
        assert!(explain_compiler_error_pt_br("warning: unused variable").is_none());
    }

    #[test]
    fn browser_tecla_r_abre_revisao_pendente_ou_avisa() {
        let mut app = app_with_curriculum();
        app.screen = Screen::Browser;
        press(&mut app, KeyCode::Char('r'));
        assert_eq!(app.screen, Screen::Browser);
        assert!(
            app.status
                .as_ref()
                .unwrap()
                .message
                .contains("Nenhuma revisão")
        );

        app.progress.srs_cards.insert(
            "B-T09-P".to_string(),
            crate::storage::SrsCard {
                mastery_level: 1,
                interval_steps: 2,
                due_at_step: 0,
                independent_passes: 1,
                review_count: 1,
            },
        );
        press(&mut app, KeyCode::Char('r'));
        assert_eq!(app.screen, Screen::LessonView);
        assert_eq!(lesson_of(&app).map(|l| l.id.as_str()), Some("mutabilidade"));
    }

    #[test]
    fn trilha_v3_navega_6_estagios_abre_licao_e_respeita_gating() {
        let mut app = app_with_curriculum();
        app.screen = Screen::Browser;
        // Já começa diretamente na Trilha V3 (213 entradas)
        assert!(app.browser.v3_mode);
        assert_eq!(app.browser.focus, BrowserPane::Chapters);

        // `v` ou `V` alterna para a visão clássica (10 capítulos) e volta para a V3
        press(&mut app, KeyCode::Char('V'));
        assert!(!app.browser.v3_mode);
        press(&mut app, KeyCode::Char('v'));
        assert!(app.browser.v3_mode);

        // Entra em O BÁSICO e abre B-T01 (desbloqueada)
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.browser.focus, BrowserPane::Lessons);
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.screen, Screen::LessonView);
        assert_eq!(
            app.lesson_view.as_ref().and_then(|v| v.v3_entry_id),
            Some("B-T01")
        );

        // Responder quiz 1 (correto = 0) e avançar leva para B-T02
        press(&mut app, KeyCode::Char('1'));
        press(&mut app, KeyCode::Enter);
        assert_eq!(
            app.lesson_view.as_ref().and_then(|v| v.v3_entry_id),
            Some("B-T02")
        );

        // Volta ao Browser e tenta abrir M01 sem concluir B-P02: bloqueado até Alt+V
        press(&mut app, KeyCode::Esc);
        assert_eq!(app.screen, Screen::Browser);
        press(&mut app, KeyCode::Tab);
        app.browser.chapter_idx = 1; // O MEIO
        press(&mut app, KeyCode::Enter); // foca entradas
        press(&mut app, KeyCode::Enter); // M01-T01 bloqueada sem B-P02
        assert_eq!(app.screen, Screen::Browser);

        // Alt+V libera pré-visualização e permite abrir M01-T01
        app.handle_input(KeyEvent::new(KeyCode::Char('v'), KeyModifiers::ALT));
        assert!(app.browser.v3_preview_unlocked);
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.screen, Screen::LessonView);
        assert_eq!(
            app.lesson_view.as_ref().and_then(|v| v.v3_entry_id),
            Some("M01-T01")
        );

        assert_eq!(PracticeOutcome::Passed.status_badge(), "[Aprovado]");
        assert_eq!(
            PracticeOutcome::CompileFailed.status_badge(),
            "[Erro de compilação]"
        );
    }
}
