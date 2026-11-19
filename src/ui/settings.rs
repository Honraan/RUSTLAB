//! Tela de configurações — editável por teclado, salva ao primeiro ajuste.
//!
//! Cada linha aceita ←/→ (ou h/l). Mudanças de tema/animação têm efeito
//! imediato; som/volume controlam o tom de XP. IA alterna entre Desativada, Ollama local e API remota.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState};

use crate::app::{App, SettingRow};
use crate::config::{AiProvider, GamificationVisibility};
use crate::ui::titled_block;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let rows = Layout::vertical([
        Constraint::Min(7),
        Constraint::Length(3), // relatório de capacidade de áudio
        Constraint::Length(1), // legenda
    ])
    .split(area);

    let rows_list = crate::app::visible_settings_rows(app.snapshot_mode);
    let items: Vec<ListItem> = rows_list
        .iter()
        .enumerate()
        .map(|(i, row)| ListItem::new(setting_line(app, *row, i == app.settings_row)))
        .collect();

    let list = List::new(items)
        .highlight_symbol(if app.unicode { "▶ " } else { "> " })
        .highlight_style(theme.selection());

    let block = titled_block(app.t("settings.title"), theme, true);
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
    let audio_block = titled_block(app.t("settings.audio_report"), theme, false);
    let audio_inner = audio_block.inner(rows[1]);
    frame.render_widget(audio_block, rows[1]);
    let report_line = Line::from(vec![
        mark,
        Span::styled(report.headline_for(app.is_english()), theme.base()),
        Span::styled(format!(" — {}", report.detail), theme.dim()),
    ]);
    let note = Line::from(Span::styled(app.t("help.audio_note"), theme.dim()));
    frame.render_widget(
        ratatui::widgets::Paragraph::new(vec![report_line, note])
            .wrap(ratatui::widgets::Wrap { trim: true }),
        audio_inner,
    );

    // ---- legenda --------------------------------------------------------
    let legend = Line::from(Span::styled(
        app.tf(
            "settings.legend",
            &[&if app.unicode { "←/→" } else { "<->" }],
        ),
        theme.dim(),
    ));
    frame.render_widget(ratatui::widgets::Paragraph::new(legend), rows[2]);
}

