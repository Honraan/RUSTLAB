import { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { api, errorMessage } from "./api";

export default function TerminalPanel({ allowed, download, running, onRunning, report, fontSize }: {
  allowed: boolean;
  download: boolean;
  running: boolean;
  onRunning: (running: boolean) => void;
  report: (message: string) => void;
  fontSize: number;
}) {
  const container = useRef<HTMLDivElement>(null);
  const terminal = useRef<Terminal | null>(null);
  const fit = useRef<FitAddon | null>(null);
  const active = useRef(false);
  const queue = useRef({ pending: 0, tail: Promise.resolve() });
  const [status, setStatus] = useState("");
  const [starting, setStarting] = useState(false);

  useEffect(() => {
    const mount = container.current;
    if (!mount) return;
    const emulator = new Terminal({
      cursorBlink: false,
      fontFamily: "Cascadia Code, Consolas, monospace",
      scrollback: 1200,
      theme: { background: "#211d1c", foreground: "#fff4eb", cursor: "#ffad86", selectionBackground: "#674838" },
      screenReaderMode: true,
      linkHandler: { activate: () => {} },
    });
    const addon = new FitAddon();
    emulator.loadAddon(addon);
    emulator.open(mount);
    terminal.current = emulator;
    fit.current = addon;
    const resize = () => {
      if (!mount.clientWidth) return;
      addon.fit();
      if (active.current) void api.terminalResize(emulator.cols, emulator.rows).catch((error: unknown) => report(errorMessage(error)));
    };
    const observer = new ResizeObserver(resize);
    observer.observe(mount);
    const data = emulator.onData((text) => {
      if (!active.current) return;
      const bytes = new TextEncoder().encode(text);
      if (bytes.length > 8192) {
        report("Limite de 8 KiB por envio ao terminal.");
        return;
      }
      const writes = queue.current;
      if (writes.pending >= 32) {
        report("Fila de entrada cheia. Aguarde o terminal.");
        return;
      }
      writes.pending += 1;
      writes.tail = writes.tail.then(async () => {
        if (active.current && queue.current === writes) await api.terminalWrite(Array.from(bytes));
      }).catch((error: unknown) => report(errorMessage(error))).finally(() => { writes.pending -= 1; });
    });
    return () => {
      observer.disconnect();
      data.dispose();
      emulator.dispose();
      terminal.current = null;
      fit.current = null;
    };
  }, [report]);

  useEffect(() => {
    const emulator = terminal.current;
    if (emulator) {
      emulator.options.fontSize = fontSize;
      fit.current?.fit();
      if (active.current) void api.terminalResize(emulator.cols, emulator.rows).catch((error: unknown) => report(errorMessage(error)));
    }
  }, [fontSize, report]);

  useEffect(() => {
    if (!running) return;
    let stopped = false;
    let timer = 0;
    async function poll() {
      try {
        const batch = await api.terminalPoll();
        if (stopped) return;
        const emulator = terminal.current;
        if (emulator && batch.bytes.length) {
          await new Promise<void>((resolve) => emulator.write(Uint8Array.from(batch.bytes), resolve));
        }
        if (stopped) return;
        if (batch.finished) {
          active.current = false;
          setStatus(batch.finished);
          onRunning(false);
          return;
        }
      } catch (error) {
        if (!stopped) report(errorMessage(error));
      }
      if (!stopped) timer = window.setTimeout(() => { void poll(); }, 60);
    }
    void poll();
    return () => { stopped = true; window.clearTimeout(timer); };
  }, [running, onRunning, report]);

  async function start() {
    const emulator = terminal.current;
    if (!emulator) return;
    setStarting(true);
    try {
      fit.current?.fit();
      await api.terminalStart(download, emulator.cols, emulator.rows);
      emulator.reset();
      queue.current = { pending: 0, tail: Promise.resolve() };
      active.current = true;
      setStatus("Terminal ativo.");
      onRunning(true);
      emulator.focus();
    } catch (error) {
      report(errorMessage(error));
    } finally {
      setStarting(false);
    }
  }

  return <section className="terminal-panel" aria-label="Terminal interativo do projeto">
    <h3>Terminal interativo</h3>
    <div className="toolbar">
      <button disabled={!allowed || starting || running} onClick={() => void start()}>Executar no terminal</button>
      <button disabled={!running} onClick={() => void api.terminalCancel().catch((error: unknown) => report(errorMessage(error)))}>Encerrar terminal</button>
      <span role="status">{status}</span>
    </div>
    <div className="term" ref={container} />
  </section>;
}
