import React, { useCallback, useEffect, useState } from 'react';
import {
  Award,
  BookOpen,
  Code2,
  EyeOff,
  FolderGit2,
  HelpCircle,
  Home,
  Library,
  Map,
  Settings,
  ShieldAlert,
  Stethoscope,
} from 'lucide-react';
import { formatIpcError, ipc } from './lib/ipc';
import type {
  AppRoute,
  CourseOverviewDto,
  MentorAskRequest,
  ProgressSummaryDto,
  SettingsDto,
  SettingsPatchDto,
  WindowsCapabilityReport,
} from './lib/types';
import { CourseMapView } from './components/CourseMapView';
import { CreateView } from './components/CreateView';
import { ExercisesView } from './components/ExercisesView';
import { HelpAboutView } from './components/HelpAboutView';
import { HomeView } from './components/HomeView';
import { LessonView } from './components/LessonView';
import { LibrariesDrawer } from './components/LibrariesDrawer';
import { MentorDrawer } from './components/MentorDrawer';
import { OnboardingModal } from './components/OnboardingModal';
import { ProgressView } from './components/ProgressView';
import { SettingsView } from './components/SettingsView';

const DEFAULT_SETTINGS: SettingsDto = {
  language: 'pt-BR',
  originLanguage: 'none',
  theme: 'cuia',
  desktopTheme: 'light',
  animations: 'full',
  gamificationVisible: true,
  focusMode: false,
  soundEnabled: true,
  soundVolume: 70,
  uiScale: 100,
  editorFontSize: 14,
  firstRunCompleted: true,
  execConsentGranted: false,
  aiProvider: 'disabled',
  ollamaUrl: 'http://127.0.0.1:11434',
  ollamaModel: 'llama3.2',
  remoteUrl: '',
  remoteModel: '',
  remoteConsent: 'unset',
  hasRemoteApiKey: false,
  dataRoot: '',
};

