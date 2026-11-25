use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use rustlab::config::ExecConsent;
use rustlab::curriculum::v3_blueprint::{
    build_v3_practice_for_any_entry, build_v3_practice_spec_for, find_v3_entry,
};
use rustlab::execution::{DiagLevel, ExecEvent, StreamKind, ToolCommand, parse_diagnostics};
use rustlab::exercises::{
    bugfix_challenges, fresh_temp_dir, idiomatic_solution_for, is_compile_error,
    pedagogical_clippy_for, prepare_fixture,
};
use rustlab::learn::{
    FerrisMood, PracticeMode, calculate_v3_practice_xp, explain_compiler_error_for,
    explain_compiler_error_socratic_for, fresh_practice_temp_dir, is_practice_compile_error,
    prepare_practice_project, validate_practice_run_for,
};

use crate::error::{CmdResult, CommandError};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticDto {
    pub level: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub col: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeRunResultDto {
    pub outcome: String,
    pub output_lines: Vec<String>,
    pub diagnostics: Vec<DiagnosticDto>,
    pub ferris_explanation: Option<String>,
    pub socratic_eli10: Option<String>,
    pub clippy_tips: Vec<String>,
    pub idiomatic_solution: Option<(String, String)>,
    pub xp_awarded: u32,
    pub independence_bonus: u32,
    pub total_xp: u32,
    pub level: u32,
    pub ferris_mood: String,
    pub ferris_ascii: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeStreamEvent {
    pub kind: String,
    pub text: String,
}

pub fn save_practice_draft(state: &AppState, scope: &str, id: &str, code: &str) -> CmdResult<()> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    let key = if scope == "practice" {
        format!("practice:{id}")
    } else {
        id.to_string()
    };
    inner.write_draft(&format!("{scope}:{id}"), code);
    inner
        .progress
        .exercise_answers
        .insert(key, code.to_string());
    inner
        .save_progress()
        .map_err(|e| CommandError::io("Falha ao salvar rascunho no disco", e))?;
    Ok(())
}

pub fn reveal_practice_hint(state: &AppState, scope: &str, id: &str) -> CmdResult<usize> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    let key = if scope == "practice" {
        format!("practice:{id}")
    } else {
        id.to_string()
    };
    let current = inner.progress.hints_used.get(&key).copied().unwrap_or(0) as usize;
    let next = (current + 1).min(4);
    inner.progress.hints_used.insert(key, next as u8);
    let _ = inner.save_progress();
    Ok(next)
}

pub fn get_idiomatic_solution(
    state: &AppState,
    _scope: &str,
    id: &str,
) -> CmdResult<Option<(String, String)>> {
    let inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    let en = inner.is_english();
    if !inner.progress.exercises_completed.contains(id) {
        return Err(CommandError::forbidden(
            "Complete e passe nos testes reais primeiro para desbloquear a comparação idiomática.",
        ));
    }
    Ok(idiomatic_solution_for(id, en).map(|(c, w)| (c.to_string(), w.to_string())))
}

pub fn cancel_active_execution(state: &AppState) -> CmdResult<()> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
    inner.exec.cancel();
    inner.append_log("exec", "Execução cancelada pelo usuário");
    Ok(())
}

enum PreparedTarget {
    Practice {
        spec_id: &'static str,
        lesson_id: &'static str,
        mode: PracticeMode,
        dir: PathBuf,
    },
    Exercise {
        ex_id: String,
        dir: PathBuf,
    },
}

impl PreparedTarget {
    fn dir(&self) -> &PathBuf {
        match self {
            PreparedTarget::Practice { dir, .. } | PreparedTarget::Exercise { dir, .. } => dir,
        }
    }
}

