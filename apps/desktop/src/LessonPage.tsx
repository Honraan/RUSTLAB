import { useState } from "react";
import { ChevronRight, CircleHelp, Play, Save } from "lucide-react";
import { api, errorMessage } from "./api";
import type { Content, Lesson, QuizResult } from "./api";
import type { RunTask } from "./App";
import { Editor } from "./Editor";

export function LessonPage({ lesson, fontSize, report, run, running, refreshed, celebrate }: {
  lesson: Lesson;
  fontSize: number;
  report: (message: string) => void;
  run: RunTask;
  running: boolean;
  refreshed: () => Promise<void>;
  celebrate: (xp: number) => void;
}) {
  const draftKey = `rustlab-draft-${lesson.entry.id}`;
  const [source, setSource] = useState(() => localStorage.getItem(draftKey) ?? lesson.practice?.source ?? "");
  const [hints, setHints] = useState(lesson.practice?.hints ?? []);
  const [picks, setPicks] = useState<Record<number, number>>({});
  const [result, setResult] = useState<QuizResult | null>(null);
  const [working, setWorking] = useState(false);
  const [consent, setConsent] = useState(false);
  const id = lesson.entry.id;

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

  function edit(text: string) {
    setSource(text);
    try {
      localStorage.setItem(draftKey, text);
    } catch {
      report("O armazenamento de recuperação está cheio. Use Salvar rascunho.");
    }
  }

  const save = () => act(async () => {
    await api.draft(id, source);
    if (localStorage.getItem(draftKey) === source) localStorage.removeItem(draftKey);
    report("Rascunho salvo.");
  });

  return <>
    <header className="lesson-header">
      <div>
        <p className="eyebrow">{lesson.stage} · {lesson.entry.kind} · {lesson.minutes} min</p>
        <h1>{lesson.entry.title}</h1>
      </div>
      <div className="ferris-reaction" aria-label="Reação do Ferris">{result?.ferris ?? "c(o.o)c"}</div>
    </header>
    <div className="lesson-layout">
      <article className="card reading">
        <h2>Explicação</h2>
        {lesson.content.map((block, index) => <LessonBlock key={index} block={block} />)}
      </article>
      <section className="card questions">
        <h2>{lesson.questions.length ? "Teste sua compreensão" : "Finalizar leitura"}</h2>
        <form onSubmit={(event) => {
          event.preventDefault();
          void act(async () => {
            const answered = await api.answer(id, lesson.questions.map((_, index) => picks[index] ?? -1));
            setResult(answered);
            celebrate(answered.awarded);
            await refreshed();
          });
        }}>
          {lesson.questions.map((q, index) => (
            <fieldset key={index}>
              <legend>{q.question}</legend>
              {q.options.map((option, choice) => <label className="quiz-option" key={choice}>
                <input type="radio" name={`question-${index}`} checked={picks[index] === choice}
                  onChange={() => { setPicks({ ...picks, [index]: choice }); setResult(null); }}
                  required />{option}
              </label>)}
              {result && <p className={result.correct[index] ? "feedback done" : "feedback"} role="status">
                {result.correct[index] ? result.explanations[index] || "Correto." : "Ainda não. Confira a explicação e tente outra opção."}
              </p>}
            </fieldset>
          ))}
          <button className="primary" disabled={working} type="submit">
            {lesson.questions.length ? "Conferir respostas" : "Registrar leitura"} <ChevronRight size={16} aria-hidden="true" />
          </button>
          {result?.completed && <p role="status">{result.awarded ? `Lição concluída. +${result.awarded} XP.` : "Lição concluída."}</p>}
          {result?.correct.every(Boolean) && lesson.practice && !result.completed && <p role="status">Agora conclua a prática com Cargo.</p>}
        </form>
      </section>
    </div>
    {lesson.practice && <section className="card practice">
      <h2>Pratique no editor</h2>
      <p>{lesson.practice.prompt}</p>
      <Editor label="Código da prática" value={source} language="rust" fontSize={fontSize} onChange={edit} onSave={() => void save()} />
      <label className="consent"><input type="checkbox" checked={consent} onChange={(event) => setConsent(event.target.checked)} />
        Autorizo compilar e executar este código com as permissões da minha conta. A tarefa usa Cargo em modo offline.</label>
      <div className="toolbar">
        <button className="primary" disabled={working || !consent || running} onClick={() => void act(async () => {
          await run("Validar prática", () => api.practice(id, source));
          if (localStorage.getItem(draftKey) === source) localStorage.removeItem(draftKey);
        })}><Play size={16} aria-hidden="true" /> Validar prática</button>
        <button disabled={working} onClick={() => void save()}><Save size={16} aria-hidden="true" /> Salvar rascunho</button>
        <button disabled={working || hints.length === 4 || running} onClick={() => void act(async () => setHints(await api.hint(id)))}>
          <CircleHelp size={16} aria-hidden="true" /> Revelar dica ({hints.length}/4)
        </button>
      </div>
      {hints.length > 0 && <ol className="hints">{hints.map((hint) => <li key={hint}>{hint}</li>)}</ol>}
    </section>}
  </>;
}

function LessonBlock({ block }: { block: Content }) {
  const [step, setStep] = useState(0);
  switch (block.type) {
    case "text": return <p className="prewrap">{block.text}</p>;
    case "callout": return <aside className="inset"><strong>{block.label}</strong><p className="prewrap">{block.text}</p></aside>;
    case "code": return <figure>
      {block.file && <figcaption><code>{block.file}</code></figcaption>}
      <pre tabIndex={0}><code>{block.code}</code></pre>
      {block.explain && <p>{block.explain}</p>}
    </figure>;
    case "demo": return <div className="inset">
      <strong>{block.name}</strong>
      {block.caption && <p>{block.caption}</p>}
      {block.steps.length > 0 && <>
        <p aria-live="polite">{block.steps[step]}</p>
        <button onClick={() => setStep((step + 1) % block.steps.length)} disabled={block.steps.length === 1}>
          Próximo passo <ChevronRight size={16} aria-hidden="true" />
        </button>
      </>}
    </div>;
  }
}
