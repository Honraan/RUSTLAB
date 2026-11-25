import React, { useState } from 'react';
import { ExternalLink, FileDown, RefreshCw, Stethoscope } from 'lucide-react';
import { formatIpcError, ipc } from '../lib/ipc';
import type { WindowsCapabilityReport } from '../lib/types';

interface HelpAboutViewProps {
  capabilities: WindowsCapabilityReport | null;
  onRefreshCapabilities: () => Promise<void>;
  onOpenDoctor: () => void;
}

export function HelpAboutView({
  capabilities,
  onRefreshCapabilities,
  onOpenDoctor,
}: HelpAboutViewProps): React.ReactElement {
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleExportLogs = async () => {
    setError(null);
    try {
      const dest = await ipc.pickSaveFile('rustlab-diagnostico.log', 'log');
      if (!dest) return;
      await ipc.exportDiagnosticsLog(dest);
      setStatus(`Log redigido (sem chaves/segredos) salvo em ${dest}`);
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  return (
    <div className="stack" style={{ gap: 16 }}>
      <div className="card stack">
        <div className="between">
          <div>
            <h1 style={{ margin: 0, fontSize: '1.4rem' }}>Sobre e Diagnóstico Windows</h1>
            <p className="muted" style={{ margin: '4px 0 0' }}>
              RustLab 4.0.0 — Licença GPL-3.0-only · TUI Classic preservada no binário principal.
            </p>
          </div>
          <div className="row">
            <button type="button" className="btn btn-sm" onClick={() => void onRefreshCapabilities()}>
              <RefreshCw size={14} /> Atualizar
            </button>
            <button type="button" className="btn btn-sm btn-primary" onClick={onOpenDoctor}>
              <Stethoscope size={14} /> Abrir Toolchain Doctor
            </button>
          </div>
        </div>
      </div>

      {status ? <div className="banner banner-ok">{status}</div> : null}
      {error ? <div className="banner banner-danger">{error}</div> : null}

      {capabilities ? (
        <div className="card stack">
          <h2 style={{ margin: 0, fontSize: '1.1rem' }}>Capacidades Detectadas neste Host</h2>
          <div className="grid-2">
            <div className="card-muted stack" style={{ gap: 4 }}>
              <div><strong>Sistema:</strong> {capabilities.osCaption} (Build {capabilities.osBuild}, {capabilities.arch})</div>
              <div><strong>WebView2:</strong> {capabilities.webview2Version ?? 'Não detectado'}</div>
              <div><strong>ConPTY:</strong> {capabilities.conptySupported ? 'Suportado' : 'Indisponível'}</div>
              <div><strong>Currículo 213 entradas:</strong> {capabilities.offlineCurriculumReady ? 'Offline pronto' : 'Erro'}</div>
            </div>
            <div className="card-muted stack" style={{ gap: 4 }}>
              <div><strong>rustc:</strong> {capabilities.rustcVersion ?? 'Ausente'}</div>
              <div><strong>cargo:</strong> {capabilities.cargoVersion ?? 'Ausente'}</div>
              <div><strong>Target:</strong> {capabilities.defaultHostTriple ?? 'N/D'}</div>
              <div>
                <strong>Linkers:</strong> MSVC={capabilities.msvcLinkerAvailable ? 'Sim' : 'Não'} ·
                SDK={capabilities.windowsSdkAvailable ? 'Sim' : 'Não'} ·
                GNU={capabilities.gnuLinkerAvailable ? 'Sim' : 'Não'}
              </div>
              <div>
                <strong>Ferramentas:</strong> rustfmt={capabilities.rustfmtAvailable ? 'Sim' : 'Não'} ·
                clippy={capabilities.clippyAvailable ? 'Sim' : 'Não'} ·
                rust-analyzer={capabilities.rustAnalyzerAvailable ? 'Sim' : 'Não'}
              </div>
            </div>
          </div>
          <div className="banner banner-info" style={{ marginBottom: 0 }}>
            <span>{capabilities.recommendedActionPt}</span>
          </div>
        </div>
      ) : null}

      <div className="grid-2">
        <div className="card stack">
          <h2 style={{ margin: 0, fontSize: '1.1rem' }}>Atalhos de Teclado</h2>
          <ul style={{ margin: 0, paddingLeft: 18 }}>
            <li><code className="inline-code">F5</code> / <code className="inline-code">Ctrl+Enter</code> — Executar / validar prática ou projeto</li>
            <li><code className="inline-code">F1</code> — Revelar próxima dica progressiva</li>
            <li><code className="inline-code">F4</code> — Abrir projeto gráfico (Mini Paint / CaveGame) em janela própria</li>
            <li><code className="inline-code">F6 / F7 / F8</code> — Test / Rustfmt / Clippy no Modo Criar</li>
            <li><code className="inline-code">F9</code> — Abrir/fechar Catálogo de Bibliotecas Rust</li>
            <li><code className="inline-code">Ctrl+Shift+F</code> — Alternar Modo Foco</li>
            <li><code className="inline-code">Ctrl+S</code> — Salvar arquivo ativo no Mini-IDE</li>
          </ul>
        </div>

        <div className="card stack">
          <h2 style={{ margin: 0, fontSize: '1.1rem' }}>Documentação Oficial e Suporte</h2>
          <div className="row">
            <button
              type="button"
              className="btn btn-sm"
              onClick={() => void ipc.openSafeUrl('https://doc.rust-lang.org/book/')}
            >
              <ExternalLink size={14} /> The Rust Book
            </button>
            <button
              type="button"
              className="btn btn-sm"
              onClick={() => void ipc.openSafeUrl('https://rust-lang.org/install')}
            >
              <ExternalLink size={14} /> Instalar Rust
            </button>
            <button type="button" className="btn btn-sm" onClick={() => void handleExportLogs()}>
              <FileDown size={14} /> Exportar Log de Diagnóstico
            </button>
          </div>
          <p className="muted" style={{ margin: 0, fontSize: '0.86rem' }}>
            Aviso transparente: compilar ou rodar código no Modo Prática ou Modo Criar executa com
            os privilégios da sua conta Windows (não é uma sandbox). Capstones Bevy 3D e projetos
            gráficos dependem de driver OpenGL/DX12/Vulkan e pré-requisitos MSVC no host.
          </p>
        </div>
      </div>
    </div>
  );
}
