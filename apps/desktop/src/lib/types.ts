export interface CommandError {
  code: string;
  messagePt: string;
  detail?: string | null;
}

export interface ToolchainFix {
  title: string;
  description: string;
  command?: string | null;
  docsUrl?: string | null;
}

export interface WindowsCapabilityReport {
  osCaption: string;
  osBuild: string;
  arch: string;
  webview2Version?: string | null;
  offlineCurriculumReady: boolean;
  rustupFound: boolean;
  rustcVersion?: string | null;
  cargoVersion?: string | null;
  cargoPath?: string | null;
  defaultHostTriple?: string | null;
  rustfmtAvailable: boolean;
  clippyAvailable: boolean;
  rustAnalyzerAvailable: boolean;
  msvcLinkerAvailable: boolean;
  gnuLinkerAvailable: boolean;
  windowsSdkAvailable: boolean;
  canCompileNative: boolean;
  conptySupported: boolean;
  recommendedActionPt: string;
  copyableFixes: ToolchainFix[];
}

export interface HelloWorldCheckResult {
  success: boolean;
  category: string;
  stdout: string;
  stderr: string;
  suggestionPt: string;
}

export interface CourseEntrySummaryDto {
  id: string;
  title: string;
  stageId: string;
  kind: string;
  cycle?: string | null;
  prerequisites: string[];
  learningGoal: string;
  completionEvidence: string;
  unlocked: boolean;
  completed: boolean;
  read: boolean;
  isInteractiveStdin: boolean;
}

export interface CourseStageDto {
  id: string;
  label: string;
  subtitle: string;
  unlocked: boolean;
  completedCount: number;
  totalCount: number;
  entries: CourseEntrySummaryDto[];
}

export interface CourseOverviewDto {
  totalEntries: number;
  completedEntries: number;
  xp: number;
  level: number;
  lastEntryId?: string | null;
  dueSrsItems: string[];
  stages: CourseStageDto[];
}

export type LessonBlockDto =
  | { type: 'text'; text: string }
  | { type: 'code'; code: string; file?: string | null; explain?: string | null }
  | { type: 'callout'; kind: string; text: string }
  | { type: 'demo'; demo: string; name: string; steps: string[]; caption?: string | null }
  | { type: 'quiz'; quizIndex: number; question: string; options: string[]; correct: number; why?: string | null };

export interface PracticeSpecDto {
  id: string;
  lessonId: string;
  title: string;
  prompt: string;
  starter: string;
  currentCode: string;
  hints: string[];
  shownHints: number;
  mode: string;
  stdinPreview: string[];
  completed: boolean;
  hasIdiomatic: boolean;
}

export interface LessonDetailDto {
  id: string;
  title: string;
  stageId: string;
  stageLabel: string;
  kind: string;
  completed: boolean;
  requiresQuiz: boolean;
  ferrisMood: string;
  ferrisAscii: string;
  blocks: LessonBlockDto[];
  practice?: PracticeSpecDto | null;
  prevEntryId?: string | null;
  nextEntryId?: string | null;
}

export interface ClassicExerciseDto {
  id: string;
  title: string;
  statement: string;
  starter: string;
  currentCode: string;
  hints: string[];
  shownHints: number;
  completed: boolean;
  hasIdiomatic: boolean;
  isBugfix: boolean;
}

export interface QuizSubmitResultDto {
  correct: boolean;
  explanation?: string | null;
  lessonCompleted: boolean;
  xpAwarded: number;
  totalXp: number;
  level: number;
  ferrisMood: string;
  ferrisAscii: string;
}

export interface DiagnosticDto {
  level: string;
  message: string;
  file?: string | null;
  line?: number | null;
  col?: number | null;
}

export interface PracticeRunResultDto {
  outcome: string;
  outputLines: string[];
  diagnostics: DiagnosticDto[];
  ferrisExplanation?: string | null;
  socraticEli10?: string | null;
  clippyTips: string[];
  idiomaticSolution?: [string, string] | null;
  xpAwarded: number;
  independenceBonus: number;
  totalXp: number;
  level: number;
  ferrisMood: string;
  ferrisAscii: string;
}

export interface PracticeStreamEvent {
  kind: string;
  text: string;
}

export interface TemplateDto {
  id: string;
  name: string;
  description: string;
}

export interface ProjectCapabilitiesDto {
  hasPlanMd: boolean;
  sourceFileCount: number;
  hasStructsOrEnums: boolean;
  hasErrorHandling: boolean;
  hasUnitTests: boolean;
  hasDocComments: boolean;
}

