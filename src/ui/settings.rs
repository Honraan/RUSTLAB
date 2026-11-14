//! Tela de configurações — editável por teclado, salva ao primeiro ajuste.
//!
//! Cada linha aceita ←/→ (ou h/l). Mudanças de tema/animação têm efeito
//! imediato; som/volume controlam o tom de XP. IA alterna entre Desativada, Ollama local e API remota.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState};

use crate::app::{App, SETTING_ROWS, SettingRow};
use crate::config::{AiProvider, GamificationVisibility};
use crate::i18n::{t, tf};
use crate::ui::titled_block;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let rows = Layout::vertical([
        Constraint::Min(7),
        Constraint::Length(3), // relatório de capacidade de áudio
        Constraint::Length(1), // legenda
    ])
    .split(area);

    let items: Vec<ListItem> = SETTING_ROWS
        .iter()
        .enumerate()
        .map(|(i, row)| ListItem::new(setting_line(app, *row, i == app.settings_row)))
        .collect();

    let list = List::new(items)
        .highlight_symbol(if app.unicode { "▶ " } else { "> " })
        .highlight_style(theme.selection());

    let block = titled_block(t("settings.title"), theme, true);
    let inner = block.inner(rows[0]);
    frame.render_widget(block, rows[0]);
    let mut state = ListState::default().with_selected(Some(app.settings_row));
    frame.render_stateful_widget(list, inner, &mut state);

    // ---- capacidade de áudio (checagem da Fase 1) ----------------------
    let report = &app.audio_report;
    let mark = if report.available() {
        Span::styled(if app.unicode { "✓ " } else { "[ok] " }, theme.success())
    } else {
        Span::styled(if app.unicode { "○ " } else { "[--] " }, theme.dim())
    };
    let audio_block = titled_block(t("settings.audio_report"), theme, false);
    let audio_inner = audio_block.inner(rows[1]);
    frame.render_widget(audio_block, rows[1]);
    let report_line = Line::from(vec![
        mark,
        Span::styled(report.headline(), theme.base()),
        Span::styled(format!(" — {}", report.detail), theme.dim()),
    ]);
    let note = Line::from(Span::styled(t("help.audio_note"), theme.dim()));
    frame.render_widget(
        ratatui::widgets::Paragraph::new(vec![report_line, note])
            .wrap(ratatui::widgets::Wrap { trim: true }),
        audio_inner,
    );

    // ---- legenda --------------------------------------------------------
    let legend = Line::from(Span::styled(
        tf(
            "settings.legend",
            &[&if app.unicode { "←/→" } else { "<->" }],
        ),
        theme.dim(),
    ));
    frame.render_widget(ratatui::widgets::Paragraph::new(legend), rows[2]);
}

fn setting_line(app: &App, row: SettingRow, selected: bool) -> Line<'static> {
    let theme = &app.theme;
    let (label, value, value_style) = match row {
        SettingRow::Theme => {
            let name = app.config.ui.theme.label();
            (
                t("settings.theme").to_string(),
                name.to_string(),
                theme.base(),
            )
        }
        SettingRow::Animations => (
            t("settings.animations").to_string(),
            t(app.config.ui.animations.label_key()).to_string(),
            theme.base(),
        ),
        SettingRow::Sound => (
            t("settings.sound").to_string(),
            if app.config.sound.enabled {
                t("settings.sound_on").to_string()
            } else {
                t("settings.sound_off").to_string()
            },
            theme.base(),
        ),
        SettingRow::Volume => (
            t("settings.volume").to_string(),
            format!("{}% (tom suave)", app.config.sound.volume),
            theme.base(),
        ),
        SettingRow::Gamification => (
            t("settings.gamification").to_string(),
            match app.config.ui.gamification {
                GamificationVisibility::Visible => t("settings.gam_on").to_string(),
                GamificationVisibility::Hidden => t("settings.gam_off").to_string(),
            },
            theme.base(),
        ),
        SettingRow::Ai => (
            t("settings.ai_provider").to_string(),
            match app.config.ai.provider {
                AiProvider::Disabled => "Desligado (guia local determinístico)".to_string(),
                AiProvider::Ollama => format!("Ollama ({})", app.config.ai.ollama_model),
                AiProvider::Remote => format!(
                    "API remota ({} · {})",
                    app.config.ai.remote_model,
                    match app.config.ai.remote_consent {
                        crate::config::RemoteAiConsent::Granted => "autorizada",
                        crate::config::RemoteAiConsent::Ask => "pedir consentimento",
                        crate::config::RemoteAiConsent::Denied => "recusada",
                    }
                ),
            },
            theme.base(),
        ),
        SettingRow::OllamaModel => (
            t("settings.ollama_model").to_string(),
            if app.ollama_models.is_empty() {
                format!(
                    "{} (←/→ alternar · r buscar no Ollama)",
                    app.config.ai.ollama_model
                )
            } else {
                format!(
                    "{} ({} no Ollama · r atualiza)",
                    app.config.ai.ollama_model,
                    app.ollama_models.len()
                )
            },
            theme.base(),
        ),
    };

    let mut line = Line::from(vec![
        Span::styled(format!("  {label:<26}"), theme.base()),
        Span::styled(value, value_style),
    ]);
    if selected {
        // Pista não-cromática de edição ativa (acessibilidade).
        let spans: Vec<Span> = line
            .spans
            .into_iter()
            .map(|s| {
                let style = s.style.add_modifier(ratatui::style::Modifier::BOLD);
                Span::styled(s.content.to_string(), style)
            })
            .collect();
        line = Line::from(spans);
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{MotionMode, ThemeId};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn render_linhas_de_config() {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        let mut app = crate::ui::tests_util::test_app();
        app.screen = crate::app::Screen::Settings;
        terminal.draw(|f| crate::ui::draw(f, &mut app)).unwrap();
        let content = crate::ui::tests_util::buffer_text(terminal.backend().buffer());
        assert!(content.contains("Tema"));
        assert!(content.contains("Animações"));
        assert!(content.contains("Cuia"));
    }

    #[test]
    fn nomes_de_tema_estao_na_tabela() {
        // A UI de configurações deve sempre conseguir nomear todas as opções.
        for id in ThemeId::ALL {
            assert!(!id.label().is_empty());
        }
        for m in MotionMode::ALL {
            assert_ne!(t(m.label_key()), m.label_key()); // chave existe
        }
    }
}
