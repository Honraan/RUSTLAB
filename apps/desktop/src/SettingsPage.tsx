import { useEffect, useState } from "react";
import { Download, Search, Upload } from "lucide-react";
import { api, errorMessage } from "./api";
import type { Doctor, Library } from "./api";
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
            if (window.confirm("Substituir o progresso local pela cópia selecionada? Os rascunhos de recuperação do editor serão limpos.")) {
              void act(async () => {
                const result = await api.import();
                if (!result) return;
                for (const key of Object.keys(localStorage)) {
                  if (key.startsWith("rustlab-draft-")) localStorage.removeItem(key);
                }
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

export function LibrariesPage({ report }: { report: (message: string) => void }) {
  const [libraries, setLibraries] = useState<Library[]>([]);
  const [search, setSearch] = useState("");
  useEffect(() => {
    void api.libraries().then(setLibraries).catch((error: unknown) => report(errorMessage(error)));
  }, [report]);
  const term = search.toLocaleLowerCase("pt-BR").trim();
  return <>
    <h1>Bibliotecas</h1>
    <p className="lead">Explore o catálogo F9 da trilha. As versões exibidas vêm da edição Classic.</p>
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
        </article>
      ))}
    </div>
  </>;
}
