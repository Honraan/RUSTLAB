import React from 'react';
import { Award, Flame, RefreshCw, ShieldCheck } from 'lucide-react';
import type { ProgressSummaryDto } from '../lib/types';

interface ProgressViewProps {
  summary: ProgressSummaryDto | null;
  onOpenLesson: (id: string) => void;
}

export function ProgressView({ summary, onOpenLesson }: ProgressViewProps): React.ReactElement {
  if (!summary) {
    return <div className="card">Carregando progresso e estatísticas...</div>;
  }

  const pct = summary.totalV3 > 0 ? Math.round((summary.completedV3 / summary.totalV3) * 100) : 0;

  return (
    <div className="stack" style={{ gap: 16 }}>
      <div className="card stack">
        <div className="between">
          <div>
            <h1 style={{ margin: 0, fontSize: '1.4rem' }}>Seu Progresso e Conquistas</h1>
            <p className="muted" style={{ margin: '4px 0 0' }}>
              Acompanhamento fiel de maestria, revisões espaçadas (SRS) e independência sem dicas.
            </p>
          </div>
          <span className="pill pill-info">
            <Flame size={14} /> Nível {summary.level} · {summary.xp} XP
          </span>
        </div>

        <div className="progress-bar-track">
          <div className="progress-bar-fill" style={{ width: `${pct}%` }} />
        </div>
        <div className="row">
          <span className="pill">Currículo: {summary.completedV3}/{summary.totalV3} ({pct}%)</span>
          <span className="pill">Exercícios práticos: {summary.completedExercises}</span>
          <span className="pill pill-ok">
            <ShieldCheck size={14} /> Sem dicas / IA: {summary.independentCompletions}
          </span>
        </div>
      </div>

      <div className="grid-2">
        <div className="card stack">
          <h2 style={{ margin: 0, fontSize: '1.1rem' }}>Progresso por Estágio</h2>
          {summary.stageProgress.map((st) => {
            const p = st.total > 0 ? Math.round((st.completed / st.total) * 100) : 0;
            return (
              <div key={st.id} className="stack" style={{ gap: 4 }}>
                <div className="between">
                  <strong>{st.label}</strong>
                  <span className="muted">{st.completed}/{st.total}</span>
                </div>
                <div className="progress-bar-track">
                  <div className="progress-bar-fill" style={{ width: `${p}%` }} />
                </div>
              </div>
            );
          })}
        </div>

        <div className="card stack">
          <h2 style={{ margin: 0, fontSize: '1.1rem' }}>Revisões Espaçadas (SRS) & Erros Frequentes</h2>
          {summary.dueSrsItems.length > 0 ? (
            <div className="row">
              {summary.dueSrsItems.map((id) => (
                <button
                  key={id}
                  type="button"
                  className="btn btn-sm"
                  onClick={() => onOpenLesson(id)}
                >
                  <RefreshCw size={13} /> Revisar {id}
                </button>
              ))}
            </div>
          ) : (
            <span className="muted">Nenhuma revisão vencida no momento.</span>
          )}

          <strong>Códigos de erro que você mais enfrentou</strong>
          {summary.topErrors.length === 0 ? (
            <span className="muted">Sem erros registrados ainda.</span>
          ) : (
            <div className="row">
              {summary.topErrors.map(([code, count]) => (
                <span key={code} className="pill pill-warn">
                  {code}: {count}x
                </span>
              ))}
            </div>
          )}
        </div>
      </div>

      <div className="card stack">
        <h2 style={{ margin: 0, fontSize: '1.1rem' }}>Conquistas</h2>
        <div className="grid-3">
          {summary.achievements.map((ach) => (
            <div
              key={ach.id}
              className="card-muted row"
              style={{ opacity: ach.unlocked ? 1 : 0.6 }}
            >
              <Award size={18} color={ach.unlocked ? 'var(--ok)' : 'var(--fg-muted)'} />
              <div>
                <strong>{ach.title}</strong>
                <div className="muted" style={{ fontSize: '0.8rem' }}>
                  {ach.unlocked ? 'Desbloqueada' : 'Ainda bloqueada'}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
