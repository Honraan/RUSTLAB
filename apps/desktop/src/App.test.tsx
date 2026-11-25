import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { SafeAnsiOutput } from './lib/safeRender';
import { MentorDrawer } from './components/MentorDrawer';
import { App } from './App';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => invokeMock(cmd, args),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => {}),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(async () => null),
  save: vi.fn(async () => null),
}));

const STAGES_213 = [
  { id: 'basico', label: 'Básico', subtitle: 'Fundamentos', unlocked: true, completedCount: 0, totalCount: 36 },
  { id: 'meio', label: 'Meio', subtitle: 'Ownership', unlocked: false, completedCount: 0, totalCount: 42 },
  { id: 'dificil', label: 'Difícil', subtitle: 'Traits e Erros', unlocked: false, completedCount: 0, totalCount: 40 },
  { id: 'fim', label: 'Fim', subtitle: 'Projetos TUI', unlocked: false, completedCount: 0, totalCount: 41 },
  { id: 'mini-paint', label: 'Mini Paint', subtitle: 'GUI 2D', unlocked: false, completedCount: 0, totalCount: 24 },
  { id: 'cave-game', label: 'CaveGame', subtitle: 'Voxel Bevy', unlocked: false, completedCount: 0, totalCount: 30 },
].map((s, si) => ({
  ...s,
  entries: [
    {
      id: si === 0 ? 'B-T01' : `S${si}-T01`,
      title: si === 0 ? 'O que é programar?' : `Lição ${s.label}`,
      stageId: s.id,
      kind: 'Teoria',
      cycle: null,
      prerequisites: [],
      learningGoal: 'Compreender fundamentos reais.',
      completionEvidence: 'Quiz',
      unlocked: si === 0,
      completed: false,
      read: false,
      isInteractiveStdin: false,
    },
  ],
}));

