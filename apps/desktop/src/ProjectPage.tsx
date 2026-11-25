import { useState } from "react";
import { FileCode2, FolderOpen, Plus, RefreshCw, Save } from "lucide-react";
import { api, errorMessage } from "./api";
import type { Action, Project } from "./api";
import type { RunTask } from "./App";
import { Editor } from "./Editor";

const actions: { action: Action; title: string }[] = [
  { action: "check", title: "Verificar" }, { action: "run", title: "Executar" },
  { action: "test", title: "Testar" }, { action: "fmt", title: "Formatar" },
  { action: "clippy", title: "Clippy" },
];
type Buffer = { base: string; text: string };

export function ProjectPage({ report, nameChanged, run, running, fontSize }: {
  report: (message: string) => void;
  nameChanged: (name: string) => void;
  run: RunTask;
  running: boolean;
  fontSize: number;
}) {
  const [project, setProject] = useState<Project | null>(null);
  const [buffers, setBuffers] = useState<Record<string, Buffer>>({});
  const [selected, setSelected] = useState<string | null>(null);
  const [newProject, setNewProject] = useState("");
  const [newFile, setNewFile] = useState("");
  const [working, setWorking] = useState(false);
  const [consent, setConsent] = useState(false);
  const [download, setDownload] = useState(false);
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

  async function select(file: string, refresh = false) {
    if (refresh || !buffers[file]) {
      const text = await api.read(file);
      setBuffers((existing) => ({ ...existing, [file]: { base: text, text } }));
    }
    setSelected(file);
  }

  async function changeProject(pick: () => Promise<Project | null>) {
    if (dirty && !window.confirm("Há arquivos com alterações. Deseja trocar de projeto e descartar esses buffers?")) return;
    const next = await pick();
    if (!next) return;
    setProject(next);
    nameChanged(next.name);
    setSelected(null);
    setBuffers({});
    setConsent(false);
    setDownload(false);
    const first = next.files.includes("src/main.rs") ? "src/main.rs" : next.files[0];
    if (first) await select(first, true);
  }

  const save = () => act(async () => {
    if (!selected || !active || running) return;
    const { base, text } = active;
    await api.save(selected, base, text);
    setBuffers((existing) => ({ ...existing, [selected]: { base: text, text: existing[selected]?.text ?? text } }));
    report("Arquivo salvo.");
  });

  return <>
    <h1>Modo Criar</h1>
    <p className="lead">Trabalhe com seus próprios projetos Cargo.</p>
    <section className="card project-pickers">
      <button disabled={working || running} onClick={() => void act(() => changeProject(api.openProject))}>
        <FolderOpen size={17} aria-hidden="true" /> Abrir projeto Cargo
      </button>
      <form className="toolbar" onSubmit={(event) => {
        event.preventDefault();
        void act(() => changeProject(() => api.createProject(newProject)));
      }}>
        <input aria-label="Nome do novo projeto" placeholder="Nome do novo projeto" value={newProject}
          maxLength={64} onChange={(event) => setNewProject(event.target.value)} required />
        <button type="submit" disabled={working || running}><Plus size={17} aria-hidden="true" /> Criar projeto</button>
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
          {selected && active && <>
            <h3>{selected}{active.text !== active.base && " · Alterado"}</h3>
            <Editor key={selected} label={selected} value={active.text} language={selected.endsWith(".rs") ? "rust" : "text"} fontSize={fontSize}
              onChange={(text) => setBuffers((existing) => ({ ...existing, [selected]: { base: existing[selected]?.base ?? active.base, text } }))}
              onSave={() => void save()} />
            <div className="toolbar">
              <button disabled={working || running || active.text === active.base} onClick={() => void save()}><Save size={16} aria-hidden="true" /> Salvar arquivo</button>
              <button disabled={working || running} onClick={() => {
                if (active.text === active.base || window.confirm("Recarregar o arquivo e descartar sua edição aberta?")) {
                  void act(() => select(selected, true));
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
          await select(newFile.replaceAll("\\", "/"), true);
          setNewFile("");
        });
      }}>
        <input aria-label="Caminho do novo arquivo" placeholder="src/modulo.rs" value={newFile} maxLength={240} onChange={(e) => setNewFile(e.target.value)} required />
        <button type="submit" disabled={working || running}><Plus size={16} aria-hidden="true" /> Criar arquivo</button>
      </form>
      <div className="task-settings">
        <label className="consent"><input type="checkbox" checked={consent} onChange={(e) => setConsent(e.target.checked)} />
          Confio neste projeto e autorizo tarefas do Cargo. O código, scripts de compilação e macros podem executar com as permissões da minha conta.</label>
        <label className="consent"><input type="checkbox" checked={download} onChange={(e) => setDownload(e.target.checked)} />
          Permitir que Cargo baixe dependências para este projeto.</label>
        {dirty && <p>Salve os arquivos alterados antes de iniciar uma tarefa.</p>}
        <div className="toolbar">
          {actions.map(({ action, title }) => <button key={action} disabled={working || running || dirty || !consent} onClick={() => {
            if (action === "fmt" && !window.confirm("Autorizar cargo fmt a reformatar os arquivos no disco?")) return;
            void act(() => run(`cargo ${action}`, () => api.projectTask(action, download)));
          }}>{title}</button>)}
        </div>
      </div>
    </section>}
  </>;
}
