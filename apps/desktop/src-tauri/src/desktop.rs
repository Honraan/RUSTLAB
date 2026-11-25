use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use rustlab::doc_hint::extra_libraries;
use rustlab::execution::{ToolCommand, find_cargo};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::course::{Course, CourseMap, Lesson, QuizResult};
use crate::tasks::{Batch, Tasks};
use crate::workspace::{ProjectInfo, Workspace};

struct AppState {
    _instance: File,
    data: Mutex<Desktop>,
}

struct Desktop {
    course: Course,
    tasks: Tasks,
    project: Option<Workspace>,
}

impl AppState {
    fn lock(&self) -> Result<MutexGuard<'_, Desktop>, String> {
        self.data
            .lock()
            .map_err(|_| "Sessão indisponível. Reinicie o RustLab.".into())
    }
}

impl Desktop {
    fn idle(&self) -> Result<(), String> {
        if self.tasks.busy() {
            return Err("Aguarde ou cancele a tarefa atual.".into());
        }
        Ok(())
    }

    fn project(&self) -> Result<&Workspace, String> {
        self.project
            .as_ref()
            .ok_or_else(|| "Abra um projeto primeiro.".into())
    }
}

#[tauri::command]
async fn course_map(state: State<'_, AppState>) -> Result<CourseMap, String> {
    Ok(state.lock()?.course.map())
}

#[tauri::command]
async fn lesson(id: String, state: State<'_, AppState>) -> Result<Lesson, String> {
    state.lock()?.course.lesson(&id)
}

#[tauri::command]
async fn answer(
    id: String,
    picks: Vec<usize>,
    state: State<'_, AppState>,
) -> Result<QuizResult, String> {
    state.lock()?.course.answer(&id, &picks)
}

#[tauri::command]
async fn save_draft(id: String, source: String, state: State<'_, AppState>) -> Result<(), String> {
    state.lock()?.course.save_draft(&id, source)
}

