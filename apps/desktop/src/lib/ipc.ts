import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open, save } from '@tauri-apps/plugin-dialog';
import type {
  ClassicExerciseDto,
  CommandError,
  CourseOverviewDto,
  FileContentDto,
  HelloWorldCheckResult,
  ImportPreviewDto,
  LessonDetailDto,
  LibraryApplyResultDto,
  LibraryChangePreviewDto,
  LibraryItemDto,
  LspInspectDto,
  MentorAskRequest,
  MentorReplyDto,
  PracticeRunResultDto,
  PracticeStreamEvent,
  ProgressSummaryDto,
  ProjectInfoDto,
  PtyDataEvent,
  PtyExitEvent,
  QuizSubmitResultDto,
  SaveFileResultDto,
  SearchHitDto,
  SettingsDto,
  SettingsPatchDto,
  TaskHistoryEntry,
  TemplateDto,
  WindowsCapabilityReport,
  WorkspaceExecResultDto,
} from './types';

export function formatIpcError(err: unknown): string {
  if (typeof err === 'string') return err;
  if (err && typeof err === 'object') {
    const ce = err as Partial<CommandError> & { message?: string };
    if (ce.messagePt) {
      return ce.detail ? `${ce.messagePt} (${ce.detail})` : ce.messagePt;
    }
    if (ce.message) return ce.message;
  }
  return 'Erro inesperado na comunicação com o motor RustLab.';
}

export function isConsentError(err: unknown): boolean {
  return Boolean(
    err && typeof err === 'object' && (err as Partial<CommandError>).code === 'CONSENT_REQUIRED',
  );
}

