import React, { useEffect, useState } from 'react';
import { ExternalLink, Library, RotateCcw, ShieldCheck, X } from 'lucide-react';
import { formatIpcError, ipc } from '../lib/ipc';
import type { LibraryChangePreviewDto, LibraryItemDto } from '../lib/types';

interface LibrariesDrawerProps {
  open: boolean;
  activeProjectRoot: string | null;
  onClose: () => void;
  onProjectMutated?: () => void;
}

export function LibrariesDrawer({
  open,
  activeProjectRoot,
  onClose,
  onProjectMutated,
}: LibrariesDrawerProps): React.ReactElement | null {
  const [libs, setLibs] = useState<LibraryItemDto[]>([]);
  const [category, setCategory] = useState('Todas');
  const [selectedName, setSelectedName] = useState<string | null>(null);
  const [includeTemplate, setIncludeTemplate] = useState(false);
  const [preview, setPreview] = useState<LibraryChangePreviewDto | null>(null);
  const [lastRevertId, setLastRevertId] = useState<string | null>(null);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = async () => {
    setError(null);
    try {
      const list = await ipc.listLibraries(activeProjectRoot);
      setLibs(list);
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  useEffect(() => {
    if (open) {
      setPreview(null);
      setStatusMsg(null);
      void load();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, activeProjectRoot]);

  if (!open) return null;

  const categories = ['Todas', ...Array.from(new Set(libs.map((l) => l.category)))];
  const visible = libs.filter((l) => category === 'Todas' || l.category === category);

  const handlePreview = async (name: string) => {
    if (!activeProjectRoot) return;
    setError(null);
    setSelectedName(name);
    try {
      const p = await ipc.previewAddLibrary(activeProjectRoot, name, includeTemplate);
      setPreview(p);
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const handleApply = async () => {
    if (!activeProjectRoot || !preview) return;
    setError(null);
    try {
      const res = await ipc.applyAddLibrary(activeProjectRoot, preview.name, includeTemplate);
      setLastRevertId(res.revertId);
      setPreview(null);
      setStatusMsg(`Biblioteca ${preview.name} adicionada ao Cargo.toml e ${res.targetRsRel}.`);
      await load();
      onProjectMutated?.();
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const handleRevert = async () => {
    if (!lastRevertId) return;
    setError(null);
    try {
      await ipc.revertLibraryChange(lastRevertId);
      setLastRevertId(null);
      setStatusMsg('Última adição de biblioteca revertida com segurança.');
      await load();
      onProjectMutated?.();
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  return (
    <div className="drawer-backdrop" role="dialog" aria-modal="true" aria-label="Catálogo F9 de Bibliotecas">
      <div className="drawer-panel stack">
        <div className="between">
          <div className="row">
            <Library size={18} color="var(--accent)" />
            <h2 style={{ margin: 0 }}>Bibliotecas Rust (F9)</h2>
          </div>
          <button type="button" className="btn btn-sm" onClick={onClose} aria-label="Fechar F9">
            <X size={15} /> Fechar
          </button>
        </div>

        <p className="muted" style={{ margin: 0 }}>
          Consulta 100% offline. Adicionar ao projeto exige pré-visualização e seu consentimento.
        </p>

        {lastRevertId ? (
          <div className="banner banner-info">
            <span>Alteração aplicada recentemente.</span>
            <button type="button" className="btn btn-sm" onClick={() => void handleRevert()}>
              <RotateCcw size={14} /> Desfazer última alteração
            </button>
          </div>
        ) : null}

        {statusMsg ? <div className="banner banner-ok">{statusMsg}</div> : null}
        {error ? <div className="banner banner-danger">{error}</div> : null}

        <div className="row">
          {categories.map((cat) => (
            <button
              key={cat}
              type="button"
              className={`btn btn-sm ${category === cat ? 'btn-primary' : ''}`}
              onClick={() => setCategory(cat)}
            >
              {cat}
            </button>
          ))}
        </div>

        {activeProjectRoot ? (
          <label className="row" style={{ cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={includeTemplate}
              onChange={(e) => {
                setIncludeTemplate(e.target.checked);
                setPreview(null);
              }}
            />
            <span>Incluir código-exemplo completo em `src/main.rs` (opcional)</span>
          </label>
        ) : (
          <div className="banner banner-info">
            <span>Abra um projeto no Modo Criar para poder instalar crates no `Cargo.toml`.</span>
          </div>
        )}

        {preview ? (
          <div className="card stack" style={{ borderColor: 'var(--accent)' }}>
            <strong>Prévia de alteração para `{preview.name}` ({preview.version})</strong>
            <span className="muted">{preview.requiresDownloadNote}</span>
            <span className="field-label">Cargo.toml (depois)</span>
            <pre className="code-block">{preview.cargoTomlAfter}</pre>
            <span className="field-label">{preview.targetRsRel} (depois)</span>
            <pre className="code-block">{preview.targetRsAfter}</pre>
            <div className="row">
              <button type="button" className="btn btn-primary" onClick={() => void handleApply()}>
                <ShieldCheck size={15} /> Confirmar e Aplicar
              </button>
              <button type="button" className="btn" onClick={() => setPreview(null)}>
                Cancelar
              </button>
            </div>
          </div>
        ) : null}

        <div className="stack">
          {visible.map((lib) => (
            <div
              key={lib.name}
              className="card-muted stack"
              style={{ borderColor: selectedName === lib.name ? 'var(--accent)' : undefined }}
            >
              <div className="between">
                <div className="row">
                  <strong>{lib.name}</strong>
                  <span className="pill">{lib.version}</span>
                  <span className="pill">{lib.category}</span>
                </div>
                {lib.installedInProject ? <span className="pill pill-ok">No Cargo.toml</span> : null}
              </div>
              <span>{lib.summary}</span>
              <span className="muted" style={{ fontSize: '0.86rem' }}>{lib.docsPt}</span>
              <div className="banner banner-info" style={{ marginBottom: 0 }}>
                <span><strong>Windows:</strong> {lib.windowsNotes}</span>
              </div>
              {lib.versionNote ? (
                <div className="banner banner-warn" style={{ marginBottom: 0 }}>
                  <span>{lib.versionNote}</span>
                </div>
              ) : null}
              <div className="between">
                <button
                  type="button"
                  className="btn btn-sm"
                  onClick={() => void ipc.openSafeUrl(lib.docUrl)}
                >
                  <ExternalLink size={13} /> Docs oficiais
                </button>
                {activeProjectRoot ? (
                  <button
                    type="button"
                    className="btn btn-sm btn-primary"
                    onClick={() => void handlePreview(lib.name)}
                  >
                    Pré-visualizar adição
                  </button>
                ) : null}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
