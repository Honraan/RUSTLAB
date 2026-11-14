//! Tela de ajuda: tabela de atalhos + estado honesto do projeto.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};

use crate::app::App;
use crate::ui::titled_block;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let arrow = if app.unicode { "↑" } else { "^" };
    let arrow2 = if app.unicode { "↓" } else { "v" };

    let keys: &[(&str, &str)] = &[
        (app.t("help.k_nav"), &format!("{arrow}/{arrow2} · k/j")),
        (
            app.t("help.k_enter"),
            app.bi("confirmar / abrir", "confirm / open"),
        ),
        ("Esc", app.bi("voltar uma tela", "go back one screen")),
        (
            "q",
            app.bi(
                "sair (no menu) / voltar (em telas)",
                "quit (menu) / back (screens)",
            ),
        ),
        (
            "Ctrl+C",
            app.bi("sair de qualquer lugar", "quit from anywhere"),
        ),
        (
            "F4–F8",
            app.bi(
                "build / run / test / check / fmt no Modo Criar",
                "build / run / test / check / fmt in Create Mode",
            ),
        ),
        (
            "F1/F11/Ctrl+G",
            app.bi(
                "doc / definição / busca global no Modo Criar",
                "docs / definition / global search in Create Mode",
            ),
        ),
        (
            "Ctrl+L/Alt+M",
            app.bi(
                "rodar cargo clippy / gerar PLANO.md (Etapa 10)",
                "run cargo clippy / generate PLANO.md (Stage 10)",
            ),
        ),
        (
            "Ctrl+S/K",
            app.bi("salvar / cancelar execução", "save / cancel execution"),
        ),
        (
            "Ctrl+T / m",
            app.bi(
                "abrir Mentor opcional (guia local / Ollama / API)",
                "open optional Mentor (local guide / Ollama / API)",
            ),
        ),
        (
            "Alt+A / Ctrl+T",
            app.bi(
                "abrir/fechar Agente IA lateral no Modo Criar",
                "toggle side AI Agent in Create Mode",
            ),
        ),
        (
            "Alt+O",
            app.bi(
                "escolher modelo Ollama no Agente (o/m no Mentor)",
                "pick Ollama model in Agent (o/m in Mentor)",
            ),
        ),
        (
            "Alt+Enter",
            app.bi(
                "aplicar código sugerido pelo Agente no editor",
                "apply Agent suggested code to editor",
            ),
        ),
        (
            "Alt+D / d",
            app.bi(
                "definir pasta padrão dos novos projetos (Criar)",
                "set default directory for new projects (Create)",
            ),
        ),
        (
            "e (Aprender)",
            app.bi(
                "abrir exercícios com testes Cargo determinísticos",
                "open exercises with deterministic Cargo tests",
            ),
        ),
        (
            "F1 exercício",
            app.bi(
                "revelar a próxima dica progressiva",
                "reveal next progressive hint",
            ),
        ),
        (
            app.t("help.k_adjust"),
            app.bi(
                "←/→ ou h/l, na tela Configurações",
                "←/→ or h/l, on the Settings screen",
            ),
        ),
    ];

    let rows = Layout::vertical([Constraint::Min(5), Constraint::Length(6)]).split(area);
    // Em telas baixas, priorize atalhos: corte enfeites antes de dados.
    let compact = rows[0].height < 11;

    // --- atalhos -------------------------------------------------------
    let block = titled_block(app.t("help.title"), theme, true);
    let inner = block.inner(rows[0]);
    frame.render_widget(block, rows[0]);

    let mut lines = Vec::new();
    if !compact {
        lines.push(Line::from(Span::styled(app.t("help.intro"), theme.dim())));
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
        Line::from(Span::styled(
            app.bi("  Situação do projeto", "  Project status"),
            theme.title(),
        )),
        Line::from(Span::styled(
            app.bi(
                "  Fases 1-9: etapas 1-10 (pt-BR), 12 exercícios Cargo e widgets visuais.",
                "  Phases 1-9: stages 1-10 (pt-BR), 12 Cargo exercises and visual widgets.",
            ),
            theme.base(),
        )),
    ];
    if compact {
        status.push(Line::from(Span::styled(
            app.bi(
                "  Console: entrada por linha/EOF; Shift+Tab troca foco (não é PTY).",
                "  Console: line/EOF input; Shift+Tab switches focus (not a full PTY).",
            ),
            theme.base(),
        )));
    } else {
        status.push(Line::from(Span::styled(
            app.bi(
                "  Criar: rust-analyzer/índice local, Clippy, Alt+M plano; Cargo sem sandbox.",
                "  Create: rust-analyzer/local index, Clippy, Alt+M plan; unsandboxed Cargo.",
            ),
            theme.base(),
        )));
    }
    status.push(Line::from(Span::styled(
        app.t("help.audio_note"),
        theme.dim(),
    )));

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
