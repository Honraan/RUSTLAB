use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::App;
use crate::exercises::{ExerciseOutcome, ExerciseState, ExerciseView};

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme;
    let Some(mut state) = app.exercises.take() else {
        return;
    };
    let view = state.view;
    let consent_pending = state.consent_pending;
    match view {
        ExerciseView::List => draw_list(frame, app, &state, area),
        ExerciseView::Work => draw_work(frame, app, &mut state, area),
    }
    let en = app.is_english();
    if consent_pending {
        let r = centered(area, 72, 9);
        frame.render_widget(Clear, r);
        frame.render_widget(
            Paragraph::new(if en {
                "Running these tests compiles and executes your local code; this is NOT sandboxed. The process runs with your user permissions.\n\nNothing has run yet. Enter accepts; Esc declines."
            } else {
                "Executar estes testes compila e executa seu código local; isso NÃO é sandbox. O processo tem as permissões da sua conta.\n\nNada foi rodado ainda. Enter aceita; Esc recusa."
            })
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .title(if en {
                        " Consent required "
                    } else {
                        " Consentimento necessário "
                    })
                    .border_style(theme.border_style(true)),
            ),
            r,
        );
    }
    app.exercises = Some(state);
}

fn draw_list(frame: &mut Frame, app: &App, state: &ExerciseState, area: Rect) {
    let en = app.is_english();
    let theme = &app.theme;
    let Some(cur) = app.curriculum.as_ref() else {
        return;
    };
    let exercises: Vec<_> = cur.exercises().collect();
    if exercises.is_empty() {
        frame.render_widget(
            Paragraph::new(if en {
                "No exercises available yet."
            } else {
                "Ainda não há exercícios disponíveis."
            })
            .block(Block::bordered().title(if en {
                " Exercises "
            } else {
                " Exercícios "
            })),
            area,
        );
        return;
    }
    let panels = Layout::vertical([
        Constraint::Min(4),
        Constraint::Length(if area.height >= 16 { 10 } else { 0 }),
    ])
    .split(area);
    let items: Vec<ListItem> = exercises
        .iter()
        .map(|e| {
            let done = app.progress.exercises_completed.contains(&e.id);
            let marker = if done { "✓ " } else { "· " };
            ListItem::new(Line::from(vec![
                Span::styled(marker, if done { theme.success() } else { theme.dim() }),
                Span::styled(e.title.clone(), theme.base()),
            ]))
        })
        .collect();
    let block = Block::bordered()
        .title(if en {
            " Exercises · deterministic Cargo tests "
        } else {
            " Exercícios · testes Cargo determinísticos "
        })
        .border_style(theme.border_style(true));
    let mut list =
        ListState::default().with_selected(Some(state.selected.min(items.len().saturating_sub(1))));
    frame.render_stateful_widget(
        List::new(items)
            .highlight_symbol("▶ ")
            .highlight_style(theme.selection())
            .block(block),
        panels[0],
        &mut list,
    );
    if panels[1].height > 0 {
        let exercise = exercises[state.selected.min(exercises.len() - 1)];
        let lessons = exercise
            .lesson_ids
            .iter()
            .filter_map(|id| cur.lesson(id))
            .map(|lesson| lesson.title.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let saved = app.progress.exercise_answers.contains_key(&exercise.id)
            || state.drafts.contains_key(&exercise.id)
            || state
                .session
                .as_ref()
                .is_some_and(|session| session.id == exercise.id);
        let preview = vec![
            Line::styled(exercise.title.clone(), theme.title()),
            Line::from(exercise.statement.clone()),
            Line::styled(
                if en {
                    format!("Review: {lessons}")
                } else {
                    format!("Revisar: {lessons}")
                },
                theme.dim(),
            ),
            Line::styled(
                match (saved, en) {
                    (true, true) => "Enter resume answer · F5 runs tests only after consent",
                    (true, false) => "Enter retomar resposta · F5 testa apenas após consentimento",
                    (false, true) => "Enter open starter code · four hints available",
                    (false, false) => "Enter abrir código inicial · quatro dicas disponíveis",
                },
                theme.dim(),
            ),
            Line::styled(
                match (&state.return_screen, en) {
                    (crate::app::Screen::LessonView, true) => {
                        "Esc returns to the lesson, keeping your scroll and quizzes."
                    }
                    (crate::app::Screen::LessonView, false) => {
                        "Esc volta à lição, preservando sua posição e quizzes."
                    }
                    (crate::app::Screen::Progress, true) => "Esc returns to My Progress.",
                    (crate::app::Screen::Progress, false) => "Esc volta a Meu Progresso.",
                    (_, true) => "Esc returns to the course.",
                    (_, false) => "Esc volta ao curso.",
                },
                theme.dim(),
            ),
        ];
        // O enunciado quebra em linhas; mantenha um bloco separado da lista.
        frame.render_widget(
            Paragraph::new(preview).wrap(Wrap { trim: true }).block(
                Block::bordered()
                    .title(if en {
                        " Before you practice "
                    } else {
                        " Antes de praticar "
                    })
                    .border_style(theme.border_style(false)),
            ),
            panels[1],
        );
    }
}

fn draw_work(frame: &mut Frame, app: &App, state: &mut ExerciseState, area: Rect) {
    let en = app.is_english();
    let theme = &app.theme;
    let Some(exercise) = state.exercise(app).cloned() else {
        return;
    };
    let Some(session) = state.session.as_mut() else {
        return;
    };
    let rows = Layout::vertical([
        Constraint::Length(5),
        Constraint::Percentage(58),
        Constraint::Percentage(42),
    ])
    .split(area);
    let statement = Paragraph::new(format!("{}\n{}", exercise.title, exercise.statement))
        .wrap(Wrap { trim: true });
    let statement_max = statement
        .line_count(rows[0].width.saturating_sub(2))
        .saturating_sub(rows[0].height.saturating_sub(2) as usize);
    session.statement_scroll = session
        .statement_scroll
        .min(statement_max.min(u16::MAX as usize) as u16);
    frame.render_widget(
        statement.scroll((session.statement_scroll, 0)).block(
            Block::bordered()
                .title(if en {
                    " Prompt · F9 scroll · Shift+F9 scroll up "
                } else {
                    " Enunciado · F9 rolar · Shift+F9 subir "
                })
                .border_style(theme.border_style(false)),
        ),
        rows[0],
    );

    let middle =
        Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)]).split(rows[1]);
    let editor_rect = Rect {
        x: middle[0].x.saturating_add(1),
        y: middle[0].y.saturating_add(1),
        width: middle[0].width.saturating_sub(2),
        height: middle[0].height.saturating_sub(2),
    };
    session.editor.ensure_visible(
        editor_rect.height as usize,
        editor_rect.width.saturating_sub(6) as usize,
    );
    let editor = &session.editor;
    let lines = (0..editor.buf.line_count())
        .map(|row| {
            let source = editor.buf.line_str(row);
            let mut spans = vec![Span::styled(format!("{:>3} │ ", row + 1), theme.dim())];
            let cursor_col = (editor.cursor.line == row).then_some(editor.cursor.col);
            let mut col = 0;
            for span in crate::highlight::highlight_line(&source, theme) {
                for ch in span.content.chars() {
                    let style = if cursor_col == Some(col) {
                        span.style.add_modifier(Modifier::REVERSED)
                    } else {
                        span.style
                    };
                    spans.push(Span::styled(ch.to_string(), style));
                    col += 1;
                }
            }
            if cursor_col == Some(col) {
                spans.push(Span::styled(" ", theme.selection()));
            }
            Line::from(spans)
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(lines)
            .scroll((editor.scroll_row as u16, editor.scroll_col as u16))
            .block(
                Block::bordered()
                    .title(match (editor.dirty, en) {
                        (true, true) => " Your code · unsaved · Ctrl+S ",
                        (true, false) => " Seu código · não salvo · Ctrl+S ",
                        (false, true) => " Your code · Ctrl+S save · F5 test ",
                        (false, false) => " Seu código · Ctrl+S salvar · F5 testar ",
                    })
                    .border_style(theme.border_style(true)),
            ),
        middle[0],
    );

    let (hints_area, ferris_area) = if middle[1].height >= 10 {
        let right = Layout::vertical([Constraint::Min(5), Constraint::Length(5)]).split(middle[1]);
        (right[0], Some(right[1]))
    } else {
        (middle[1], None)
    };

    let hint = exercise.hints.get(session.hint_index);
    let hint_text = if session.shown_hints == 0 {
        if en {
            "F1 reveals the first hint.\nF2/F3 review revealed hints.".to_string()
        } else {
            "F1 revela a primeira dica.\nF2/F3 revisam dicas já reveladas.".to_string()
        }
    } else if en {
        format!(
            "Hint {}/{} · {} revealed\n{}\n\nF2 previous · F3 next\nF1 reveals another hint",
            session.hint_index + 1,
            exercise.hints.len(),
            session.shown_hints,
            hint.map(String::as_str).unwrap_or("")
        )
    } else {
        format!(
            "Dica {}/{} · {} reveladas\n{}\n\nF2 anterior · F3 próxima\nF1 revela outra dica",
            session.hint_index + 1,
            exercise.hints.len(),
            session.shown_hints,
            hint.map(String::as_str).unwrap_or("")
        )
    };
    let hints = Paragraph::new(hint_text).wrap(Wrap { trim: true });
    let hint_max = hints
        .line_count(hints_area.width.saturating_sub(2))
        .saturating_sub(hints_area.height.saturating_sub(2) as usize);
    session.hint_scroll = session
        .hint_scroll
        .min(hint_max.min(u16::MAX as usize) as u16);
    frame.render_widget(
        hints.scroll((session.hint_scroll, 0)).block(
            Block::bordered()
                .title(if en {
                    " Hints · F4/Shift+F4 scroll "
                } else {
                    " Dicas · F4/Shift+F4 rolar "
                })
                .border_style(theme.border_style(false)),
        ),
        hints_area,
    );

    if let Some(f_area) = ferris_area {
        let mood = match session.outcome {
            ExerciseOutcome::Passed => crate::learn::FerrisMood::Happy,
            ExerciseOutcome::TestsFailed
            | ExerciseOutcome::CompileFailed
            | ExerciseOutcome::TimedOut => crate::learn::FerrisMood::Encouraging,
            ExerciseOutcome::StartFailed => crate::learn::FerrisMood::Confused,
            ExerciseOutcome::NotRun | ExerciseOutcome::Running | ExerciseOutcome::Cancelled => {
                crate::learn::FerrisMood::Neutral
            }
        };
        let (base, bonus) =
            crate::learn::calculate_v3_practice_xp(session.shown_hints, session.ai_used);
        let xp_note = match (bonus > 0, en) {
            (true, true) => format!("XP: {base}+{bonus} no hints"),
            (true, false) => format!("XP: {base}+{bonus} s/ dicas"),
            (false, true) => format!("Hints: {} (indep. bonus used)", session.shown_hints),
            (false, false) => format!("Dicas: {} (bônus indep. usado)", session.shown_hints),
        };
        let mood_style = match mood {
            crate::learn::FerrisMood::Neutral => theme.title().add_modifier(Modifier::BOLD),
            crate::learn::FerrisMood::Happy => theme.success().add_modifier(Modifier::BOLD),
            _ => theme.warning().add_modifier(Modifier::BOLD),
        };
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(format!(" {} ", mood.ascii()), mood_style),
                    Span::styled(mood.label_for(en), theme.base()),
                ]),
                Line::from(Span::styled(xp_note, theme.dim())),
            ])
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .title(" Ferris ")
                    .border_style(theme.border_style(false)),
            ),
            f_area,
        );
    }

    let result = match (session.outcome, en) {
        (ExerciseOutcome::NotRun, true) => "Not run yet".to_string(),
        (ExerciseOutcome::NotRun, false) => "Ainda não executado".to_string(),
        (ExerciseOutcome::Running, true) => "Running tests…".to_string(),
        (ExerciseOutcome::Running, false) => "Testes em execução…".to_string(),
        (ExerciseOutcome::Passed, true) => "✓ All tests passed".to_string(),
        (ExerciseOutcome::Passed, false) => "✓ Todos os testes passaram".to_string(),
        (ExerciseOutcome::TestsFailed, true) => "✗ One or more tests failed".to_string(),
        (ExerciseOutcome::TestsFailed, false) => "✗ Um ou mais testes falharam".to_string(),
        (ExerciseOutcome::CompileFailed, true) => "! Code did not compile".to_string(),
        (ExerciseOutcome::CompileFailed, false) => "! O código não compilou".to_string(),
        (ExerciseOutcome::TimedOut, true) => "⏱ Tests exceeded the time limit".to_string(),
        (ExerciseOutcome::TimedOut, false) => "⏱ Testes excederam o limite de tempo".to_string(),
        (ExerciseOutcome::Cancelled, true) => "Execution cancelled".to_string(),
        (ExerciseOutcome::Cancelled, false) => "Execução cancelada".to_string(),
        (ExerciseOutcome::StartFailed, true) => "Could not start the test runner".to_string(),
        (ExerciseOutcome::StartFailed, false) => "Não foi possível iniciar os testes".to_string(),
    };
    let result_line = Line::from(Span::styled(
        result,
        match session.outcome {
            ExerciseOutcome::Passed => theme.success(),
            ExerciseOutcome::NotRun | ExerciseOutcome::Running => theme.dim(),
            _ => theme.warning(),
        },
    ));
    let result_block = Block::bordered()
        .title(if en {
            format!(
                " Output · PgUp/PgDn · {} ",
                if session.output_scroll == 0 {
                    "end".to_string()
                } else {
                    format!("{} lines above", session.output_scroll)
                }
            )
        } else {
            format!(
                " Resultado · PgUp/PgDn · {} ",
                if session.output_scroll == 0 {
                    "fim".to_string()
                } else {
                    format!("{} linhas acima", session.output_scroll)
                }
            )
        })
        .border_style(theme.border_style(false));
    let body = result_block.inner(rows[2]);
    frame.render_widget(result_block, rows[2]);
    let output_rows = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(body);
    frame.render_widget(Paragraph::new(result_line), output_rows[0]);
    session.output_width = output_rows[1].width.max(1);
    session.output_page_rows = output_rows[1].height.max(1) as usize;
    let max_scroll = session
        .output_line_count()
        .saturating_sub(session.output_page_rows);
    session.output_scroll = session.output_scroll.min(max_scroll);
    let offset = max_scroll
        .saturating_sub(session.output_scroll)
        .min(u16::MAX as usize) as u16;
    frame.render_widget(
        Paragraph::new(session.output.join("\n"))
            .wrap(Wrap { trim: false })
            .scroll((offset, 0)),
        output_rows[1],
    );
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width.saturating_sub(2));
    let h = height.min(area.height.saturating_sub(2));
    Rect {
        x: area.x + area.width.saturating_sub(w) / 2,
        y: area.y + area.height.saturating_sub(h) / 2,
        width: w,
        height: h,
    }
}
