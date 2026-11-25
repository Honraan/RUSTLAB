import React, { useEffect, useRef, useState } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import { Play, Square, Terminal as TerminalIcon } from 'lucide-react';
import { formatIpcError, ipc } from '../lib/ipc';

interface InteractiveTerminalProps {
  rootDir: string;
  trusted: boolean;
  onConsentRequired?: () => void;
}

export function InteractiveTerminal({
  rootDir,
  trusted,
  onConsentRequired,
}: InteractiveTerminalProps): React.ReactElement {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const sessionIdRef = useRef<string | null>(null);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [status, setStatus] = useState<string>('Terminal ConPTY pronto');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    sessionIdRef.current = sessionId;
  }, [sessionId]);

  useEffect(() => {
    const host = containerRef.current;
    if (!host) return;
    let term: Terminal | null = null;
    try {
      term = new Terminal({
        convertEol: true,
        cursorBlink: true,
        fontFamily: "'Cascadia Code', 'Consolas', monospace",
        fontSize: 13,
        theme: {
          background: '#0b0d13',
          foreground: '#edf2f7',
          cursor: '#ff703d',
        },
      });
      const fit = new FitAddon();
      term.loadAddon(fit);
      term.open(host);
      try {
        fit.fit();
      } catch {
        // jsdom ou container oculto
      }
      term.writeln('RustLab ConPTY — clique em "Iniciar cargo run" para terminal interativo Windows.');
      termRef.current = term;
      fitRef.current = fit;
    } catch {
      setStatus('Terminal visual indisponível neste ambiente de teste.');
    }

    const onResize = () => {
      try {
        fitRef.current?.fit();
      } catch {
        // noop
      }
    };
    window.addEventListener('resize', onResize);

    let active = true;
    const unlisteners: (() => void)[] = [];
    const dataDisposable = term?.onData((chunk) => {
      const sid = sessionIdRef.current;
      if (sid) {
        void ipc.ptyWrite(sid, chunk).catch(() => {});
      }
    });

    void ipc
      .listenPtyData((ev) => {
        if (active && sessionIdRef.current && ev.sessionId === sessionIdRef.current) {
          termRef.current?.write(ev.data);
        }
      })
      .then((u) => unlisteners.push(u));

    void ipc
      .listenPtyExit((ev) => {
        if (active && sessionIdRef.current && ev.sessionId === sessionIdRef.current) {
          termRef.current?.writeln(`\r\n[Processo encerrado com código ${ev.code ?? 0}]`);
          sessionIdRef.current = null;
          setSessionId(null);
          setStatus(`Encerrado (código ${ev.code ?? 0})`);
        }
      })
      .then((u) => unlisteners.push(u));

    return () => {
      active = false;
      dataDisposable?.dispose();
      for (const u of unlisteners) u();
      window.removeEventListener('resize', onResize);
      term?.dispose();
      termRef.current = null;
      fitRef.current = null;
    };
  }, []);

  const startSession = async () => {
    setError(null);
    try {
      const cols = termRef.current?.cols || 100;
      const rows = termRef.current?.rows || 24;
      termRef.current?.clear();
      termRef.current?.writeln(`$ cargo run  (em ${rootDir})\r\n`);
      const id = await ipc.ptyStart(rootDir, cols, rows);
      sessionIdRef.current = id;
      setSessionId(id);
      setStatus('ConPTY em execução (digite aqui para stdin / jogos TUI)');
    } catch (err) {
      const msg = formatIpcError(err);
      setError(msg);
      if (msg.toLowerCase().includes('consentimento')) {
        onConsentRequired?.();
      }
    }
  };

  const stopSession = async () => {
    if (!sessionId) return;
    try {
      await ipc.ptyKill(sessionId);
    } catch {
      // noop
    } finally {
      setSessionId(null);
      setStatus('Sessão ConPTY encerrada');
    }
  };

  return (
    <div className="stack" style={{ height: '100%' }}>
      <div className="between">
        <div className="row">
          <TerminalIcon size={15} />
          <span style={{ fontWeight: 600 }}>Terminal Interativo (Windows ConPTY)</span>
          <span className="muted" style={{ fontSize: '0.82rem' }}>{status}</span>
        </div>
        <div className="row">
          {!sessionId ? (
            <button
              type="button"
              className="btn btn-primary btn-sm"
              onClick={() => void startSession()}
              disabled={!trusted}
            >
              <Play size={14} /> Iniciar cargo run (ConPTY)
            </button>
          ) : (
            <button type="button" className="btn btn-danger btn-sm" onClick={() => void stopSession()}>
              <Square size={14} /> Encerrar sessão
            </button>
          )}
        </div>
      </div>
      {error ? <div className="banner banner-danger">{error}</div> : null}
      <div
        ref={containerRef}
        style={{
          flex: 1,
          minHeight: 165,
          background: '#0b0d13',
          borderRadius: 6,
          overflow: 'hidden',
          padding: 4,
        }}
      />
    </div>
  );
}
