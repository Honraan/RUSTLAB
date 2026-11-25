pub mod curriculum_cmd;
pub mod error;
pub mod libraries_cmd;
pub mod mentor_cmd;
pub mod migration_cmd;
pub mod practice_cmd;
pub mod pty_cmd;
pub mod state;
pub mod templates;
pub mod toolchain;
pub mod workspace_cmd;

use tauri::{AppHandle, Manager, State};

use crate::error::CmdResult;
use crate::state::AppState;

#[tauri::command]
fn get_windows_capabilities() -> toolchain::WindowsCapabilityReport {
    toolchain::inspect_capabilities()
}

#[tauri::command]
fn run_toolchain_hello_world() -> CmdResult<toolchain::HelloWorldCheckResult> {
    toolchain::run_hello_world_test()
}

#[tauri::command]
fn get_course_overview(
    state: State<'_, AppState>,
    preview_unlocked: bool,
) -> CmdResult<curriculum_cmd::CourseOverviewDto> {
    curriculum_cmd::get_course_overview(&state, preview_unlocked)
}

#[tauri::command]
fn get_lesson_detail(
    state: State<'_, AppState>,
    entry_id: String,
) -> CmdResult<curriculum_cmd::LessonDetailDto> {
    curriculum_cmd::get_lesson_detail(&state, &entry_id)
}

#[tauri::command]
fn get_exercises_catalog(
    state: State<'_, AppState>,
    bugfix_mode: bool,
) -> CmdResult<Vec<curriculum_cmd::ClassicExerciseDto>> {
    curriculum_cmd::get_exercises_catalog(&state, bugfix_mode)
}

#[tauri::command]
fn submit_quiz_answer(
    state: State<'_, AppState>,
    entry_id: String,
    quiz_index: usize,
    option_index: usize,
) -> CmdResult<curriculum_cmd::QuizSubmitResultDto> {
    curriculum_cmd::submit_quiz_answer(&state, &entry_id, quiz_index, option_index)
}

#[tauri::command]
fn complete_reading_lesson(
    state: State<'_, AppState>,
    entry_id: String,
) -> CmdResult<curriculum_cmd::QuizSubmitResultDto> {
    curriculum_cmd::complete_reading_lesson(&state, &entry_id)
}

#[tauri::command]
fn save_practice_draft(
    state: State<'_, AppState>,
    scope: String,
    id: String,
    code: String,
) -> CmdResult<()> {
    practice_cmd::save_practice_draft(&state, &scope, &id, &code)
}

#[tauri::command]
fn reveal_practice_hint(state: State<'_, AppState>, scope: String, id: String) -> CmdResult<usize> {
    practice_cmd::reveal_practice_hint(&state, &scope, &id)
}

#[tauri::command]
fn get_idiomatic_solution(
    state: State<'_, AppState>,
    scope: String,
    id: String,
) -> CmdResult<Option<(String, String)>> {
    practice_cmd::get_idiomatic_solution(&state, &scope, &id)
}

#[tauri::command]
fn cancel_active_execution(state: State<'_, AppState>) -> CmdResult<()> {
    practice_cmd::cancel_active_execution(&state)
}

#[tauri::command]
async fn run_practice_check(
    app: AppHandle,
    state: State<'_, AppState>,
    scope: String,
    id: String,
    code: String,
    ai_used: bool,
) -> CmdResult<practice_cmd::PracticeRunResultDto> {
    let shared = state.inner.clone();
    tokio::task::spawn_blocking(move || {
        let wrapper = AppState { inner: shared };
        practice_cmd::run_practice_check_core(Some(&app), &wrapper, &scope, &id, &code, ai_used)
    })
    .await
    .map_err(|e| {
        error::CommandError::new(
            "INTERNAL",
            "Falha de tarefa em background",
            Some(e.to_string()),
        )
    })?
}

#[tauri::command]
fn get_default_projects_dir() -> String {
    workspace_cmd::get_default_projects_dir()
}

#[tauri::command]
fn list_project_templates() -> Vec<workspace_cmd::TemplateDto> {
    workspace_cmd::list_project_templates()
}

#[tauri::command]
fn create_project(
    state: State<'_, AppState>,
    parent_dir: String,
    name: String,
    template_id: String,
) -> CmdResult<workspace_cmd::ProjectInfoDto> {
    workspace_cmd::create_project(&state, &parent_dir, &name, &template_id)
}

#[tauri::command]
fn open_project(
    state: State<'_, AppState>,
    root_dir: String,
) -> CmdResult<workspace_cmd::ProjectInfoDto> {
    workspace_cmd::open_project(&state, &root_dir)
}

#[tauri::command]
fn trust_project(state: State<'_, AppState>, root_dir: String) -> CmdResult<()> {
    workspace_cmd::trust_project(&state, &root_dir)
}

#[tauri::command]
fn get_recent_projects(state: State<'_, AppState>) -> CmdResult<Vec<String>> {
    workspace_cmd::get_recent_projects(&state)
}

