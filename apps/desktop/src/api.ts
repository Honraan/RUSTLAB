import { invoke } from "@tauri-apps/api/core";

export type Entry = {
  id: string;
  title: string;
  prerequisites: string[];
  kind: "Teoria" | "Prática" | "Marco de projeto";
  completed: boolean;
  unlocked: boolean;
};
export type Stage = { title: string; entries: Entry[] };
export type Course = { stages: Stage[]; xp: number; next: string | null; reviews: string[]; ferris: string; warning: string | null };
export type Content =
  | { type: "text"; text: string }
  | { type: "code"; code: string; file: string | null; explain: string | null }
  | { type: "callout"; text: string; label: string }
  | { type: "demo"; name: string; steps: string[]; caption: string | null };
export type Lesson = {
  entry: Entry;
  stage: string;
  minutes: number;
  content: Content[];
  questions: { question: string; options: string[] }[];
  practice: { prompt: string; source: string; hints: string[] } | null;
};
export type QuizResult = {
  correct: boolean[];
  explanations: (string | null)[];
  completed: boolean;
  awarded: number;
  ferris: string;
};
export type Project = { name: string; root: string; files: string[] };
export type Action = "check" | "run" | "test" | "fmt" | "clippy" | "gui";
export type Template = "basic" | "paint" | "cavegame";
export type Line = { stream: "stdout" | "stderr" | "system"; text: string };
export type Marker = { file: string; line: number; column: number; severity: string; message: string };
export type Jump = Pick<Marker, "file" | "line" | "column"> & { sequence: number };
export type Outcome = { state: string; code: number | null; message: string; awarded: number; markers: Marker[] };
export type Batch = { lines: Line[]; result: Outcome | null };
export type Doctor = Record<"cargo" | "rustc" | "rustup" | "rustfmt" | "clippy" | "analyzer", string | null>;
export type Library = {
  name: string; version: string; category: string; summary: string; warnings: string; lessons: string[]; docs: string;
};

export type LibraryPlan = { before: string; after: string; present: boolean };

export const api = {
  course: () => invoke<Course>("course_map"),
  lesson: (id: string) => invoke<Lesson>("lesson", { id }),
  answer: (id: string, picks: number[]) => invoke<QuizResult>("answer", { id, picks }),
  draft: (id: string, source: string) => invoke<void>("save_draft", { id, source }),
  hint: (id: string) => invoke<string[]>("reveal_hint", { id }),
  practice: (id: string, source: string) => invoke<void>("start_practice", { id, source, consent: true }),
  openProject: () => invoke<Project | null>("open_project"),
  createProject: (name: string, template: Template) => invoke<Project | null>("create_project", { name, template }),
  read: (relative: string) => invoke<string>("read_source", { relative }),
  save: (relative: string, expected: string, source: string) =>
    invoke<void>("save_source", { relative, expected, source }),
  newFile: (relative: string) => invoke<Project>("create_source", { relative }),
  projectTask: (action: Action, allowDownloads: boolean) =>
    invoke<void>("run_project", { action, consent: true, allowDownloads }),
  poll: () => invoke<Batch>("poll_task"),
  cancel: () => invoke<void>("cancel_task"),
  terminalStart: (allowDownloads: boolean, columns: number, rows: number) =>
    invoke<void>("terminal_start", { consent: true, allowDownloads, columns, rows }),
  terminalWrite: (input: number[]) => invoke<void>("terminal_write", { input }),
  terminalResize: (columns: number, rows: number) => invoke<void>("terminal_resize", { columns, rows }),
  terminalPoll: () => invoke<{ bytes: number[]; finished: string | null }>("terminal_poll"),
  terminalCancel: () => invoke<void>("terminal_cancel"),
  doctor: () => invoke<Doctor>("doctor"),
  toolchainTest: () => invoke<void>("test_toolchain", { consent: true }),
  libraries: () => invoke<Library[]>("libraries"),
  libraryPreview: (name: string) => invoke<LibraryPlan>("library_preview", { name }),
  libraryApply: (name: string, plan: LibraryPlan, allowDownloads: boolean) =>
    invoke<void>("library_apply", { name, before: plan.before, after: plan.after, consent: true, allowDownloads }),
  libraryUndo: () => invoke<void>("library_undo"),
  import: () => invoke<string | null>("import_progress", { replace: true }),
  export: () => invoke<boolean>("export_progress"),
};

export function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