describe('RustLab Desktop V4 UI', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockImplementation(async (cmd: string) => {
      switch (cmd) {
        case 'get_settings':
          return {
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
            execConsentGranted: true,
            aiProvider: 'disabled',
            ollamaUrl: 'http://127.0.0.1:11434',
            ollamaModel: 'llama3.2',
            remoteUrl: '',
            remoteModel: '',
            remoteConsent: 'unset',
            hasRemoteApiKey: false,
            dataRoot: 'C:\\Users\\demo\\AppData\\Local\\RustLab',
          };
        case 'get_windows_capabilities':
          return {
            osCaption: 'Microsoft Windows 11',
            osBuild: '22631',
            arch: 'x86_64',
            webview2Version: '158.0.0.0',
            offlineCurriculumReady: true,
            rustupFound: true,
            rustcVersion: 'rustc 1.100.0',
            cargoVersion: 'cargo 1.100.0',
            cargoPath: 'C:\\Users\\demo\\.cargo\\bin\\cargo.exe',
            defaultHostTriple: 'x86_64-pc-windows-msvc',
            rustfmtAvailable: true,
            clippyAvailable: true,
            rustAnalyzerAvailable: true,
            msvcLinkerAvailable: true,
            gnuLinkerAvailable: false,
            windowsSdkAvailable: true,
            canCompileNative: true,
            conptySupported: true,
            recommendedActionPt: 'Ambiente pronto para compilar.',
            copyableFixes: [],
          };
        case 'get_course_overview':
          return {
            totalEntries: 213,
            completedEntries: 0,
            xp: 0,
            level: 1,
            lastEntryId: 'B-T01',
            dueSrsItems: [],
            stages: STAGES_213,
          };
        case 'get_progress_summary':
          return {
            xp: 0,
            level: 1,
            completedV3: 0,
            totalV3: 213,
            completedExercises: 0,
            independentCompletions: 0,
            dueSrsItems: [],
            topErrors: [],
            stageProgress: STAGES_213.map((s) => ({
              id: s.id,
              label: s.label,
              completed: 0,
              total: s.totalCount,
            })),
            achievements: [],
          };
        case 'get_lesson_detail':
          return {
            id: 'B-T01',
            title: 'O que é programar?',
            stageId: 'basico',
            stageLabel: 'Básico',
            kind: 'Teoria',
            completed: false,
            requiresQuiz: true,
            ferrisMood: 'neutral',
            ferrisAscii: 'c(o.o)c',
            blocks: [
              { type: 'text', text: 'Programar é dar instruções exatas ao computador.' },
              {
                type: 'demo',
                demo: 'Sequencia',
                name: 'Fluxo de instruções',
                steps: ['Passo 1: Entrada', 'Passo 2: Saída'],
                caption: 'Execução passo a passo',
              },
              {
                type: 'quiz',
                quizIndex: 0,
                question: 'O que faz println!?',
                options: ['Imprime no terminal', 'Apaga o arquivo'],
                correct: 0,
                why: 'println! escreve texto na saída padrão.',
              },
            ],
            practice: null,
            prevEntryId: null,
            nextEntryId: 'B-P01',
          };
        case 'submit_quiz_answer':
          return {
            correct: true,
            explanation: 'Exato! println! escreve na saída padrão.',
            lessonCompleted: true,
            xpAwarded: 40,
            totalXp: 40,
            level: 1,
            ferrisMood: 'happy',
            ferrisAscii: 'c(ovo)c',
          };
        case 'list_libraries':
          return [
            {
              name: 'rand',
              version: '0.8',
              category: 'Aleatoriedade',
              summary: 'Geração de números aleatórios.',
              docsPt: 'Ideal para jogos.',
              useLine: 'use rand::Rng;',
              docUrl: 'https://docs.rs/rand',
              associatedV3Lessons: ['M-P01'],
              commonPitfalls: 'Seed fixa para testes.',
              windowsNotes: 'Pure Rust.',
              installedInProject: false,
              versionNote: null,
            },
          ];
        case 'mentor_ask':
          return {
            source: 'Mentor Local (Offline)',
            body: 'Observe que variáveis são imutáveis por padrão em Rust.',
            readOnlyEnforced: true,
            proposedCode: null,
            probabilisticNotice: 'Modo somente leitura.',
          };
        case 'update_settings':
          return {
            language: 'pt-BR',
            originLanguage: 'none',
            theme: 'cuia',
            desktopTheme: 'light',
            animations: 'full',
            gamificationVisible: true,
            focusMode: true,
            soundEnabled: true,
            soundVolume: 70,
            uiScale: 100,
            editorFontSize: 14,
            firstRunCompleted: true,
            execConsentGranted: true,
            aiProvider: 'disabled',
            ollamaUrl: 'http://127.0.0.1:11434',
            ollamaModel: 'llama3.2',
            remoteUrl: '',
            remoteModel: '',
            remoteConsent: 'unset',
            hasRemoteApiKey: false,
            dataRoot: 'C:\\Users\\demo\\AppData\\Local\\RustLab',
          };
        default:
          return null;
      }
    });
  });

  it('renderiza Início em pt-BR, abre B-T01 e responde quiz com Ferris feliz', async () => {
    render(<App />);
    const cta = await screen.findByRole('button', { name: /Continuar aprendendo \(B-T01\)/i });
    expect(screen.getByText(/Os 6 Estágios do Currículo Canônico/i)).toBeInTheDocument();

    fireEvent.click(cta);
    expect(await screen.findByText(/O que é programar\?/i)).toBeInTheDocument();
    expect(screen.getByText(/c\(o\.o\)c/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: /Imprime no terminal/i }));
    expect(await screen.findByText(/Resposta correta!/i)).toBeInTheDocument();
    expect(await screen.findByText(/c\(ovo\)c/)).toBeInTheDocument();
  });

  it('mostra os 6 estágios e 213 entradas no Mapa do Curso', async () => {
    render(<App />);
    const navButtons = await screen.findAllByRole('button', { name: /^Curso \(213\)$/i });
    fireEvent.click(navButtons[0]);

    expect(await screen.findByText(/Mapa do Curso \(0\/213 entradas\)/i)).toBeInTheDocument();
    expect(screen.getByRole('region', { name: 'Mini Paint' })).toBeInTheDocument();
    expect(screen.getByRole('region', { name: 'CaveGame' })).toBeInTheDocument();
  });

  it('escapa HTML malicioso na saída ANSI e não injeta script', () => {
    const { container } = render(
      <SafeAnsiOutput lines={['\x1b[31merro:\x1b[0m <script>alert(1)</script>']} />,
    );
    expect(container.querySelector('script')).toBeNull();
    expect(screen.getByText('<script>alert(1)</script>')).toBeInTheDocument();
  });

  it('aplica política somente leitura no MentorDrawer quando learnMode=true', async () => {
    render(
      <MentorDrawer
        open
        context={{
          learnMode: true,
          origin: 'Lição B-P02',
          topicTitle: 'Variáveis',
          summary: 'Mutabilidade',
          code: 'let x = 1;',
          errors: '',
          console: '',
        }}
        onClose={() => {}}
      />,
    );
    expect(screen.getByText(/Modo Aprender \(Somente Leitura\)/i)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /Explicar conceito/i }));
    await waitFor(() => {
      expect(screen.getByText(/Somente leitura ativo/i)).toBeInTheDocument();
    });
    expect(screen.queryByRole('button', { name: /Aplicar/i })).toBeNull();
  });

  it('abre bibliotecas com F9 e ativa Modo Foco pelo teclado', async () => {
    render(<App />);
    await screen.findByRole('button', { name: /Continuar aprendendo/i });
    fireEvent.keyDown(window, { key: 'F9' });
    expect(await screen.findByRole('dialog', { name: /Catálogo F9 de Bibliotecas/i })).toBeInTheDocument();
    fireEvent.keyDown(window, { key: 'Escape' });
    fireEvent.keyDown(window, { key: 'f', ctrlKey: true, shiftKey: true });
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('update_settings', {
        patch: { focusMode: true },
      });
    });
  });
});