export interface ProjectInfoDto {
  name: string;
  rootDir: string;
  trusted: boolean;
  files: string[];
  capabilities: ProjectCapabilitiesDto;
}

export interface FileContentDto {
  relPath: string;
  content: string;
  sha256: string;
}

export interface SaveFileResultDto {
  saved: boolean;
  conflict: boolean;
  diskSha256: string;
  diskContent?: string | null;
}

export interface SearchHitDto {
  relPath: string;
  line: number;
  column: number;
  preview: string;
}

export interface WorkspaceExecResultDto {
  status: string;
  code?: number | null;
  outputLines: string[];
  diagnostics: DiagnosticDto[];
}

export interface TaskHistoryEntry {
  id: string;
  command: string;
  cwd: string;
  status: string;
  timestamp: string;
  outputPreview: string;
}

export interface InlayHintDto {
  line: number;
  col: number;
  label: string;
}

export interface LspInspectDto {
  lspActive: boolean;
  completions: string[];
  inlayHints: InlayHintDto[];
  hoverDoc?: string | null;
}

export interface PtyDataEvent {
  sessionId: string;
  data: string;
}

export interface PtyExitEvent {
  sessionId: string;
  code?: number | null;
}

export interface LibraryItemDto {
  name: string;
  version: string;
  category: string;
  summary: string;
  docsPt: string;
  useLine: string;
  docUrl: string;
  associatedV3Lessons: string[];
  commonPitfalls: string;
  windowsNotes: string;
  installedInProject: boolean;
  versionNote?: string | null;
}

export interface LibraryChangePreviewDto {
  name: string;
  version: string;
  cargoTomlBefore: string;
  cargoTomlAfter: string;
  targetRsRel: string;
  targetRsBefore: string;
  targetRsAfter: string;
  requiresDownloadNote: string;
}

export interface LibraryApplyResultDto {
  revertId: string;
  targetRsRel: string;
}

export interface MentorAskRequest {
  learnMode: boolean;
  action: string;
  origin: string;
  topicTitle?: string | null;
  summary: string;
  code: string;
  errors: string;
  console: string;
  question: string;
}

export interface MentorReplyDto {
  source: string;
  body: string;
  readOnlyEnforced: boolean;
  proposedCode?: string | null;
  probabilisticNotice: string;
}

export interface StageProgressDto {
  id: string;
  label: string;
  completed: number;
  total: number;
}

export interface AchievementDto {
  id: string;
  title: string;
  unlocked: boolean;
}

export interface ProgressSummaryDto {
  xp: number;
  level: number;
  completedV3: number;
  totalV3: number;
  completedExercises: number;
  independentCompletions: number;
  dueSrsItems: string[];
  topErrors: [string, number][];
  stageProgress: StageProgressDto[];
  achievements: AchievementDto[];
}

export interface SettingsDto {
  language: string;
  originLanguage: string;
  theme: string;
  desktopTheme: string;
  animations: string;
  gamificationVisible: boolean;
  focusMode: boolean;
  soundEnabled: boolean;
  soundVolume: number;
  uiScale: number;
  editorFontSize: number;
  firstRunCompleted: boolean;
  execConsentGranted: boolean;
  aiProvider: string;
  ollamaUrl: string;
  ollamaModel: string;
  remoteUrl: string;
  remoteModel: string;
  remoteConsent: string;
  hasRemoteApiKey: boolean;
  dataRoot: string;
}

export interface SettingsPatchDto {
  language?: string;
  originLanguage?: string;
  desktopTheme?: string;
  animations?: string;
  gamificationVisible?: boolean;
  focusMode?: boolean;
  soundEnabled?: boolean;
  soundVolume?: number;
  uiScale?: number;
  editorFontSize?: number;
  firstRunCompleted?: boolean;
  aiProvider?: string;
  ollamaUrl?: string;
  ollamaModel?: string;
  remoteUrl?: string;
  remoteModel?: string;
  remoteConsent?: string;
}

export interface ImportPreviewDto {
  sourceFormat: string;
  incomingLessons: number;
  incomingExercises: number;
  incomingXp: number;
  newXpAfterMerge: number;
  warning?: string | null;
}

export type AppRoute =
  | 'home'
  | 'course'
  | 'lesson'
  | 'exercises'
  | 'create'
  | 'progress'
  | 'settings'
  | 'help';
