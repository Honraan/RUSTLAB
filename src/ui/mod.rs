//! Camada de desenho: decide o layout, delega por tela e aplica os
//! efeitos do motor de animação ao buffer já renderizado.

mod browser;
mod create;
mod exercises;
mod help;
mod lesson;
mod mentor;
mod menu;
mod placeholder;
pub(crate) mod progress;
mod settings;
mod welcome;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph, Wrap};

use crate::app::{App, Screen, StatusKind};
use crate::config::ThemeId;
use crate::theme::Theme;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Terminal ridiculamente pequeno: mensagem em vez de layout quebrado.
    if area.width < 40 || area.height < 8 {
        let tiny = Theme::for_id(ThemeId::Cuia);
        let p = Paragraph::new(app.bi(
            "Terminal muito pequeno.\nAmplie a janela (mín. 40x8).",
            "Terminal too small.\nResize window (min 40x8).",
        ))
        .style(tiny.error());
        frame.render_widget(p, area);
        // IMPORTANTE: mesmo aqui os efeitos precisam avançar — se
        // saltássemos o tick, um efeito em andamento nunca completaria e
        // o loop de eventos giraria a 40ms esperando o fim da animação.
        app.anim.tick(frame.buffer_mut(), area);
        return;
    }

    // Estrutura comum: conteúdo + barra de status opcional + rodapé.
    let show_status = app.status.is_some() || app.load_warning.is_some();
    let footer_height = footer(app)
        .line_count(area.width)
        .max(1)
        .min(area.height.saturating_sub(6).max(1) as usize) as u16;
    let column = Layout::vertical([
        Constraint::Min(3),
        Constraint::Length(if show_status { 1 } else { 0 }),
        Constraint::Length(footer_height),
    ])
    .split(area);

    match app.screen {
        Screen::Welcome => welcome::draw(frame, app, column[0]),
        Screen::Menu => menu::draw(frame, app, column[0]),
        Screen::Settings => settings::draw(frame, app, column[0]),
        Screen::Help => help::draw(frame, app, column[0]),
        Screen::Browser => browser::draw(frame, app, column[0]),
        Screen::LessonView => lesson::draw(frame, app, column[0]),
        Screen::Create => create::draw(frame, app, column[0]),
        Screen::Exercises => exercises::draw(frame, app, column[0]),
        Screen::Progress => progress::draw(frame, app, column[0]),
        Screen::Mentor => mentor::draw(frame, app, column[0]),
        Screen::Placeholder(p) => placeholder::draw(frame, app, column[0], p),
    }

    if show_status {
        draw_status_bar(frame, app, column[1]);
    }
    draw_footer(frame, app, column[2]);

    // Animações por cima do conteúdo já renderizado (conceito do TachyonFX:
    // widgets primeiro; efeitos transformam as células depois).
    app.anim.tick(frame.buffer_mut(), area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    if area.height == 0 {
        return;
    }
    let theme = &app.theme;
    // O aviso de config tem prioridade sobre status momentâneos: é mais
    // importante o usuário saber que as preferências dele não carregaram.
    let (msg, style) = match (&app.load_warning, &app.status) {
        (Some(w), _) => (w.clone(), theme.warning()),
        (None, Some(s)) => {
            let style = match s.kind {
                StatusKind::Info => theme.dim(),
                StatusKind::Success => theme.success(),
                StatusKind::Warning => theme.warning(),
            };
            (s.message.clone(), style)
        }
        (None, None) => return,
    };
    frame.render_widget(Paragraph::new(format!("  {msg}")).style(style), area);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    frame.render_widget(footer(app), area);
}

