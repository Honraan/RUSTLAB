//! Esquema de lições e currículo embutido.
//!
//! As lições vivem em arquivos TOML compilados dentro do binário
//! (`include_str!`): são dados, não código — assim a validação é
//! determinística, testa-se com `cargo test`, e nada de conteúdo
//! desconhecido pode ser interpretado como instrução.
//!
//! Formato (resumo):
//!
//! ```toml
//! id = "variaveis"
//! title = "Variáveis"
//! estimated_minutes = 6
//!
//! [[blocks]]
//! type = "text"
//! text = "..."
//!
//! [[blocks]]
//! type = "code"
//! code = "let x = 3;"
//! file = "src/main.rs"
//! explain = "..."
//!
//! [[blocks]]
//! type = "callout"
//! kind = "note"        # note | tip | warning
//! text = "..."
//!
//! [[blocks]]
//! type = "demo"
//! demo = "variables"
//! name = "idade"
//! steps = ["20", "31"]
//!
//! [[blocks]]
//! type = "quiz"
//! question = "..."
//! options = ["...", "..."]
//! correct = 1
//! why = "..."
//! ```

pub mod en_tr;
pub mod v3_blueprint;

use std::collections::HashMap;

use serde::Deserialize;

/// Nível de preparação de um capítulo. Capítulos `Planned` são exibidos
/// no navegador com a fase prevista — sem fingir conteúdo inexistente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChapterStatus {
    Available,
    Planned,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub order: u32,
    pub status: ChapterStatus,
    /// Quando Planned: em qual fase do roadmap este capítulo chega.
    #[serde(default)]
    pub planned_phase: Option<u8>,
    /// ids das lições (nesta ordem)
    #[serde(default)]
    pub lessons: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CalloutKind {
    #[default]
    Note,
    Tip,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DemoKind {
    /// Caixa rotulada segurando um valor; passos avançam com `d`.
    Variables,
    /// Iteração passo a passo em um laço (`for` / `while` / `loop`).
    Loop,
    /// Transformação entrada -> função -> saída.
    Function,
    /// Comparação conceitual entre Stack (tamanho fixo) e Heap (alocação dinâmica).
    StackHeap,
    /// Transferência de posse (move semantics).
    Ownership,
    /// Empréstimo imutável (`&T`) e mutável (`&mut T`).
    Borrowing,
    /// Inserção e remoção em um `Vec<T>`.
    Vector,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Block {
    Text {
        text: String,
    },
    Code {
        code: String,
        #[serde(default)]
        file: Option<String>,
        #[serde(default)]
        explain: Option<String>,
    },
    Callout {
        #[serde(default)]
        kind: CalloutKind,
        text: String,
    },
    Quiz {
        question: String,
        options: Vec<String>,
        correct: usize,
        #[serde(default)]
        why: Option<String>,
    },
    Demo {
        demo: DemoKind,
        name: String,
        steps: Vec<String>,
        #[serde(default)]
        caption: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub estimated_minutes: u32,
    pub blocks: Vec<Block>,
}

/// Exercício autocontido: o código de teste é compilado com o esqueleto do
/// aluno num crate temporário e validado pelo cargo test real.
#[derive(Debug, Clone, Deserialize)]
pub struct Exercise {
    pub id: String,
    /// Ordem pedagógica; IDs persistidos não mudam ao reordenar o catálogo.
    #[serde(default)]
    pub order: u32,
    #[serde(default)]
    pub lesson_ids: Vec<String>,
    pub title: String,
    pub statement: String,
    pub skeleton: String,
    pub tests: String,
    #[serde(default)]
    pub hints: Vec<String>,
}

impl Lesson {
    /// Lições sem quiz podem ser concluídas por leitura; com quiz, o
    /// quiz é a validação determinística de compreensão (requisito 8).
    pub fn requires_quiz(&self) -> bool {
        self.blocks.iter().any(|b| matches!(b, Block::Quiz { .. }))
    }

    pub fn count_demos(&self) -> usize {
        self.blocks
            .iter()
            .filter(|b| matches!(b, Block::Demo { .. }))
            .count()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ChapterFile {
    #[serde(flatten)]
    chapter: Chapter,
}

#[derive(Debug, Clone, Deserialize)]
struct LessonFile {
    #[serde(flatten)]
    lesson: Lesson,
}

#[derive(Debug, Clone, Deserialize)]
struct ExerciseFile {
    #[serde(flatten)]
    exercise: Exercise,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurriculumError {
    pub message: String,
}

impl std::fmt::Display for CurriculumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub struct Curriculum {
    pub chapters: Vec<Chapter>,
    lessons: HashMap<String, Lesson>,
    exercises: HashMap<String, Exercise>,
}

impl Curriculum {
    pub fn lesson(&self, id: &str) -> Option<&Lesson> {
        self.lessons.get(id)
    }

    pub fn exercise(&self, id: &str) -> Option<&Exercise> {
        self.exercises.get(id)
    }

    pub fn exercises(&self) -> impl Iterator<Item = &Exercise> {
        let mut list: Vec<_> = self.exercises.values().collect();
        list.sort_by(|a, b| a.order.cmp(&b.order).then_with(|| a.id.cmp(&b.id)));
        list.into_iter()
    }

    /// Índice plano de lições (capítulo, índice_na_lição, id) na ordem do
    /// curso — usado por `n`/`p` para atravessar capítulos.
    pub fn lesson_order(&self) -> Vec<(usize, usize, &str)> {
        let mut out = Vec::new();
        for (ci, c) in self.chapters.iter().enumerate() {
            if c.status != ChapterStatus::Available {
                continue;
            }
            for (li, id) in c.lessons.iter().enumerate() {
                out.push((ci, li, id.as_str()));
            }
        }
        out
    }

    /// Carrega o currículo embutido no binário e o valida
    /// integralmente. Qualquer inconsistência devolve erro descritivo —
    /// o app então mostra um aviso em vez de meio-carregar.
    pub fn embedded() -> Result<Curriculum, CurriculumError> {
        let mut chapters = Vec::new();
        let mut lessons = HashMap::new();
        let mut exercises = HashMap::new();

        for (src, text) in EMBEDDED_CHAPTERS {
            let cf: ChapterFile = toml::from_str(text)
                .map_err(|e| err(format!("capítulo {src}: {}", e.message())))?;
            chapters.push(cf.chapter);
        }
        for (src, text) in EMBEDDED_LESSONS {
            let lf: LessonFile =
                toml::from_str(text).map_err(|e| err(format!("lição {src}: {}", e.message())))?;
            lessons.insert(lf.lesson.id.clone(), lf.lesson);
        }

        for (src, text) in EMBEDDED_EXERCISES {
            let ef: ExerciseFile = toml::from_str(text)
                .map_err(|e| err(format!("exercício {src}: {}", e.message())))?;
            if exercises
                .insert(ef.exercise.id.clone(), ef.exercise)
                .is_some()
            {
                return Err(err(format!("id de exercício duplicado: {src}")));
            }
        }

        Self::validate(chapters, lessons, exercises, src_of())
    }

    /// Carrega o currículo embutido no idioma configurado (`en-US` quando `english == true`).
    pub fn embedded_for(english: bool) -> Result<Curriculum, CurriculumError> {
        let mut cur = Self::embedded()?;
        if english {
            en_tr::translate_curriculum_to_en(&mut cur);
        }
        Ok(cur)
    }

    fn validate(
        mut chapters: Vec<Chapter>,
        lessons: HashMap<String, Lesson>,
        exercises: HashMap<String, Exercise>,
        source_of: impl Fn(&str) -> &'static str,
    ) -> Result<Curriculum, CurriculumError> {
        // ordem e referências
        chapters.sort_by_key(|c| c.order);
        for c in &chapters {
            if let ChapterStatus::Available = c.status
                && c.lessons.is_empty()
            {
                return Err(err(format!("capítulo disponível '{}' sem lições", c.id)));
            }
            for id in &c.lessons {
                if !lessons.contains_key(id) {
                    return Err(err(format!(
                        "capítulo '{}' referencia lição inexistente '{id}' (fonte: {})",
                        c.id,
                        source_of(&c.id)
                    )));
                }
            }
        }
        // integridade das lições
        for (id, lesson) in &lessons {
            if lesson.blocks.is_empty() {
                return Err(err(format!("lição '{id}' sem blocos")));
            }
            for block in &lesson.blocks {
                match block {
                    Block::Quiz {
                        options, correct, ..
                    } => {
                        if options.is_empty() || *correct >= options.len() {
                            return Err(err(format!(
                                "lição '{id}': quiz com índice correto inválido ({correct} de {})",
                                options.len()
                            )));
                        }
                        if options.len() < 2 {
                            return Err(err(format!("lição '{id}': quiz com menos de 2 opções")));
                        }
                    }
                    Block::Demo { steps, .. } if steps.is_empty() => {
                        return Err(err(format!("lição '{id}': demo sem passos")));
                    }
                    _ => {}
                }
            }
        }
        for (id, exercise) in &exercises {
            for lesson_id in &exercise.lesson_ids {
                if !lessons.contains_key(lesson_id) {
                    return Err(err(format!(
                        "exercício '{id}' referencia lição inexistente '{lesson_id}'"
                    )));
                }
            }
            if exercise.title.trim().is_empty()
                || exercise.statement.trim().is_empty()
                || exercise.skeleton.trim().is_empty()
                || exercise.tests.trim().is_empty()
            {
                return Err(err(format!(
                    "exercício '{id}' precisa de título, enunciado, esqueleto e testes"
                )));
            }
            if exercise.hints.len() != 4 {
                return Err(err(format!(
                    "exercício '{id}' precisa de exatamente quatro níveis de dica"
                )));
            }
            if exercise.hints.iter().any(|hint| hint.trim().is_empty()) {
                return Err(err(format!("exercício '{id}' tem dica vazia")));
            }
            if id.is_empty()
                || !id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            {
                return Err(err(format!("id inválido no exercício '{id}'")));
            }
        }
        Ok(Curriculum {
            chapters,
            lessons,
            exercises,
        })
    }
}

fn err(message: String) -> CurriculumError {
    CurriculumError { message }
}

fn src_of() -> impl Fn(&str) -> &'static str {
    move |id| {
        for (src, text) in EMBEDDED_CHAPTERS {
            if text.contains(&format!("id = \"{id}\"")) {
                return src;
            }
        }
        "<desconhecido>"
    }
}

// ---------------------------------------------------------------------------
// Conteúdo embutido
// ---------------------------------------------------------------------------

/// Capítulos disponíveis: Etapas 1 e 2 escritas e revisadas; o resto do
/// roadmap aparece como planejado.
pub const EMBEDDED_CHAPTERS: &[(&str, &str)] = &[
    (
        "chapter/etapa-1.toml",
        include_str!("../content/chapter/etapa-1.toml"),
    ),
    (
        "chapter/etapa-2.toml",
        include_str!("../content/chapter/etapa-2.toml"),
    ),
    (
        "chapter/etapa-3.toml",
        include_str!("../content/chapter/etapa-3.toml"),
    ),
    (
        "chapter/etapa-4.toml",
        include_str!("../content/chapter/etapa-4.toml"),
    ),
    (
        "chapter/etapa-5.toml",
        include_str!("../content/chapter/etapa-5.toml"),
    ),
    (
        "chapter/etapa-6.toml",
        include_str!("../content/chapter/etapa-6.toml"),
    ),
    (
        "chapter/etapa-7.toml",
        include_str!("../content/chapter/etapa-7.toml"),
    ),
    (
        "chapter/etapa-8.toml",
        include_str!("../content/chapter/etapa-8.toml"),
    ),
    (
        "chapter/etapa-9.toml",
        include_str!("../content/chapter/etapa-9.toml"),
    ),
    (
        "chapter/etapa-10.toml",
        include_str!("../content/chapter/etapa-10.toml"),
    ),
];

pub const EMBEDDED_LESSONS: &[(&str, &str)] = &[
    (
        "o-que-e-programar",
        include_str!("../content/lesson/o-que-e-programar.toml"),
    ),
    (
        "como-um-programa-executa",
        include_str!("../content/lesson/como-um-programa-executa.toml"),
    ),
    (
        "conhecendo-o-rust",
        include_str!("../content/lesson/conhecendo-o-rust.toml"),
    ),
    (
        "hello-world",
        include_str!("../content/lesson/hello-world.toml"),
    ),
    (
        "anatomia-do-hello-world",
        include_str!("../content/lesson/anatomia-do-hello-world.toml"),
    ),
    (
        "variaveis",
        include_str!("../content/lesson/variaveis.toml"),
    ),
    (
        "mutabilidade",
        include_str!("../content/lesson/mutabilidade.toml"),
    ),
    (
        "tipos-numericos",
        include_str!("../content/lesson/tipos-numericos.toml"),
    ),
    (
        "operadores",
        include_str!("../content/lesson/operadores.toml"),
    ),
    (
        "condicoes-lacos-e-funcoes",
        include_str!("../content/lesson/condicoes-lacos-e-funcoes.toml"),
    ),
    (
        "strings-e-fatias",
        include_str!("../content/lesson/strings-e-fatias.toml"),
    ),
    (
        "tuplas-arrays-e-vetores",
        include_str!("../content/lesson/tuplas-arrays-e-vetores.toml"),
    ),
    (
        "colecoes-e-iteracao",
        include_str!("../content/lesson/colecoes-e-iteracao.toml"),
    ),
    (
        "ownership-na-pratica",
        include_str!("../content/lesson/ownership-na-pratica.toml"),
    ),
    (
        "borrowing-e-referencias",
        include_str!("../content/lesson/borrowing-e-referencias.toml"),
    ),
    (
        "tempo-de-vida-introducao",
        include_str!("../content/lesson/tempo-de-vida-introducao.toml"),
    ),
    (
        "structs-e-metodos",
        include_str!("../content/lesson/structs-e-metodos.toml"),
    ),
    (
        "enums-e-pattern-matching",
        include_str!("../content/lesson/enums-e-pattern-matching.toml"),
    ),
    (
        "traits-modulos-e-cargo",
        include_str!("../content/lesson/traits-modulos-e-cargo.toml"),
    ),
    (
        "option-e-ausencia",
        include_str!("../content/lesson/option-e-ausencia.toml"),
    ),
    (
        "result-e-propagacao",
        include_str!("../content/lesson/result-e-propagacao.toml"),
    ),
    (
        "testes-documentacao-e-depuracao",
        include_str!("../content/lesson/testes-documentacao-e-depuracao.toml"),
    ),
    (
        "genericos-e-closures",
        include_str!("../content/lesson/genericos-e-closures.toml"),
    ),
    (
        "iteradores-e-arquivos",
        include_str!("../content/lesson/iteradores-e-arquivos.toml"),
    ),
    (
        "cli-configuracao-e-serde",
        include_str!("../content/lesson/cli-configuracao-e-serde.toml"),
    ),
    (
        "rede-e-programacao-assincrona",
        include_str!("../content/lesson/rede-e-programacao-assincrona.toml"),
    ),
    (
        "projetos-guiados-calculadora-e-jogo",
        include_str!("../content/lesson/projetos-guiados-calculadora-e-jogo.toml"),
    ),
    (
        "projetos-guiados-tarefas-e-notas",
        include_str!("../content/lesson/projetos-guiados-tarefas-e-notas.toml"),
    ),
    (
        "projeto-independente-planejamento",
        include_str!("../content/lesson/projeto-independente-planejamento.toml"),
    ),
    (
        "projeto-independente-qualidade-e-entrega",
        include_str!("../content/lesson/projeto-independente-qualidade-e-entrega.toml"),
    ),
];

pub const EMBEDDED_EXERCISES: &[(&str, &str)] = &[
    ("dobro", include_str!("../content/exercise/dobro.toml")),
    ("media", include_str!("../content/exercise/media.toml")),
    (
        "intervalo",
        include_str!("../content/exercise/intervalo.toml"),
    ),
    ("soma", include_str!("../content/exercise/soma.toml")),
    (
        "paridade",
        include_str!("../content/exercise/paridade.toml"),
    ),
    ("maior", include_str!("../content/exercise/maior.toml")),
    (
        "contar-vogais",
        include_str!("../content/exercise/contar-vogais.toml"),
    ),
    (
        "filtrar-positivos",
        include_str!("../content/exercise/filtrar-positivos.toml"),
    ),
    (
        "primeira-palavra",
        include_str!("../content/exercise/primeira-palavra.toml"),
    ),
    (
        "retangulo-area",
        include_str!("../content/exercise/retangulo-area.toml"),
    ),
    (
        "divisao-segura",
        include_str!("../content/exercise/divisao-segura.toml"),
    ),
    (
        "soma-quadrados-pares",
        include_str!("../content/exercise/soma-quadrados-pares.toml"),
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curriculum_embutido_valida() {
        let c = Curriculum::embedded().expect("currículo embutido deve ser válido");
        assert!(!c.chapters.is_empty());
        // todas as etapas planejadas têm fase prevista
        for ch in &c.chapters {
            if ch.status == ChapterStatus::Planned {
                assert!(ch.planned_phase.is_some(), "{} planejado sem fase", ch.id);
            }
        }
    }

    #[test]
    fn ordem_das_lcoes_cobre_as_disponiveis() {
        let c = Curriculum::embedded().unwrap();
        let order = c.lesson_order();
        assert!(
            order.len() >= 8,
            "currículo inicial tem poucas lições: {}",
            order.len()
        );
        for (_, _, id) in &order {
            assert!(c.lesson(id).is_some(), "ordem referencia id ausente {id}");
        }
    }

    #[test]
    fn toda_lcao_explica_antes_de_exigir() {
        // Heurística estrutural do requisito 8: lições que cobram quiz
        // precisam primeiro de texto de explicação (bloco Text antes do
        // primeiro Quiz) — não exigimos uso antes de explicação.
        let c = Curriculum::embedded().unwrap();
        for (_, _, id) in c.lesson_order() {
            let l = c.lesson(id).unwrap();
            let first_quiz = l
                .blocks
                .iter()
                .position(|b| matches!(b, Block::Quiz { .. }));
            if let Some(q) = first_quiz {
                assert!(
                    l.blocks[..q]
                        .iter()
                        .any(|b| matches!(b, Block::Text { .. })),
                    "lição {id}: quiz antes de qualquer explicação"
                );
            }
        }
    }

    #[test]
    fn quiz_invalido_e_pego() {
        let bad = Lesson {
            id: "x".into(),
            title: "X".into(),
            estimated_minutes: 1,
            blocks: vec![Block::Quiz {
                question: "Q".into(),
                options: vec!["a".into()],
                correct: 5,
                why: None,
            }],
        };
        let mut lessons = HashMap::new();
        lessons.insert("x".into(), bad.clone());
        let chapters = vec![Chapter {
            id: "c".into(),
            title: "C".into(),
            summary: String::new(),
            order: 1,
            status: ChapterStatus::Available,
            planned_phase: None,
            lessons: vec!["x".into()],
        }];
        let res = Curriculum::validate(chapters, lessons, HashMap::new(), |_| "teste");
        assert!(
            res.is_err(),
            "quiz com correct fora do range deveria falhar"
        );
    }

    #[test]
    fn capitulo_referenciando_lcao_fantasma_falha() {
        let chapters = vec![Chapter {
            id: "c".into(),
            title: "C".into(),
            summary: String::new(),
            order: 1,
            status: ChapterStatus::Available,
            planned_phase: None,
            lessons: vec!["nao-existe".into()],
        }];
        let res = Curriculum::validate(chapters, HashMap::new(), HashMap::new(), |_| "teste");
        assert!(res.is_err());
    }

    #[test]
    fn requires_quiz_e_contagem_de_demos() {
        let c = Curriculum::embedded().unwrap();
        let mut com_quiz = 0;
        let mut com_demo = 0;
        for (_, _, id) in c.lesson_order() {
            let l = c.lesson(id).unwrap();
            if l.requires_quiz() {
                com_quiz += 1;
            }
            if l.count_demos() > 0 {
                com_demo += 1;
            }
        }
        assert!(com_quiz >= 3, "currículo inicial deveria ter quizzes");
        assert!(
            com_demo >= 1,
            "currículo inicial deveria ter ao menos uma demo"
        );
    }

    #[test]
    fn exercise_embedded_has_four_graduated_hints_and_real_tests() {
        let c = Curriculum::embedded().unwrap();
        let ex = c.exercise("soma").unwrap();
        assert_eq!(ex.hints.len(), 4);
        assert!(ex.tests.contains("assert_eq!"));
        assert!(ex.skeleton.contains("todo!"));
    }

    #[test]
    fn catalogo_tem_ordem_pedagogica_e_quatro_dicas_por_exercicio() {
        let curriculum = Curriculum::embedded().unwrap();
        let ids: Vec<_> = curriculum.exercises().map(|ex| ex.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "soma",
                "dobro",
                "media",
                "paridade",
                "maior",
                "intervalo",
                "contar-vogais",
                "filtrar-positivos",
                "primeira-palavra",
                "retangulo-area",
                "divisao-segura",
                "soma-quadrados-pares",
            ]
        );
        for ex in curriculum.exercises() {
            assert_eq!(ex.hints.len(), 4);
            assert!(ex.hints.iter().all(|hint| !hint.trim().is_empty()));
            assert!(ex.skeleton.contains("todo!"));
            assert!(ex.tests.contains("#[test]"));
        }
    }

    #[test]
    fn exercicio_nao_pode_referenciar_licao_inexistente() {
        let mut curriculum = Curriculum::embedded().unwrap();
        curriculum
            .exercises
            .get_mut("soma")
            .unwrap()
            .lesson_ids
            .push("fantasma".into());
        let result = Curriculum::validate(
            curriculum.chapters,
            curriculum.lessons,
            curriculum.exercises,
            |_| "teste",
        );
        assert!(result.err().unwrap().message.contains("lição inexistente"));
    }

    #[test]
    fn curriculo_classico_em_ingles_tem_capitulos_licoes_e_exercicios_traduzidos() {
        let cur_pt = Curriculum::embedded_for(false).unwrap();
        let cur_en = Curriculum::embedded_for(true).unwrap();

        assert_eq!(cur_en.chapters.len(), 10);
        assert_eq!(cur_en.lesson_order().len(), 30);
        assert_eq!(cur_en.exercises().count(), 12);

        for (ch_pt, ch_en) in cur_pt.chapters.iter().zip(cur_en.chapters.iter()) {
            assert!(!ch_en.title.trim().is_empty());
            assert_ne!(ch_pt.title, ch_en.title, "chapter {}", ch_en.id);
        }

        for (_, _, id) in cur_en.lesson_order() {
            let l_pt = cur_pt.lesson(id).unwrap();
            let l_en = cur_en.lesson(id).unwrap();
            assert!(!l_en.title.trim().is_empty());
            assert_ne!(l_pt.title, l_en.title, "lesson {id}");
            assert!(!l_en.blocks.is_empty());
        }

        for ex_en in cur_en.exercises() {
            let ex_pt = cur_pt.exercise(&ex_en.id).unwrap();
            assert!(!ex_en.title.trim().is_empty());
            assert!(!ex_en.statement.trim().is_empty());
            assert_ne!(ex_pt.title, ex_en.title, "exercise {}", ex_en.id);
            assert_ne!(ex_pt.statement, ex_en.statement, "exercise {}", ex_en.id);
            assert!(!ex_en.hints.is_empty());
            assert!(!ex_en.tests.trim().is_empty());
        }
    }
}
