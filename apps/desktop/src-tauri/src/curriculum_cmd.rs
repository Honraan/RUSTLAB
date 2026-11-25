use serde::Serialize;

use rustlab::curriculum::v3_blueprint::{
    V3EntryKind, V3Stage, build_v3_lesson_for, build_v3_practice_for_any_entry,
    build_v3_practice_spec_for, entries_for_stage, find_v3_entry, is_v3_entry_completed,
    is_v3_entry_unlocked, is_v3_stage_unlocked, v3_curriculum_blueprint,
};
use rustlab::curriculum::{Block, CalloutKind, DemoKind};
use rustlab::learn::PracticeMode;

use crate::error::{CmdResult, CommandError};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseStageDto {
    pub id: String,
    pub label: String,
    pub subtitle: String,
    pub unlocked: bool,
    pub completed_count: usize,
    pub total_count: usize,
    pub entries: Vec<CourseEntrySummaryDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseEntrySummaryDto {
    pub id: String,
    pub title: String,
    pub stage_id: String,
    pub kind: String,
    pub cycle: Option<String>,
    pub prerequisites: Vec<String>,
    pub learning_goal: String,
    pub completion_evidence: String,
    pub unlocked: bool,
    pub completed: bool,
    pub read: bool,
    pub is_interactive_stdin: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseOverviewDto {
    pub total_entries: usize,
    pub completed_entries: usize,
    pub xp: u32,
    pub level: u32,
    pub last_entry_id: Option<String>,
    pub due_srs_items: Vec<String>,
    pub stages: Vec<CourseStageDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LessonBlockDto {
    Text {
        text: String,
    },
    Code {
        code: String,
        file: Option<String>,
        explain: Option<String>,
    },
    Callout {
        kind: String,
        text: String,
    },
    Demo {
        demo: String,
        name: String,
        steps: Vec<String>,
        caption: Option<String>,
    },
    Quiz {
        quiz_index: usize,
        question: String,
        options: Vec<String>,
        correct: usize,
        why: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeSpecDto {
    pub id: String,
    pub lesson_id: String,
    pub title: String,
    pub prompt: String,
    pub starter: String,
    pub current_code: String,
    pub hints: Vec<String>,
    pub shown_hints: usize,
    pub mode: String,
    pub stdin_preview: Vec<String>,
    pub completed: bool,
    pub has_idiomatic: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonDetailDto {
    pub id: String,
    pub title: String,
    pub stage_id: String,
    pub stage_label: String,
    pub kind: String,
    pub completed: bool,
    pub requires_quiz: bool,
    pub ferris_mood: String,
    pub ferris_ascii: String,
    pub blocks: Vec<LessonBlockDto>,
    pub practice: Option<PracticeSpecDto>,
    pub prev_entry_id: Option<String>,
    pub next_entry_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassicExerciseDto {
    pub id: String,
    pub title: String,
    pub statement: String,
    pub starter: String,
    pub current_code: String,
    pub hints: Vec<String>,
    pub shown_hints: usize,
    pub completed: bool,
    pub has_idiomatic: bool,
    pub is_bugfix: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizSubmitResultDto {
    pub correct: bool,
    pub explanation: Option<String>,
    pub lesson_completed: bool,
    pub xp_awarded: u32,
    pub total_xp: u32,
    pub level: u32,
    pub ferris_mood: String,
    pub ferris_ascii: String,
}

pub fn stage_id_str(stage: V3Stage) -> &'static str {
    match stage {
        V3Stage::Basico => "basico",
        V3Stage::Meio => "meio",
        V3Stage::Dificil => "dificil",
        V3Stage::Fim => "fim",
        V3Stage::MiniPaint => "mini-paint",
        V3Stage::CaveGame => "cave-game",
    }
}

pub fn get_course_overview(
    state: &AppState,
    preview_unlocked: bool,
) -> CmdResult<CourseOverviewDto> {
    let inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    let en = inner.is_english();
    let mut total_completed = 0usize;
    let mut stages = Vec::with_capacity(V3Stage::ALL.len());

    for stage in V3Stage::ALL {
        let list = entries_for_stage(stage);
        let stage_unlocked = is_v3_stage_unlocked(&inner.progress, stage, preview_unlocked);
        let mut completed_in_stage = 0usize;
        let mut entries_dto = Vec::with_capacity(list.len());

        for e in &list {
            let completed = is_v3_entry_completed(&inner.progress, e);
            if completed {
                completed_in_stage += 1;
                total_completed += 1;
            }
            let unlocked = is_v3_entry_unlocked(&inner.progress, e, preview_unlocked);
            let read = inner.progress.read.contains(e.id);
            entries_dto.push(CourseEntrySummaryDto {
                id: e.id.to_string(),
                title: e.title_for(en).to_string(),
                stage_id: stage_id_str(stage).to_string(),
                kind: match e.kind {
                    V3EntryKind::Theory => "theory",
                    V3EntryKind::Practice => "practice",
                    V3EntryKind::CapstoneMilestone => "capstone",
                }
                .to_string(),
                cycle: e.cycle.map(str::to_string),
                prerequisites: e.prerequisites.iter().map(|p| (*p).to_string()).collect(),
                learning_goal: e.learning_goal_for(en).to_string(),
                completion_evidence: e.completion_evidence_for(en).to_string(),
                unlocked,
                completed,
                read,
                is_interactive_stdin: e.id == "D09-P01",
            });
        }

        stages.push(CourseStageDto {
            id: stage_id_str(stage).to_string(),
            label: stage.label_for(en).to_string(),
            subtitle: stage.subtitle_for(en).to_string(),
            unlocked: stage_unlocked,
            completed_count: completed_in_stage,
            total_count: list.len(),
            entries: entries_dto,
        });
    }

    let bp = v3_curriculum_blueprint();
    Ok(CourseOverviewDto {
        total_entries: bp.len(),
        completed_entries: total_completed,
        xp: inner.xp(),
        level: inner.level(),
        last_entry_id: inner.progress.last_entry_id.clone(),
        due_srs_items: inner.progress.due_srs_items(),
        stages,
    })
}

pub fn get_lesson_detail(state: &AppState, entry_id: &str) -> CmdResult<LessonDetailDto> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    let en = inner.is_english();
    let entry = find_v3_entry(entry_id)
        .ok_or_else(|| CommandError::not_found(format!("Lição `{entry_id}` não encontrada")))?;

    inner.progress.read.insert(entry.id.to_string());
    inner.progress.last_entry_id = Some(entry.id.to_string());
    let _ = inner.save_progress();

    let lesson = build_v3_lesson_for(entry, &inner.curriculum, en);
    let mut quiz_idx = 0usize;
    let blocks = lesson
        .blocks
        .iter()
        .map(|b| match b {
            Block::Text { text } => LessonBlockDto::Text { text: text.clone() },
            Block::Code {
                code,
                file,
                explain,
            } => LessonBlockDto::Code {
                code: code.clone(),
                file: file.clone(),
                explain: explain.clone(),
            },
            Block::Callout { kind, text } => LessonBlockDto::Callout {
                kind: match kind {
                    CalloutKind::Note => "note",
                    CalloutKind::Tip => "tip",
                    CalloutKind::Warning => "warning",
                }
                .to_string(),
                text: text.clone(),
            },
            Block::Demo {
                demo,
                name,
                steps,
                caption,
            } => LessonBlockDto::Demo {
                demo: match demo {
                    DemoKind::Variables => "variables",
                    DemoKind::Loop => "loop",
                    DemoKind::Function => "function",
                    DemoKind::StackHeap => "stack-heap",
                    DemoKind::Ownership => "ownership",
                    DemoKind::Borrowing => "borrowing",
                    DemoKind::Vector => "vector",
                }
                .to_string(),
                name: name.clone(),
                steps: steps.clone(),
                caption: caption.clone(),
            },
            Block::Quiz {
                question,
                options,
                correct,
                why,
            } => {
                let qi = quiz_idx;
                quiz_idx += 1;
                LessonBlockDto::Quiz {
                    quiz_index: qi,
                    question: question.clone(),
                    options: options.clone(),
                    correct: *correct,
                    why: why.clone(),
                }
            }
        })
        .collect();

    let spec = build_v3_practice_spec_for(entry, en)
        .unwrap_or_else(|| build_v3_practice_for_any_entry(entry, en));
    let draft_key = format!("practice:{}", spec.lesson_id);
    let current_code = inner
        .read_draft(&draft_key)
        .or_else(|| inner.progress.exercise_answers.get(&draft_key).cloned())
        .unwrap_or_else(|| spec.starter.to_string());
    let shown_hints = inner
        .progress
        .hints_used
        .get(&format!("practice:{}", spec.id))
        .copied()
        .unwrap_or(0) as usize;
    let practice_completed = inner.progress.exercises_completed.contains(spec.id);
    let (mode, stdin_preview) = match spec.mode {
        PracticeMode::CargoRun => ("cargo-run".to_string(), Vec::new()),
        PracticeMode::CargoRunWithInput { stdin_lines } => (
            "cargo-run-stdin".to_string(),
            stdin_lines.iter().map(|s| (*s).to_string()).collect(),
        ),
        PracticeMode::CargoTest { .. } => ("cargo-test".to_string(), Vec::new()),
    };

    let has_idiomatic = rustlab::exercises::idiomatic_solution_for(spec.id, en).is_some();
    let practice = Some(PracticeSpecDto {
        id: spec.id.to_string(),
        lesson_id: spec.lesson_id.to_string(),
        title: spec.title.to_string(),
        prompt: spec.prompt.to_string(),
        starter: spec.starter.to_string(),
        current_code,
        hints: spec.hints.iter().map(|h| (*h).to_string()).collect(),
        shown_hints: shown_hints.min(4),
        mode,
        stdin_preview,
        completed: practice_completed,
        has_idiomatic,
    });

    let bp = v3_curriculum_blueprint();
    let pos = bp.iter().position(|e| e.id == entry.id);
    let prev_entry_id = pos
        .and_then(|i| i.checked_sub(1))
        .map(|i| bp[i].id.to_string());
    let next_entry_id = pos.and_then(|i| bp.get(i + 1)).map(|e| e.id.to_string());

    let completed = is_v3_entry_completed(&inner.progress, entry);
    let ferris = if completed {
        rustlab::learn::FerrisMood::Happy
    } else {
        rustlab::learn::FerrisMood::Neutral
    };

    Ok(LessonDetailDto {
        id: entry.id.to_string(),
        title: entry.title_for(en).to_string(),
        stage_id: stage_id_str(entry.stage).to_string(),
        stage_label: entry.stage.label_for(en).to_string(),
        kind: match entry.kind {
            V3EntryKind::Theory => "theory",
            V3EntryKind::Practice => "practice",
            V3EntryKind::CapstoneMilestone => "capstone",
        }
        .to_string(),
        completed,
        requires_quiz: lesson.requires_quiz(),
        ferris_mood: format!("{ferris:?}").to_lowercase(),
        ferris_ascii: ferris.ascii().to_string(),
        blocks,
        practice,
        prev_entry_id,
        next_entry_id,
    })
}

pub fn get_exercises_catalog(
    state: &AppState,
    bugfix_mode: bool,
) -> CmdResult<Vec<ClassicExerciseDto>> {
    let inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    let en = inner.is_english();
    let list: Vec<_> = if bugfix_mode {
        rustlab::exercises::bugfix_challenges(en)
    } else {
        inner.curriculum.exercises().cloned().collect()
    };

    Ok(list
        .into_iter()
        .map(|ex| {
            let draft_key = format!("exercise:{}", ex.id);
            let current_code = inner
                .read_draft(&draft_key)
                .or_else(|| inner.progress.exercise_answers.get(&ex.id).cloned())
                .unwrap_or_else(|| ex.skeleton.clone());
            let shown_hints = inner.progress.hints_used.get(&ex.id).copied().unwrap_or(0) as usize;
            let completed = inner.progress.exercises_completed.contains(&ex.id);
            let has_idiomatic = rustlab::exercises::idiomatic_solution_for(&ex.id, en).is_some();
            ClassicExerciseDto {
                id: ex.id,
                title: ex.title,
                statement: ex.statement,
                starter: ex.skeleton,
                current_code,
                hints: ex.hints,
                shown_hints: shown_hints.min(4),
                completed,
                has_idiomatic,
                is_bugfix: bugfix_mode,
            }
        })
        .collect())
}

pub fn submit_quiz_answer(
    state: &AppState,
    entry_id: &str,
    quiz_index: usize,
    option_index: usize,
) -> CmdResult<QuizSubmitResultDto> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    let en = inner.is_english();
    let entry = find_v3_entry(entry_id)
        .ok_or_else(|| CommandError::not_found(format!("Lição `{entry_id}` não encontrada")))?;
    let lesson = build_v3_lesson_for(entry, &inner.curriculum, en);
    let quizzes: Vec<(usize, Option<String>)> = lesson
        .blocks
        .iter()
        .filter_map(|b| match b {
            Block::Quiz { correct, why, .. } => Some((*correct, why.clone())),
            _ => None,
        })
        .collect();
    let (correct_idx, why) = quizzes
        .get(quiz_index)
        .ok_or_else(|| CommandError::invalid("Índice de quiz inválido"))?;

    let is_correct = option_index == *correct_idx;
    let mut xp_awarded = 0;
    if is_correct {
        if inner.progress.completed.insert(entry.id.to_string()) {
            xp_awarded = inner.award_xp(format!("lesson:{}", entry.id), 20);
        }
        inner.progress.record_srs_mastery(entry.id, true);
        let _ = inner.save_progress();
    }

    let mood = if is_correct {
        rustlab::learn::FerrisMood::Happy
    } else {
        rustlab::learn::FerrisMood::Encouraging
    };

    Ok(QuizSubmitResultDto {
        correct: is_correct,
        explanation: if is_correct { why.clone() } else { None },
        lesson_completed: is_v3_entry_completed(&inner.progress, entry),
        xp_awarded,
        total_xp: inner.xp(),
        level: inner.level(),
        ferris_mood: format!("{mood:?}").to_lowercase(),
        ferris_ascii: mood.ascii().to_string(),
    })
}

pub fn complete_reading_lesson(state: &AppState, entry_id: &str) -> CmdResult<QuizSubmitResultDto> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    let entry = find_v3_entry(entry_id)
        .ok_or_else(|| CommandError::not_found(format!("Lição `{entry_id}` não encontrada")))?;
    inner.progress.read.insert(entry.id.to_string());
    let mut xp_awarded = 0;
    if inner.progress.completed.insert(entry.id.to_string()) {
        xp_awarded = inner.award_xp(format!("lesson:{}", entry.id), 20);
    }
    inner.progress.record_srs_mastery(entry.id, true);
    let _ = inner.save_progress();

    let mood = rustlab::learn::FerrisMood::Happy;
    Ok(QuizSubmitResultDto {
        correct: true,
        explanation: None,
        lesson_completed: true,
        xp_awarded,
        total_xp: inner.xp(),
        level: inner.level(),
        ferris_mood: "happy".to_string(),
        ferris_ascii: mood.ascii().to_string(),
    })
}
