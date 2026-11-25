import React, { useEffect, useState } from 'react';
import {
  AlertTriangle,
  Bot,
  FilePlus2,
  FolderOpen,
  FolderPlus,
  Hammer,
  Library,
  MonitorPlay,
  Play,
  Save,
  Search,
  ShieldCheck,
  Sparkles,
  Square,
  TestTube2,
  Trash2,
} from 'lucide-react';
import { formatIpcError, ipc, isConsentError } from '../lib/ipc';
import { SafeAnsiOutput } from '../lib/safeRender';
import type {
  DiagnosticDto,
  LspInspectDto,
  MentorAskRequest,
  ProjectInfoDto,
  SearchHitDto,
  TaskHistoryEntry,
  TemplateDto,
} from '../lib/types';
import { CodeEditor } from './CodeEditor';
import { InteractiveTerminal } from './InteractiveTerminal';

interface OpenTab {
  relPath: string;
  content: string;
  savedContent: string;
  sha256: string;
}

interface CreateViewProps {
  isDarkTheme: boolean;
  onOpenLibraries: (rootDir: string | null) => void;
  onOpenMentor: (req: Omit<MentorAskRequest, 'action' | 'question'>) => void;
  onConsentRequired: () => void;
}

export function CreateView({
  isDarkTheme,
  onOpenLibraries,
  onOpenMentor,
  onConsentRequired,
}: CreateViewProps): React.ReactElement {
  const [templates, setTemplates] = useState<TemplateDto[]>([]);
  const [recentProjects, setRecentProjects] = useState<string[]>([]);
  const [defaultParent, setDefaultParent] = useState('');
  const [newName, setNewName] = useState('meu_projeto_rust');
  const [newTemplate, setNewTemplate] = useState('hello');

  const [project, setProject] = useState<ProjectInfoDto | null>(null);
  const [tabs, setTabs] = useState<OpenTab[]>([]);
  const [activeRel, setActiveRel] = useState<string | null>(null);
  const [newFileRel, setNewFileRel] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [searchHits, setSearchHits] = useState<SearchHitDto[]>([]);

  const [bottomTab, setBottomTab] = useState<'output' | 'conpty' | 'history'>('output');
  const [running, setRunning] = useState(false);
  const [outputLines, setOutputLines] = useState<string[]>([]);
  const [diagnostics, setDiagnostics] = useState<DiagnosticDto[]>([]);
  const [lsp, setLsp] = useState<LspInspectDto | null>(null);
  const [taskHistory, setTaskHistory] = useState<TaskHistoryEntry[]>([]);
  const [conflictBanner, setConflictBanner] = useState<{
    relPath: string;
    diskContent: string;
    diskSha: string;
  } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [infoMsg, setInfoMsg] = useState<string | null>(null);

  const activeTab = tabs.find((t) => t.relPath === activeRel) ?? null;

  const refreshHub = async () => {
    try {
      const [tpls, dir, rec] = await Promise.all([
        ipc.listProjectTemplates(),
        ipc.getDefaultProjectsDir(),
        ipc.getRecentProjects(),
      ]);
      setTemplates(tpls);
      setDefaultParent(dir);
      setRecentProjects(rec);
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  useEffect(() => {
    void refreshHub();
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void ipc
      .listenWorkspaceEvents((ev) => {
        setOutputLines((prev) => [...prev.slice(-300), `[${ev.kind}] ${ev.text}`]);
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => {
      unlisten?.();
    };
  }, []);

  const openFileInTab = async (projRoot: string, relPath: string) => {
    const existing = tabs.find((t) => t.relPath === relPath);
    if (existing) {
      setActiveRel(relPath);
      return;
    }
    try {
      const file = await ipc.readProjectFile(projRoot, relPath);
      setTabs((prev) => [
        ...prev,
        { relPath: file.relPath, content: file.content, savedContent: file.content, sha256: file.sha256 },
      ]);
      setActiveRel(file.relPath);
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const handleOpenProjectInfo = async (info: ProjectInfoDto) => {
    setProject(info);
    setTabs([]);
    setActiveRel(null);
    setOutputLines([]);
    setDiagnostics([]);
    setConflictBanner(null);
    setError(null);
    const preferred = info.files.find((f) => f === 'src/main.rs') ?? info.files[0];
    if (preferred) {
      await openFileInTab(info.rootDir, preferred);
    }
  };

  const handleCreateProject = async () => {
    setError(null);
    try {
      const created = await ipc.createProject(defaultParent, newName.trim(), newTemplate);
      await handleOpenProjectInfo(created);
      await refreshHub();
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const handleOpenExisting = async (dirPath?: string) => {
    setError(null);
    try {
      const target = dirPath ?? (await ipc.pickDirectory(defaultParent || undefined));
      if (!target) return;
      const opened = await ipc.openProject(target);
      await handleOpenProjectInfo(opened);
      await refreshHub();
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const handleTrustProject = async () => {
    if (!project) return;
    try {
      await ipc.trustProject(project.rootDir);
      const refreshed = await ipc.openProject(project.rootDir);
      setProject(refreshed);
      setInfoMsg('Projeto marcado como confiável neste Windows.');
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const handleSave = async (force = false) => {
    if (!project || !activeTab) return;
    setError(null);
    try {
      const res = await ipc.saveProjectFile(
        project.rootDir,
        activeTab.relPath,
        activeTab.content,
        activeTab.sha256,
        force,
      );
      if (res.conflict && res.diskContent != null) {
        setConflictBanner({
          relPath: activeTab.relPath,
          diskContent: res.diskContent,
          diskSha: res.diskSha256,
        });
        return;
      }
      setConflictBanner(null);
      setTabs((prev) =>
        prev.map((t) =>
          t.relPath === activeTab.relPath
            ? { ...t, savedContent: activeTab.content, sha256: res.diskSha256 }
            : t,
        ),
      );
      setInfoMsg(`Salvo: ${activeTab.relPath}`);
      if (activeTab.relPath.endsWith('.rs')) {
        const inspected = await ipc
          .lspInspectFile(project.rootDir, activeTab.relPath, activeTab.content, null)
          .catch(() => null);
        if (inspected) setLsp(inspected);
      }
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const runCargo = async (action: string) => {
    if (!project) return;
    setRunning(true);
    setError(null);
    setInfoMsg(null);
    setOutputLines([]);
    setBottomTab('output');
    try {
      if (activeTab && activeTab.content !== activeTab.savedContent) {
        await handleSave(false);
      }
      const res = await ipc.runWorkspaceCargo(project.rootDir, action);
      setOutputLines(res.outputLines);
      setDiagnostics(res.diagnostics);
      const hist = await ipc.getTaskHistory().catch(() => []);
      setTaskHistory(hist);
      if (action === 'fmt' && activeTab) {
        const reloaded = await ipc.readProjectFile(project.rootDir, activeTab.relPath);
        setTabs((prev) =>
          prev.map((t) =>
            t.relPath === reloaded.relPath
              ? { ...t, content: reloaded.content, savedContent: reloaded.content, sha256: reloaded.sha256 }
              : t,
          ),
        );
      }
    } catch (err) {
      if (isConsentError(err)) {
        onConsentRequired();
      } else {
        setError(formatIpcError(err));
      }
    } finally {
      setRunning(false);
    }
  };

  const handleLaunchGui = async () => {
    if (!project) return;
    setRunning(true);
    setError(null);
    try {
      if (activeTab && activeTab.content !== activeTab.savedContent) {
        await handleSave(false);
      }
      const msg = await ipc.launchGuiApp(project.rootDir);
      setInfoMsg(msg);
    } catch (err) {
      if (isConsentError(err)) {
        onConsentRequired();
      } else {
        setError(formatIpcError(err));
      }
    } finally {
      setRunning(false);
    }
  };

  const handleCreateFile = async () => {
    if (!project || !newFileRel.trim()) return;
    try {
      const created = await ipc.createProjectFile(project.rootDir, newFileRel.trim());
      setNewFileRel('');
      const refreshed = await ipc.openProject(project.rootDir);
      setProject(refreshed);
      await openFileInTab(project.rootDir, created);
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const handleDeleteFile = async (rel: string) => {
    if (!project || rel === 'Cargo.toml') return;
    try {
      await ipc.deleteProjectFile(project.rootDir, rel);
      setTabs((prev) => prev.filter((t) => t.relPath !== rel));
      if (activeRel === rel) setActiveRel(null);
      const refreshed = await ipc.openProject(project.rootDir);
      setProject(refreshed);
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const handleSearch = async () => {
    if (!project || !searchQuery.trim()) return;
    try {
      const hits = await ipc.searchInProject(project.rootDir, searchQuery.trim());
      setSearchHits(hits);
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  if (!project) {
    return (
      <div className="stack" style={{ gap: 16 }}>
        <div className="card stack">
          <h1 style={{ margin: 0, fontSize: '1.4rem' }}>Modo Criar — Projetos Cargo Reais</h1>
          <p className="muted" style={{ margin: 0 }}>
            Crie projetos com modelos do curso (Console, Pong, Snake, Editor, Mini Paint GUI ou
            CaveGame Bevy 3D) ou abra qualquer pasta Cargo existente no Windows.
          </p>
        </div>

        {error ? <div className="banner banner-danger">{error}</div> : null}

        <div className="grid-2">
          <div className="card stack">
            <div className="row">
              <FolderPlus size={18} color="var(--accent)" />
              <h2 style={{ margin: 0, fontSize: '1.1rem' }}>Novo Projeto</h2>
            </div>

            <div>
              <label className="field-label" htmlFor="proj-name">Nome da crate (letras minúsculas, dígitos ou `_`)</label>
              <input
                id="proj-name"
                className="input"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
              />
            </div>

            <div>
              <label className="field-label" htmlFor="proj-dir">Pasta pai</label>
              <div className="row">
                <input
                  id="proj-dir"
                  className="input"
                  style={{ flex: 1 }}
                  value={defaultParent}
                  onChange={(e) => setDefaultParent(e.target.value)}
                />
                <button
                  type="button"
                  className="btn btn-sm"
                  onClick={() =>
                    void ipc.pickDirectory(defaultParent || undefined).then((d) => {
                      if (d) setDefaultParent(d);
                    })
                  }
                >
                  Escolher...
                </button>
              </div>
            </div>

            <div>
              <label className="field-label" htmlFor="proj-tpl">Template inicial</label>
              <select
                id="proj-tpl"
                className="input"
                value={newTemplate}
                onChange={(e) => setNewTemplate(e.target.value)}
              >
                {templates.map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.name}
                  </option>
                ))}
              </select>
              <div className="muted" style={{ fontSize: '0.84rem', marginTop: 4 }}>
                {templates.find((t) => t.id === newTemplate)?.description ?? ''}
              </div>
            </div>

            <button type="button" className="btn btn-primary" onClick={() => void handleCreateProject()}>
              <FolderPlus size={15} /> Criar e Abrir no Mini-IDE
            </button>
          </div>

          <div className="card stack">
            <div className="row">
              <FolderOpen size={18} color="var(--accent)" />
              <h2 style={{ margin: 0, fontSize: '1.1rem' }}>Abrir Projeto Existente</h2>
            </div>
            <button type="button" className="btn" onClick={() => void handleOpenExisting()}>
              <FolderOpen size={15} /> Procurar Pasta no Windows...
            </button>

            <strong style={{ marginTop: 6 }}>Projetos recentes</strong>
            {recentProjects.length === 0 ? (
              <span className="muted">Nenhum projeto aberto ainda.</span>
            ) : (
              <div className="stack">
                {recentProjects.map((dir) => (
                  <button
                    key={dir}
                    type="button"
                    className="btn"
                    style={{ justifyContent: 'flex-start' }}
                    onClick={() => void handleOpenExisting(dir)}
                  >
                    {dir}
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="stack" style={{ gap: 10 }}>
      <div className="card between" style={{ padding: '10px 14px' }}>
        <div>
          <strong>{project.name}</strong>{' '}
          <span className="muted mono" style={{ fontSize: '0.82rem' }}>{project.rootDir}</span>
          <div className="row" style={{ marginTop: 4 }}>
            <span className="pill">{project.capabilities.sourceFileCount} arquivo(s)</span>
            {project.capabilities.hasUnitTests ? <span className="pill pill-ok">#[test]</span> : null}
            {project.capabilities.hasErrorHandling ? <span className="pill pill-info">Result</span> : null}
            {lsp?.lspActive ? <span className="pill pill-ok">rust-analyzer ativo</span> : null}
          </div>
        </div>
        <div className="row">
          <button type="button" className="btn btn-sm" onClick={() => setProject(null)}>
            Trocar projeto
          </button>
          <button type="button" className="btn btn-sm" onClick={() => onOpenLibraries(project.rootDir)}>
            <Library size={14} /> Bibliotecas (F9)
          </button>
          <button
            type="button"
            className="btn btn-sm"
            onClick={() =>
              onOpenMentor({
                learnMode: false,
                origin: `Projeto ${project.name}`,
                topicTitle: activeTab?.relPath ?? 'src/main.rs',
                summary: `Pasta: ${project.rootDir}`,
                code: activeTab?.content ?? '',
                errors: diagnostics.map((d) => d.message).join('\n'),
                console: outputLines.join('\n'),
              })
            }
          >
            <Bot size={14} /> Mentor IA
          </button>
        </div>
      </div>

      {!project.trusted ? (
        <div className="banner banner-warn">
          <div>
            <strong>Projeto externo ainda não confiável:</strong> executar Cargo,{' '}
            <code>build.rs</code> ou <code>rust-analyzer</code> roda código nativo na sua conta
            Windows.
          </div>
          <button type="button" className="btn btn-primary btn-sm" onClick={() => void handleTrustProject()}>
            <ShieldCheck size={14} /> Confiar neste projeto
          </button>
        </div>
      ) : null}

      {error ? <div className="banner banner-danger">{error}</div> : null}
      {infoMsg ? <div className="banner banner-ok">{infoMsg}</div> : null}

      {conflictBanner ? (
        <div className="banner banner-warn">
          <span>
            O arquivo <code>{conflictBanner.relPath}</code> foi alterado no disco fora do RustLab.
          </span>
          <div className="row">
            <button type="button" className="btn btn-sm btn-danger" onClick={() => void handleSave(true)}>
              Sobrescrever disco
            </button>
            <button
              type="button"
              className="btn btn-sm"
              onClick={() => {
                setTabs((prev) =>
                  prev.map((t) =>
                    t.relPath === conflictBanner.relPath
                      ? {
                          ...t,
                          content: conflictBanner.diskContent,
                          savedContent: conflictBanner.diskContent,
                          sha256: conflictBanner.diskSha,
                        }
                      : t,
                  ),
                );
                setConflictBanner(null);
              }}
            >
              Recarregar do disco
            </button>
          </div>
        </div>
      ) : null}

      <div className="ide-grid">
        <aside className="card stack" style={{ overflowY: 'auto', padding: 10 }}>
          <div className="between">
            <strong>Arquivos</strong>
            {!project.capabilities.hasPlanMd ? (
              <button
                type="button"
                className="btn btn-sm"
                onClick={() =>
                  void ipc.createProjectPlanMd(project.rootDir).then(async (rel) => {
                    const refreshed = await ipc.openProject(project.rootDir);
                    setProject(refreshed);
                    await openFileInTab(project.rootDir, rel);
                  })
                }
              >
                + PLAN.md
              </button>
            ) : null}
          </div>

          <div className="row">
            <input
              className="input"
              placeholder="src/novo.rs"
              aria-label="Novo arquivo relativo"
              value={newFileRel}
              onChange={(e) => setNewFileRel(e.target.value)}
            />
            <button type="button" className="btn btn-sm" onClick={() => void handleCreateFile()}>
              <FilePlus2 size={14} />
            </button>
          </div>

          <div className="stack" style={{ gap: 4 }}>
            {project.files.map((rel) => (
              <div key={rel} className="between" style={{ gap: 4 }}>
                <button
                  type="button"
                  className={`nav-item ${activeRel === rel ? 'btn-primary' : ''}`}
                  style={{ padding: '5px 8px', fontSize: '0.86rem' }}
                  onClick={() => void openFileInTab(project.rootDir, rel)}
                >
                  {rel}
                </button>
                {rel !== 'Cargo.toml' ? (
                  <button
                    type="button"
                    className="btn btn-sm"
                    aria-label={`Excluir ${rel}`}
                    onClick={() => void handleDeleteFile(rel)}
                  >
                    <Trash2 size={12} />
                  </button>
                ) : null}
              </div>
            ))}
          </div>

          <hr style={{ width: '100%', borderColor: 'var(--border-subtle)' }} />

          <div className="stack" style={{ gap: 6 }}>
            <strong>Buscar no projeto</strong>
            <div className="row">
              <input
                className="input"
                placeholder="Texto..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
              <button type="button" className="btn btn-sm" onClick={() => void handleSearch()}>
                <Search size={14} />
              </button>
            </div>
            {searchHits.slice(0, 8).map((h, i) => (
              <button
                key={i}
                type="button"
                className="btn btn-sm"
                style={{ justifyContent: 'flex-start', textAlign: 'left' }}
                onClick={() => void openFileInTab(project.rootDir, h.relPath)}
              >
                {h.relPath}:{h.line} — {h.preview.slice(0, 32)}
              </button>
            ))}
          </div>
        </aside>

        <div className="ide-center">
          <div className="card stack" style={{ padding: '8px 10px', gap: 8 }}>
            <div className="row">
              {tabs.map((t) => {
                const dirty = t.content !== t.savedContent;
                return (
                  <button
                    key={t.relPath}
                    type="button"
                    className={`btn btn-sm ${activeRel === t.relPath ? 'btn-primary' : ''}`}
                    onClick={() => setActiveRel(t.relPath)}
                  >
                    {t.relPath}
                    {dirty ? ' *' : ''}
                  </button>
                );
              })}
            </div>

            <div className="row">
              <button type="button" className="btn btn-sm" onClick={() => void handleSave(false)}>
                <Save size={14} /> Salvar
              </button>
              <button type="button" className="btn btn-sm" disabled={running} onClick={() => void runCargo('check')}>
                Check
              </button>
              <button type="button" className="btn btn-sm" disabled={running} onClick={() => void runCargo('build')}>
                <Hammer size={14} /> Build
              </button>
              <button type="button" className="btn btn-sm" disabled={running} onClick={() => void runCargo('test')}>
                <TestTube2 size={14} /> Test (F6)
              </button>
              <button type="button" className="btn btn-sm" disabled={running} onClick={() => void runCargo('fmt')}>
                Fmt (F7)
              </button>
              <button type="button" className="btn btn-sm" disabled={running} onClick={() => void runCargo('clippy')}>
                <Sparkles size={14} /> Clippy (F8)
              </button>
              <button
                type="button"
                className="btn btn-sm btn-primary"
                disabled={running}
                onClick={() => void runCargo('run')}
              >
                <Play size={14} /> Run (F5)
              </button>
              <button
                type="button"
                className="btn btn-sm"
                disabled={running}
                onClick={() => void handleLaunchGui()}
              >
                <MonitorPlay size={14} /> Abrir GUI (F4)
              </button>
              {running ? (
                <button
                  type="button"
                  className="btn btn-sm btn-danger"
                  onClick={() => void ipc.cancelActiveExecution()}
                >
                  <Square size={14} /> Cancelar
                </button>
              ) : null}
            </div>
          </div>

          <div style={{ minHeight: 220 }}>
            {activeTab ? (
              <CodeEditor
                value={activeTab.content}
                onChange={(next) =>
                  setTabs((prev) =>
                    prev.map((t) => (t.relPath === activeTab.relPath ? { ...t, content: next } : t)),
                  )
                }
                dark={isDarkTheme}
                diagnostics={diagnostics}
                completions={lsp?.completions ?? []}
                onRunShortcut={() => void runCargo('run')}
                onSaveShortcut={() => void handleSave(false)}
                onLibrariesShortcut={() => onOpenLibraries(project.rootDir)}
              />
            ) : (
              <div className="card">Selecione um arquivo na árvore ao lado.</div>
            )}
          </div>

          <div className="card stack" style={{ padding: 10, minHeight: 0, overflow: 'hidden' }}>
            <div className="row">
              <button
                type="button"
                className={`btn btn-sm ${bottomTab === 'output' ? 'btn-primary' : ''}`}
                onClick={() => setBottomTab('output')}
              >
                Saída & Diagnósticos ({diagnostics.length})
              </button>
              <button
                type="button"
                className={`btn btn-sm ${bottomTab === 'conpty' ? 'btn-primary' : ''}`}
                onClick={() => setBottomTab('conpty')}
              >
                Terminal ConPTY
              </button>
              <button
                type="button"
                className={`btn btn-sm ${bottomTab === 'history' ? 'btn-primary' : ''}`}
                onClick={() => {
                  setBottomTab('history');
                  void ipc.getTaskHistory().then(setTaskHistory).catch(() => {});
                }}
              >
                Histórico ({taskHistory.length})
              </button>
            </div>

            {bottomTab === 'output' ? (
              <div className="stack" style={{ minHeight: 0, overflow: 'auto' }}>
                {diagnostics.length > 0 ? (
                  <div className="row">
                    {diagnostics.slice(0, 6).map((d, idx) => (
                      <button
                        key={idx}
                        type="button"
                        className={`pill ${d.level === 'error' ? 'pill-danger' : 'pill-warn'}`}
                        style={{ cursor: 'pointer' }}
                        onClick={() => {
                          if (d.file && project.files.includes(d.file)) {
                            void openFileInTab(project.rootDir, d.file);
                          }
                        }}
                      >
                        <AlertTriangle size={12} /> {d.file ?? 'cargo'}:{d.line ?? 1} — {d.message.slice(0, 50)}
                      </button>
                    ))}
                  </div>
                ) : null}
                <SafeAnsiOutput lines={outputLines} />
              </div>
            ) : null}

            {bottomTab === 'conpty' ? (
              <InteractiveTerminal
                rootDir={project.rootDir}
                trusted={project.trusted}
                onConsentRequired={onConsentRequired}
              />
            ) : null}

            {bottomTab === 'history' ? (
              <div className="stack" style={{ overflowY: 'auto' }}>
                {taskHistory.length === 0 ? (
                  <span className="muted">Nenhuma tarefa executada nesta sessão.</span>
                ) : (
                  taskHistory.map((h) => (
                    <div key={h.id} className="card-muted between">
                      <span>
                        <code>{h.command}</code> — <span className="muted">{h.cwd}</span>
                      </span>
                      <span className="pill">{h.status}</span>
                    </div>
                  ))
                )}
              </div>
            ) : null}
          </div>
        </div>
      </div>
    </div>
  );
}