#[tauri::command]
fn read_project_file(
    root_dir: String,
    rel_path: String,
) -> CmdResult<workspace_cmd::FileContentDto> {
    workspace_cmd::read_project_file(&root_dir, &rel_path)
}

#[tauri::command]
fn save_project_file(
    root_dir: String,
    rel_path: String,
    content: String,
    expected_sha256: Option<String>,
    force: bool,
) -> CmdResult<workspace_cmd::SaveFileResultDto> {
    workspace_cmd::save_project_file(
        &root_dir,
        &rel_path,
        &content,
        expected_sha256.as_deref(),
        force,
    )
}

#[tauri::command]
fn create_project_file(root_dir: String, rel_path: String) -> CmdResult<String> {
    workspace_cmd::create_project_file(&root_dir, &rel_path)
}

#[tauri::command]
fn rename_project_file(root_dir: String, old_rel: String, new_rel: String) -> CmdResult<String> {
    workspace_cmd::rename_project_file(&root_dir, &old_rel, &new_rel)
}

#[tauri::command]
fn delete_project_file(root_dir: String, rel_path: String) -> CmdResult<()> {
    workspace_cmd::delete_project_file(&root_dir, &rel_path)
}

#[tauri::command]
fn create_project_plan_md(root_dir: String) -> CmdResult<String> {
    workspace_cmd::create_project_plan_md(&root_dir)
}

#[tauri::command]
fn search_in_project(
    root_dir: String,
    query: String,
) -> CmdResult<Vec<workspace_cmd::SearchHitDto>> {
    workspace_cmd::search_in_project(&root_dir, &query)
}

#[tauri::command]
async fn run_workspace_cargo(
    app: AppHandle,
    state: State<'_, AppState>,
    root_dir: String,
    action: String,
) -> CmdResult<workspace_cmd::WorkspaceExecResultDto> {
    let shared = state.inner.clone();
    tokio::task::spawn_blocking(move || {
        let wrapper = AppState { inner: shared };
        workspace_cmd::run_workspace_cargo(Some(&app), &wrapper, &root_dir, &action)
    })
    .await
    .map_err(|e| {
        error::CommandError::new(
            "INTERNAL",
            "Falha de tarefa em background",
            Some(e.to_string()),
        )
    })?
}

#[tauri::command]
async fn launch_gui_app(
    app: AppHandle,
    state: State<'_, AppState>,
    root_dir: String,
) -> CmdResult<String> {
    let shared = state.inner.clone();
    tokio::task::spawn_blocking(move || {
        let wrapper = AppState { inner: shared };
        workspace_cmd::launch_gui_app(Some(&app), &wrapper, &root_dir)
    })
    .await
    .map_err(|e| {
        error::CommandError::new(
            "INTERNAL",
            "Falha de tarefa em background",
            Some(e.to_string()),
        )
    })?
}

#[tauri::command]
fn get_task_history(state: State<'_, AppState>) -> CmdResult<Vec<state::TaskHistoryEntry>> {
    workspace_cmd::get_task_history(&state)
}

#[tauri::command]
fn lsp_inspect_file(
    state: State<'_, AppState>,
    root_dir: String,
    rel_path: String,
    content: String,
    word_under_cursor: Option<String>,
) -> CmdResult<workspace_cmd::LspInspectDto> {
    workspace_cmd::lsp_inspect_file(
        &state,
        &root_dir,
        &rel_path,
        &content,
        word_under_cursor.as_deref(),
    )
}

#[tauri::command]
fn pty_start(
    app: AppHandle,
    state: State<'_, AppState>,
    root_dir: String,
    cols: u16,
    rows: u16,
) -> CmdResult<String> {
    pty_cmd::pty_start(&app, &state, &root_dir, cols, rows)
}

#[tauri::command]
fn pty_write(state: State<'_, AppState>, session_id: String, data: String) -> CmdResult<()> {
    pty_cmd::pty_write(&state, &session_id, &data)
}

#[tauri::command]
fn pty_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> CmdResult<()> {
    pty_cmd::pty_resize(&state, &session_id, cols, rows)
}

#[tauri::command]
fn pty_kill(state: State<'_, AppState>, session_id: String) -> CmdResult<()> {
    pty_cmd::pty_kill(&state, &session_id)
}

#[tauri::command]
fn list_libraries(
    state: State<'_, AppState>,
    root_dir: Option<String>,
) -> CmdResult<Vec<libraries_cmd::LibraryItemDto>> {
    libraries_cmd::list_libraries(&state, root_dir.as_deref())
}

#[tauri::command]
fn preview_add_library(
    state: State<'_, AppState>,
    root_dir: String,
    lib_name: String,
    include_template: bool,
) -> CmdResult<libraries_cmd::LibraryChangePreviewDto> {
    libraries_cmd::preview_add_library(&state, &root_dir, &lib_name, include_template)
}

