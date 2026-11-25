import { useEffect, useState } from "react";
import { Download, Search, Upload } from "lucide-react";
import { api, errorMessage } from "./api";
import type { Doctor, Library, LibraryPlan } from "./api";
import type { Preferences, RunTask } from "./App";

const doctorNames = { cargo: "Cargo", rustc: "rustc", rustup: "rustup", rustfmt: "rustfmt", clippy: "Clippy", analyzer: "rust-analyzer" };

export function SettingsPage({ prefs, update, report, refresh, run, running }: {
  prefs: Preferences;
  update: (prefs: Preferences) => void;
  report: (message: string) => void;
  refresh: () => Promise<void>;
  run: RunTask;
  running: boolean;
}) {
  const [doctor, setDoctor] = useState<Doctor | null>(null);
  const [working, setWorking] = useState(false);
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

  return <>
    <h1>Configurações</h1>
    <div className="settings-grid">
      <section className="card"><h2>Aparência e acessibilidade</h2>
        <label className="setting">Tema <select value={prefs.theme} onChange={(e) => update({ ...prefs, theme: e.target.value as Preferences["theme"] })}>
          <option value="system">Do sistema</option><option value="light">Claro</option><option value="dark">Escuro</option>
        </select></label>
        <label className="setting">Animações <input type="checkbox" checked={prefs.motion} onChange={(e) => update({ ...prefs, motion: e.target.checked })} /></label>
        <label className="setting">Modo foco <input type="checkbox" checked={prefs.focus} onChange={(e) => update({ ...prefs, focus: e.target.checked })} /></label>
        <label className="setting">Volume dos efeitos <input type="range" min="0" max="100" step="5" value={prefs.volume} onChange={(e) => update({ ...prefs, volume: e.target.valueAsNumber })} /> <span>{prefs.volume}%</span></label>
        <label className="setting">Escala da interface <input type="range" min="90" max="150" step="5" value={prefs.scale} onChange={(e) => update({ ...prefs, scale: e.target.valueAsNumber })} /> <span>{prefs.scale}%</span></label>
        <label className="setting">Fonte do editor <input type="range" min="12" max="28" value={prefs.font} onChange={(e) => update({ ...prefs, font: e.target.valueAsNumber })} /> <span>{prefs.font}px</span></label>
      </section>
      <section className="card"><h2>Ferramentas Rust</h2>
        <p>A trilha e os quizzes funcionam offline. Para compilar, instale o Rust com o alvo MSVC, as ferramentas C++ do Visual Studio e o SDK apropriado do Windows.</p>
        <p>Guias oficiais <code>rust-lang.org/install</code> e <code>rust-lang.github.io/rustup/installation/windows-msvc.html</code>.</p>
        <div className="toolbar">
          <button disabled={working} onClick={() => void act(async () => setDoctor(await api.doctor()))}>Detectar executáveis</button>
          <button disabled={working || running} onClick={() => {
            if (window.confirm("Autorizar a compilação e execução offline de um Hello World em pasta temporária?")) {
              void act(() => run("Teste de Cargo e linker MSVC", api.toolchainTest));
            }
          }}>Compilar Hello World</button>
        </div>
        {doctor && <dl className="doctor">
          {Object.entries(doctor).map(([key, path]) => <div key={key}><dt>{doctorNames[key as keyof Doctor]}</dt><dd>{path ?? "Ausente"}</dd></div>)}
        </dl>}
      </section>
      <section className="card"><h2>Dados de aprendizagem</h2>
        <p>Exporte o progresso e os rascunhos salvos em TOML. A importação substitui o progresso local e guarda um backup independente.</p>
        <div className="toolbar">
          <button disabled={working || running} onClick={() => {
            if (window.confirm("Substituir o progresso local salvo pela cópia selecionada? Rascunhos ainda não salvos continuam neste dispositivo.")) {
              void act(async () => {
                const result = await api.import();
                if (!result) return;
                report(result);
                await refresh();
              });
            }
          }}><Upload size={16} aria-hidden="true" /> Importar TOML</button>
          <button disabled={working} onClick={() => void act(async () => {
            if (await api.export()) report("Progresso exportado.");
          })}><Download size={16} aria-hidden="true" /> Exportar TOML</button>
        </div>
      </section>
    </div>
  </>;
}