export const ipc = {
  getWindowsCapabilities: () => invoke<WindowsCapabilityReport>('get_windows_capabilities'),
  runToolchainHelloWorld: () => invoke<HelloWorldCheckResult>('run_toolchain_hello_world'),

  getCourseOverview: (previewUnlocked: boolean) =>
    invoke<CourseOverviewDto>('get_course_overview', { previewUnlocked }),
  getLessonDetail: (entryId: string) =>
    invoke<LessonDetailDto>('get_lesson_detail', { entryId }),
  getExercisesCatalog: (bugfixMode: boolean) =>
    invoke<ClassicExerciseDto[]>('get_exercises_catalog', { bugfixMode }),
  submitQuizAnswer: (entryId: string, quizIndex: number, optionIndex: number) =>
    invoke<QuizSubmitResultDto>('submit_quiz_answer', { entryId, quizIndex, optionIndex }),
  completeReadingLesson: (entryId: string) =>
    invoke<QuizSubmitResultDto>('complete_reading_lesson', { entryId }),

  savePracticeDraft: (scope: 'practice' | 'exercise', id: string, code: string) =>
    invoke<void>('save_practice_draft', { scope, id, code }),
  revealPracticeHint: (scope: 'practice' | 'exercise', id: string) =>
    invoke<number>('reveal_practice_hint', { scope, id }),
  getIdiomaticSolution: (scope: 'practice' | 'exercise', id: string) =>
    invoke<[string, string] | null>('get_idiomatic_solution', { scope, id }),
  runPracticeCheck: (scope: 'practice' | 'exercise', id: string, code: string, aiUsed: boolean) =>
    invoke<PracticeRunResultDto>('run_practice_check', { scope, id, code, aiUsed }),
  cancelActiveExecution: () => invoke<void>('cancel_active_execution'),

  getDefaultProjectsDir: () => invoke<string>('get_default_projects_dir'),
  listProjectTemplates: () => invoke<TemplateDto[]>('list_project_templates'),
  createProject: (parentDir: string, name: string, templateId: string) =>
    invoke<ProjectInfoDto>('create_project', { parentDir, name, templateId }),
  openProject: (rootDir: string) => invoke<ProjectInfoDto>('open_project', { rootDir }),
  trustProject: (rootDir: string) => invoke<void>('trust_project', { rootDir }),
  getRecentProjects: () => invoke<string[]>('get_recent_projects'),
  readProjectFile: (rootDir: string, relPath: string) =>
    invoke<FileContentDto>('read_project_file', { rootDir, relPath }),
  saveProjectFile: (
    rootDir: string,
    relPath: string,
    content: string,
    expectedSha256: string | null,
    force: boolean,
  ) =>
    invoke<SaveFileResultDto>('save_project_file', {
      rootDir,
      relPath,
      content,
      expectedSha256,
      force,
    }),
  createProjectFile: (rootDir: string, relPath: string) =>
    invoke<string>('create_project_file', { rootDir, relPath }),
  renameProjectFile: (rootDir: string, oldRel: string, newRel: string) =>
    invoke<string>('rename_project_file', { rootDir, oldRel, newRel }),
  deleteProjectFile: (rootDir: string, relPath: string) =>
    invoke<void>('delete_project_file', { rootDir, relPath }),
  createProjectPlanMd: (rootDir: string) =>
    invoke<string>('create_project_plan_md', { rootDir }),
  searchInProject: (rootDir: string, query: string) =>
    invoke<SearchHitDto[]>('search_in_project', { rootDir, query }),
  runWorkspaceCargo: (rootDir: string, action: string) =>
    invoke<WorkspaceExecResultDto>('run_workspace_cargo', { rootDir, action }),
  launchGuiApp: (rootDir: string) => invoke<string>('launch_gui_app', { rootDir }),
  getTaskHistory: () => invoke<TaskHistoryEntry[]>('get_task_history'),
  lspInspectFile: (
    rootDir: string,
    relPath: string,
    content: string,
    wordUnderCursor: string | null,
  ) =>
    invoke<LspInspectDto>('lsp_inspect_file', { rootDir, relPath, content, wordUnderCursor }),

  ptyStart: (rootDir: string, cols: number, rows: number) =>
    invoke<string>('pty_start', { rootDir, cols, rows }),
  ptyWrite: (sessionId: string, data: string) =>
    invoke<void>('pty_write', { sessionId, data }),
  ptyResize: (sessionId: string, cols: number, rows: number) =>
    invoke<void>('pty_resize', { sessionId, cols, rows }),
  ptyKill: (sessionId: string) => invoke<void>('pty_kill', { sessionId }),

  listLibraries: (rootDir: string | null) =>
    invoke<LibraryItemDto[]>('list_libraries', { rootDir }),
  previewAddLibrary: (rootDir: string, libName: string, includeTemplate: boolean) =>
    invoke<LibraryChangePreviewDto>('preview_add_library', {
      rootDir,
      libName,
      includeTemplate,
    }),
  applyAddLibrary: (rootDir: string, libName: string, includeTemplate: boolean) =>
    invoke<LibraryApplyResultDto>('apply_add_library', {
      rootDir,
      libName,
      includeTemplate,
    }),
  revertLibraryChange: (revertId: string) =>
    invoke<void>('revert_library_change', { revertId }),

  mentorAsk: (req: MentorAskRequest) => invoke<MentorReplyDto>('mentor_ask', { req }),
  mentorListOllamaModels: () => invoke<string[]>('mentor_list_ollama_models'),
  mentorSetApiKey: (apiKey: string) => invoke<void>('mentor_set_api_key', { apiKey }),
  mentorDeleteApiKey: () => invoke<void>('mentor_delete_api_key'),

  getProgressSummary: () => invoke<ProgressSummaryDto>('get_progress_summary'),
  getSettings: () => invoke<SettingsDto>('get_settings'),
  updateSettings: (patch: SettingsPatchDto) => invoke<SettingsDto>('update_settings', { patch }),
  grantExecConsent: () => invoke<void>('grant_exec_consent'),
  resetProgress: () => invoke<void>('reset_progress'),
  exportBackup: (destPath: string) => invoke<void>('export_backup', { destPath }),
  previewImportBackup: (srcPath: string) =>
    invoke<ImportPreviewDto>('preview_import_backup', { srcPath }),
  applyImportBackup: (srcPath: string, mode: 'merge' | 'replace') =>
    invoke<ProgressSummaryDto>('apply_import_backup', { srcPath, mode }),
  openSafeUrl: (url: string) => invoke<void>('open_safe_url', { url }),
  exportDiagnosticsLog: (destPath: string) =>
    invoke<void>('export_diagnostics_log', { destPath }),

  listenPracticeEvents: (cb: (ev: PracticeStreamEvent) => void): Promise<UnlistenFn> =>
    listen<PracticeStreamEvent>('practice://event', (e) => cb(e.payload)),
  listenWorkspaceEvents: (cb: (ev: PracticeStreamEvent) => void): Promise<UnlistenFn> =>
    listen<PracticeStreamEvent>('workspace://exec', (e) => cb(e.payload)),
  listenPtyData: (cb: (ev: PtyDataEvent) => void): Promise<UnlistenFn> =>
    listen<PtyDataEvent>('pty://data', (e) => cb(e.payload)),
  listenPtyExit: (cb: (ev: PtyExitEvent) => void): Promise<UnlistenFn> =>
    listen<PtyExitEvent>('pty://exit', (e) => cb(e.payload)),

  pickDirectory: async (defaultPath?: string): Promise<string | null> => {
    const selected = await open({ directory: true, multiple: false, defaultPath });
    return typeof selected === 'string' ? selected : null;
  },
  pickBackupFile: async (): Promise<string | null> => {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Backup / Progresso RustLab', extensions: ['json', 'toml'] }],
    });
    return typeof selected === 'string' ? selected : null;
  },
  pickSaveFile: async (defaultPath: string, ext: string): Promise<string | null> => {
    const selected = await save({
      defaultPath,
      filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
    });
    return typeof selected === 'string' ? selected : null;
  },
};
