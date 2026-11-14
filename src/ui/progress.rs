//! Resumo determinístico de progresso e recompensas registradas.
use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
};

pub fn lines(app: &App) -> Vec<Line<'static>> {
    let en = app.is_english();
    let theme = &app.theme;
    let Some(curriculum) = &app.curriculum else {
        return vec![Line::from(if en {
            "Curriculum unavailable; cannot compute progress."
        } else {
            "Currículo indisponível; não é possível calcular o progresso."
        })];
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
            if en {
                format!(
                    "Completed lessons: {completed}/{} · Read: {read}/{}",
                    order.len(),
                    order.len()
                )
            } else {
                format!(
                    "Lições concluídas: {completed}/{} · Lidas: {read}/{}",
                    order.len(),
                    order.len()
                )
            },
            theme.title(),
        ),
        Line::styled(
            if en {
                format!("Completed exercises: {passed}/{}", exercises.len())
            } else {
                format!("Exercícios concluídos: {passed}/{}", exercises.len())
            },
            theme.title(),
        ),
        Line::from(if en {
            "Completions and hints are saved to disk; save warnings appear in the status bar."
        } else {
            "Conclusões e dicas são persistidas em disco; avisos de salvamento aparecem na interface."
        }),
    ];
    if app.config.ui.gamification == crate::config::GamificationVisibility::Visible {
        let xp = app.xp();
        let within = xp % 100;
        lines.extend([
            Line::styled(
                if en {
                    format!("Level {} · {xp} XP · {within}/100 to next", app.level())
                } else {
                    format!(
                        "Nível {} · {xp} XP · {within}/100 para o próximo",
                        app.level()
                    )
                },
                theme.accent,
            ),
            Line::styled(
                if en {
                    format!("Achievements unlocked: {}/8", app.achievement_count())
                } else {
                    format!("Marcos conquistados: {}/8", app.achievement_count())
                },
                theme.title(),
            ),
            achievement_line(
                app,
                if en {
                    "First steps"
                } else {
                    "Primeiros passos"
                },
                completed >= 1,
            ),
            achievement_line(
                app,
                if en { "Five lessons" } else { "Cinco lições" },
                completed >= 5,
            ),
            achievement_line(
                app,
                if en { "Full track" } else { "Trilha completa" },
                !order.is_empty() && completed == order.len(),
            ),
            achievement_line(
                app,
                if en {
                    "First challenge"
                } else {
                    "Primeiro desafio"
                },
                passed >= 1,
            ),
            achievement_line(
                app,
                if en {
                    "Six challenges"
                } else {
                    "Seis desafios"
                },
                passed >= 6,
            ),
            achievement_line(
                app,
                if en {
                    "All challenges"
                } else {
                    "Todos os desafios"
                },
                !exercises.is_empty() && passed == exercises.len(),
            ),
            achievement_line(
                app,
                if en {
                    "Steady practice"
                } else {
                    "Prática constante"
                },
                completed + passed >= 10,
            ),
            achievement_line(
                app,
                if en {
                    "Independent project completed"
                } else {
                    "Projeto independente concluído"
                },
                app.has_independent_project_complete(),
            ),
        ]);
    }
    lines.extend([
        Line::from(""),
        Line::styled(
            if en {
                "AVAILABLE LESSONS"
            } else {
                "LIÇÕES DISPONÍVEIS"
            },
            theme.title(),
        ),
    ]);
    for (_, _, id) in &order {
        let Some(lesson) = curriculum.lesson(id) else {
            continue;
        };
        let (marker, style) = if app.progress.completed.contains(*id) {
            ("[ok]", theme.success())
        } else if app.progress.read.contains(*id) {
            (if en { "[read]" } else { "[lida]" }, theme.dim())
        } else {
            (if en { "[pending]" } else { "[pendente]" }, theme.base())
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{marker} "), style),
            Span::raw(lesson.title.clone()),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::styled(
        if en {
            "EXERCISES · e to practice"
        } else {
            "EXERCÍCIOS · e para praticar"
        },
        theme.title(),
    ));
    for ex in exercises {
        let done = app.progress.exercises_completed.contains(&ex.id);
        let hints = usize::from(app.progress.hints_used.get(&ex.id).copied().unwrap_or(0))
            .min(ex.hints.len());
        let tag = if done {
            "[ok]"
        } else if en {
            "[pending]"
        } else {
            "[pendente]"
        };
        lines.push(Line::styled(
            if en {
                format!(
                    "{tag} {} · hints revealed: {hints}/{}",
                    ex.title,
                    ex.hints.len()
                )
            } else {
                format!(
                    "{tag} {} · dicas reveladas: {hints}/{}",
                    ex.title,
                    ex.hints.len()
                )
            },
            if done { theme.success() } else { theme.base() },
        ));
    }
    lines.extend([
        Line::from(""),
        Line::styled(
            if en {
                "INDEPENDENT PROJECT (CHAPTER 10) · Demonstrated skills"
            } else {
                "PROJETO INDEPENDENTE (ETAPA 10) · Habilidades demonstradas"
            },
            theme.title(),
        ),
    ]);
    for (key, desc) in if en {
        [
            ("plan", "PLANO.md created (Alt+M)"),
            ("multi-module", "Multi-file / multi-module project"),
            ("types", "Custom types (struct/enum)"),
            ("error-handling", "Explicit error handling (Option/Result)"),
            ("docs", "API documentation (///)"),
            ("build", "Clean build/check (F4/F7)"),
            ("clippy", "Clean Clippy (Ctrl+L)"),
            ("tests", "Green unit tests (F6)"),
            ("run", "Application executed (F5)"),
        ]
    } else {
        [
            ("plan", "PLANO.md criado (Alt+M)"),
            ("multi-module", "Projeto em múltiplos arquivos/módulos"),
            ("types", "Tipos próprios (struct/enum)"),
            ("error-handling", "Tratamento explícito (Option/Result)"),
            ("docs", "Documentação de API (///)"),
            ("build", "Build/check limpo (F4/F7)"),
            ("clippy", "Clippy limpo (Ctrl+L)"),
            ("tests", "Testes unitários verdes (F6)"),
            ("run", "Aplicação executada (F5)"),
        ]
    } {
        lines.push(capability_line(
            app,
            desc,
            app.progress.project_milestones.contains(key),
        ));
    }

    lines.extend([
        Line::from(""),
        Line::styled(
            if en {
                "MASTERY & SPACED REVISION (V3)"
            } else {
                "MAESTRIA & REVISÃO ESPAÇADA (V3)"
            },
            theme.title(),
        ),
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
        if en {
            format!(
                "  SRS cards: {total_cards} · Avg mastery: {avg_mastery:.1}/5 · Due: {}",
                due.len()
            )
        } else {
            format!(
                "  Cartões SRS: {total_cards} · Maestria média: {avg_mastery:.1}/5 · Pendentes: {}",
                due.len()
            )
        },
        theme.base(),
    ));
    if due.is_empty() {
        lines.push(Line::styled(
            if en {
                "  No spaced reviews due right now."
            } else {
                "  Nenhuma revisão espaçada pendente no momento."
            },
            theme.dim(),
        ));
    } else {
        let preview = due.iter().take(5).cloned().collect::<Vec<_>>().join(", ");
        lines.push(Line::styled(
            if en {
                format!("  Review with 'r' in catalog: {preview}")
            } else {
                format!("  Revisar com 'r' no catálogo: {preview}")
            },
            theme.success(),
        ));
    }
    lines
}

fn capability_line(app: &App, label: &str, ok: bool) -> Line<'static> {
    let tag = if ok {
        "[ok]"
    } else if app.is_english() {
        "[pending]"
    } else {
        "[pendente]"
    };
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
    let title = app.bi("Meu Progresso", "My Progress").to_string();
    frame.render_widget(
        content
            .scroll((app.progress_scroll.min(u16::MAX as usize) as u16, 0))
            .block(super::titled_block(&title, &app.theme, true)),
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

    #[test]
    fn renderiza_progresso_em_ingles_quando_configurado() {
        let mut app = App::for_snapshot();
        app.config.language = "en-US".to_string();
        app.curriculum = crate::curriculum::Curriculum::embedded_for(true).ok();
        app.progress.srs_cards.insert(
            "B-T09-P".into(),
            crate::storage::SrsCard {
                mastery_level: 3,
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
        assert!(content.contains("MASTERY & SPACED REVISION (V3)"));
        assert!(content.contains("SRS cards: 1 · Avg mastery: 3.0/5 · Due: 1"));
        assert!(content.contains("Review with 'r' in catalog: B-T09-P"));
    }
}