fn footer(app: &App) -> Paragraph<'static> {
    let theme = &app.theme;
    let keys: &[&str] = match app.screen {
        Screen::Welcome => &[app.t("footer.quit"), app.t("welcome.prompt")],
        Screen::Menu => &[
            app.t("footer.nav"),
            app.t("footer.enter"),
            app.t("footer.quit"),
        ],
        Screen::Settings => &[
            app.t("footer.nav"),
            app.bi("←/→ ou h/l ajustar", "←/→ or h/l adjust"),
            app.t("footer.esc"),
        ],
        Screen::Browser => {
            if app.browser.v3_mode {
                &[
                    app.t("footer.nav"),
                    app.t("footer.enter"),
                    "Tab trocar · v 10 caps · Alt+V prévia · r revisar",
                    app.t("footer.esc"),
                ]
            } else {
                &[
                    app.t("footer.nav"),
                    app.t("footer.enter"),
                    "Tab trocar · e exercícios · v Trilha V3 · r revisar",
                    app.t("footer.esc"),
                ]
            }
        }
        Screen::LessonView => {
            if app.lesson_view.as_ref().is_some_and(|v| v.practice_focused) {
                &[
                    "F5 Executar",
                    "Ctrl+S Salvar",
                    "F1..F3 Dicas",
                    "Tab Leitura/Quiz",
                    "Esc Voltar",
                ]
            } else {
                &[
                    "Tab/e praticar",
                    "↑↓ ler",
                    "1-9 responder",
                    "d demo",
                    "m mentor",
                    "←/→ lição",
                    app.t("footer.esc"),
                ]
            }
        }
        Screen::Create => match app.create.as_ref().map(|state| state.view) {
            Some(crate::create::CreateView::Start) => &[
                "↑↓ escolher ação",
                "d alterar pasta",
                "Enter confirmar/abrir",
                "Esc voltar ao menu",
            ],
            Some(crate::create::CreateView::Workspace) => &[
                "F5 Executar",
                "Ctrl+S Salvar",
                "F4 Compilar (Linux/Win)",
                "F6 Testar",
                "Alt+A Agente IA",
                "F9 Bibliotecas (▼)",
                "F1 Doc/Ajuda",
                "Shift+Tab Foco",
                "Esc Fechar projeto",
            ],
            None => &[app.t("footer.esc")],
        },
        Screen::Exercises if app.exercises.as_ref().is_some_and(|s| s.consent_pending) => &[
            "Enter autorizar execução local",
            "Esc recusar (nada será executado)",
        ],
        Screen::Exercises
            if app
                .exercises
                .as_ref()
                .is_some_and(|s| s.view == crate::exercises::ExerciseView::List) =>
        {
            &["↑↓ escolher", "Enter abrir/retomar", "Esc voltar"]
        }
        Screen::Exercises => &[
            "F5 testar",
            "F1 dica",
            "F2/F3 rever dicas",
            "F4/Shift+F4 rolar dica",
            "F9/Shift+F9 enunciado",
            "PgUp/PgDn saída",
            "Ctrl+S salvar",
            "Ctrl+K cancelar",
            "Esc voltar",
        ],
        Screen::Help | Screen::Placeholder(_) => &[app.t("footer.esc")],
        Screen::Progress => &[
            "↑↓ rolar",
            "Home início",
            "e exercícios",
            "m mentor",
            "Esc menu",
        ],
        Screen::Mentor => &[
            "1–5 ação",
            "6/p pergunta livre",
            "o/m modelo Ollama",
            "Enter executar",
            "PgUp/PgDn rolar",
            "Ctrl+K cancelar",
            "Esc/Ctrl+T voltar",
        ],
    };
    let mut spans = Vec::new();
    if !matches!(app.screen, Screen::Create) {
        spans.push(Span::styled(
            format!(" {} ", app.t("menu.title")),
            theme.dim(),
        ));
    }
    for key in keys {
        spans.push(Span::styled(format!(" · {key} "), theme.dim()));
    }
    // Sem borda: inclusive um rodapé de uma linha deve ter área útil.
    Paragraph::new(Line::from(spans))
        .wrap(Wrap { trim: true })
        .style(Style::default())
}

/// Moldura arredondada com título, reutilizada por todas as telas.
pub fn titled_block<'a>(title: &str, theme: &Theme, focused: bool) -> Block<'a> {
    Block::bordered()
        .title(format!(" {title} "))
        .title_style(theme.title())
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style(focused))
        .style(Style::default())
}