export function LibrariesPage({ report, run, running, project }: {
  report: (message: string) => void;
  run: RunTask;
  running: boolean;
  project: string;
}) {
  const [libraries, setLibraries] = useState<Library[]>([]);
  const [search, setSearch] = useState("");
  const [selection, setSelection] = useState<{ library: Library; plan: LibraryPlan } | null>(null);
  const [download, setDownload] = useState(false);
  const [consent, setConsent] = useState(false);
  const [working, setWorking] = useState(false);
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
  useEffect(() => {
    void api.libraries().then(setLibraries).catch((error: unknown) => report(errorMessage(error)));
  }, [report]);
  const term = search.toLocaleLowerCase("pt-BR").trim();
  return <>
    <h1>Bibliotecas</h1>
    <p className="lead">Explore o catálogo F9 da trilha. As versões exibidas vêm da edição Classic.</p>
    {project
      ? <div className="toolbar"><p>Projeto ativo: <b>{project}</b></p>
        <button disabled={working || running} onClick={() => {
          if (window.confirm("Desfazer a última alteração F9 em Cargo.toml? O Cargo.lock pode ter sido atualizado pela verificação.")) {
            void act(async () => { await api.libraryUndo(); setSelection(null); report("Alteração F9 desfeita em Cargo.toml."); });
          }
        }}>Desfazer alteração F9</button>
      </div>
      : <p>Abra um projeto no Modo Criar para preparar uma alteração no manifesto.</p>}
    {selection && <section className="card">
      <h2>{selection.library.name} no projeto {project}</h2>
      {selection.plan.present ? <p>Dependência já declarada no manifesto.</p> : <>
        <details open><summary>Alteração proposta em Cargo.toml</summary>
          <div className="library-grid">
            <figure><figcaption>Atual</figcaption><pre tabIndex={0}>{selection.plan.before}</pre></figure>
            <figure><figcaption>Proposto</figcaption><pre tabIndex={0}>{selection.plan.after}</pre></figure>
          </div>
        </details>
        <label className="consent"><input type="checkbox" checked={consent} onChange={(e) => setConsent(e.target.checked)} />
          Autorizo alterar Cargo.toml e verificar o projeto. Scripts de compilação e macros podem executar código com as permissões da minha conta.</label>
        <label className="consent"><input type="checkbox" checked={download} onChange={(e) => setDownload(e.target.checked)} />
          Permitir downloads de crates durante a verificação.</label>
        <button className="primary" disabled={!consent || working || running} onClick={() => void act(async () => {
          await run("Verificar dependência", () => api.libraryApply(selection.library.name, selection.plan, download));
          setSelection(null);
        })}>Aplicar e verificar</button>
      </>}
    </section>}
    <label className="search"><Search size={18} aria-hidden="true" /><input placeholder="Buscar bibliotecas" value={search}
      onChange={(e) => setSearch(e.target.value)} /></label>
    <div className="library-grid">
      {libraries.filter((lib) => `${lib.name} ${lib.category}`.toLocaleLowerCase("pt-BR").includes(term)).map((lib) => (
        <article className="card" key={lib.name}>
          <h2>{lib.name} <small>{lib.version}</small></h2>
          <p><strong>{lib.category}</strong></p><p>{lib.summary}</p>
          <p>{lib.warnings}</p><p>Trilha · {lib.lessons.join(", ")}</p>
          <p className="project-root">{lib.docs}</p>
          <p>Compatibilidade Windows a verificar.</p>
          <button disabled={!project || working || running} onClick={() => void act(async () => {
            const plan = await api.libraryPreview(lib.name);
            setSelection({ library: lib, plan });
            setDownload(false);
            setConsent(false);
            window.requestAnimationFrame(() => document.querySelector("main")?.scrollTo({ top: 0 }));
          })}>Ver alteração no manifesto</button>
        </article>
      ))}
    </div>
  </>;
}