pub fn run_practice_check_core(
    app: Option<&AppHandle>,
    state: &AppState,
    scope: &str,
    id: &str,
    code: &str,
    ai_used: bool,
) -> CmdResult<PracticeRunResultDto> {
    let (en, shown_hints, prepared) = {
        let mut inner = state
            .inner
            .lock()
            .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
        if inner.config.execution.consent != ExecConsent::Granted {
            return Err(CommandError::consent(
                "Execução local requer consentimento explícito (executar Cargo não é sandbox).",
            ));
        }
        if inner.exec.running {
            return Err(CommandError::busy(
                "Já existe um comando Cargo em execução. Aguarde ou cancele antes.",
            ));
        }
        let en = inner.is_english();
        let draft_key = if scope == "practice" {
            format!("practice:{id}")
        } else {
            id.to_string()
        };
        inner.write_draft(&format!("{scope}:{id}"), code);
        inner
            .progress
            .exercise_answers
            .insert(draft_key.clone(), code.to_string());
        let _ = inner.save_progress();

        let shown_hints = inner
            .progress
            .hints_used
            .get(&draft_key)
            .copied()
            .unwrap_or(0) as usize;

        if scope == "practice" {
            let entry = find_v3_entry(id)
                .ok_or_else(|| CommandError::not_found(format!("Prática `{id}` não encontrada")))?;
            let spec = build_v3_practice_spec_for(entry, en)
                .unwrap_or_else(|| build_v3_practice_for_any_entry(entry, en));
            let dir = fresh_practice_temp_dir(spec.id)
                .map_err(|e| CommandError::io("Falha ao criar pasta temporária", e))?;
            if let Err(e) = prepare_practice_project(&dir, &spec, code) {
                let _ = std::fs::remove_dir_all(&dir);
                return Err(CommandError::io("Falha ao preparar projeto Cargo", e));
            }
            let cmd = match spec.mode {
                PracticeMode::CargoRun => ToolCommand::Run,
                PracticeMode::CargoRunWithInput { .. } => ToolCommand::RunInteractive,
                PracticeMode::CargoTest { .. } => ToolCommand::Test,
            };
            if let Err(msg) = inner.exec.submit(cmd, dir.clone()) {
                let _ = std::fs::remove_dir_all(&dir);
                return Err(CommandError::new("INTERNAL", msg, None));
            }
            if let PracticeMode::CargoRunWithInput { stdin_lines } = spec.mode {
                for line in stdin_lines {
                    let _ = inner.exec.send_line(line);
                }
                let _ = inner.exec.close_input();
            }
            (
                en,
                shown_hints,
                PreparedTarget::Practice {
                    spec_id: spec.id,
                    lesson_id: spec.lesson_id,
                    mode: spec.mode,
                    dir,
                },
            )
        } else {
            let ex = inner
                .curriculum
                .exercise(id)
                .cloned()
                .or_else(|| bugfix_challenges(en).into_iter().find(|e| e.id == id))
                .ok_or_else(|| {
                    CommandError::not_found(format!("Exercício `{id}` não encontrado"))
                })?;
            let dir = fresh_temp_dir(&ex.id)
                .map_err(|e| CommandError::io("Falha ao criar pasta temporária", e))?;
            if let Err(e) = prepare_fixture(&dir, &ex, code) {
                let _ = std::fs::remove_dir_all(&dir);
                return Err(CommandError::io("Falha ao preparar suíte de testes", e));
            }
            if let Err(msg) = inner.exec.submit(ToolCommand::ExerciseTest, dir.clone()) {
                let _ = std::fs::remove_dir_all(&dir);
                return Err(CommandError::new("INTERNAL", msg, None));
            }
            (
                en,
                shown_hints,
                PreparedTarget::Exercise { ex_id: ex.id, dir },
            )
        }
    };

    let mut output_lines: Vec<String> = Vec::new();
    let mut stdout_lines: Vec<String> = Vec::new();
    let mut stderr_lines: Vec<String> = Vec::new();
    let mut compile_error_seen = false;
    let mut final_event: Option<ExecEvent> = None;

    let deadline = Instant::now() + Duration::from_secs(180);
    while Instant::now() < deadline {
        let mut batch = Vec::new();
        let still_running = {
            let mut inner = state
                .inner
                .lock()
                .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
            inner.exec.poll_events(&mut batch);
            inner.exec.running
        };

        for ev in batch {
            match &ev {
                ExecEvent::Chunk { kind, text } => {
                    let kind_str = match kind {
                        StreamKind::Stdout => {
                            stdout_lines.push(text.clone());
                            "stdout"
                        }
                        StreamKind::Stderr => {
                            stderr_lines.push(text.clone());
                            let is_err = match &prepared {
                                PreparedTarget::Practice { .. } => is_practice_compile_error(text),
                                PreparedTarget::Exercise { .. } => is_compile_error(text),
                            };
                            if is_err {
                                compile_error_seen = true;
                            }
                            "stderr"
                        }
                        StreamKind::System => "system",
                    };
                    output_lines.push(text.clone());
                    if let Some(handle) = app {
                        let _ = handle.emit(
                            "practice://event",
                            PracticeStreamEvent {
                                kind: kind_str.to_string(),
                                text: text.clone(),
                            },
                        );
                    }
                }
                ExecEvent::Finished { .. }
                | ExecEvent::TimedOut { .. }
                | ExecEvent::Cancelled { .. }
                | ExecEvent::SpawnFailed { .. } => {
                    final_event = Some(ev);
                }
                ExecEvent::Started { .. } => {}
            }
        }
        if final_event.is_some() && !still_running {
            break;
        }
        std::thread::sleep(Duration::from_millis(15));
    }

    let _ = std::fs::remove_dir_all(prepared.dir());

    let joined_err = stderr_lines.join("\n");
    let diagnostics: Vec<DiagnosticDto> = parse_diagnostics(&joined_err)
        .into_iter()
        .map(|d| DiagnosticDto {
            level: match d.level {
                DiagLevel::Error => "error",
                DiagLevel::Warning => "warning",
                DiagLevel::Note => "note",
            }
            .to_string(),
            message: d.message,
            file: d.file,
            line: d.line,
            col: d.col,
        })
        .collect();

    let outcome;
    let ferris_mood;
    let mut ferris_explanation = None;
    let mut socratic_eli10 = None;
    let mut clippy_tips = Vec::new();
    let mut idiomatic_solution = None;
    let mut xp_awarded = 0u32;
    let mut independence_bonus = 0u32;

    match final_event {
        Some(ExecEvent::Finished { success, .. }) => {
            if !success {
                if compile_error_seen {
                    outcome = "compile-failed".to_string();
                    ferris_mood = FerrisMood::Encouraging;
                    let full = output_lines.join("\n");
                    ferris_explanation = explain_compiler_error_for(&full, en);
                    socratic_eli10 = explain_compiler_error_socratic_for(&full, true, en);
                    if let Ok(mut inner) = state.inner.lock() {
                        inner.progress.record_compiler_errors(&full);
                        let _ = inner.save_progress();
                    }
                } else {
                    outcome = match &prepared {
                        PreparedTarget::Practice {
                            mode: PracticeMode::CargoRun | PracticeMode::CargoRunWithInput { .. },
                            ..
                        } => "run-failed".to_string(),
                        _ => "tests-failed".to_string(),
                    };
                    ferris_mood = FerrisMood::Encouraging;
                }
            } else {
                let validation_err = match &prepared {
                    PreparedTarget::Practice { spec_id, .. } => {
                        let stdout_joined = stdout_lines.join("\n");
                        validate_practice_run_for(spec_id, code, &stdout_joined, en).err()
                    }
                    PreparedTarget::Exercise { .. } => None,
                };
                if let Some(msg) = validation_err {
                    outcome = "validation-failed".to_string();
                    output_lines.push(format!("Validação RustLab: {msg}"));
                    ferris_mood = FerrisMood::Encouraging;
                } else {
                    outcome = "passed".to_string();
                    ferris_mood = FerrisMood::Happy;
                    clippy_tips = pedagogical_clippy_for(code, en);

                    let mut inner = state.inner.lock().map_err(|_| {
                        CommandError::new("INTERNAL", "Falha de lock interno", None)
                    })?;
                    match &prepared {
                        PreparedTarget::Practice {
                            spec_id, lesson_id, ..
                        } => {
                            inner.progress.read.insert((*lesson_id).to_string());
                            inner
                                .progress
                                .exercises_completed
                                .insert((*spec_id).to_string());
                            let (base, bonus) = calculate_v3_practice_xp(shown_hints, ai_used);
                            xp_awarded = inner.award_xp(format!("practice:{spec_id}"), base);
                            if bonus > 0 {
                                independence_bonus = inner
                                    .award_xp(format!("practice:{spec_id}:independent"), bonus);
                            }
                            inner.progress.record_srs_mastery(spec_id, bonus > 0);
                            idiomatic_solution = idiomatic_solution_for(spec_id, en)
                                .map(|(c, w)| (c.to_string(), w.to_string()));
                        }
                        PreparedTarget::Exercise { ex_id, .. } => {
                            let newly = inner.progress.exercises_completed.insert(ex_id.clone());
                            if newly {
                                let (base, bonus) = calculate_v3_practice_xp(shown_hints, ai_used);
                                xp_awarded = inner.award_xp(format!("exercise:{ex_id}"), base);
                                if bonus > 0 {
                                    independence_bonus = inner
                                        .award_xp(format!("exercise:{ex_id}:independent"), bonus);
                                }
                            }
                            inner
                                .progress
                                .record_srs_mastery(ex_id, shown_hints == 0 && !ai_used);
                            idiomatic_solution = idiomatic_solution_for(ex_id, en)
                                .map(|(c, w)| (c.to_string(), w.to_string()));
                        }
                    }
                    let _ = inner.save_progress();
                }
            }
        }
        Some(ExecEvent::TimedOut { .. }) => {
            outcome = "timed-out".to_string();
            ferris_mood = FerrisMood::Encouraging;
            output_lines.push("Tempo limite atingido ao executar Cargo.".into());
        }
        Some(ExecEvent::Cancelled { .. }) => {
            outcome = "cancelled".to_string();
            ferris_mood = FerrisMood::Neutral;
            output_lines.push("Execução cancelada pelo aluno.".into());
        }
        Some(ExecEvent::SpawnFailed { reason, .. }) => {
            outcome = "start-failed".to_string();
            ferris_mood = FerrisMood::Confused;
            output_lines.push(reason);
        }
        _ => {
            outcome = "timed-out".to_string();
            ferris_mood = FerrisMood::Encouraging;
        }
    }

    let (total_xp, level) = {
        let inner = state
            .inner
            .lock()
            .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
        (inner.xp(), inner.level())
    };

    Ok(PracticeRunResultDto {
        outcome,
        output_lines,
        diagnostics,
        ferris_explanation,
        socratic_eli10,
        clippy_tips,
        idiomatic_solution,
        xp_awarded,
        independence_bonus,
        total_xp,
        level,
        ferris_mood: format!("{ferris_mood:?}").to_lowercase(),
        ferris_ascii: ferris_mood.ascii().to_string(),
    })
}
