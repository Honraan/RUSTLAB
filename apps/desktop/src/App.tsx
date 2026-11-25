import { useCallback, useEffect, useRef, useState } from "react";
import { ArrowLeft, ArrowRight, BookOpen, Boxes, ChartNoAxesColumnIncreasing, ChevronRight, CircleHelp, CodeXml, FolderOpen, House, Settings, Sparkles, Terminal, X } from "lucide-react";
import { api, errorMessage } from "./api";
import type { Course, Entry, Lesson, Line, Outcome } from "./api";
import { LessonPage } from "./LessonPage";
import { ProjectPage } from "./ProjectPage";
import { SettingsPage, LibrariesPage } from "./SettingsPage";
import ferris from "./ferris.png";

export type RunTask = (label: string, start: () => Promise<void>) => Promise<void>;
export type Preferences = { theme: "system" | "dark" | "light"; motion: boolean; focus: boolean; volume: number; font: number; scale: number };
const defaults: Preferences = { theme: "system", motion: true, focus: false, volume: 0, font: 15, scale: 100 };

type Page = "home" | "course" | "lesson" | "project" | "progress" | "libraries" | "settings" | "help";
const routes = [
  { id: "home", title: "Início", icon: House },
  { id: "course", title: "Trilha", icon: BookOpen },
  { id: "project", title: "Criar", icon: CodeXml },
  { id: "progress", title: "Progresso", icon: ChartNoAxesColumnIncreasing },
  { id: "libraries", title: "Bibliotecas", icon: Boxes },
  { id: "settings", title: "Configurações", icon: Settings },
  { id: "help", title: "Ajuda", icon: CircleHelp },
] as const;
const stageDetails = [
  "Escreva suas primeiras linhas de Rust.",
  "Pratique tipos, funções e ownership.",
  "Construa aplicações para o terminal.",
  "Explore Rust assíncrono e avançado.",
  "Desenvolva um editor de desenho.",
  "Construa um mundo de blocos em 3D.",
];

function loadPreferences(): Preferences {
  try {
    const stored = JSON.parse(localStorage.getItem("rustlab-preferences") ?? "{}") as Partial<Preferences>;
    return { ...defaults, ...stored };
  } catch {
    return defaults;
  }
}

function playXp(volume: number) {
  if (!volume) return;
  try {
    const audio = new AudioContext();
    void audio.resume().then(() => {
      const now = audio.currentTime;
      for (const [index, frequency] of [440, 587.33].entries()) {
        const tone = audio.createOscillator();
        const envelope = audio.createGain();
        const start = now + index * 0.075;
        tone.frequency.value = frequency;
        envelope.gain.setValueAtTime(0, start);
        envelope.gain.linearRampToValueAtTime(volume / 1200, start + 0.025);
        envelope.gain.linearRampToValueAtTime(0, start + 0.13);
        tone.connect(envelope).connect(audio.destination);
        tone.start(start);
        tone.stop(start + 0.14);
        if (index === 1) tone.onended = () => { void audio.close(); };
      }
    }).catch(() => audio.close());
  } catch {
    // Audio is optional and can be unavailable.
  }
}

