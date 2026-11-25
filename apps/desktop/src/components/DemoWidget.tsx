import React, { useEffect, useState } from 'react';
import { ChevronLeft, ChevronRight, Pause, Play, RotateCcw } from 'lucide-react';
import { SafeRichText } from '../lib/safeRender';

interface DemoWidgetProps {
  demo: string;
  name: string;
  steps: string[];
  caption?: string | null;
  reducedMotion: boolean;
}

export function DemoWidget({
  demo,
  name,
  steps,
  caption,
  reducedMotion,
}: DemoWidgetProps): React.ReactElement {
  const safeSteps = steps.length > 0 ? steps : ['Passo único desta demonstração visual.'];
  const [stepIndex, setStepIndex] = useState(0);
  const [playing, setPlaying] = useState(false);

  useEffect(() => {
    setStepIndex(0);
    setPlaying(false);
  }, [name, demo]);

  useEffect(() => {
    if (!playing || reducedMotion) return;
    const timer = window.setInterval(() => {
      setStepIndex((prev) => (prev + 1) % safeSteps.length);
    }, 1800);
    return () => window.clearInterval(timer);
  }, [playing, reducedMotion, safeSteps.length]);

  const progressPct = Math.round(((stepIndex + 1) / safeSteps.length) * 100);

  return (
    <div className="card-muted stack" aria-label={`Demonstração interativa: ${name}`}>
      <div className="between">
        <div>
          <span className="pill pill-info">Demo · {demo}</span> <strong>{name}</strong>
        </div>
        <span className="muted" style={{ fontSize: '0.84rem' }}>
          Passo {stepIndex + 1} de {safeSteps.length}
        </span>
      </div>

      <div className="progress-bar-track" aria-hidden="true">
        <div className="progress-bar-fill" style={{ width: `${progressPct}%` }} />
      </div>

      <div
        className="card"
        style={{ background: 'var(--bg-surface)', minHeight: 74, display: 'flex', alignItems: 'center' }}
      >
        <div style={{ fontSize: '0.96rem' }}>
          <SafeRichText text={safeSteps[stepIndex]} />
        </div>
      </div>

      <div className="between">
        <div className="row">
          <button
            type="button"
            className="btn btn-sm"
            onClick={() => {
              setPlaying(false);
              setStepIndex((i) => Math.max(0, i - 1));
            }}
            disabled={stepIndex === 0}
          >
            <ChevronLeft size={15} /> Anterior
          </button>
          <button
            type="button"
            className="btn btn-sm"
            onClick={() => {
              setPlaying(false);
              setStepIndex((i) => Math.min(safeSteps.length - 1, i + 1));
            }}
            disabled={stepIndex >= safeSteps.length - 1}
          >
            Próximo <ChevronRight size={15} />
          </button>
          {!reducedMotion ? (
            <button type="button" className="btn btn-sm" onClick={() => setPlaying((p) => !p)}>
              {playing ? <Pause size={15} /> : <Play size={15} />}
              {playing ? 'Pausar' : 'Auto'}
            </button>
          ) : null}
          <button
            type="button"
            className="btn btn-sm"
            onClick={() => {
              setPlaying(false);
              setStepIndex(0);
            }}
          >
            <RotateCcw size={14} /> Reiniciar
          </button>
        </div>
        {caption ? <span className="muted" style={{ fontSize: '0.82rem' }}>{caption}</span> : null}
      </div>
    </div>
  );
}
