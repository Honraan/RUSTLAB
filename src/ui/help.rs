//! Tela de ajuda: tabela de atalhos + estado honesto do projeto.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};

use crate::app::App;
use crate::i18n::t;
use crate::ui::titled_block;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let arrow = if app.unicode { "↑" } else { "^" };
    let arrow2 = if app.unicode { "↓" } else { "v" };

    let keys: &[(&str, &str)] = &[
        (t("help.k_nav"), &format!("{arrow}/{arrow2} · k/j")),
        (t("help.k_enter"), "confirmar / abrir"),
        ("Esc", "voltar uma tela"),
        ("q", "sair (no menu) / voltar (em telas)"),
        ("Ctrl+C", "sair de qualquer lugar"),
        ("F4–F8", "build / run / test / check / fmt no Modo Criar"),
        (
            "F1/F11/Ctrl+G",
            "doc / definição / busca global no Modo Criar",
        ),
        (
            "Ctrl+L/Alt+M",
            "rodar cargo clippy / gerar PLANO.md (Etapa 10)",
        ),
        ("Ctrl+S/K", "salvar / cancelar execução"),
        (
            "Ctrl+T / m",
            "abrir Mentor opcional (guia local / Ollama / API)",
        ),
        (
            "Alt+A / Ctrl+T",
            "abrir/fechar Agente IA lateral no Modo Criar",
        ),
        ("Alt+O", "escolher modelo Ollama no Agente (o/m no Mentor)"),
        ("Alt+Enter", "aplicar código sugerido pelo Agente no editor"),
        (
            "Alt+D / d",
            "definir pasta padrão dos novos projetos (Criar)",
        ),
        (
            "e (Aprender)",
            "abrir exercícios com testes Cargo determinísticos",
        ),
        ("F1 exercício", "revelar a próxima dica progressiva"),
        (t("help.k_adjust"), "←/→ ou h/l, na tela Configurações"),
    ];

    let rows = Layout::vertical([Constraint::Min(5), Constraint::Length(6)]).split(area);
    // Em telas baixas, priorize atalhos: corte enfeites antes de dados.
    let compact = rows[0].height < 11;

    // --- atalhos -------------------------------------------------------
    let block = titled_block(t("help.title"), theme, true);
    let inner = block.inner(rows[0]);
    frame.render_widget(block, rows[0]);

    let mut lines = Vec::new();
    if !compact {
        lines.push(Line::from(Span::styled(t("help.intro"), theme.dim())));
        lines.push(Line::default());
    }
    for (key, desc) in keys {
        lines.push(Line::from(vec![
            Span::styled(format!("  {key:<15}"), theme.title()),
            Span::styled(*desc, theme.base()),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), inner);

    // --- situação do projeto --------------------------------------------
    let mut status = vec![
        Line::from(Span::styled("  Situação do projeto", theme.title())),
        Line::from(Span::styled(
            "  Fases 1-9: etapas 1-10 (pt-BR), 12 exercícios Cargo e widgets visuais.",
            theme.base(),
        )),
    ];
    if compact {
        status.push(Line::from(Span::styled(
            "  Console: entrada por linha/EOF; Shift+Tab troca foco (não é PTY).",
            theme.base(),
        )));
    } else {
        status.push(Line::from(Span::styled(
            "  Criar: rust-analyzer/índice local, Clippy, Alt+M plano; Cargo sem sandbox.",
            theme.base(),
        )));
    }
    status.push(Line::from(Span::styled(t("help.audio_note"), theme.dim())));

    let block2 = Block::bordered()
        .border_type(if app.unicode {
            BorderType::Rounded
        } else {
            BorderType::Plain
        })
        .border_style(theme.border_style(false));
    let inner2 = block2.inner(rows[1]);
    frame.render_widget(block2, rows[1]);
    frame.render_widget(Paragraph::new(status), inner2);
}