#[tauri::command]
async fn reveal_hint(id: String, state: State<'_, AppState>) -> Result<Vec<&'static str>, String> {
    state.lock()?.course.reveal_hint(&id)
}

#[tauri::command]
async fn start_practice(
    id: String,
    source: String,
    consent: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    require_consent(consent)?;
    let mut desktop = state.lock()?;
    let Desktop { course, tasks, .. } = &mut *desktop;
    tasks.start_practice(course, &id, source)
}

#[tauri::command]
async fn import_progress(
    app: AppHandle,
    replace: bool,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    require_consent(replace)?;
    state.lock()?.idle()?;
    let picked = app
        .dialog()
        .file()
        .add_filter("Progresso RustLab", &["toml"])
        .blocking_pick_file();
    let Some(path) = local_path(picked)? else {
        return Ok(None);
    };
    let mut desktop = state.lock()?;
    desktop.idle()?;
    let backup = desktop.course.import(&path)?;
    Ok(Some(backup.map_or_else(
        || "Progresso importado.".into(),
        |path| format!("Progresso importado. Backup salvo em {path}"),
    )))
}

#[tauri::command]
async fn export_progress(app: AppHandle, state: State<'_, AppState>) -> Result<bool, String> {
    let picked = app
        .dialog()
        .file()
        .add_filter("Progresso RustLab", &["toml"])
        .set_file_name("rustlab-progresso.toml")
        .blocking_save_file();
    let Some(path) = local_path(picked)? else {
        return Ok(false);
    };
    state.lock()?.course.export(&path)?;
    Ok(true)
}

#[tauri::command]
async fn open_project(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<ProjectInfo>, String> {
    state.lock()?.idle()?;
    let Some(path) = local_path(app.dialog().file().blocking_pick_folder())? else {
        return Ok(None);
    };
    let mut desktop = state.lock()?;
    desktop.idle()?;
    let project = Workspace::open(&path)?;
    let info = project.info();
    desktop.project = Some(project);
    Ok(Some(info))
}

#[tauri::command]
async fn create_project(
    app: AppHandle,
    name: String,
    state: State<'_, AppState>,
) -> Result<Option<ProjectInfo>, String> {
    state.lock()?.idle()?;
    let Some(path) = local_path(
        app.dialog()
            .file()
            .set_title("Escolha a pasta que receberá o projeto")
            .blocking_pick_folder(),
    )?
    else {
        return Ok(None);
    };
    let mut desktop = state.lock()?;
    desktop.idle()?;
    let project = Workspace::create(&path, &name)?;
    let info = project.info();
    desktop.project = Some(project);
    Ok(Some(info))
}

#[tauri::command]
async fn read_source(relative: String, state: State<'_, AppState>) -> Result<String, String> {
    state.lock()?.project()?.read(&relative)
}

#[tauri::command]
async fn save_source(
    relative: String,
    expected: String,
    source: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let desktop = state.lock()?;
    desktop.idle()?;
    desktop.project()?.save(&relative, &expected, &source)
}

#[tauri::command]
async fn create_source(
    relative: String,
    state: State<'_, AppState>,
) -> Result<ProjectInfo, String> {
    let desktop = state.lock()?;
    desktop.idle()?;
    desktop.project()?.create_source(&relative)
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Action {
    Check,
    Run,
    Test,
    Fmt,
    Clippy,
}

#[tauri::command]
async fn run_project(
    action: Action,
    consent: bool,
    allow_downloads: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    require_consent(consent)?;
    let mut desktop = state.lock()?;
    let root = desktop.project()?.root().to_owned();
    let tool = match action {
        Action::Check => ToolCommand::Check,
        Action::Run => ToolCommand::Run,
        Action::Test => ToolCommand::Test,
        Action::Fmt => ToolCommand::Fmt,
        Action::Clippy => ToolCommand::Clippy,
    };
    desktop.tasks.start(tool, root, allow_downloads)
}

#[tauri::command]
async fn poll_task(state: State<'_, AppState>) -> Result<Batch, String> {
    let mut desktop = state.lock()?;
    let Desktop { course, tasks, .. } = &mut *desktop;
    Ok(tasks.poll(course))
}

#[tauri::command]
async fn cancel_task(state: State<'_, AppState>) -> Result<(), String> {
    let mut desktop = state.lock()?;
    if desktop.tasks.busy() {
        desktop.tasks.cancel();
    }
    Ok(())
}

#[derive(Serialize)]
struct DoctorReport {
    cargo: Option<String>,
    rustc: Option<String>,
    rustup: Option<String>,
    rustfmt: Option<String>,
    clippy: Option<String>,
    analyzer: Option<String>,
}

#[tauri::command]
async fn doctor() -> DoctorReport {
    let cargo = find_cargo();
    let bin = cargo.as_ref().and_then(|p| p.parent());
    let find = |name: &str| {
        let executable = format!("{name}.exe");
        bin.into_iter()
            .map(PathBuf::from)
            .chain(
                std::env::var_os("PATH")
                    .into_iter()
                    .flat_map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
                    .filter(|path| path.is_absolute()),
            )
            .map(|p| p.join(&executable))
            .find(|p| p.is_file())
            .map(|p| p.display().to_string())
    };
    DoctorReport {
        rustc: find("rustc"),
        rustup: find("rustup"),
        rustfmt: find("rustfmt"),
        clippy: find("cargo-clippy"),
        analyzer: find("rust-analyzer"),
        cargo: cargo.map(|p| p.display().to_string()),
    }
}

#[tauri::command]
async fn test_toolchain(consent: bool, state: State<'_, AppState>) -> Result<(), String> {
    require_consent(consent)?;
    state.lock()?.tasks.test_toolchain()
}

#[derive(Serialize)]
struct Library {
    name: &'static str,
    version: &'static str,
    category: &'static str,
    summary: &'static str,
    warnings: &'static str,
    lessons: &'static [&'static str],
    docs: &'static str,
}

#[tauri::command]
fn libraries() -> Vec<Library> {
    extra_libraries(false)
        .iter()
        .map(|lib| {
            let meta = lib.v3_metadata();
            Library {
                name: lib.name,
                version: lib.version,
                category: lib.category,
                summary: lib.summary,
                warnings: meta.common_pitfalls,
                lessons: meta.associated_v3_lessons,
                docs: lib.doc_url,
            }
        })
        .collect()
}

fn local_path(picked: Option<FilePath>) -> Result<Option<PathBuf>, String> {
    picked
        .map(|path| path.into_path().map_err(|error| error.to_string()))
        .transpose()
}

fn require_consent(consent: bool) -> Result<(), String> {
    if !consent {
        return Err("Confirme a operação para continuar.".into());
    }
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_local_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let instance = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(data_dir.join("rustlab.lock"))?;
            instance.try_lock()?;
            let course =
                Course::load(data_dir.join("progresso.toml")).map_err(std::io::Error::other)?;
            app.manage(AppState {
                _instance: instance,
                data: Mutex::new(Desktop {
                    course,
                    tasks: Tasks::default(),
                    project: None,
                }),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            course_map,
            lesson,
            answer,
            save_draft,
            reveal_hint,
            start_practice,
            import_progress,
            export_progress,
            open_project,
            create_project,
            read_source,
            save_source,
            create_source,
            run_project,
            poll_task,
            cancel_task,
            doctor,
            test_toolchain,
            libraries
        ])
        .run(tauri::generate_context!())
        .expect("Não foi possível iniciar o RustLab Desktop");
}
