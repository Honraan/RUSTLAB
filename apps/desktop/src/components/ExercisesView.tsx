import React, { useEffect, useState } from 'react';
import { Bot, Bug, CheckCircle2, Code2, Lightbulb, Play, RotateCcw, Square } from 'lucide-react';
import { formatIpcError, ipc, isConsentError } from '../lib/ipc';
import { SafeAnsiOutput, SafeRichText } from '../lib/safeRender';
import { playXpCue } from '../lib/sound';
import type {
  ClassicExerciseDto,
  MentorAskRequest,
  PracticeRunResultDto,
  SettingsDto,
} from '../lib/types';
import { CodeEditor } from './CodeEditor';
import { FerrisMascot } from './FerrisMascot';

interface ExercisesViewProps {
  settings: SettingsDto;
  isDarkTheme: boolean;
  onProgressUpdated: () => Promise<void>;
  onOpenMentor: (req: Omit<MentorAskRequest, 'action' | 'question'>) => void;
  onConsentRequired: () => void;
}

export function ExercisesView({
  settings,
  isDarkTheme,
  onProgressUpdated,
  onOpenMentor,
  onConsentRequired,
}: ExercisesViewProps): React.ReactElement {
  const [bugfixMode, setBugfixMode] = useState(false);
  const [items, setItems] = useState<ClassicExerciseDto[]>([]);
  const [selectedId, setSelectedId] = useState<string>('');
  const [code, setCode] = useState('');
  const [shownHints, setShownHints] = useState(0);
  const [running, setRunning] = useState(false);
  const [streamLines, setStreamLines] = useState<string[]>([]);
  const [runResult, setRunResult] = useState<PracticeRunResultDto | null>(null);
  const [idiomatic, setIdiomatic] = useState<[string, string] | null>(null);
  const [aiUsed, setAiUsed] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const selected = items.find((i) => i.id === selectedId) ?? items[0] ?? null;

  const loadCatalog = async (bf: boolean) => {
    setError(null);
    try {
      const list = await ipc.getExercisesCatalog(bf);
      setItems(list);
      if (list[0]) {
        setSelectedId(list[0].id);
        setCode(list[0].currentCode);
        setShownHints(list[0].shownHints);
        setRunResult(null);
        setStreamLines([]);
        setIdiomatic(null);
      }
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  useEffect(() => {
    void loadCatalog(bugfixMode);
  }, [bugfixMode]);

  useEffect(() => {
    if (!selected) return;
    setCode(selected.currentCode);
    setShownHints(selected.shownHints);
    setRunResult(null);
    setStreamLines([]);
    setIdiomatic(null);
    setAiUsed(false);
    if (selected.completed && selected.hasIdiomatic) {
      void ipc
        .getIdiomaticSolution('exercise', selected.id)
        .then(setIdiomatic)
        .catch(() => {});
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selected?.id]);

  useEffect(() => {
    if (!selected) return;
    const id = selected.id;
    const h = window.setTimeout(() => {
      void ipc.savePracticeDraft('exercise', id, code).catch(() => {});
    }, 650);
    return () => window.clearTimeout(h);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [code, selected?.id]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void ipc
      .listenPracticeEvents((ev) => {
        setStreamLines((prev) => [...prev.slice(-240), `[${ev.kind}] ${ev.text}`]);
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => {
      unlisten?.();
    };
  }, []);

  const handleRun = async () => {
    if (!selected) return;
    setRunning(true);
    setError(null);
    setStreamLines([]);
    setRunResult(null);
    try {
      const res = await ipc.runPracticeCheck('exercise', selected.id, code, aiUsed);
      setRunResult(res);
      setStreamLines(res.outputLines);
      if (res.outcome === 'passed') {
        playXpCue(settings.soundEnabled, settings.soundVolume, settings.focusMode);
        if (res.idiomaticSolution) setIdiomatic(res.idiomaticSolution);
        await onProgressUpdated();
        const refreshed = await ipc.getExercisesCatalog(bugfixMode);
        setItems(refreshed);
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

  const handleHint = async () => {
    if (!selected) return;
    try {
      const n = await ipc.revealPracticeHint('exercise', selected.id);
      setShownHints(n);
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  return (
    <div className="stack" style={{ gap: 14 }}>
      <div className="card between">
        <div>
          <h1 style={{ margin: 0, fontSize: '1.35rem' }}>Exercícios Clássicos & Bugfix</h1>
          <p className="muted" style={{ margin: '2px 0 0' }}>
            Testes reais com Cargo local, comparação idiomática e bônus de independência (+100 XP).
          </p>
        </div>
        <div className="row">
          <button
            type="button"
            className={`btn btn-sm ${!bugfixMode ? 'btn-primary' : ''}`}
            onClick={() => setBugfixMode(false)}
          >
            <Code2 size={15} /> Clássicos
          </button>
          <button
            type="button"
            className={`btn btn-sm ${bugfixMode ? 'btn-primary' : ''}`}
            onClick={() => setBugfixMode(true)}
          >
            <Bug size={15} /> Caça aos Bugs
          </button>
        </div>
      </div>

      {error ? <div className="banner banner-danger">{error}</div> : null}

      <div className="lesson-split">
        <div className="lesson-pane">
          <div className="stack">
            {items.map((item) => (
              <button
                key={item.id}
                type="button"
                className="card-muted"
                style={{
                  textAlign: 'left',
                  cursor: 'pointer',
                  borderColor: selected?.id === item.id ? 'var(--accent)' : undefined,
                }}
                onClick={() => setSelectedId(item.id)}
              >
                <div className="between">
                  <strong>{item.title}</strong>
                  {item.completed ? (
                    <span className="pill pill-ok">
                      <CheckCircle2 size={13} /> Passou
                    </span>
                  ) : (
                    <span className="pill">{item.isBugfix ? 'Bugfix' : 'Desafio'}</span>
                  )}
                </div>
                <div className="muted" style={{ fontSize: '0.84rem', marginTop: 4 }}>
                  <SafeRichText text={item.statement} />
                </div>
              </button>
            ))}
          </div>
        </div>

        {selected ? (
          <div className="lesson-pane">
            <FerrisMascot
              mood={runResult?.ferrisMood ?? (selected.completed ? 'happy' : 'neutral')}
              ascii={runResult?.ferrisAscii}
              title={selected.title}
              message={runResult?.ferrisExplanation ?? selected.statement}
            />

            <div className="row">
              <button
                type="button"
                className="btn btn-primary"
                disabled={running}
                onClick={() => void handleRun()}
              >
                <Play size={15} /> {running ? 'Executando testes...' : 'Executar cargo test (F5)'}
              </button>
              {running ? (
                <button
                  type="button"
                  className="btn btn-danger"
                  onClick={() => void ipc.cancelActiveExecution()}
                >
                  <Square size={14} /> Cancelar
                </button>
              ) : null}
              <button
                type="button"
                className="btn"
                disabled={shownHints >= selected.hints.length}
                onClick={() => void handleHint()}
              >
                <Lightbulb size={15} /> Dica (F1) [{shownHints}/{selected.hints.length}]
              </button>
              <button
                type="button"
                className="btn"
                onClick={() => setCode(selected.starter)}
              >
                <RotateCcw size={15} /> Restaurar
              </button>
              <button
                type="button"
                className="btn"
                onClick={() => {
                  setAiUsed(true);
                  onOpenMentor({
                    learnMode: true,
                    origin: `Exercício ${selected.id}`,
                    topicTitle: selected.title,
                    summary: selected.statement,
                    code,
                    errors: runResult?.diagnostics.map((d) => d.message).join('\n') ?? '',
                    console: streamLines.join('\n'),
                  });
                }}
              >
                <Bot size={15} /> Mentor
              </button>
            </div>

            {shownHints > 0 ? (
              <div className="card-muted stack">
                {selected.hints.slice(0, shownHints).map((h, i) => (
                  <div key={i}>
                    <span className="pill">Dica {i + 1}</span> <SafeRichText text={h} />
                  </div>
                ))}
              </div>
            ) : null}

            <div style={{ minHeight: 240 }}>
              <CodeEditor
                value={code}
                onChange={setCode}
                dark={isDarkTheme}
                diagnostics={runResult?.diagnostics ?? []}
                onRunShortcut={() => void handleRun()}
                onHintShortcut={() => void handleHint()}
              />
            </div>

            <SafeAnsiOutput lines={streamLines} />

            {idiomatic ? (
              <div className="card stack">
                <strong>Solução idiomática</strong>
                <pre className="code-block">{idiomatic[0]}</pre>
                <span className="muted">{idiomatic[1]}</span>
              </div>
            ) : null}
          </div>
        ) : null}
      </div>
    </div>
  );
}
