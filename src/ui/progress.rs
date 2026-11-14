//! Resumo determinístico de progresso e recompensas registradas.
use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
};

pub fn lines(app: &App) -> Vec<Line<'static>> {
    let theme = &app.theme;
    let Some(curriculum) = &app.curriculum else {
        return vec![Line::from(
            "Currículo indisponível; não é possível calcular o progresso.",
        )];
    };
    let order = curriculum.lesson_order();
    let completed = order
        .iter()
        .filter(|(_, _, id)| app.progress.completed.contains(*id))
        .count();
    let read = order
        .iter()
        .filter(|(_, _, id)| app.progress.read.contains(*id))
        .count();
    let exercises: Vec<_> = curriculum.exercises().collect();
    let passed = exercises
        .iter()
        .filter(|ex| app.progress.exercises_completed.contains(&ex.id))
        .count();
    let mut lines = vec![
        Line::styled(
            format!(
                "Lições concluídas: {completed}/{} · Lidas: {read}/{}",
                order.len(),
                order.len()
            ),
            theme.title(),
        ),
        Line::styled(
            format!("Exercícios concluídos: {passed}/{}", exercises.len()),
            theme.title(),
        ),
        Line::from(
            "Conclusões e dicas são persistidas em disco; avisos de salvamento aparecem na interface.",
        ),
    ];
    if app.config.ui.gamification == crate::config::GamificationVisibility::Visible {
        let xp = app.xp();
        let within = xp % 100;
        lines.extend([
            Line::styled(
                format!(
                    "Nível {} · {xp} XP · {within}/100 para o próximo",
                    app.level()
                ),
                theme.accent,
            ),
            Line::styled(
                format!("Marcos conquistados: {}/8", app.achievement_count()),
                theme.title(),
            ),
            achievement_line(app, "Primeiros passos", completed >= 1),
            achievement_line(app, "Cinco lições", completed >= 5),
            achievement_line(
                app,
                "Trilha completa",
                !order.is_empty() && completed == order.len(),
            ),
            achievement_line(app, "Primeiro desafio", passed >= 1),
            achievement_line(app, "Seis desafios", passed >= 6),
            achievement_line(
                app,
                "Todos os desafios",
                !exercises.is_empty() && passed == exercises.len(),
            ),
            achievement_line(app, "Prática constante", completed + passed >= 10),
            achievement_line(
                app,
                "Projeto independente concluído",
                app.has_independent_project_complete(),
            ),
        ]);
    }
    lines.extend([
        Line::from(""),
        Line::styled("LIÇÕES DISPONÍVEIS", theme.title()),
    ]);
    for (_, _, id) in &order {
        let Some(lesson) = curriculum.lesson(id) else {
            continue;
        };
        let (marker, style) = if app.progress.completed.contains(*id) {
            ("[ok]", theme.success())
        } else if app.progress.read.contains(*id) {
            ("[lida]", theme.dim())
        } else {
            ("[pendente]", theme.base())
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{marker} "), style),
            Span::raw(lesson.title.clone()),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::styled("EXERCÍCIOS · e para praticar", theme.title()));
    for ex in exercises {
        let done = app.progress.exercises_completed.contains(&ex.id);
        let hints = usize::from(app.progress.hints_used.get(&ex.id).copied().unwrap_or(0))
            .min(ex.hints.len());
        lines.push(Line::styled(
            format!(
                "{} {} · dicas reveladas: {hints}/{}",
                if done { "[ok]" } else { "[pendente]" },
                ex.title,
                ex.hints.len()
            ),
            if done { theme.success() } else { theme.base() },
        ));
    }
    lines.extend([
        Line::from(""),
        Line::styled(
            "PROJETO INDEPENDENTE (ETAPA 10) · Habilidades demonstradas",
            theme.title(),
        ),
    ]);
    for (key, desc) in [
        ("plan", "PLANO.md criado (Alt+M)"),
        ("multi-module", "Projeto em múltiplos arquivos/módulos"),
        ("types", "Tipos próprios (struct/enum)"),
        ("error-handling", "Tratamento explícito (Option/Result)"),
        ("docs", "Documentação de API (///)"),
        ("build", "Build/check limpo (F4/F7)"),
        ("clippy", "Clippy limpo (Ctrl+L)"),
        ("tests", "Testes unitários verdes (F6)"),
        ("run", "Aplicação executada (F5)"),
    ] {
        lines.push(capability_line(
            app,
            desc,
            app.progress.project_milestones.contains(key),
        ));
    }

    lines.extend([
        Line::from(""),
        Line::styled("MAESTRIA & REVISÃO ESPAÇADA (V3)", theme.title()),
    ]);
    let due = app.progress.due_srs_items();
    let total_cards = app.progress.srs_cards.len();
    let avg_mastery = if total_cards == 0 {
        0.0
    } else {
        app.progress
            .srs_cards
            .values()
            .map(|c| f64::from(c.mastery_level))
            .sum::<f64>()
            / total_cards as f64
    };
    lines.push(Line::styled(
        format!(
            "  Cartões SRS: {total_cards} · Maestria média: {avg_mastery:.1}/5 · Pendentes: {}",
            due.len()
        ),
        theme.base(),
    ));
    if due.is_empty() {
        lines.push(Line::styled(
            "  Nenhuma revisão espaçada pendente no momento.",
            theme.dim(),
        ));
    } else {
        let preview = due.iter().take(5).cloned().collect::<Vec<_>>().join(", ");
        lines.push(Line::styled(
            format!("  Revisar com 'r' no catálogo: {preview}"),
            theme.success(),
        ));
    }
    lines
}

