use std::path::PathBuf;

use rustlab::execution::{
    DiagLevel, ExecEvent, ExecutionManager, StreamKind, ToolCommand, parse_diagnostics,
};
use rustlab::learn::{
    PracticeMode, PracticeSpec, explain_compiler_error_pt_br, is_practice_compile_error,
    prepare_practice_project, validate_practice_run,
};
use serde::Serialize;

use crate::course::Course;

#[derive(Default)]
pub struct Tasks {
    executor: ExecutionManager,
    active: Option<Active>,
}

struct Active {
    practice: Option<PracticeRun>,
    stderr: Vec<String>,
    _scratch: Option<tempfile::TempDir>,
}

struct PracticeRun {
    spec: PracticeSpec,
    source: String,
    stdout: Vec<String>,
}

#[derive(Serialize)]
pub struct Batch {
    pub lines: Vec<Line>,
    pub result: Option<Outcome>,
}

#[derive(Serialize)]
pub struct Line {
    pub stream: &'static str,
    pub text: String,
}

#[derive(Serialize)]
pub struct Marker {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub severity: &'static str,
    pub message: String,
}

#[derive(Serialize)]
pub struct Outcome {
    pub state: &'static str,
    pub code: Option<i32>,
    pub message: String,
    pub awarded: u32,
    pub markers: Vec<Marker>,
}

impl Tasks {
    pub fn busy(&self) -> bool {
        self.active.is_some()
    }

    pub fn start(
        &mut self,
        command: ToolCommand,
        root: PathBuf,
        online: bool,
    ) -> Result<(), String> {
        self.executor.submit_with_network(command, root, online)?;
        self.active = Some(Active {
            _scratch: None,
            practice: None,
            stderr: Vec::new(),
        });
        Ok(())
    }

    pub fn start_practice(
        &mut self,
        course: &mut Course,
        id: &str,
        source: String,
    ) -> Result<(), String> {
        if self.busy() {
            return Err("Aguarde ou cancele a tarefa atual.".into());
        }
        let spec = course.practice_spec(id)?;
        course.save_draft(id, source.clone())?;
        let scratch = tempfile::tempdir().map_err(|error| error.to_string())?;
        prepare_practice_project(scratch.path(), &spec, &source)
            .map_err(|error| error.to_string())?;
        let (command, stdin) = match spec.mode {
            PracticeMode::CargoRun => (ToolCommand::Run, &[][..]),
            PracticeMode::CargoRunWithInput { stdin_lines } => {
                (ToolCommand::RunInteractive, stdin_lines)
            }
            PracticeMode::CargoTest { .. } => (ToolCommand::Test, &[][..]),
        };
        self.start(command, scratch.path().into(), false)?;
        self.active = Some(Active {
            _scratch: Some(scratch),
            stderr: Vec::new(),
            practice: Some(PracticeRun {
                spec,
                source,
                stdout: Vec::new(),
            }),
        });
        if command_needs_input(&self.active) {
            let result = stdin
                .iter()
                .try_for_each(|line| self.executor.send_line(line))
                .and_then(|()| self.executor.close_input());
            if result.is_err() {
                self.executor.cancel();
            }
        }
        Ok(())
    }

    pub fn test_toolchain(&mut self) -> Result<(), String> {
        let scratch = tempfile::tempdir().map_err(|error| error.to_string())?;
        let project = crate::workspace::Workspace::create(
            scratch.path(),
            "rustlab_hello",
            crate::templates::Template::Basic,
        )?;
        #[cfg(windows)]
        {
            let config = project.root().join(".cargo");
            std::fs::create_dir(&config).map_err(|error| error.to_string())?;
            std::fs::write(
                config.join("config.toml"),
                "[build]\ntarget = \"x86_64-pc-windows-msvc\"\n",
            )
            .map_err(|error| error.to_string())?;
        }
        self.start(ToolCommand::Run, project.root().into(), false)?;
        self.active.as_mut().expect("task started")._scratch = Some(scratch);
        Ok(())
    }

    pub fn cancel(&mut self) {
        self.executor.cancel();
    }

    pub fn poll(&mut self, course: &mut Course) -> Batch {
        let mut events = Vec::new();
        self.executor.poll_events(&mut events);
        let mut batch = Batch {
            lines: Vec::new(),
            result: None,
        };
        for event in events {
            match event {
                ExecEvent::Started { label } => batch.lines.push(Line {
                    stream: "system",
                    text: label,
                }),
                ExecEvent::Chunk { kind, text } => {
                    if let Some(active) = self.active.as_mut() {
                        match kind {
                            StreamKind::Stdout => {
                                if let Some(practice) = active.practice.as_mut() {
                                    practice.stdout.push(text.clone());
                                }
                            }
                            StreamKind::Stderr => active.stderr.push(text.clone()),
                            StreamKind::System => {}
                        }
                    }
                    batch.lines.push(Line {
                        stream: match kind {
                            StreamKind::Stdout => "stdout",
                            StreamKind::Stderr => "stderr",
                            StreamKind::System => "system",
                        },
                        text,
                    });
                }
                ExecEvent::Finished { code, success, .. } => {
                    let running = self.active.take();
                    let stderr = running
                        .as_ref()
                        .map(|a| a.stderr.join("\n"))
                        .unwrap_or_default();
                    batch.result = Some(finish(course, running, &stderr, code, success));
                }
                ExecEvent::TimedOut { .. } => {
                    self.active = None;
                    batch.result = Some(outcome("timeout", None, "Tempo limite atingido."));
                }
                ExecEvent::Cancelled { .. } => {
                    self.active = None;
                    batch.result = Some(outcome("cancelled", None, "Tarefa cancelada."));
                }
                ExecEvent::SpawnFailed { reason, .. } => {
                    self.active = None;
                    batch.result = Some(outcome("error", None, reason));
                }
            }
        }
        batch
    }
}

