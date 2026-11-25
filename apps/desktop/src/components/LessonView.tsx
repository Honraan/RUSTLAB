import React, { useEffect, useState } from 'react';
import {
  Bot,
  CheckCircle2,
  ChevronLeft,
  ChevronRight,
  Lightbulb,
  Play,
  RotateCcw,
  Square,
} from 'lucide-react';
import { formatIpcError, ipc, isConsentError } from '../lib/ipc';
import { SafeAnsiOutput, SafeRichText } from '../lib/safeRender';
import { playXpCue } from '../lib/sound';
import type {
  LessonDetailDto,
  MentorAskRequest,
  PracticeRunResultDto,
  QuizSubmitResultDto,
  SettingsDto,
} from '../lib/types';
import { CodeEditor } from './CodeEditor';
import { DemoWidget } from './DemoWidget';
import { FerrisMascot } from './FerrisMascot';

interface LessonViewProps {
  lessonId: string;
  settings: SettingsDto;
  isDarkTheme: boolean;
  onSelectLesson: (id: string) => void;
  onProgressUpdated: () => Promise<void>;
  onOpenMentor: (req: Omit<MentorAskRequest, 'action' | 'question'>) => void;
  onConsentRequired: () => void;
}

export function LessonView({
  lessonId,
  settings,
  isDarkTheme,
  onSelectLesson,
  onProgressUpdated,
  onOpenMentor,
  onConsentRequired,
}: LessonViewProps): React.ReactElement {
  const [lesson, setLesson] = useState<LessonDetailDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [code, setCode] = useState('');
  const [shownHints, setShownHints] = useState(0);
  const [running, setRunning] = useState(false);
  const [streamLines, setStreamLines] = useState<string[]>([]);
  const [runResult, setRunResult] = useState<PracticeRunResultDto | null>(null);
  const [idiomatic, setIdiomatic] = useState<[string, string] | null>(null);
  const [quizFeedback, setQuizFeedback] = useState<Record<number, QuizSubmitResultDto>>({});
  const [lastQuizResult, setLastQuizResult] = useState<QuizSubmitResultDto | null>(null);
  const [aiUsed, setAiUsed] = useState(false);

  const reducedMotion = settings.animations !== 'full';

  const loadLesson = async (id: string) => {
    setLoading(true);
    setError(null);
    setRunResult(null);
    setStreamLines([]);
    setIdiomatic(null);
    setQuizFeedback({});
    setLastQuizResult(null);
    setAiUsed(false);
    try {
      const detail = await ipc.getLessonDetail(id);
      setLesson(detail);
      if (detail.practice) {
        setCode(detail.practice.currentCode);
        setShownHints(detail.practice.shownHints);
        if (detail.practice.completed && detail.practice.hasIdiomatic) {
          const sol = await ipc.getIdiomaticSolution('practice', detail.id).catch(() => null);
          setIdiomatic(sol);
        }
      }
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadLesson(lessonId);
  }, [lessonId]);

  useEffect(() => {
    if (!lesson?.practice) return;
    const id = lesson.id;
    const handle = window.setTimeout(() => {
      void ipc.savePracticeDraft('practice', id, code).catch(() => {});
    }, 650);
    return () => window.clearTimeout(handle);
  }, [code, lesson?.id, lesson?.practice]);

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

  const handleQuiz = async (quizIndex: number, optionIndex: number) => {
    if (!lesson) return;
    try {
      const res = await ipc.submitQuizAnswer(lesson.id, quizIndex, optionIndex);
      setQuizFeedback((prev) => ({ ...prev, [quizIndex]: res }));
      setLastQuizResult(res);
      if (res.xpAwarded > 0) {
        playXpCue(settings.soundEnabled, settings.soundVolume, settings.focusMode);
      }
      if (res.lessonCompleted) {
        await onProgressUpdated();
        const refreshed = await ipc.getLessonDetail(lesson.id);
        setLesson(refreshed);
      }
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const handleCompleteReading = async () => {
    if (!lesson) return;
    try {
      const res = await ipc.completeReadingLesson(lesson.id);
      if (res.xpAwarded > 0) {
        playXpCue(settings.soundEnabled, settings.soundVolume, settings.focusMode);
      }
      await onProgressUpdated();
      const refreshed = await ipc.getLessonDetail(lesson.id);
      setLesson(refreshed);
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const handleRunPractice = async () => {
    if (!lesson?.practice) return;
    setRunning(true);
    setError(null);
    setStreamLines([]);
    setRunResult(null);
    try {
      const res = await ipc.runPracticeCheck('practice', lesson.id, code, aiUsed);
      setRunResult(res);
      setStreamLines(res.outputLines);
      if (res.outcome === 'passed') {
        playXpCue(settings.soundEnabled, settings.soundVolume, settings.focusMode);
        if (res.idiomaticSolution) setIdiomatic(res.idiomaticSolution);
        await onProgressUpdated();
        const refreshed = await ipc.getLessonDetail(lesson.id);
        setLesson(refreshed);
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
    if (!lesson?.practice) return;
    try {
      const count = await ipc.revealPracticeHint('practice', lesson.id);
      setShownHints(count);
    } catch (err) {
      setError(formatIpcError(err));
    }
  };

  const askMentor = () => {
    if (!lesson) return;
    setAiUsed(true);
    onOpenMentor({
      learnMode: true,
      origin: `Lição ${lesson.id}`,
      topicTitle: lesson.title,
      summary: lesson.practice?.prompt ?? lesson.stageLabel,
      code,
      errors: runResult?.diagnostics.map((d) => d.message).join('\n') ?? '',
      console: streamLines.join('\n'),
    });
  };

  if (loading) {
    return <div className="card">Carregando lição {lessonId}...</div>;
  }

  if (!lesson) {
    return <div className="banner banner-danger">{error ?? `Lição ${lessonId} não encontrada.`}</div>;
  }

  const ferrisMood = runResult?.ferrisMood ?? lastQuizResult?.ferrisMood ?? lesson.ferrisMood;
  const ferrisAscii = runResult?.ferrisAscii ?? lastQuizResult?.ferrisAscii ?? lesson.ferrisAscii;
  const ferrisMsg =
    runResult?.ferrisExplanation ??
    lastQuizResult?.explanation ??
    (runResult?.outcome === 'passed'
      ? `Parabéns! Você ganhou **${runResult.xpAwarded} XP** + **${runResult.independenceBonus} XP** de independência.`
      : lesson.completed
        ? 'Lição concluída! Você pode revisar ou seguir para a próxima etapa.'
        : 'Leia os blocos abaixo e teste seu código com segurança.');

  return (
    <div className="stack" style={{ gap: 12 }}>
      <div className="card between">
        <div>
          <div className="row">
            <span className="pill pill-info">{lesson.stageLabel}</span>
            <code className="inline-code">{lesson.id}</code>
            <span className="pill">{lesson.kind}</span>
            {lesson.completed ? (
              <span className="pill pill-ok">
                <CheckCircle2 size={13} /> Concluída
              </span>
            ) : null}
          </div>
          <h1 style={{ margin: '6px 0 0', fontSize: '1.3rem' }}>{lesson.title}</h1>
        </div>
        <div className="row">
          <button
            type="button"
            className="btn btn-sm"
            disabled={!lesson.prevEntryId}
            onClick={() => lesson.prevEntryId && onSelectLesson(lesson.prevEntryId)}
          >
            <ChevronLeft size={15} /> Anterior ({lesson.prevEntryId ?? '-'})
          </button>
          <button
            type="button"
            className="btn btn-sm btn-primary"
            disabled={!lesson.nextEntryId}
            onClick={() => lesson.nextEntryId && onSelectLesson(lesson.nextEntryId)}
          >
            Próxima ({lesson.nextEntryId ?? '-'}) <ChevronRight size={15} />
          </button>
        </div>
      </div>

      {error ? <div className="banner banner-danger">{error}</div> : null}

      <div className={lesson.practice ? 'lesson-split' : 'stack'}>
        <div className="lesson-pane">
          <FerrisMascot mood={ferrisMood} ascii={ferrisAscii} message={ferrisMsg} />

          {lesson.blocks.map((block, idx) => {
            if (block.type === 'text') {
              return (
                <div key={idx} className="card">
                  <SafeRichText text={block.text} />
                </div>
              );
            }
            if (block.type === 'code') {
              return (
                <div key={idx} className="card stack">
                  {block.file ? <code className="inline-code">{block.file}</code> : null}
                  <pre className="code-block">{block.code}</pre>
                  {block.explain ? (
                    <div className="muted">
                      <SafeRichText text={block.explain} />
                    </div>
                  ) : null}
                </div>
              );
            }
            if (block.type === 'callout') {
              const cls =
                block.kind === 'warning'
                  ? 'banner-warn'
                  : block.kind === 'tip'
                    ? 'banner-ok'
                    : 'banner-info';
              return (
                <div key={idx} className={`banner ${cls}`}>
                  <SafeRichText text={block.text} />
                </div>
              );
            }
            if (block.type === 'demo') {
              return (
                <DemoWidget
                  key={idx}
                  demo={block.demo}
                  name={block.name}
                  steps={block.steps}
                  caption={block.caption}
                  reducedMotion={reducedMotion}
                />
              );
            }
            if (block.type === 'quiz') {
              const fb = quizFeedback[block.quizIndex];
              return (
                <div key={idx} className="card stack" aria-label={`Quiz ${block.quizIndex + 1}`}>
                  <strong>Quiz: {block.question}</strong>
                  <div className="stack">
                    {block.options.map((opt, oi) => (
                      <button
                        key={oi}
                        type="button"
                        className="btn"
                        style={{ justifyContent: 'flex-start', textAlign: 'left' }}
                        onClick={() => void handleQuiz(block.quizIndex, oi)}
                      >
                        <code className="inline-code">{String.fromCharCode(65 + oi)}</code> {opt}
                      </button>
                    ))}
                  </div>
                  {fb ? (
                    <div className={`banner ${fb.correct ? 'banner-ok' : 'banner-danger'}`}>
                      <div>
                        <strong>{fb.correct ? 'Resposta correta!' : 'Tente novamente.'}</strong>{' '}
                        {fb.explanation ?? block.why ?? ''}
                        {fb.xpAwarded > 0 ? ` (+${fb.xpAwarded} XP)` : ''}
                      </div>
                    </div>
                  ) : null}
                </div>
              );
            }
            return null;
          })}

          {!lesson.requiresQuiz && !lesson.practice && !lesson.completed ? (
            <div className="card between">
              <span>Terminou de estudar esta seção?</span>
              <button
                type="button"
                className="btn btn-primary"
                onClick={() => void handleCompleteReading()}
              >
                <CheckCircle2 size={15} /> Concluir leitura
              </button>
            </div>
          ) : null}
        </div>

        {lesson.practice ? (
          <div className="lesson-pane">
            <div className="card stack">
              <div className="between">
                <strong>Prática: {lesson.practice.title}</strong>
                <span className="pill">{lesson.practice.mode}</span>
              </div>
              <SafeRichText text={lesson.practice.prompt} />
              {lesson.practice.stdinPreview.length > 0 ? (
                <div className="banner banner-info">
                  <span>
                    Entrada simulada (`stdin`) enviada ao executar:{' '}
                    <code>{lesson.practice.stdinPreview.join(' · ')}</code>
                  </span>
                </div>
              ) : null}
            </div>

            <div className="row">
              <button
                type="button"
                className="btn btn-primary"
                disabled={running}
                onClick={() => void handleRunPractice()}
              >
                <Play size={15} /> {running ? 'Compilando...' : 'Executar e Validar (F5)'}
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
                disabled={shownHints >= lesson.practice.hints.length}
                onClick={() => void handleHint()}
              >
                <Lightbulb size={15} /> Dica (F1) [{shownHints}/{lesson.practice.hints.length}]
              </button>
              <button
                type="button"
                className="btn"
                onClick={() => setCode(lesson.practice?.starter ?? '')}
              >
                <RotateCcw size={15} /> Restaurar inicial
              </button>
              <button type="button" className="btn" onClick={askMentor}>
                <Bot size={15} /> Mentor (Somente Leitura)
              </button>
            </div>

            {shownHints > 0 ? (
              <div className="card-muted stack">
                <strong>Dicas reveladas ({shownHints}):</strong>
                {lesson.practice.hints.slice(0, shownHints).map((h, i) => (
                  <div key={i}>
                    <span className="pill">Dica {i + 1}</span> <SafeRichText text={h} />
                  </div>
                ))}
              </div>
            ) : null}

            <div style={{ minHeight: 250 }}>
              <CodeEditor
                value={code}
                onChange={setCode}
                dark={isDarkTheme}
                diagnostics={runResult?.diagnostics ?? []}
                onRunShortcut={() => void handleRunPractice()}
                onHintShortcut={() => void handleHint()}
              />
            </div>

            <div className="stack">
              <strong>Saída real do Cargo / compilador</strong>
              <SafeAnsiOutput lines={streamLines} />
            </div>

            {runResult?.socraticEli10 ? (
              <div className="banner banner-info">
                <div>
                  <strong>Explicação simples (Ferris ELI10):</strong>{' '}
                  <SafeRichText text={runResult.socraticEli10} />
                </div>
              </div>
            ) : null}

            {runResult && runResult.clippyTips.length > 0 ? (
              <div className="card-muted stack">
                <strong>Dicas pedagógicas de estilo (Clippy):</strong>
                {runResult.clippyTips.map((tip, i) => (
                  <div key={i}>• {tip}</div>
                ))}
              </div>
            ) : null}

            {idiomatic ? (
              <div className="card stack">
                <strong>Comparação idiomática desbloqueada</strong>
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
