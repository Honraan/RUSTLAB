import React from 'react';
import { BookOpen, Code2, FolderGit2, Map, Play, Sparkles, Stethoscope } from 'lucide-react';
import type { CourseOverviewDto, WindowsCapabilityReport } from '../lib/types';
import { FerrisMascot } from './FerrisMascot';

interface HomeViewProps {
  overview: CourseOverviewDto | null;
  capabilities: WindowsCapabilityReport | null;
  onOpenLesson: (entryId: string) => void;
  onNavigate: (route: 'course' | 'exercises' | 'create' | 'progress') => void;
  onOpenDoctor: () => void;
}

export function HomeView({
  overview,
  capabilities,
  onOpenLesson,
  onNavigate,
  onOpenDoctor,
}: HomeViewProps): React.ReactElement {
  const resumeId = overview?.lastEntryId || 'B-T01';
  const completed = overview?.completedEntries ?? 0;
  const total = overview?.totalEntries ?? 213;
  const pct = total > 0 ? Math.round((completed / total) * 100) : 0;

  return (
    <div className="stack" style={{ gap: 18 }}>
      <div className="card stack">
        <div className="between">
          <div>
            <span className="pill pill-info">RustLab 4.0 · Edição Windows Desktop</span>
            <h1 style={{ margin: '8px 0 4px', fontSize: '1.6rem' }}>
              Aprenda Rust na prática, do zero aos seus projetos reais
            </h1>
            <p className="muted" style={{ margin: 0 }}>
              Currículo canônico de {total} entradas em português, compilação real via Cargo local e
              terminal interativo Windows ConPTY.
            </p>
          </div>
          <button
            type="button"
            className="btn btn-primary"
            onClick={() => onOpenLesson(resumeId)}
          >
            <Play size={16} /> Continuar aprendendo ({resumeId})
          </button>
        </div>

        <FerrisMascot
          mood="neutral"
          title="Ferris acompanha sua jornada"
          message={`Progresso atual: **${completed}/${total}** entradas concluídas (**${pct}%**) · **${overview?.xp ?? 0} XP** (Nível **${overview?.level ?? 1}**).`}
        />

        {capabilities && !capabilities.canCompileNative ? (
          <div className="banner banner-warn">
            <div>
              <strong>Compilador nativo ainda pendente de configuração:</strong>{' '}
              {capabilities.recommendedActionPt} Você já pode ler todas as lições e responder
              quizzes offline.
            </div>
            <button type="button" className="btn btn-sm" onClick={onOpenDoctor}>
              <Stethoscope size={14} /> Abrir Toolchain Doctor
            </button>
          </div>
        ) : null}

        {overview && overview.dueSrsItems.length > 0 ? (
          <div className="banner banner-info">
            <div>
              <strong>Revisões SRS recomendadas ({overview.dueSrsItems.length}):</strong>{' '}
              {overview.dueSrsItems.slice(0, 5).join(', ')}
            </div>
            <button
              type="button"
              className="btn btn-sm"
              onClick={() => onNavigate('progress')}
            >
              Ver revisões
            </button>
          </div>
        ) : null}
      </div>

      <div className="grid-3">
        <div className="card stack">
          <div className="row">
            <Map size={18} color="var(--accent)" />
            <strong>Mapa do Curso ({total})</strong>
          </div>
          <p className="muted" style={{ margin: 0 }}>
            Explore os 6 estágios: Básico, Meio, Difícil, Fim, Mini Paint e CaveGame.
          </p>
          <button type="button" className="btn" onClick={() => onNavigate('course')}>
            <BookOpen size={15} /> Abrir Trilha Completa
          </button>
        </div>

        <div className="card stack">
          <div className="row">
            <Code2 size={18} color="var(--accent)" />
            <strong>Exercícios & Bugfix</strong>
          </div>
          <p className="muted" style={{ margin: 0 }}>
            Desafios clássicos com compilação real, dicas progressivas e comparação idiomática.
          </p>
          <button type="button" className="btn" onClick={() => onNavigate('exercises')}>
            <Sparkles size={15} /> Praticar Desafios
          </button>
        </div>

        <div className="card stack">
          <div className="row">
            <FolderGit2 size={18} color="var(--accent)" />
            <strong>Modo Criar (Mini-IDE)</strong>
          </div>
          <p className="muted" style={{ margin: 0 }}>
            Crie projetos Cargo reais, jogos ConPTY, Mini Paint GUI e CaveGame 3D.
          </p>
          <button type="button" className="btn" onClick={() => onNavigate('create')}>
            <FolderGit2 size={15} /> Abrir Projetos
          </button>
        </div>
      </div>

      {overview ? (
        <div className="card stack">
          <h2 style={{ margin: 0, fontSize: '1.15rem' }}>Os 6 Estágios do Currículo Canônico</h2>
          <div className="grid-3">
            {overview.stages.map((st) => {
              const stagePct =
                st.totalCount > 0 ? Math.round((st.completedCount / st.totalCount) * 100) : 0;
              return (
                <div key={st.id} className="card-muted stack">
                  <div className="between">
                    <strong>{st.label}</strong>
                    <span className={`pill ${st.unlocked ? 'pill-ok' : 'pill-warn'}`}>
                      {st.completedCount}/{st.totalCount}
                    </span>
                  </div>
                  <div className="muted" style={{ fontSize: '0.84rem' }}>{st.subtitle}</div>
                  <div className="progress-bar-track" aria-hidden="true">
                    <div className="progress-bar-fill" style={{ width: `${stagePct}%` }} />
                  </div>
                  {st.entries[0] ? (
                    <button
                      type="button"
                      className="btn btn-sm"
                      onClick={() => onOpenLesson(st.entries[0].id)}
                    >
                      Ir para {st.entries[0].id}
                    </button>
                  ) : null}
                </div>
              );
            })}
          </div>
        </div>
      ) : null}
    </div>
  );
}
