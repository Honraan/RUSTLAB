//! Tela do Mentor opcional de IA (Fase 8): ações pedagógicas, resposta
//! rolável, pergunta livre e consentimento explícito para API remota.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::App;
use crate::mentor::{MentorAction, MentorState, ModelPickerState};
use crate::ui::titled_block;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let Some(state) = &app.mentor else {
        frame.render_widget(
            Paragraph::new("Mentor não inicializado. Pressione Ctrl+T para abrir.")
                .block(titled_block("Mentor", theme, true)),
            area,
        );
        return;
    };

    let rows = Layout::vertical([Constraint::Length(4), Constraint::Min(8)]).split(area);

    let ctx_lines = vec![
        Line::from(vec![
            Span::styled("Contexto: ", theme.title()),
            Span::styled(state.context.origin.as_str(), theme.base()),
        ]),
        Line::from(Span::styled(
            "O mentor explica conceitos e dá pistas, mas nunca substitui `cargo check` / `cargo test`.",
            theme.dim(),
        )),
    ];
    frame.render_widget(
        Paragraph::new(ctx_lines)
            .wrap(Wrap { trim: true })
            .block(titled_block("Mentor RustLab", theme, false)),
        rows[0],
    );

    let cols =
        Layout::horizontal([Constraint::Percentage(36), Constraint::Percentage(64)]).split(rows[1]);

    let items: Vec<ListItem> = MentorAction::ALL
        .iter()
        .enumerate()
        .map(|(i, act)| {
            let selected = i == state.selected_action;
            let style = if selected {
                theme.selection()
            } else {
                theme.base()
            };
            ListItem::new(format!("[{}] {}", act.shortcut_digit(), act.label())).style(style)
        })
        .collect();
    let mut list_state = ListState::default().with_selected(Some(state.selected_action));
    let actions_panel =
        Layout::vertical([Constraint::Min(8), Constraint::Length(4)]).split(cols[0]);
    frame.render_stateful_widget(
        List::new(items)
            .highlight_symbol(if app.unicode { "▶ " } else { "> " })
            .block(titled_block("Ações (1–6 · Enter)", theme, true)),
        actions_panel[0],
        &mut list_state,
    );
    let current_desc = MentorAction::ALL[state.selected_action].description();
    frame.render_widget(
        Paragraph::new(current_desc)
            .style(theme.dim())
            .wrap(Wrap { trim: true })
            .block(titled_block("Sobre a ação", theme, false)),
        actions_panel[1],
    );

    draw_response(frame, app, state, cols[1]);

    if state.model_picker.open {
        draw_model_picker(
            frame,
            &app.theme,
            &state.model_picker,
            &app.config.ai.ollama_model,
            area,
        );
    } else if state.asking_custom {
        draw_custom_question(frame, state, area, app);
    } else if state.consent_pending {
        draw_remote_consent(frame, app, area);
    }
}

fn draw_response(frame: &mut Frame, app: &App, state: &MentorState, area: Rect) {
    let theme = &app.theme;
    let mut lines = vec![
        Line::from(vec![
            Span::styled(state.response_title.as_str(), theme.title()),
            Span::styled(format!(" · {}", state.response_source), theme.dim()),
        ]),
        Line::from(""),
    ];
    if let Some(err) = &state.last_error {
        lines.push(Line::styled(format!("Aviso: {err}"), theme.warning()));
        lines.push(Line::from(""));
    }
    for line in &state.response_lines {
        lines.push(Line::styled(line.as_str(), theme.base()));
    }
    let block = titled_block("Resposta do Mentor", theme, !state.manager.running);
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((state.scroll, 0))
            .block(block),
        area,
    );
}

fn draw_custom_question(frame: &mut Frame, state: &MentorState, area: Rect, app: &App) {
    let theme = &app.theme;
    let popup = centered(area, 72, 7);
    frame.render_widget(Clear, popup);
    let msg = format!(
        "Digite sua dúvida sobre Rust ou sobre o contexto atual:\n> {}_\nEnter envia · Esc cancela",
        state.custom_input
    );
    frame.render_widget(
        Paragraph::new(msg).wrap(Wrap { trim: true }).block(
            Block::bordered()
                .title(" Pergunta ao Mentor ")
                .border_style(theme.border_style(true)),
        ),
        popup,
    );
}

fn draw_remote_consent(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let popup = centered(area, 74, 11);
    frame.render_widget(Clear, popup);
    let msg = format!(
        "Consentimento para API remota ({})\n\nCódigo e mensagens de erro do contexto atual serão enviados para:\n{}\n\nNenhum dado foi enviado ainda.\nEnter / s autoriza envio remoto · Esc / n recusa e mantém guia local",
        app.config.ai.remote_model, app.config.ai.remote_url
    );
    frame.render_widget(
        Paragraph::new(msg)
            .style(theme.warning())
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .title(" Consentimento de IA Remota ")
                    .border_style(theme.border_style(true)),
            ),
        popup,
    );
}

/// Overlay compartilhado de seleção de modelo Ollama (usado pelo Mentor e pelo Agente no Modo Criar).
pub fn draw_model_picker(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    picker: &ModelPickerState,
    active_model: &str,
    area: Rect,
) {
    let list_len = picker.candidates.len().min(10) as u16;
    let popup = centered(area, 64, (list_len + 8).max(11));
    frame.render_widget(Clear, popup);

    let mut lines: Vec<Line> = Vec::new();
    let source = if picker.online {
        "Ollama local conectado (/api/tags)"
    } else {
        "Ollama offline — sugestões rápidas ou digite manualmente"
    };
    lines.push(Line::styled(source, theme.dim()));
    if picker.typing_custom {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            format!("Nome do modelo: > {}_", picker.custom_input),
            theme.title(),
        ));
        lines.push(Line::from(""));
        lines.push(Line::styled(
            "Enter confirma e ativa Ollama · Esc volta para a lista",
            theme.dim(),
        ));
    } else {
        for (idx, model) in picker.candidates.iter().enumerate() {
            let selected = idx == picker.selected;
            let active_mark = if model == active_model {
                " (ativo)"
            } else {
                ""
            };
            lines.push(Line::styled(
                format!("{}{model}{active_mark}", if selected { "▶ " } else { "  " }),
                if selected {
                    theme.selection()
                } else {
                    theme.base()
                },
            ));
        }
        lines.push(Line::from(""));
        lines.push(Line::styled(
            "↑↓ escolher · Enter ativar · t digitar outro · r atualizar · Esc fechar",
            theme.dim(),
        ));
    }

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::bordered()
                .title(" Escolher modelo local (Ollama) ")
                .border_style(theme.border_style(true)),
        ),
        popup,
    );
}

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let width = w.min(area.width.saturating_sub(2)).max(24);
    let height = h.min(area.height.saturating_sub(2)).max(5);
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}