fn command_needs_input(active: &Option<Active>) -> bool {
    active
        .as_ref()
        .and_then(|a| a.practice.as_ref())
        .is_some_and(|run| matches!(run.spec.mode, PracticeMode::CargoRunWithInput { .. }))
}

fn finish(
    course: &mut Course,
    running: Option<Active>,
    stderr: &str,
    code: Option<i32>,
    success: bool,
) -> Outcome {
    let is_project = running
        .as_ref()
        .is_some_and(|active| active._scratch.is_none());
    let practice = running.and_then(|active| active.practice);
    let compile_failed = !success && stderr.lines().any(is_practice_compile_error);
    let Some(practice) = practice else {
        let mut result = match (success, compile_failed) {
            (true, _) => outcome("success", code, "Cargo concluiu a tarefa."),
            (false, true) => outcome(
                "compile-error",
                code,
                "A compilação falhou. Confira os diagnósticos.",
            ),
            _ => outcome(
                "failure",
                code,
                "Cargo retornou uma falha. Confira a saída.",
            ),
        };
        if is_project {
            result.markers = parse_diagnostics(stderr)
                .into_iter()
                .filter_map(|d| {
                    Some(Marker {
                        file: d.file?.replace('\\', "/"),
                        line: d.line?,
                        column: d.col?,
                        message: d.message,
                        severity: match d.level {
                            DiagLevel::Error => "Erro",
                            DiagLevel::Warning => "Aviso",
                            DiagLevel::Note => "Nota",
                        },
                    })
                })
                .take(100)
                .collect();
        }
        return result;
    };
    if !success {
        if !compile_failed {
            return outcome("failure", code, "A execução ou os testes falharam.");
        }
        let explanation = explain_compiler_error_pt_br(stderr).unwrap_or_default();
        return outcome(
            "compile-error",
            code,
            format!("O código não compilou. {explanation}"),
        );
    }
    match validate_practice_run(
        practice.spec.id,
        &practice.source,
        &practice.stdout.join("\n"),
    ) {
        Err(reason) => outcome("validation-error", code, reason),
        Ok(()) => match course.pass_practice(&practice.spec) {
            Ok(awarded) => Outcome {
                awarded,
                ..outcome("passed", code, "Prática aprovada.")
            },
            Err(error) => outcome(
                "error",
                code,
                format!("Prática aprovada. Falha ao salvar o progresso: {error}"),
            ),
        },
    }
}

fn outcome(state: &'static str, code: Option<i32>, message: impl Into<String>) -> Outcome {
    Outcome {
        state,
        code,
        message: message.into(),
        awarded: 0,
        markers: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use rustlab::storage::{self, ProgressData};

    fn wait(tasks: &mut Tasks, course: &mut Course) -> Outcome {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(25) {
            if let Some(result) = tasks.poll(course).result {
                return result;
            }
            std::thread::sleep(Duration::from_millis(15));
        }
        panic!("Cargo task did not finish")
    }

    #[test]
    fn project_cargo_reports_positions_and_can_cancel() {
        let dir = tempfile::tempdir().unwrap();
        let mut course = Course::load(dir.path().join("progress.toml")).unwrap();
        let project = crate::workspace::Workspace::create(
            dir.path(),
            "diagnostics",
            crate::templates::Template::Basic,
        )
        .unwrap();
        std::fs::write(
            project.root().join("src/main.rs"),
            "fn main() { missing(); }",
        )
        .unwrap();
        let mut tasks = Tasks::default();
        tasks
            .start(ToolCommand::Check, project.root().into(), false)
            .unwrap();
        let result = wait(&mut tasks, &mut course);
        assert_eq!(result.state, "compile-error");
        assert!(
            result
                .markers
                .iter()
                .any(|marker| marker.file.ends_with("src/main.rs") && marker.line == 1)
        );

        tasks.test_toolchain().unwrap();
        assert_eq!(wait(&mut tasks, &mut course).state, "success");
        tasks
            .start(ToolCommand::Check, project.root().into(), false)
            .unwrap();
        tasks.cancel();
        assert_eq!(wait(&mut tasks, &mut course).state, "cancelled");
    }

    #[test]
    fn cargo_practice_rejects_wrong_output_and_awards_once() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("progress.toml");
        let mut data = ProgressData::current();
        data.completed_lessons.insert("B-T05".into());
        storage::save_to(&data, &path).unwrap();
        let mut course = Course::load(path).unwrap();
        let mut tasks = Tasks::default();
        tasks
            .start_practice(&mut course, "B-P01", "fn main() { nope(); }".into())
            .unwrap();
        assert_eq!(wait(&mut tasks, &mut course).state, "compile-error");
        tasks
            .start_practice(
                &mut course,
                "B-P01",
                "fn main() { println!(\"...\"); }".into(),
            )
            .unwrap();
        assert_eq!(wait(&mut tasks, &mut course).state, "validation-error");
        let good = "fn main() { println!(\"Hello, world!\"); }";
        for expected in [200, 0] {
            tasks
                .start_practice(&mut course, "B-P01", good.into())
                .unwrap();
            let result = wait(&mut tasks, &mut course);
            assert_eq!(result.state, "passed", "{}", result.message);
            assert_eq!(result.awarded, expected);
        }
        assert_eq!(course.map().xp, 200);
        course.reveal_hint("B-P01").unwrap();
        tasks
            .start_practice(&mut course, "B-P01", good.into())
            .unwrap();
        assert_eq!(wait(&mut tasks, &mut course).awarded, 0);
        assert_eq!(course.map().xp, 200);
    }
}