#[tauri::command]
fn apply_add_library(
    state: State<'_, AppState>,
    root_dir: String,
    lib_name: String,
    include_template: bool,
) -> CmdResult<libraries_cmd::LibraryApplyResultDto> {
    libraries_cmd::apply_add_library(&state, &root_dir, &lib_name, include_template)
}

#[tauri::command]
fn revert_library_change(state: State<'_, AppState>, revert_id: String) -> CmdResult<()> {
    libraries_cmd::revert_library_change(&state, &revert_id)
}

#[tauri::command]
async fn mentor_ask(
    state: State<'_, AppState>,
    req: mentor_cmd::MentorAskRequest,
) -> CmdResult<mentor_cmd::MentorReplyDto> {
    mentor_cmd::mentor_ask(&state, req).await
}

#[tauri::command]
async fn mentor_list_ollama_models(state: State<'_, AppState>) -> CmdResult<Vec<String>> {
    mentor_cmd::list_ollama_models(&state).await
}

#[tauri::command]
fn mentor_set_api_key(api_key: String) -> CmdResult<()> {
    mentor_cmd::set_api_key(&api_key)
}

#[tauri::command]
fn mentor_delete_api_key() -> CmdResult<()> {
    mentor_cmd::delete_api_key()
}

#[tauri::command]
fn get_progress_summary(
    state: State<'_, AppState>,
) -> CmdResult<migration_cmd::ProgressSummaryDto> {
    migration_cmd::get_progress_summary(&state)
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> CmdResult<migration_cmd::SettingsDto> {
    migration_cmd::get_settings(&state)
}

#[tauri::command]
fn update_settings(
    state: State<'_, AppState>,
    patch: migration_cmd::SettingsPatchDto,
) -> CmdResult<migration_cmd::SettingsDto> {
    migration_cmd::update_settings(&state, patch)
}

#[tauri::command]
fn grant_exec_consent(state: State<'_, AppState>) -> CmdResult<()> {
    migration_cmd::grant_exec_consent(&state)
}

#[tauri::command]
fn reset_progress(state: State<'_, AppState>) -> CmdResult<()> {
    migration_cmd::reset_progress(&state)
}

#[tauri::command]
fn export_backup(state: State<'_, AppState>, dest_path: String) -> CmdResult<()> {
    migration_cmd::export_backup(&state, &dest_path)
}

#[tauri::command]
fn preview_import_backup(
    state: State<'_, AppState>,
    src_path: String,
) -> CmdResult<migration_cmd::ImportPreviewDto> {
    migration_cmd::preview_import_backup(&state, &src_path)
}

#[tauri::command]
fn apply_import_backup(
    state: State<'_, AppState>,
    src_path: String,
    mode: String,
) -> CmdResult<migration_cmd::ProgressSummaryDto> {
    migration_cmd::apply_import_backup(&state, &src_path, &mode)
}

#[tauri::command]
fn open_safe_url(app: AppHandle, url: String) -> CmdResult<()> {
    use tauri_plugin_opener::OpenerExt;
    let safe = migration_cmd::validate_safe_external_url(&url)?;
    app.opener()
        .open_url(safe, None::<&str>)
        .map_err(|e| error::CommandError::io("Falha ao abrir link no navegador", e))
}

#[tauri::command]
fn export_diagnostics_log(state: State<'_, AppState>, dest_path: String) -> CmdResult<()> {
    migration_cmd::export_diagnostics_log(&state, &dest_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let state = window.state::<AppState>();
                if let Ok(mut inner) = state.inner.lock() {
                    inner.exec.cancel();
                    for (_, mut sess) in inner.pty_sessions.drain() {
                        let _ = sess.killer.kill();
                    }
                    if let Some(mut lsp) = inner.lsp.take()
                        && let Some(mut client) = lsp.client.take()
                    {
                        client.shutdown();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_windows_capabilities,
            run_toolchain_hello_world,
            get_course_overview,
            get_lesson_detail,
            get_exercises_catalog,
            submit_quiz_answer,
            complete_reading_lesson,
            save_practice_draft,
            reveal_practice_hint,
            get_idiomatic_solution,
            cancel_active_execution,
            run_practice_check,
            get_default_projects_dir,
            list_project_templates,
            create_project,
            open_project,
            trust_project,
            get_recent_projects,
            read_project_file,
            save_project_file,
            create_project_file,
            rename_project_file,
            delete_project_file,
            create_project_plan_md,
            search_in_project,
            run_workspace_cargo,
            launch_gui_app,
            get_task_history,
            lsp_inspect_file,
            pty_start,
            pty_write,
            pty_resize,
            pty_kill,
            list_libraries,
            preview_add_library,
            apply_add_library,
            revert_library_change,
            mentor_ask,
            mentor_list_ollama_models,
            mentor_set_api_key,
            mentor_delete_api_key,
            get_progress_summary,
            get_settings,
            update_settings,
            grant_exec_consent,
            reset_progress,
            export_backup,
            preview_import_backup,
            apply_import_backup,
            open_safe_url,
            export_diagnostics_log,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao executar RustLab Desktop");
}