fn capability_line(app: &App, label: &str, ok: bool) -> Line<'static> {
    let tag = if ok { "[ok]" } else { "[pendente]" };
    Line::styled(
        format!("{tag} {label}"),
        if ok {
            app.theme.success()
        } else {
            app.theme.dim()
        },
    )
}

fn achievement_line(app: &App, title: &str, unlocked: bool) -> Line<'static> {
    let marker = if unlocked { "[★]" } else { "[ ]" };
    Line::styled(
        format!("{marker} {title}"),
        if unlocked {
            app.theme.success()
        } else {
            app.theme.dim()
        },
    )
}

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let content = Paragraph::new(lines(app)).wrap(ratatui::widgets::Wrap { trim: true });
    let max_scroll = content
        .line_count(area.width.saturating_sub(2))
        .saturating_sub(area.height.saturating_sub(2) as usize);
    app.progress_scroll = app.progress_scroll.min(max_scroll);
    frame.render_widget(
        content
            .scroll((app.progress_scroll.min(u16::MAX as usize) as u16, 0))
            .block(super::titled_block("Meu Progresso", &app.theme, true)),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conta_apenas_ids_do_curriculo_atual() {
        let mut app = App::for_snapshot();
        app.progress.completed.insert("hello-world".into());
        app.progress.completed.insert("removida".into());
        app.progress.exercises_completed.insert("soma".into());
        let content = lines(&app)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(content.contains("Lições concluídas: 1/30"));
        assert!(content.contains("Exercícios concluídos: 1/12"));
        assert!(content.contains("Marcos conquistados: 2/8"));
        assert!(content.contains("PROJETO INDEPENDENTE (ETAPA 10)"));
        assert!(content.contains("[pendente] PLANO.md criado (Alt+M)"));

        for key in ["plan", "build", "clippy", "tests", "run"] {
            app.progress.project_milestones.insert(key.into());
        }
        let updated = lines(&app)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(updated.contains("Marcos conquistados: 3/8"));
        assert!(updated.contains("[★] Projeto independente concluído"));
        assert!(updated.contains("[ok] PLANO.md criado (Alt+M)"));
    }

    #[test]
    fn mostra_secao_de_maestria_e_revisao_srs() {
        let mut app = App::for_snapshot();
        app.progress.srs_cards.insert(
            "B-T09-P".into(),
            crate::storage::SrsCard {
                mastery_level: 2,
                interval_steps: 4,
                due_at_step: 0,
                independent_passes: 2,
                review_count: 2,
            },
        );
        let content = lines(&app)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(content.contains("MAESTRIA & REVISÃO ESPAÇADA (V3)"));
        assert!(content.contains("Cartões SRS: 1 · Maestria média: 2.0/5 · Pendentes: 1"));
        assert!(content.contains("Revisar com 'r' no catálogo: B-T09-P"));
    }
}
