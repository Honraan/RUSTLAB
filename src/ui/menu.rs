//! Menu principal: o hub entre os modos Aprender/Criar e as telas do app.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, List, ListState};

use crate::app::{App, MenuAction};
use crate::ui::titled_block;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let border = if app.unicode {
        BorderType::Rounded
    } else {
        BorderType::Plain
    };

    let rows = Layout::vertical([
        Constraint::Min(7),
        Constraint::Length(if app.unicode { 4 } else { 6 }),
    ])
    .split(area);

    // Lista de itens
    let items: Vec<Line> = crate::app::MENU_ITEMS
        .iter()
        .map(|action| item_line(app, action))
        .collect();

    let list = List::new(items)
        .highlight_symbol(if app.unicode { "▶ " } else { "> " })
        .highlight_style(theme.selection());

    let menu_title = format!("{} v{}", app.t("menu.title"), env!("CARGO_PKG_VERSION"));
    let block = titled_block(&menu_title, theme, true).border_type(border);
    let inner = block.inner(rows[0]);
    frame.render_widget(block, rows[0]);
    let mut state = ListState::default().with_selected(Some(app.menu_index));
    frame.render_stateful_widget(list, inner, &mut state);

    // Cartão de contexto da seleção: o que espera o aluno ali.
    draw_context_card(frame, app, rows[1]);

    // Dica inicial para quem acabou de chegar — colocada no vazio abaixo
    // da lista, nunca sobre os itens.
    if app.menu_index == 0 && app.status.is_none() && rows[0].height >= 9 {
        let hint = Line::from(Span::styled(
            format!(
                "{} — {}",
                app.t("menu.learn"),
                app.bi("comece por aqui :)", "start here :)")
            ),
            theme.accent_dim,
        ));
        let hint_area = Rect {
            x: rows[0].x,
            y: rows[0].bottom() - 1,
            width: rows[0].width,
            height: 1,
        };
        frame.render_widget(ratatui::widgets::Paragraph::new(hint).centered(), hint_area);
    }
}

fn item_line(app: &App, action: &MenuAction) -> Line<'static> {
    let theme = &app.theme;
    let (glyph, label, note) = match action {
        MenuAction::Learn => (
            "◆",
            app.t("menu.learn"),
            app.bi("currículo guiado", "guided curriculum"),
        ),
        MenuAction::Create => (
            "◆",
            app.t("menu.create"),
            app.bi("mini-IDE com Cargo", "Cargo mini-IDE"),
        ),
        MenuAction::Progress => (
            "◆",
            app.t("menu.progress"),
            app.bi("lições, exercícios e dicas", "lessons, exercises & hints"),
        ),
        MenuAction::Settings => ("⚙", app.t("menu.settings"), ""),
        MenuAction::Help => ("?", app.t("menu.help"), ""),
        MenuAction::Quit => ("x", app.t("menu.quit"), ""),
    };
    let glyph = if app.unicode {
        glyph
    } else {
        ascii_glyph(glyph)
    };
    let mut spans = vec![
        Span::styled(format!("  {glyph} "), theme.accent),
        Span::styled(label, theme.base()),
    ];
    if !note.is_empty() {
        spans.push(Span::styled(format!("   — {note}"), theme.dim()));
    }
    Line::from(spans)
}

fn ascii_glyph(glyph: &str) -> &'static str {
    match glyph {
        "◆" => "*",
        "⚙" => "#",
        _ => " ",
    }
}

/// Um cartão honesto sobre o que cada item realmente contém hoje.
/// O texto vive *dentro* da moldura: em janelas estreitas é recortado com
/// elegância, sem invadir as bordas.
fn draw_context_card(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let action = app.menu_action();
    let text: Line = match action {
        MenuAction::Learn => Line::from(Span::styled(
            app.bi(
                "Currículo por etapas — comece pela Etapa 1; progresso desta sessão.",
                "Step-by-step curriculum — start at Stage 1; session progress.",
            ),
            theme.dim(),
        )),
        MenuAction::Create => Line::from(Span::styled(
            app.bi(
                "Editor multilinha e comandos Cargo reais; execução exige consentimento.",
                "Multi-line editor and real Cargo commands; execution asks consent.",
            ),
            theme.dim(),
        )),
        MenuAction::Progress => {
            if app.config.ui.gamification == crate::config::GamificationVisibility::Visible {
                Line::from(format!(
                    "{} {} · {} XP · {} {}",
                    app.bi("Nível", "Level"),
                    app.level(),
                    app.xp(),
                    app.achievement_count(),
                    app.bi("marcos", "milestones")
                ))
            } else {
                Line::from(app.bi(
                    "Resumo de lições e exercícios; gamificação oculta nas configurações.",
                    "Lessons and exercises summary; gamification hidden in settings.",
                ))
            }
        }
        MenuAction::Settings => Line::from(Span::styled(
            app.bi(
                "Aparência, movimento, som e IA. Tudo se aplica e se salva na hora.",
                "Appearance, motion, sound, language, and AI. Applies and saves immediately.",
            ),
            theme.dim(),
        )),
        MenuAction::Help => Line::from(Span::styled(
            app.bi(
                "Atalhos de teclado e o estado atual do projeto.",
                "Keyboard shortcuts and current project status.",
            ),
            theme.dim(),
        )),
        MenuAction::Quit => Line::from(Span::styled(
            app.bi(
                "Até logo! O terminal volta ao normal.",
                "See you later! Terminal state will be restored.",
            ),
            theme.dim(),
        )),
    };
    let block = Block::bordered()
        .title(" · ")
        .border_type(if app.unicode {
            BorderType::Rounded
        } else {
            BorderType::Plain
        })
        .border_style(theme.border_style(false));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(ratatui::widgets::Paragraph::new(text.centered()), inner);
}