export function App() {
  const [page, setPage] = useState<Page>("home");
  const [course, setCourse] = useState<Course | null>(null);
  const [lesson, setLesson] = useState<Lesson | null>(null);
  const [stage, setStage] = useState<string | null>(null);
  const [preferences, setPreferences] = useState(loadPreferences);
  const [notice, setNotice] = useState("");
  const [lines, setLines] = useState<Line[]>([]);
  const [task, setTask] = useState<{ label: string; running: boolean; result: Outcome | null } | null>(null);
  const [consoleOpen, setConsoleOpen] = useState(false);
  const [projectName, setProjectName] = useState("");
  const pickingLesson = useRef(0);
  const taskPending = useRef(false);
  const heading = useRef<HTMLElement>(null);

  const refresh = useCallback(() => api.course().then(setCourse), []);
  const celebrate = useCallback((xp: number) => {
    if (xp && !preferences.focus) playXp(preferences.volume);
  }, [preferences.focus, preferences.volume]);
  const navigate = useCallback((destination: Page) => {
    pickingLesson.current += 1;
    setPage(destination);
  }, []);

  useEffect(() => { void refresh().catch((error: unknown) => setNotice(errorMessage(error))); }, [refresh]);
  useEffect(() => {
    const root = document.documentElement;
    root.dataset.theme = preferences.theme;
    root.dataset.motion = preferences.motion && !preferences.focus ? "on" : "off";
    root.style.fontSize = `${preferences.scale}%`;
    localStorage.setItem("rustlab-preferences", JSON.stringify(preferences));
  }, [preferences]);
  useEffect(() => {
    heading.current?.focus();
  }, [page, lesson?.entry.id]);
  useEffect(() => {
    const shortcuts = (event: KeyboardEvent) => {
      if (event.key === "F9") {
        event.preventDefault();
        navigate("libraries");
      }
    };
    window.addEventListener("keydown", shortcuts);
    return () => window.removeEventListener("keydown", shortcuts);
  }, [navigate]);

  const run: RunTask = async (label, start) => {
    if (taskPending.current) throw new Error("Uma tarefa já está em andamento.");
    taskPending.current = true;
    try {
      await start();
      setLines([]);
      setTask({ label, running: true, result: null });
      setConsoleOpen(true);
    } catch (error) {
      taskPending.current = false;
      throw error;
    }
  };

  const running = task?.running ?? false;
  useEffect(() => {
    if (!running) return;
    let disposed = false;
    let timer = 0;
    const poll = async () => {
      try {
        const batch = await api.poll();
        if (disposed) return;
        if (batch.lines.length) setLines((previous) => [...previous, ...batch.lines].slice(-1600));
        if (batch.result) {
          const result = batch.result;
          setTask((current) => current ? { ...current, running: false, result } : current);
          taskPending.current = false;
          celebrate(result.awarded);
          await refresh();
          return;
        }
      } catch (error) {
        if (disposed) return;
        setNotice(errorMessage(error));
      }
      if (!disposed) timer = window.setTimeout(() => { void poll(); }, 150);
    };
    void poll();
    return () => { disposed = true; window.clearTimeout(timer); };
  }, [running, refresh, celebrate]);

  async function openLesson(id: string) {
    const request = ++pickingLesson.current;
    try {
      const selected = await api.lesson(id);
      if (request === pickingLesson.current) {
        setLesson(selected);
        setPage("lesson");
      }
    } catch (error) {
      if (request === pickingLesson.current) setNotice(errorMessage(error));
    }
  }

  const entries = course?.stages.flatMap((group) => group.entries) ?? [];
  const next = entries.find((entry) => entry.id === course?.next);
  const complete = entries.filter((entry) => entry.completed).length;

  return (
    <div className="shell">
      <a href="#conteudo" className="skip">Pular para o conteúdo</a>
      <aside className="rail">
        <button className="brand" onClick={() => navigate("home")}>
          <img src={ferris} alt="" />
          <span>RustLab<small>DESKTOP</small></span>
        </button>
        <nav aria-label="Navegação principal">
          {routes.map(({ id, title, icon: Icon }) => (
            <button key={id} aria-current={page === id || (page === "lesson" && id === "course") ? "page" : undefined}
              onClick={() => navigate(id)}><Icon size={19} aria-hidden="true" />{title}{id === "libraries" && <kbd>F9</kbd>}</button>
          ))}
        </nav>
        <div className="rail-footer"><span className="status-dot" />Dados locais</div>
      </aside>
      <div className="body">
        <header className="topbar">
          <span>{page === "lesson" ? "Trilha / Lição" : routes.find((r) => r.id === page)?.title}</span>
          <div className="top-actions">
            {task && <button className="ghost" onClick={() => setConsoleOpen((open) => !open)} aria-expanded={consoleOpen}>
              <Terminal size={16} aria-hidden="true" /> Saída {running && "· em execução"}
            </button>}
            <span className="xp"><Sparkles size={16} aria-hidden="true" /> {course?.xp ?? 0} XP</span>
          </div>
        </header>
        <main id="conteudo" tabIndex={-1} ref={heading} className="main">
          {notice && <div className="notice" role="status"><span>{notice}</span>
            <button className="icon-button" aria-label="Dispensar aviso" onClick={() => setNotice("")}><X size={17} /></button>
          </div>}
          {course?.warning && <div className="notice" role="status">{course.warning}</div>}
          {page === "home" && <>
            <section className="hero">
              <div className="hero-text">
                <p className="eyebrow">Seu laboratório de Rust</p>
                <h1>Aprenda Rust<br /><em>escrevendo Rust.</em></h1>
                <p>Leia, experimente e descubra o que seu código faz. O Ferris acompanha você da primeira linha até o seu próprio projeto.</p>
                <button className="primary" onClick={() => next ? void openLesson(next.id) : navigate("course")}>
                  {complete ? "Retomar trilha" : "Começar a aprender"} <ArrowRight size={18} aria-hidden="true" />
                </button>
              </div>
              <div className="hero-art" aria-hidden="true">
                <div className="mascot-frame"><img src={ferris} alt="" /></div>
                <code>{course?.ferris || "c(o.o)c"}</code>
              </div>
            </section>
            <div className="home-links">
              {next && <button className="feature" onClick={() => void openLesson(next.id)}>
                <BookOpen size={22} aria-hidden="true" /><span><small>CONTINUAR TRILHA</small><strong>{next.title}</strong></span><ChevronRight size={18} />
              </button>}
              <button className="feature" onClick={() => navigate("project")}>
                <FolderOpen size={22} aria-hidden="true" /><span><small>ESPAÇO DE TRABALHO</small><strong>{projectName || "Abra seu primeiro projeto"}</strong></span><ChevronRight size={18} />
              </button>
            </div>
            {course && <StageGrid course={course} open={(title) => { setStage(title); navigate("course"); }} />}
          </>}
          {page === "course" && <>
            <h1>Trilha de aprendizagem</h1>
            <p className="lead">Avance pelas lições e práticas conforme conclui os pré-requisitos.</p>
            {course && <>
              <StageGrid course={course} open={setStage} selected={stage ?? course.stages[0]?.title} />
              <div className="entries">
                {(course.stages.find((group) => group.title === stage) ?? course.stages[0])?.entries.map((entry) =>
                  <EntryRow key={entry.id} entry={entry} open={openLesson} />)}
              </div>
            </>}
          </>}
          {page === "lesson" && lesson && <>
            <button className="ghost back" onClick={() => { setStage(lesson.stage); navigate("course"); }}>
              <ArrowLeft size={16} aria-hidden="true" /> Voltar à trilha
            </button>
            <LessonPage key={lesson.entry.id} lesson={lesson} fontSize={preferences.font} run={run} running={running}
              report={setNotice} refreshed={refresh} celebrate={celebrate} />
          </>}
          <div hidden={page !== "project"}>
            <ProjectPage report={setNotice} nameChanged={setProjectName} run={run} running={running} fontSize={preferences.font} />
          </div>
          {page === "progress" && <>
            <h1>Seu progresso</h1>
            <p className="lead">{complete} de {entries.length} lições e marcos concluídos · {course?.xp ?? 0} XP</p>
            {course && <StageGrid course={course} open={(title) => { setStage(title); navigate("course"); }} />}
          </>}
          {page === "libraries" && <LibrariesPage report={setNotice} />}
          {page === "settings" && <SettingsPage prefs={preferences} update={setPreferences} report={setNotice} refresh={refresh} run={run} running={running} />}
          {page === "help" && <>
            <h1>Ajuda e sobre</h1>
            <section className="card"><h2>Primeiros passos</h2>
              <p>Abra a Trilha para estudar sem conta e sem conexão. Para compilar no Windows, instale o Rust com Cargo, as ferramentas C++ do Visual Studio e o SDK do Windows.</p>
              <button onClick={() => navigate("settings")}>Verificar ferramentas <ArrowRight size={16} /></button>
            </section>
            <section className="card"><h2>Atalhos</h2>
              <dl className="keys"><dt><kbd>Ctrl</kbd> <kbd>S</kbd></dt><dd>Salvar no editor</dd><dt><kbd>F9</kbd></dt><dd>Abrir bibliotecas</dd></dl>
            </section>
            <section className="card"><h2>RustLab Desktop</h2>
              <p>Prévia 4.0 para Windows · Código-fonte GPL-3.0-only</p>
              <p>Projeto e contribuições em <code>github.com/Honraan/RUSTLAB</code>.</p>
            </section>
          </>}
        </main>
        {task && consoleOpen && <section className="console" aria-label="Saída do Cargo">
          <div className="console-bar"><strong><Terminal size={17} aria-hidden="true" /> {task.label}</strong>
            <div className="top-actions">
              {running && <button onClick={() => void api.cancel().catch((error: unknown) => setNotice(errorMessage(error)))}>Cancelar</button>}
              <button className="icon-button" aria-label="Ocultar saída" onClick={() => setConsoleOpen(false)}><X size={18} /></button>
            </div>
          </div>
          <pre tabIndex={0} aria-label="Log da tarefa">{lines.map((line) => line.text).join("\n")}</pre>
          <div className="console-status" role="status">{task.result
            ? `${task.result.message}${task.result.code !== null ? ` · código ${task.result.code}` : ""}${task.result.awarded ? ` · +${task.result.awarded} XP` : ""}`
            : "Executando Cargo…"}</div>
        </section>}
      </div>
    </div>
  );
}