export function App(): React.ReactElement {
  const [route, setRoute] = useState<AppRoute>('home');
  const [activeLessonId, setActiveLessonId] = useState<string>('B-T01');
  const [previewUnlocked, setPreviewUnlocked] = useState(false);

  const [settings, setSettings] = useState<SettingsDto>(DEFAULT_SETTINGS);
  const [capabilities, setCapabilities] = useState<WindowsCapabilityReport | null>(null);
  const [overview, setOverview] = useState<CourseOverviewDto | null>(null);
  const [progress, setProgress] = useState<ProgressSummaryDto | null>(null);
  const [bootError, setBootError] = useState<string | null>(null);

  const [doctorOpen, setDoctorOpen] = useState(false);
  const [consentOpen, setConsentOpen] = useState(false);
  const [librariesOpen, setLibrariesOpen] = useState(false);
  const [librariesRoot, setLibrariesRoot] = useState<string | null>(null);
  const [mentorCtx, setMentorCtx] = useState<Omit<MentorAskRequest, 'action' | 'question'> | null>(
    null,
  );

  const resolveEffectiveTheme = (themePref: string): 'light' | 'dark' | 'high-contrast' => {
    if (themePref === 'high-contrast') return 'high-contrast';
    if (themePref === 'dark' || themePref === 'cuia') return 'dark';
    if (themePref === 'system') {
      return window.matchMedia?.('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return 'light';
  };

  const effectiveTheme = resolveEffectiveTheme(settings.desktopTheme);
  const isDarkTheme = effectiveTheme !== 'light';

  useEffect(() => {
    const root = document.documentElement;
    root.setAttribute('data-theme', effectiveTheme);
    root.setAttribute('data-motion', settings.animations === 'full' ? 'full' : 'reduced');
    root.style.setProperty('--ui-scale', `${(settings.uiScale || 100) / 100}`);
    root.style.setProperty('--editor-font-size', `${settings.editorFontSize || 14}px`);
  }, [effectiveTheme, settings.animations, settings.uiScale, settings.editorFontSize]);

  const refreshAll = useCallback(async () => {
    try {
      const [cfg, cap, ov, prog] = await Promise.all([
        ipc.getSettings(),
        ipc.getWindowsCapabilities(),
        ipc.getCourseOverview(previewUnlocked),
        ipc.getProgressSummary(),
      ]);
      setSettings(cfg);
      setCapabilities(cap);
      setOverview(ov);
      setProgress(prog);
      if (ov.lastEntryId) {
        setActiveLessonId((curr) => (curr === 'B-T01' ? ov.lastEntryId || curr : curr));
      }
      if (!cfg.firstRunCompleted) {
        setDoctorOpen(true);
      }
    } catch (err) {
      setBootError(formatIpcError(err));
    }
  }, [previewUnlocked]);

  useEffect(() => {
    void refreshAll();
  }, [refreshAll]);

  const refreshProgressAndCourse = useCallback(async () => {
    try {
      const [ov, prog] = await Promise.all([
        ipc.getCourseOverview(previewUnlocked),
        ipc.getProgressSummary(),
      ]);
      setOverview(ov);
      setProgress(prog);
    } catch (err) {
      setBootError(formatIpcError(err));
    }
  }, [previewUnlocked]);

  const patchSettings = async (patch: SettingsPatchDto) => {
    try {
      const next = await ipc.updateSettings(patch);
      setSettings(next);
      if (patch.language !== undefined) {
        await refreshProgressAndCourse();
      }
    } catch (err) {
      setBootError(formatIpcError(err));
    }
  };

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'F9') {
        e.preventDefault();
        setLibrariesOpen((prev) => !prev);
      } else if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'f') {
        e.preventDefault();
        void patchSettings({ focusMode: !settings.focusMode });
      } else if (e.key === 'Escape') {
        setLibrariesOpen(false);
        setMentorCtx(null);
        setConsentOpen(false);
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [settings.focusMode]);

  const openLesson = (id: string) => {
    setActiveLessonId(id);
    setRoute('lesson');
  };

  const navItems: { id: AppRoute; label: string; icon: React.ReactNode }[] = [
    { id: 'home', label: 'Início', icon: <Home size={16} /> },
    { id: 'course', label: 'Curso (213)', icon: <Map size={16} /> },
    { id: 'lesson', label: `Lição (${activeLessonId})`, icon: <BookOpen size={16} /> },
    { id: 'exercises', label: 'Exercícios', icon: <Code2 size={16} /> },
    { id: 'create', label: 'Modo Criar', icon: <FolderGit2 size={16} /> },
    { id: 'progress', label: 'Progresso', icon: <Award size={16} /> },
    { id: 'settings', label: 'Configurações', icon: <Settings size={16} /> },
    { id: 'help', label: 'Ajuda / Sobre', icon: <HelpCircle size={16} /> },
  ];

  return (
    <div className="app-shell" data-focus={settings.focusMode ? 'true' : 'false'}>
      <header className="app-header">
        <div className="brand-group">
          <strong>RustLab</strong>
          <span className="brand-badge">4.0 Desktop</span>
          {settings.gamificationVisible && !settings.focusMode && overview ? (
            <span className="pill pill-info">
              Nível {overview.level} · {overview.xp} XP · {overview.completedEntries}/
              {overview.totalEntries}
            </span>
          ) : null}
        </div>

        <div className="header-actions">
          <button
            type="button"
            className="btn btn-sm"
            onClick={() => {
              setLibrariesRoot(null);
              setLibrariesOpen(true);
            }}
          >
            <Library size={14} /> F9 Bibliotecas
          </button>
          <button type="button" className="btn btn-sm" onClick={() => setDoctorOpen(true)}>
            <Stethoscope size={14} /> Toolchain Doctor
          </button>
          <button
            type="button"
            className="btn btn-sm"
            onClick={() => void patchSettings({ focusMode: !settings.focusMode })}
          >
            <EyeOff size={14} /> {settings.focusMode ? 'Sair Modo Foco' : 'Modo Foco'}
          </button>
        </div>
      </header>

      {!settings.focusMode ? (
        <aside className="app-sidebar" aria-label="Navegação principal">
          <nav className="nav-list">
            {navItems.map((item) => (
              <button
                key={item.id}
                type="button"
                className="nav-item"
                aria-current={route === item.id ? 'page' : undefined}
                onClick={() => setRoute(item.id)}
              >
                {item.icon}
                <span>{item.label}</span>
              </button>
            ))}
          </nav>

          <div className="card-muted" style={{ fontSize: '0.8rem' }}>
            <div><strong>Edição:</strong> Windows Desktop</div>
            <div className="muted">GPL-3.0-only · 100% offline-first</div>
          </div>
        </aside>
      ) : null}

      <main className="app-main" id="main-content">
        {bootError ? (
          <div className="banner banner-danger">
            <span>{bootError}</span>
          </div>
        ) : null}

        {route === 'home' && (
          <HomeView
            overview={overview}
            capabilities={capabilities}
            onOpenLesson={openLesson}
            onNavigate={(r) => setRoute(r)}
            onOpenDoctor={() => setDoctorOpen(true)}
          />
        )}

        {route === 'course' && (
          <CourseMapView
            overview={overview}
            previewUnlocked={previewUnlocked}
            onTogglePreview={(next) => {
              setPreviewUnlocked(next);
            }}
            onOpenLesson={openLesson}
          />
        )}

        {route === 'lesson' && (
          <LessonView
            lessonId={activeLessonId}
            settings={settings}
            isDarkTheme={isDarkTheme}
            onSelectLesson={openLesson}
            onProgressUpdated={refreshProgressAndCourse}
            onOpenMentor={(ctx) => setMentorCtx(ctx)}
            onConsentRequired={() => setConsentOpen(true)}
          />
        )}

        {route === 'exercises' && (
          <ExercisesView
            settings={settings}
            isDarkTheme={isDarkTheme}
            onProgressUpdated={refreshProgressAndCourse}
            onOpenMentor={(ctx) => setMentorCtx(ctx)}
            onConsentRequired={() => setConsentOpen(true)}
          />
        )}

        {route === 'create' && (
          <CreateView
            isDarkTheme={isDarkTheme}
            onOpenLibraries={(rootDir) => {
              setLibrariesRoot(rootDir);
              setLibrariesOpen(true);
            }}
            onOpenMentor={(ctx) => setMentorCtx(ctx)}
            onConsentRequired={() => setConsentOpen(true)}
          />
        )}

        {route === 'progress' && (
          <ProgressView summary={progress} onOpenLesson={openLesson} />
        )}

        {route === 'settings' && (
          <SettingsView
            settings={settings}
            onPatch={patchSettings}
            onProgressUpdated={refreshProgressAndCourse}
          />
        )}

        {route === 'help' && (
          <HelpAboutView
            capabilities={capabilities}
            onRefreshCapabilities={async () => {
              const cap = await ipc.getWindowsCapabilities();
              setCapabilities(cap);
            }}
            onOpenDoctor={() => setDoctorOpen(true)}
          />
        )}
      </main>

      <OnboardingModal
        open={doctorOpen}
        capabilities={capabilities}
        settings={settings}
        onPatchSettings={patchSettings}
        onRefreshCapabilities={async () => {
          const cap = await ipc.getWindowsCapabilities();
          setCapabilities(cap);
        }}
        onClose={() => setDoctorOpen(false)}
      />

      <LibrariesDrawer
        open={librariesOpen}
        activeProjectRoot={librariesRoot}
        onClose={() => setLibrariesOpen(false)}
      />

      <MentorDrawer
        open={Boolean(mentorCtx)}
        context={mentorCtx}
        onClose={() => setMentorCtx(null)}
      />

      {consentOpen ? (
        <div className="modal-backdrop" role="dialog" aria-modal="true" aria-label="Consentimento de Execução Local">
          <div className="modal-card stack">
            <div className="row">
              <ShieldAlert size={20} color="var(--warn)" />
              <h2 style={{ margin: 0 }}>Consentimento para Compilar e Executar Código</h2>
            </div>
            <p>
              Executar <code>cargo check</code>, <code>cargo run</code> ou <code>cargo test</code>{' '}
              compila e roda código nativo na sua conta Windows atual — <strong>não é uma sandbox isolada</strong>.
            </p>
            <div className="row">
              <button
                type="button"
                className="btn btn-primary"
                onClick={() =>
                  void ipc.grantExecConsent().then(() => {
                    setSettings((s) => ({ ...s, execConsentGranted: true }));
                    setConsentOpen(false);
                  })
                }
              >
                Compreendo e Autorizo Execução Local
              </button>
              <button type="button" className="btn" onClick={() => setConsentOpen(false)}>
                Agora não
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
}
