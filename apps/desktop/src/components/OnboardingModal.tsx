import React, { useState } from 'react';
import { CheckCircle2, Copy, ExternalLink, RefreshCw, ShieldAlert, Wrench } from 'lucide-react';
import { formatIpcError, ipc } from '../lib/ipc';
import type {
  HelloWorldCheckResult,
  SettingsDto,
  SettingsPatchDto,
  WindowsCapabilityReport,
} from '../lib/types';
import { FerrisMascot } from './FerrisMascot';

interface OnboardingModalProps {
  open: boolean;
  capabilities: WindowsCapabilityReport | null;
  settings: SettingsDto;
  onPatchSettings: (patch: SettingsPatchDto) => Promise<void>;
  onRefreshCapabilities: () => Promise<void>;
  onClose: () => void;
}

export function OnboardingModal({
  open,
  capabilities,
  settings,
  onPatchSettings,
  onRefreshCapabilities,
  onClose,
}: OnboardingModalProps): React.ReactElement | null {
  const [step, setStep] = useState(1);
  const [testingHello, setTestingHello] = useState(false);
  const [helloResult, setHelloResult] = useState<HelloWorldCheckResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copiedCmd, setCopiedCmd] = useState<string | null>(null);

  if (!open) return null;

  const copyCommand = async (cmd: string) => {
    try {
      await navigator.clipboard.writeText(cmd);
      setCopiedCmd(cmd);
      window.setTimeout(() => setCopiedCmd(null), 2000);
    } catch {
      // noop
    }
  };

  const runHelloTest = async () => {
    setError(null);
    setTestingHello(true);
    try {
      if (!settings.execConsentGranted) {
        await ipc.grantExecConsent();
      }
      const res = await ipc.runToolchainHelloWorld();
      setHelloResult(res);
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setTestingHello(false);
    }
  };

  const finishWizard = async () => {
    if (!settings.firstRunCompleted) {
      await onPatchSettings({ firstRunCompleted: true });
    }
    onClose();
  };

  return (
    <div className="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="wizard-title">
      <div className="modal-card stack">
        <div className="between">
          <div>
            <span className="pill pill-info">Toolchain Doctor · Etapa {step} de 7</span>
            <h2 id="wizard-title" style={{ margin: '6px 0 0' }}>
              Bem-vindo ao RustLab Desktop (Windows)
            </h2>
          </div>
          <button type="button" className="btn btn-sm" onClick={() => void finishWizard()}>
            Configurar depois
          </button>
        </div>

        <div className="progress-bar-track" aria-hidden="true">
          <div className="progress-bar-fill" style={{ width: `${Math.round((step / 7) * 100)}%` }} />
        </div>

        {step === 1 && (
          <div className="stack">
            <FerrisMascot
              mood="happy"
              message="Oi! Eu sou o **Ferris**. Antes de começar, escolha conforto visual e som para estudar Rust sem pressa."
            />
            <div className="grid-2">
              <div>
                <label className="field-label" htmlFor="onb-theme">Tema da interface</label>
                <select
                  id="onb-theme"
                  className="input"
                  value={settings.desktopTheme}
                  onChange={(e) => void onPatchSettings({ desktopTheme: e.target.value })}
                >
                  <option value="light">Claro (padrão Windows)</option>
                  <option value="dark">Escuro (Cuia)</option>
                  <option value="system">Seguir sistema Windows</option>
                  <option value="high-contrast">Alto Contraste (WCAG AAA)</option>
                </select>
              </div>
              <div>
                <label className="field-label" htmlFor="onb-motion">Animações</label>
                <select
                  id="onb-motion"
                  className="input"
                  value={settings.animations}
                  onChange={(e) => void onPatchSettings({ animations: e.target.value })}
                >
                  <option value="full">Completas</option>
                  <option value="reduced">Reduzidas</option>
                  <option value="off">Desligadas</option>
                </select>
              </div>
            </div>
            <div className="row">
              <label className="row" style={{ cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={settings.soundEnabled}
                  onChange={(e) => void onPatchSettings({ soundEnabled: e.target.checked })}
                />
                <span>Sons suaves ao ganhar XP</span>
              </label>
              <label className="row" style={{ cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={settings.focusMode}
                  onChange={(e) => void onPatchSettings({ focusMode: e.target.checked })}
                />
                <span>Modo Foco (sem barra lateral)</span>
              </label>
            </div>
          </div>
        )}

        {step === 2 && (
          <div className="stack">
            <h3>Você já pode aprender 100% offline agora mesmo</h3>
            <p>
              Todas as <strong>213 entradas do curso</strong>, demonstrações interativas, quizzes,
              glossário e salvamento de progresso funcionam <strong>sem internet</strong> e mesmo
              antes de instalar o compilador Rust.
            </p>
            <div className="banner banner-info">
              <span>
                Somente compilar código (`cargo check`, `cargo run`, `cargo test`) exige o
                compilador Rust local instalado no Windows.
              </span>
            </div>
          </div>
        )}

        {step === 3 && (
          <div className="stack">
            <div className="between">
              <h3>Inspeção do ambiente Windows</h3>
              <button
                type="button"
                className="btn btn-sm"
                onClick={() => void onRefreshCapabilities()}
              >
                <RefreshCw size={14} /> Reinspecionar
              </button>
            </div>
            {capabilities ? (
              <div className="grid-2">
                <div className="card-muted">
                  <div><strong>Sistema:</strong> {capabilities.osCaption} (Build {capabilities.osBuild})</div>
                  <div><strong>WebView2:</strong> {capabilities.webview2Version ?? 'Detectado via host'}</div>
                  <div><strong>Currículo offline:</strong> {capabilities.offlineCurriculumReady ? 'Pronto (213 entradas)' : 'Indisponível'}</div>
                </div>
                <div className="card-muted">
                  <div><strong>rustup:</strong> {capabilities.rustupFound ? 'Instalado' : 'Não encontrado'}</div>
                  <div><strong>rustc:</strong> {capabilities.rustcVersion ?? 'Ausente'}</div>
                  <div><strong>cargo:</strong> {capabilities.cargoVersion ?? 'Ausente'}</div>
                  <div><strong>Target ativo:</strong> {capabilities.defaultHostTriple ?? 'Desconhecido'}</div>
                </div>
              </div>
            ) : (
              <p className="muted">Verificando ferramentas locais...</p>
            )}
          </div>
        )}

        {step === 4 && (
          <div className="stack">
            <h3>Linkers Windows (MSVC C++ Build Tools / MinGW) e Política Segura</h3>
            <div className="banner banner-warn">
              <ShieldAlert size={18} />
              <span>
                O RustLab nunca altera seu <code>PATH</code> nem pede permissão de administrador
                escondida. Comandos abaixo são para você copiar e rodar no PowerShell quando desejar.
              </span>
            </div>
            {capabilities ? (
              <div className="stack">
                <div className="row">
                  <span className={`pill ${capabilities.msvcLinkerAvailable ? 'pill-ok' : 'pill-warn'}`}>
                    MSVC link.exe: {capabilities.msvcLinkerAvailable ? 'Presente' : 'Ausente'}
                  </span>
                  <span className={`pill ${capabilities.windowsSdkAvailable ? 'pill-ok' : 'pill-warn'}`}>
                    Windows SDK: {capabilities.windowsSdkAvailable ? 'Presente' : 'Ausente'}
                  </span>
                  <span className={`pill ${capabilities.gnuLinkerAvailable ? 'pill-ok' : 'pill-warn'}`}>
                    Linker GNU/LLVM: {capabilities.gnuLinkerAvailable ? 'Presente' : 'Ausente'}
                  </span>
                </div>
                <p>{capabilities.recommendedActionPt}</p>
                {capabilities.copyableFixes.map((fix, i) => (
                  <div key={i} className="card-muted stack">
                    <strong>{fix.title}</strong>
                    <span className="muted">{fix.description}</span>
                    {fix.command ? (
                      <div className="between">
                        <code className="inline-code">{fix.command}</code>
                        <button
                          type="button"
                          className="btn btn-sm"
                          onClick={() => void copyCommand(fix.command!)}
                        >
                          <Copy size={13} /> {copiedCmd === fix.command ? 'Copiado!' : 'Copiar'}
                        </button>
                      </div>
                    ) : null}
                    {fix.docsUrl ? (
                      <button
                        type="button"
                        className="btn btn-sm"
                        style={{ alignSelf: 'flex-start' }}
                        onClick={() => void ipc.openSafeUrl(fix.docsUrl!)}
                      >
                        <ExternalLink size={13} /> Abrir documentação oficial
                      </button>
                    ) : null}
                  </div>
                ))}
              </div>
            ) : null}
          </div>
        )}

        {step === 5 && (
          <div className="stack">
            <h3>Teste opcional de compilação (&quot;Olá, Mundo&quot;)</h3>
            <p>
              Executar <code>cargo run</code> roda código nativo na sua conta Windows (não é uma
              sandbox). Ao clicar abaixo, você autoriza esse teste isolado em pasta temporária.
            </p>
            <div className="row">
              <button
                type="button"
                className="btn btn-primary"
                disabled={testingHello}
                onClick={() => void runHelloTest()}
              >
                <Wrench size={15} /> {testingHello ? 'Compilando Olá Mundo...' : 'Autorizar e Testar Cargo Agora'}
              </button>
            </div>
            {error ? <div className="banner banner-danger">{error}</div> : null}
            {helloResult ? (
              <div className={`banner ${helloResult.success ? 'banner-ok' : 'banner-danger'}`}>
                <div className="stack">
                  <strong>
                    {helloResult.success ? 'Sucesso na compilação real!' : `Falha (${helloResult.category})`}
                  </strong>
                  <span>{helloResult.suggestionPt}</span>
                  {helloResult.stdout ? <pre className="code-block">{helloResult.stdout}</pre> : null}
                  {!helloResult.success && helloResult.stderr ? (
                    <pre className="code-block">{helloResult.stderr}</pre>
                  ) : null}
                </div>
              </div>
            ) : null}
          </div>
        )}

        {step === 6 && (
          <div className="stack">
            <h3>Diagnóstico rápido de falhas comuns no Windows</h3>
            <ul>
              <li><strong>PATH incompleto:</strong> reinicie o RustLab após instalar o `rustup`.</li>
              <li><strong>link.exe ausente (`-msvc`):</strong> instale &quot;Desenvolvimento para desktop com C++&quot; no Visual Studio Build Tools + Windows 10/11 SDK.</li>
              <li><strong>Antivírus / Acesso negado:</strong> exclua sua pasta de projetos (`Documentos\RustLab Projetos`) de bloqueios em tempo real se `.exe` recém-compilado for bloqueado.</li>
              <li><strong>Modo offline:</strong> lições e exercícios já compilados em cache funcionam com `--offline`, mas crates novas de `F9` precisam de rede na primeira vez.</li>
            </ul>
          </div>
        )}

        {step === 7 && (
          <div className="stack">
            <FerrisMascot
              mood="happy"
              message="Tudo pronto! Clique em **Concluir e Ir para Lições** para abrir o curso em português."
            />
            <div className="row">
              <CheckCircle2 size={18} color="var(--ok)" />
              <span>Você pode reabrir o Toolchain Doctor quando quiser na barra superior ou em Ajuda.</span>
            </div>
          </div>
        )}

        <div className="between" style={{ paddingTop: 8, borderTop: '1px solid var(--border-subtle)' }}>
          <button
            type="button"
            className="btn"
            disabled={step === 1}
            onClick={() => setStep((s) => Math.max(1, s - 1))}
          >
            Anterior
          </button>
          <div className="row">
            {step < 7 ? (
              <button
                type="button"
                className="btn btn-primary"
                onClick={() => setStep((s) => Math.min(7, s + 1))}
              >
                Próximo
              </button>
            ) : (
              <button
                type="button"
                className="btn btn-primary"
                onClick={() => void finishWizard()}
              >
                Concluir e Ir para Lições
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