function StageGrid({ course, open, selected }: { course: Course; open: (stage: string) => void; selected?: string | undefined }) {
  return <section aria-label="Etapas da trilha" className="stage-section">
    {!selected && <h2>Etapas da trilha</h2>}
    <div className="stages">{course.stages.map((stage, index) => {
      const completed = stage.entries.filter((entry) => entry.completed).length;
      return <button className="stage" key={stage.title} aria-pressed={selected ? stage.title === selected : undefined} onClick={() => open(stage.title)}>
        <span className="stage-heading"><strong>{stage.title}</strong><ChevronRight size={16} aria-hidden="true" /></span>
        <span>{stageDetails[index]}</span>
        <progress value={completed} max={stage.entries.length} aria-label={`Progresso em ${stage.title}`} />
        <small>{completed} de {stage.entries.length} concluídas</small>
      </button>;
    })}</div>
  </section>;
}

function EntryRow({ entry, open }: { entry: Entry; open: (id: string) => Promise<void> }) {
  const status = entry.completed ? "Concluída" : entry.unlocked ? "Disponível" : "Bloqueada";
  return <div className="entry">
    <span className="entry-id">{entry.id}</span>
    <div>{entry.unlocked
      ? <button className="text-button" onClick={() => void open(entry.id)}>{entry.title}</button>
      : <strong>{entry.title}</strong>}
      <small>{entry.kind}{!entry.unlocked && entry.prerequisites.length > 0 && ` · Requer ${entry.prerequisites.join(", ")}`}</small>
    </div>
    <span className={entry.completed ? "entry-status done" : "entry-status"}>{status}</span>
  </div>;
}
