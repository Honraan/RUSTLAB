import React, { useMemo, useState } from 'react';
import { CheckCircle2, Eye, Lock, Search, Unlock } from 'lucide-react';
import type { CourseOverviewDto } from '../lib/types';

interface CourseMapViewProps {
  overview: CourseOverviewDto | null;
  previewUnlocked: boolean;
  onTogglePreview: (next: boolean) => void;
  onOpenLesson: (entryId: string) => void;
}

export function CourseMapView({
  overview,
  previewUnlocked,
  onTogglePreview,
  onOpenLesson,
}: CourseMapViewProps): React.ReactElement {
  const [selectedStage, setSelectedStage] = useState<string>('all');
  const [query, setQuery] = useState('');

  const filteredStages = useMemo(() => {
    if (!overview) return [];
    const q = query.trim().toLowerCase();
    return overview.stages
      .filter((st) => selectedStage === 'all' || st.id === selectedStage)
      .map((st) => ({
        ...st,
        entries: st.entries.filter(
          (e) =>
            !q ||
            e.id.toLowerCase().includes(q) ||
            e.title.toLowerCase().includes(q) ||
            e.learningGoal.toLowerCase().includes(q),
        ),
      }))
      .filter((st) => st.entries.length > 0);
  }, [overview, selectedStage, query]);

  if (!overview) {
    return <div className="card">Carregando currículo canônico de 213 entradas...</div>;
  }

  return (
    <div className="stack" style={{ gap: 14 }}>
      <div className="card stack">
        <div className="between">
          <div>
            <h1 style={{ margin: 0, fontSize: '1.4rem' }}>
              Mapa do Curso ({overview.completedEntries}/{overview.totalEntries} entradas)
            </h1>
            <p className="muted" style={{ margin: '4px 0 0' }}>
              Progressão por pré-requisitos (DAG) nos 6 estágios canônicos do RustLab.
            </p>
          </div>
          <label className="row" style={{ cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={previewUnlocked}
              onChange={(e) => onTogglePreview(e.target.checked)}
            />
            <Eye size={15} />
            <span>Pré-visualizar todo o curso</span>
          </label>
        </div>

        <div className="between">
          <div className="row" role="tablist" aria-label="Filtrar por estágio">
            <button
              type="button"
              role="tab"
              aria-selected={selectedStage === 'all'}
              className={`btn btn-sm ${selectedStage === 'all' ? 'btn-primary' : ''}`}
              onClick={() => setSelectedStage('all')}
            >
              Todos ({overview.totalEntries})
            </button>
            {overview.stages.map((st) => (
              <button
                key={st.id}
                type="button"
                role="tab"
                aria-selected={selectedStage === st.id}
                className={`btn btn-sm ${selectedStage === st.id ? 'btn-primary' : ''}`}
                onClick={() => setSelectedStage(st.id)}
              >
                {st.label} ({st.completedCount}/{st.totalCount})
              </button>
            ))}
          </div>

          <div className="row" style={{ minWidth: 260 }}>
            <Search size={15} />
            <input
              type="search"
              className="input"
              aria-label="Buscar lição ou tópico"
              placeholder="Buscar por ID ou título (ex: B-T01, Cargo, Bevy)..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
          </div>
        </div>
      </div>

      {filteredStages.map((stage) => (
        <section key={stage.id} className="card stack" aria-label={stage.label}>
          <div className="between">
            <div>
              <h2 style={{ margin: 0, fontSize: '1.15rem' }}>{stage.label}</h2>
              <span className="muted">{stage.subtitle}</span>
            </div>
            <span className={`pill ${stage.unlocked ? 'pill-ok' : 'pill-warn'}`}>
              {stage.completedCount}/{stage.totalCount} concluídas
            </span>
          </div>

          <div className="grid-2">
            {stage.entries.map((entry) => {
              const canOpen = entry.unlocked || previewUnlocked;
              return (
                <div
                  key={entry.id}
                  className="card-muted stack"
                  style={{ opacity: canOpen ? 1 : 0.7 }}
                >
                  <div className="between">
                    <div className="row">
                      <code className="inline-code">{entry.id}</code>
                      <span className="pill">{entry.kind}</span>
                      {entry.isInteractiveStdin ? (
                        <span className="pill pill-info">Stdin</span>
                      ) : null}
                    </div>
                    {entry.completed ? (
                      <span className="pill pill-ok">
                        <CheckCircle2 size={13} /> Concluída
                      </span>
                    ) : canOpen ? (
                      <span className="pill">
                        <Unlock size={13} /> Disponível
                      </span>
                    ) : (
                      <span className="pill pill-warn">
                        <Lock size={13} /> Bloqueada
                      </span>
                    )}
                  </div>

                  <strong>{entry.title}</strong>
                  <span className="muted" style={{ fontSize: '0.86rem' }}>
                    {entry.learningGoal}
                  </span>

                  <div className="between">
                    <span className="muted" style={{ fontSize: '0.78rem' }}>
                      {entry.prerequisites.length > 0
                        ? `Pré-requisitos: ${entry.prerequisites.join(', ')}`
                        : 'Sem pré-requisitos'}
                    </span>
                    <button
                      type="button"
                      className={`btn btn-sm ${entry.completed ? '' : 'btn-primary'}`}
                      onClick={() => onOpenLesson(entry.id)}
                      disabled={!canOpen}
                    >
                      {entry.completed ? 'Revisar' : 'Abrir'}
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        </section>
      ))}
    </div>
  );
}