#[cfg(test)]
pub(crate) mod tests_util {
    use crate::app::App;
    use crate::audio::{AudioBackend, AudioReport};
    use crate::config::{Config, MotionMode};

    /// Despeja o buffer em texto (ratatui 0.30 não implementa Display em Buffer).
    pub fn buffer_text(buffer: &ratatui::buffer::Buffer) -> String {
        let mut out = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                out.push_str(buffer[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    /// App isolado para testes: sem animação, sem escrever config real.
    pub fn test_app() -> App {
        let mut app = App::from_config(
            Config {
                language: "pt-BR".to_string(),
                ..Config::default()
            },
            None,
            true,
            AudioReport {
                backend: AudioBackend::None,
                detail: String::new(),
            },
        );
        app.anim.set_mode(MotionMode::Off);
        app.set_config_path(
            std::env::temp_dir().join(format!("rustlab-ui-test-{}.toml", std::process::id())),
        );
        app.set_storage_path(
            std::env::temp_dir().join(format!("rustlab-ui-progress-{}.toml", std::process::id())),
        );
        app
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;
    use crate::app::App;

    fn harness(w: u16, h: u16) -> (Terminal<TestBackend>, App) {
        let terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        (terminal, super::tests_util::test_app())
    }

    fn render(terminal: &mut Terminal<TestBackend>, app: &mut App) -> String {
        terminal.draw(|f| draw(f, app)).unwrap();
        super::tests_util::buffer_text(terminal.backend().buffer())
    }

    #[test]
    fn smoke_render_por_tela_e_tamanho() {
        use crate::i18n::t;
        for (w, h) in [(90, 30), (60, 14), (42, 9)] {
            let (mut terminal, mut app) = harness(w, h);
            // Em todos os tamanhos desenhar nunca pode entrar em pânico.
            // Abaixo de ~12 linhas as telas compactam de propósito, então
            // âncoras fundo-acima só valem onde há espaço para elas.
            let roomy = h >= 12;
            // Bem-vindo
            let content = render(&mut terminal, &mut app);
            assert!(content.contains("RustLab"), "welcome {w}x{h}");
            // Menu
            app.screen = Screen::Menu;
            let content = render(&mut terminal, &mut app);
            assert!(content.contains(t("menu.learn")), "menu {w}x{h}");
            // Configurações
            app.screen = Screen::Settings;
            let content = render(&mut terminal, &mut app);
            assert!(content.contains(t("settings.title")), "settings {w}x{h}");
            // Ajuda — a tabela de atalhos é o conteúdo essencial.
            app.screen = Screen::Help;
            let content = render(&mut terminal, &mut app);
            if roomy {
                assert!(content.contains("Ctrl+C"), "help {w}x{h}");
            }
            // Placeholder
            app.screen = Screen::Create;
            app.create = Some(crate::create::CreateState::new());
            let content = render(&mut terminal, &mut app);
            assert!(content.contains("projeto") || !roomy, "create {w}x{h}");
            // Mentor
            app.mentor = Some(crate::mentor::MentorState::open(
                Screen::Menu,
                crate::mentor::MentorContext::default(),
            ));
            app.screen = Screen::Mentor;
            let content = render(&mut terminal, &mut app);
            assert!(content.contains("Mentor") || !roomy, "mentor {w}x{h}");
        }
    }

    #[test]
    fn terminal_minimo_mostra_aviso() {
        let (mut terminal, mut app) = harness(30, 5);
        let content = render(&mut terminal, &mut app);
        assert!(content.contains("muito pequeno"));
    }

    #[test]
    fn status_bar_aparece_so_quando_ha_algo() {
        let (mut terminal, mut app) = harness(80, 24);
        let without = render(&mut terminal, &mut app);
        app.set_status("teste de status", StatusKind::Info);
        let with = render(&mut terminal, &mut app);
        assert!(!without.contains("teste de status"));
        assert!(with.contains("teste de status"));
    }
}
