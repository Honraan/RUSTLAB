//! Placeholder honesto para funcionalidades de fases futuras.
//!
//! Diz *o que virá*, *em qual fase*, e o que já funciona hoje — sem
//! simular a interface da feature (requisito 7: "não finja que etapas
//! inexistentes estão prontas").

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{App, Placeholder};
use crate::i18n::{t, tf};
use crate::ui::titled_block;

pub fn draw(frame: &mut Frame, app: &App, area: Rect, p: Placeholder) {
    let theme = &app.theme;
    let block = titled_block(t(p.title_key), theme, true);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height < 5 {
        return;
    }

    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(p.planned.len() as u16),
        Constraint::Length(2),
        Constraint::Min(1),
    ])
    .split(inner);

    // Cabeçalho "em construção" bem visível.
    let heading = Line::from(vec![
        Span::styled(if app.unicode { " ◐ " } else { " (o) " }, theme.warning()),
        Span::styled(
            tf("placeholder.phase", &[&p.phase.to_string()]),
            theme.warning(),
        ),
    ]);
    let summary = Line::from(Span::styled(p.summary, theme.dim()));
    frame.render_widget(Paragraph::new(vec![heading, summary]), rows[0]);

    let planned = p
        .planned
        .iter()
        .map(|line| Line::from(Span::styled(format!("   · {line}"), theme.base())))
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(planned), rows[2]);

    let mut notes = vec![Line::from(Span::styled(
        t("placeholder.footer"),
        theme.dim(),
    ))];
    notes.push(Line::from(Span::styled(
        "Gamificação, som e progresso ficam em Meu Progresso/Configurações.",
        theme.dim(),
    )));
    frame.render_widget(Paragraph::new(notes), rows[4]);
}
