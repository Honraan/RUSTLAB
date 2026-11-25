import React, { useState } from 'react';
import { Download, KeyRound, RotateCcw, Trash2, Upload, Volume2 } from 'lucide-react';
import { formatIpcError, ipc } from '../lib/ipc';
import { playXpCue } from '../lib/sound';
import type { ImportPreviewDto, SettingsDto, SettingsPatchDto } from '../lib/types';

interface SettingsViewProps {
  settings: SettingsDto;
  onPatch: (patch: SettingsPatchDto) => Promise<void>;
  onProgressUpdated: () => Promise<void>;
}

export function SettingsView({
  settings,
  onPatch,
  onProgressUpdated,
}: SettingsViewProps): React.ReactElement {
  const [apiKeyInput, setApiKeyInput] = useState('');
  const [ollamaModels, setOllamaModels] = useState<string[]>([]);
  const [importPath, setImportPath] = useState<string | null>(null);
  const [importPreview, setImportPreview] = useState<ImportPreviewDto | null>(null);
  const [confirmReset, setConfirmReset] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleSaveKey = async () => {
    setError(null);
    setStatus(null);
    try {
      await ipc.mentorSetApiKey(apiKeyInput);
      setApiKeyInput('');
      setStatus('Chave gravada com segurança no Windows Credential Manager.');
      await onPatch({});
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const handleDeleteKey = async () => {
    setError(null);
    try {
      await ipc.mentorDeleteApiKey();
      setStatus('Chave removida do Windows Credential Manager.');
      await onPatch({});
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const handleExport = async () => {
    setError(null);
    try {
      const dest = await ipc.pickSaveFile('rustlab-backup-v4.json', 'json');
      if (!dest) return;
      await ipc.exportBackup(dest);
      setStatus(`Backup portátil exportado para ${dest}`);
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const handlePickImport = async () => {
    setError(null);
    setImportPreview(null);
    try {
      const src = await ipc.pickBackupFile();
      if (!src) return;
      setImportPath(src);
      const prev = await ipc.previewImportBackup(src);
      setImportPreview(prev);
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const handleApplyImport = async (mode: 'merge' | 'replace') => {
    if (!importPath) return;
    setError(null);
    try {
      const res = await ipc.applyImportBackup(importPath, mode);
      setImportPreview(null);
      setImportPath(null);
      await onProgressUpdated();
      setStatus(`Importação concluída (${mode}). XP atual: ${res.xp} (Nível ${res.level}).`);
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const handleReset = async () => {
    setError(null);
    try {
      await ipc.resetProgress();
      setConfirmReset(false);
      await onProgressUpdated();
      setStatus('Progresso redefinido após backup automático em %LOCALAPPDATA%\\RustLab\\backups.');
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  return (
    <div className="stack" style={{ gap: 16 }}>
      <div className="card">
        <h1 style={{ margin: 0, fontSize: '1.4rem' }}>Configurações e Dados</h1>
        <p className="muted" style={{ margin: '4px 0 0' }}>
          Pasta de dados Windows: <code className="inline-code">{settings.dataRoot}</code>
        </p>
      </div>

      {status ? <div className="banner banner-ok">{status}</div> : null}
      {error ? <div className="banner banner-danger">{error}</div> : null}

      <div className="grid-2">
        <div className="card stack">
          <h2 style={{ margin: 0, fontSize: '1.1rem' }}>Aparência & Acessibilidade</h2>
          <div>
            <label className="field-label" htmlFor="cfg-theme">Tema</label>
            <select
              id="cfg-theme"
              className="input"
              value={settings.desktopTheme}
              onChange={(e) => void onPatch({ desktopTheme: e.target.value })}
            >
              <option value="light">Claro</option>
              <option value="dark">Escuro (Cuia)</option>
              <option value="system">Seguir Windows</option>
              <option value="high-contrast">Alto Contraste (WCAG AAA)</option>
            </select>
          </div>

          <div className="grid-2">
            <div>
              <label className="field-label" htmlFor="cfg-zoom">Zoom da UI ({settings.uiScale}%)</label>
              <input
                id="cfg-zoom"
                type="range"
                min={85}
                max={150}
                step={5}
                value={settings.uiScale}
                onChange={(e) => void onPatch({ uiScale: Number(e.target.value) })}
              />
            </div>
            <div>
              <label className="field-label" htmlFor="cfg-font">Fonte do editor ({settings.editorFontSize}px)</label>
              <input
                id="cfg-font"
                type="range"
                min={12}
                max={24}
                step={1}
                value={settings.editorFontSize}
                onChange={(e) => void onPatch({ editorFontSize: Number(e.target.value) })}
              />
            </div>
          </div>

          <div>
            <label className="field-label" htmlFor="cfg-anim">Animações</label>
            <select
              id="cfg-anim"
              className="input"
              value={settings.animations}
              onChange={(e) => void onPatch({ animations: e.target.value })}
            >
              <option value="full">Completas</option>
              <option value="reduced">Reduzidas</option>
              <option value="off">Desligadas</option>
            </select>
          </div>

          <div className="row">
            <label className="row" style={{ cursor: 'pointer' }}>
              <input
                type="checkbox"
                checked={settings.focusMode}
                onChange={(e) => void onPatch({ focusMode: e.target.checked })}
              />
              <span>Modo Foco (Ctrl+Shift+F)</span>
            </label>
            <label className="row" style={{ cursor: 'pointer' }}>
              <input
                type="checkbox"
                checked={settings.gamificationVisible}
                onChange={(e) => void onPatch({ gamificationVisible: e.target.checked })}
              />
              <span>Exibir XP e nível</span>
            </label>
          </div>

          <div className="row">
            <label className="row" style={{ cursor: 'pointer' }}>
              <input
                type="checkbox"
                checked={settings.soundEnabled}
                onChange={(e) => void onPatch({ soundEnabled: e.target.checked })}
              />
              <span>Som ao conquistar XP ({settings.soundVolume}%)</span>
            </label>
            <button
              type="button"
              className="btn btn-sm"
              onClick={() => playXpCue(true, settings.soundVolume || 70, false)}
            >
              <Volume2 size={14} /> Testar som
            </button>
          </div>
        </div>

        <div className="card stack">
          <h2 style={{ margin: 0, fontSize: '1.1rem' }}>Idioma & Mentor de IA (Opcional)</h2>
          <div className="grid-2">
            <div>
              <label className="field-label" htmlFor="cfg-lang">Idioma do curso</label>
              <select
                id="cfg-lang"
                className="input"
                value={settings.language}
                onChange={(e) => void onPatch({ language: e.target.value })}
              >
                <option value="pt-BR">Português (Brasil)</option>
                <option value="en-US">English (US)</option>
              </select>
            </div>
            <div>
              <label className="field-label" htmlFor="cfg-origin">Linguagem de origem</label>
              <select
                id="cfg-origin"
                className="input"
                value={settings.originLanguage}
                onChange={(e) => void onPatch({ originLanguage: e.target.value })}
              >
                <option value="none">Iniciante em programação</option>
                <option value="python">Vindo de Python</option>
                <option value="javascript">Vindo de JavaScript</option>
                <option value="cpp">Vindo de C / C++</option>
              </select>
            </div>
          </div>

          <div>
            <label className="field-label" htmlFor="cfg-ai">Provedor de IA</label>
            <select
              id="cfg-ai"
              className="input"
              value={settings.aiProvider}
              onChange={(e) => void onPatch({ aiProvider: e.target.value })}
            >
              <option value="disabled">Desativado (Mentor determinístico offline)</option>
              <option value="ollama">Ollama Local (127.0.0.1)</option>
              <option value="remote">API Remota (OpenAI-compatível)</option>
            </select>
          </div>

          {settings.aiProvider === 'ollama' ? (
            <div className="stack">
              <div className="row">
                <input
                  className="input"
                  style={{ flex: 1 }}
                  value={settings.ollamaModel}
                  placeholder="Modelo (ex: llama3.2)"
                  onChange={(e) => void onPatch({ ollamaModel: e.target.value })}
                />
                <button
                  type="button"
                  className="btn btn-sm"
                  onClick={() =>
                    void ipc
                      .mentorListOllamaModels()
                      .then(setOllamaModels)
                      .catch((e) => setError(formatIpcError(e)))
                  }
                >
                  Listar modelos
                </button>
              </div>
              {ollamaModels.length > 0 ? (
                <div className="row">
                  {ollamaModels.map((m) => (
                    <button
                      key={m}
                      type="button"
                      className="btn btn-sm"
                      onClick={() => void onPatch({ ollamaModel: m })}
                    >
                      {m}
                    </button>
                  ))}
                </div>
              ) : null}
            </div>
          ) : null}

          {settings.aiProvider === 'remote' ? (
            <div className="stack">
              <label className="row" style={{ cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={settings.remoteConsent === 'granted'}
                  onChange={(e) =>
                    void onPatch({ remoteConsent: e.target.checked ? 'granted' : 'unset' })
                  }
                />
                <span>Autorizo enviar código e erros para a URL remota configurada</span>
              </label>
              <div className="row">
                <input
                  type="password"
                  className="input"
                  style={{ flex: 1 }}
                  placeholder={
                    settings.hasRemoteApiKey
                      ? 'Chave já salva no Windows Credential Manager'
                      : 'Cole sua API Key...'
                  }
                  value={apiKeyInput}
                  onChange={(e) => setApiKeyInput(e.target.value)}
                />
                <button type="button" className="btn btn-sm" onClick={() => void handleSaveKey()}>
                  <KeyRound size={14} /> Salvar no Credential Manager
                </button>
                {settings.hasRemoteApiKey ? (
                  <button
                    type="button"
                    className="btn btn-sm btn-danger"
                    onClick={() => void handleDeleteKey()}
                  >
                    <Trash2 size={14} /> Remover
                  </button>
                ) : null}
              </div>
            </div>
          ) : null}
        </div>
      </div>

      <div className="card stack">
        <h2 style={{ margin: 0, fontSize: '1.1rem' }}>Backup, Migração (TUI V3 → V4) e Restauração</h2>
        <p className="muted" style={{ margin: 0 }}>
          Exporte seu progresso em JSON portátil ou importe um arquivo <code>progresso.toml</code>{' '}
          da edição terminal V3 com prévia segura antes de aplicar.
        </p>

        <div className="row">
          <button type="button" className="btn" onClick={() => void handleExport()}>
            <Download size={15} /> Exportar Backup (.json)
          </button>
          <button type="button" className="btn" onClick={() => void handlePickImport()}>
            <Upload size={15} /> Importar Backup ou progresso.toml (TUI V3)
          </button>
          {!confirmReset ? (
            <button
              type="button"
              className="btn btn-danger"
              onClick={() => setConfirmReset(true)}
            >
              <RotateCcw size={15} /> Redefinir progresso
            </button>
          ) : (
            <div className="row">
              <button type="button" className="btn btn-danger" onClick={() => void handleReset()}>
                Confirmar Reset (salva backup antes)
              </button>
              <button type="button" className="btn" onClick={() => setConfirmReset(false)}>
                Cancelar
              </button>
            </div>
          )}
        </div>

        {importPreview ? (
          <div className="card-muted stack">
            <strong>Prévia de importação ({importPreview.sourceFormat})</strong>
            <span>
              Lições: {importPreview.incomingLessons} · Exercícios:{' '}
              {importPreview.incomingExercises} · XP no arquivo: {importPreview.incomingXp} · XP
              após mescla: <strong>{importPreview.newXpAfterMerge}</strong>
            </span>
            {importPreview.warning ? (
              <div className="banner banner-warn">{importPreview.warning}</div>
            ) : null}
            <div className="row">
              <button
                type="button"
                className="btn btn-primary"
                onClick={() => void handleApplyImport('merge')}
              >
                Mesclar sem perder progresso
              </button>
              <button
                type="button"
                className="btn btn-danger"
                onClick={() => void handleApplyImport('replace')}
              >
                Substituir atual (faz backup antes)
              </button>
            </div>
          </div>
        ) : null}
      </div>
    </div>
  );
}
