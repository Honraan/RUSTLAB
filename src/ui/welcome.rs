//! Tela de boas-vindas: identidade textual e cortina de entrada.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let block = Block::bordered()
        .title(Line::from(Span::styled(" RustLab ", theme.title())))
        .border_style(theme.border_style(true));

    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height < 4 || inner.width < 24 {
        return;
    }

    let rows = Layout::vertical([Constraint::Min(2), Constraint::Length(1)]).split(inner);

    // --- logotipo / tagline ---------------------------------------------
    let logo = Paragraph::new(vec![
        Line::from(Span::styled("c(o.o)c", theme.title())),
        Line::from(""),
        Line::from(Span::styled(
            "R U S T L A B",
            theme.title().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(app.t("app.tagline"), theme.dim())),
        Line::from(Span::styled(app.t("welcome.caption"), theme.dim())),
    ])
    .centered();
    frame.render_widget(logo, rows[0]);

    // --- prompt ----------------------------------------------------------
    let prompt = Paragraph::new(Line::from(Span::styled(
        format!("· {} ·", app.t("welcome.prompt")),
        theme.accent_dim,
    )))
    .centered();
    frame.render_widget(prompt, rows[1]);

    // --- rodapé de contexto: versão + idioma (requisito 14) -------------
    let meta = Paragraph::new(Span::styled(
        format!("v{} · {}", env!("CARGO_PKG_VERSION"), app.config.language),
        theme.dim(),
    ));
    let corner = Rect {
        x: area.x + 2,
        y: area.y + area.height.saturating_sub(1),
        width: area.width.saturating_sub(4).min(30),
        height: 1,
    };
    frame.render_widget(meta, corner);
}