fn setting_line(app: &App, row: SettingRow, selected: bool) -> Line<'static> {
    let theme = &app.theme;
    let english = app.is_english();
    let (label, value, value_style) = match row {
        SettingRow::Theme => {
            let name = app.config.ui.theme.label_for(english);
            (
                app.t("settings.theme").to_string(),
                name.to_string(),
                theme.base(),
            )
        }
        SettingRow::Animations => (
            app.t("settings.animations").to_string(),
            app.t(app.config.ui.animations.label_key()).to_string(),
            theme.base(),
        ),
        SettingRow::Sound => (
            app.t("settings.sound").to_string(),
            if app.config.sound.enabled {
                app.t("settings.sound_on").to_string()
            } else {
                app.t("settings.sound_off").to_string()
            },
            theme.base(),
        ),
        SettingRow::Volume => (
            app.t("settings.volume").to_string(),
            format!(
                "{}% ({})",
                app.config.sound.volume,
                app.bi("tom suave", "soft tone")
            ),
            theme.base(),
        ),
        SettingRow::Gamification => (
            app.t("settings.gamification").to_string(),
            match app.config.ui.gamification {
                GamificationVisibility::Visible => app.t("settings.gam_on").to_string(),
                GamificationVisibility::Hidden => app.t("settings.gam_off").to_string(),
            },
            theme.base(),
        ),
        SettingRow::Language => (
            app.t("settings.language").to_string(),
            app.config.language_label().to_string(),
            theme.base(),
        ),
        SettingRow::OriginLanguage => (
            app.bi("Linguagem de origem", "Origin language").to_string(),
            app.config.origin_language.label_for(english).to_string(),
            theme.base(),
        ),
        SettingRow::InlayHints => (
            app.bi("Inlay hints de tipo", "Inline type inlay hints")
                .to_string(),
            if app.config.ui.inlay_hints {
                app.bi("Ativados (sempre legível)", "Enabled (type annotations)")
                    .to_string()
            } else {
                app.bi("Desligados", "Disabled").to_string()
            },
            theme.base(),
        ),
        SettingRow::SharedTargetCache => (
            app.bi("Cache target compartilhado", "Shared Cargo target cache")
                .to_string(),
            if app.config.execution.shared_target_cache {
                app.bi(
                    "Ativo (reuso rápido entre práticas)",
                    "Active (fast dependency reuse)",
                )
                .to_string()
            } else {
                app.bi("Isolado por projeto", "Isolated per project")
                    .to_string()
            },
            theme.base(),
        ),
        SettingRow::Ai => (
            app.t("settings.ai_provider").to_string(),
            match app.config.ai.provider {
                AiProvider::Disabled => app
                    .bi(
                        "Desligado (guia local determinístico)",
                        "Disabled (deterministic local guide)",
                    )
                    .to_string(),
                AiProvider::Ollama => format!("Ollama ({})", app.config.ai.ollama_model),
                AiProvider::Remote => format!(
                    "{} ({} · {})",
                    app.bi("API remota", "Remote API"),
                    app.config.ai.remote_model,
                    match app.config.ai.remote_consent {
                        crate::config::RemoteAiConsent::Granted =>
                            app.bi("autorizada", "authorized"),
                        crate::config::RemoteAiConsent::Ask =>
                            app.bi("pedir consentimento", "ask consent"),
                        crate::config::RemoteAiConsent::Denied => app.bi("recusada", "denied"),
                    }
                ),
            },
            theme.base(),
        ),
        SettingRow::OllamaModel => (
            app.t("settings.ollama_model").to_string(),
            if app.ollama_models.is_empty() {
                format!(
                    "{} ({})",
                    app.config.ai.ollama_model,
                    app.bi(
                        "←/→ alternar · r buscar no Ollama",
                        "←/→ switch · r query Ollama"
                    )
                )
            } else {
                format!(
                    "{} ({} {})",
                    app.config.ai.ollama_model,
                    app.ollama_models.len(),
                    app.bi("no Ollama · r atualiza", "in Ollama · r refreshes")
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
    use crate::config::{MotionMode, ThemeId};
    use crate::i18n::t_for;
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
        assert!(content.contains("Idioma"));
        assert!(content.contains("Cuia"));

        assert!(content.contains('\n'));
        app.snapshot_mode = false;
        terminal.draw(|f| crate::ui::draw(f, &mut app)).unwrap();
        let full_pt = crate::ui::tests_util::buffer_text(terminal.backend().buffer());
        assert!(full_pt.contains("Linguagem de origem"));

        app.config.language = "en-US".to_string();
        terminal.draw(|f| crate::ui::draw(f, &mut app)).unwrap();
        let content_en = crate::ui::tests_util::buffer_text(terminal.backend().buffer());
        assert!(content_en.contains("Language"));
        assert!(content_en.contains("English (en-US)"));
        assert!(content_en.contains("Animations"));
        assert!(content_en.contains("Full"));
        assert!(content_en.contains("Origin language"));
        assert!(content_en.contains("Inline type inlay hints"));
    }

    #[test]
    fn nomes_de_tema_estao_na_tabela() {
        // A UI de configurações deve sempre conseguir nomear todas as opções.
        for id in ThemeId::ALL {
            assert!(!id.label().is_empty());
            assert!(!id.label_for(true).is_empty());
        }
        for m in MotionMode::ALL {
            assert_ne!(t_for(m.label_key(), false), m.label_key());
            assert_ne!(t_for(m.label_key(), true), m.label_key());
        }
    }
}
