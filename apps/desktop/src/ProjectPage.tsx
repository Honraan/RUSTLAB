import { lazy, Suspense, useEffect, useEffectEvent, useState } from "react";
import { FileCode2, FolderOpen, Plus, RefreshCw, Save } from "lucide-react";
import { api, errorMessage } from "./api";
import type { Action, Jump, Project, Template } from "./api";
import type { RunTask } from "./App";
import { Editor } from "./Editor";

const actions: { action: Action; title: string }[] = [
  { action: "check", title: "Verificar" }, { action: "run", title: "Executar" },
  { action: "test", title: "Testar" }, { action: "fmt", title: "Formatar" },
  { action: "clippy", title: "Clippy" }, { action: "gui", title: "Executar em janela" },
];
const TerminalPanel = lazy(() => import("./TerminalPanel"));

type Buffer = { base: string; text: string };
const recoveryKey = (root: string, file: string) => `rustlab-project-draft:${JSON.stringify([root, file])}`;

export function ProjectPage({ report, nameChanged, run, running, fontSize, jump }: {
  report: (message: string) => void;
  nameChanged: (project: Project) => void;
  run: RunTask;
  running: boolean;
  fontSize: number;
  jump: Jump | null;
}) {
  const [project, setProject] = useState<Project | null>(null);
  const [buffers, setBuffers] = useState<Record<string, Buffer>>({});
  const [editorJump, setEditorJump] = useState<Jump | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [newProject, setNewProject] = useState("");
  const [template, setTemplate] = useState<Template>("basic");
  const [newFile, setNewFile] = useState("");
  const [working, setWorking] = useState(false);
  const [consent, setConsent] = useState(false);
  const [download, setDownload] = useState(false);
  const [terminalOpen, setTerminalOpen] = useState(false);
  const [terminalRunning, setTerminalRunning] = useState(false);
  const busy = running || terminalRunning;
  const active = selected === null ? null : buffers[selected];
  const dirty = Object.values(buffers).some((buffer) => buffer.text !== buffer.base);

  async function act(action: () => Promise<void>) {
    if (working) return;
    setWorking(true);
    try {
      await action();
    } catch (error) {
      report(errorMessage(error));
    } finally {
      setWorking(false);
    }
  }

  async function select(file: string, mode: "open" | "reload" | "switch" = "open", root = project?.root) {
    if (mode !== "open" || !buffers[file]) {
      const text = await api.read(file);
      let buffer = { base: text, text };
      if (root) {
        const key = recoveryKey(root, file);
        if (mode === "reload") {
          localStorage.removeItem(key);
        } else {
          try {
            const recovery = JSON.parse(localStorage.getItem(key) ?? "null") as Buffer | null;
            if (recovery && typeof recovery.base === "string" && typeof recovery.text === "string") {
              if (recovery.text !== text) {
                buffer = recovery;
                report(recovery.base === text ? "Arquivo restaurado de um rascunho local." : "Rascunho restaurado. O arquivo original mudou no disco.");
              } else {
                localStorage.removeItem(key);
              }
            }
          } catch {
            report("Não foi possível ler o rascunho local do arquivo.");
          }
        }
      }
      setBuffers((existing) => ({ ...existing, [file]: buffer }));
    }
    setEditorJump(null);
    setSelected(file);
  }

  function edit(file: string, text: string, buffer: Buffer) {
    const next = { ...buffer, text };
    setBuffers((existing) => ({ ...existing, [file]: next }));
    if (project) {
      try {
        const key = recoveryKey(project.root, file);
        if (next.base === next.text) localStorage.removeItem(key);
        else localStorage.setItem(key, JSON.stringify(next));
      } catch {
        report("A recuperação do arquivo não pôde ser atualizada. Salve o arquivo antes de sair.");
      }
    }
  }

  async function changeProject(pick: () => Promise<Project | null>) {
    if (dirty && !window.confirm("Há arquivos com alterações. Deseja fechar os buffers e trocar de projeto? Rascunhos recuperáveis continuam neste dispositivo.")) return;
    const next = await pick();
    if (!next) return;
    setProject(next);
    nameChanged(next);
    setSelected(null);
    setBuffers({});
    setConsent(false);
    setDownload(false);
    const first = next.files.includes("src/main.rs") ? "src/main.rs" : next.files[0];
    if (first) await select(first, "switch", next.root);
  }

  const openMarker = useEffectEvent(async (marker: Jump) => {
    if (!project) return;
    const root = project.root.replaceAll("\\", "/").replace(/^\/\/\?\//, "").replace(/\/$/, "");
    const path = marker.file.replace(/^\/\/\?\//, "");
    const relative = path.toLowerCase().startsWith(`${root.toLowerCase()}/`) ? path.slice(root.length + 1) : path;
    const file = project.files.find((known) => known.toLowerCase() === relative.toLowerCase());
    if (!file) {
      report("O diagnóstico aponta para um arquivo fora das fontes editáveis deste projeto.");
      return;
    }
    try {
      await select(file);
      setEditorJump(marker);
    } catch (error) {
      report(errorMessage(error));
    }
  });

  useEffect(() => {
    if (jump) void openMarker(jump);
  }, [jump]);

  const save = () => act(async () => {
    if (!selected || !active || !project || busy) return;
    const { base, text } = active;
    await api.save(selected, base, text);
    setBuffers((existing) => ({ ...existing, [selected]: { base: text, text: existing[selected]?.text ?? text } }));
    const key = recoveryKey(project.root, selected);
    try {
      const recovery = JSON.parse(localStorage.getItem(key) ?? "null") as Buffer | null;
      if (recovery?.text === text) localStorage.removeItem(key);
      else if (recovery) localStorage.setItem(key, JSON.stringify({ base: text, text: recovery.text }));
    } catch {
      report("Arquivo salvo. Não foi possível atualizar o rascunho local.");
      return;
    }
    report("Arquivo salvo.");
  });

  return <>
    <h1>Modo Criar</h1>
    <p className="lead">Trabalhe com seus próprios projetos Cargo.</p>
    <section className="card project-pickers">
      <button disabled={working || busy} onClick={() => void act(() => changeProject(api.openProject))}>
        <FolderOpen size={17} aria-hidden="true" /> Abrir projeto Cargo
      </button>
      <form className="toolbar" onSubmit={(event) => {
        event.preventDefault();
        void act(() => changeProject(() => api.createProject(newProject, template)));
      }}>
        <input aria-label="Nome do novo projeto" placeholder="Nome do novo projeto" value={newProject}
          maxLength={64} onChange={(event) => setNewProject(event.target.value)} required />
        <select aria-label="Modelo de projeto" value={template} onChange={(event) => setTemplate(event.target.value as Template)}>
          <option value="basic">Rust básico</option><option value="paint">Mini Paint · minifb</option><option value="cavegame">CaveGame · Bevy</option>
        </select>
        <button type="submit" disabled={working || busy}><Plus size={17} aria-hidden="true" /> Criar projeto</button>
      </form>
    </section>
    {project && <section className="card">
      <h2>{project.name}</h2>
      <p className="project-root">{project.root}</p>
      <div className="workspace">
        <div className="file-tree" aria-label="Arquivos do projeto">
          {project.files.map((file) => <button key={file} aria-pressed={selected === file} disabled={working}
            onClick={() => void act(() => select(file))}>
            <FileCode2 size={16} aria-hidden="true" />{file}{buffers[file] && buffers[file].text !== buffers[file].base && " •"}
          </button>)}
        </div>
        <div className="file-work">
          {Object.keys(buffers).length > 1 && <div className="file-tabs toolbar" aria-label="Arquivos abertos">
            {Object.keys(buffers).map((file) => <button aria-pressed={file === selected} key={file}
              onClick={() => void act(() => select(file))}>{file.split("/").at(-1)}</button>)}
          </div>}
          {selected && active && <>
            <h3>{selected}{active.text !== active.base && " · Alterado"}</h3>
            <Editor key={selected} label={selected} value={active.text} language={selected.endsWith(".rs") ? "rust" : "text"} fontSize={fontSize} jump={editorJump}
              onChange={(text) => edit(selected, text, active)}
              onSave={() => void save()} />
            <div className="toolbar">
              <button disabled={working || busy || active.text === active.base} onClick={() => void save()}><Save size={16} aria-hidden="true" /> Salvar arquivo</button>
              <button disabled={working || busy} onClick={() => {
                if (active.text === active.base || window.confirm("Recarregar o arquivo e descartar sua edição aberta?")) {
                  void act(() => select(selected, "reload"));
                }
              }}><RefreshCw size={16} aria-hidden="true" /> Recarregar</button>
            </div>
          </>}
        </div>
      </div>
      <form className="toolbar" onSubmit={(event) => {
        event.preventDefault();
        void act(async () => {
          setProject(await api.newFile(newFile));
          await select(newFile.replaceAll("\\", "/"), "switch");
          setNewFile("");
        });
      }}>
        <input aria-label="Caminho do novo arquivo" placeholder="src/modulo.rs" value={newFile} maxLength={240} onChange={(e) => setNewFile(e.target.value)} required />
        <button type="submit" disabled={working || busy}><Plus size={16} aria-hidden="true" /> Criar arquivo</button>
      </form>
      <div className="task-settings">
        <label className="consent"><input type="checkbox" checked={consent} onChange={(e) => setConsent(e.target.checked)} />
          Confio neste projeto e autorizo tarefas do Cargo. O código, scripts de compilação e macros podem executar com as permissões da minha conta.</label>
        <label className="consent"><input type="checkbox" checked={download} onChange={(e) => setDownload(e.target.checked)} />
          Permitir que Cargo baixe dependências para este projeto.</label>
        {dirty && <p>Salve os arquivos alterados antes de iniciar uma tarefa.</p>}
        <div className="toolbar">
          {actions.map(({ action, title }) => <button key={action} disabled={working || busy || dirty || !consent} onClick={() => {
            if (action === "fmt" && !window.confirm("Autorizar cargo fmt a reformatar os arquivos no disco?")) return;
            void act(() => run(action === "gui" ? "cargo run · janela" : `cargo ${action}`, () => api.projectTask(action, download)));
          }}>{title}</button>)}
          <button onClick={() => setTerminalOpen((open) => !open)} aria-expanded={terminalOpen}>
            {terminalOpen ? "Ocultar terminal" : "Terminal interativo"}
          </button>
        </div>
      </div>
      {(terminalOpen || terminalRunning) && <div hidden={!terminalOpen}>
        <Suspense fallback={<p>Carregando terminal…</p>}>
          <TerminalPanel allowed={consent && !dirty && !running && !working} download={download}
            running={terminalRunning} onRunning={setTerminalRunning} report={report} fontSize={fontSize} />
        </Suspense>
      </div>}
    </section>}
  </>;
}
