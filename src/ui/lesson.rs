//! Renderizador de lição: transforma blocos do currículo em linhas
//! roláveis, com código realçado, caixas de chamada, demos interativas e
//! quizzes com feedback imediato.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Gauge, Paragraph, Wrap};

use crate::app::App;
use crate::curriculum::{Block as CBlock, CalloutKind, DemoKind, Lesson};
use crate::highlight;
use crate::i18n::{t_for, tf_for};
use crate::learn::{FerrisMood, LessonPractice, LessonView, PracticeOutcome};
use crate::theme::Theme;

/// Constrói as linhas do corpo da lição. Separado do desenho porque o
/// `LessonView` precisa saber a altura (scroll/progresso) e os testes
/// querem verificar conteúdo sem terminal.
#[allow(dead_code)]
pub fn build_lines(
    lesson: &Lesson,
    view: &LessonView,
    theme: &Theme,
    unicode: bool,
) -> Vec<Line<'static>> {
    build_section_lines(lesson, view, theme, unicode, true, true, false)
}

pub fn build_lines_for(
    lesson: &Lesson,
    view: &LessonView,
    theme: &Theme,
    unicode: bool,
    english: bool,
) -> Vec<Line<'static>> {
    build_section_lines(lesson, view, theme, unicode, true, true, english)
}

/// Constrói apenas os blocos conceituais (texto, código, callouts e demos)
/// para o painel superior de explicação.
#[allow(dead_code)]
pub fn build_explanation_lines(
    lesson: &Lesson,
    view: &LessonView,
    theme: &Theme,
    unicode: bool,
) -> Vec<Line<'static>> {
    build_section_lines(lesson, view, theme, unicode, true, false, false)
}

pub fn build_explanation_lines_for(
    lesson: &Lesson,
    view: &LessonView,
    theme: &Theme,
    unicode: bool,
    english: bool,
) -> Vec<Line<'static>> {
    build_section_lines(lesson, view, theme, unicode, true, false, english)
}

/// Constrói apenas os blocos de quiz para o painel inferior de tarefa.
#[allow(dead_code)]
pub fn build_quiz_lines(
    lesson: &Lesson,
    view: &LessonView,
    theme: &Theme,
    unicode: bool,
) -> Vec<Line<'static>> {
    build_section_lines(lesson, view, theme, unicode, false, true, false)
}

pub fn build_quiz_lines_for(
    lesson: &Lesson,
    view: &LessonView,
    theme: &Theme,
    unicode: bool,
    english: bool,
) -> Vec<Line<'static>> {
    build_section_lines(lesson, view, theme, unicode, false, true, english)
}

fn build_section_lines(
    lesson: &Lesson,
    view: &LessonView,
    theme: &Theme,
    unicode: bool,
    include_explanation: bool,
    include_quiz: bool,
    english: bool,
) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = Vec::new();
    let mut quiz_serial = 0;
    let mut demo_serial = 0;

    for block in &lesson.blocks {
        match block {
            CBlock::Text { text } => {
                if !include_explanation {
                    continue;
                }
                for l in text.lines() {
                    lines.push(Line::from(Span::styled(plain(l, unicode), theme.base())));
                }
                lines.push(Line::default());
            }
            CBlock::Code {
                file,
                code,
                explain,
            } => {
                if !include_explanation {
                    continue;
                }
                let label = file.clone().unwrap_or_default();
                if !label.is_empty() {
                    lines.push(Line::from(Span::styled(
                        format!("  {} {label}", if unicode { "▚" } else { "#" }),
                        theme.accent_dim,
                    )));
                }
                for (i, l) in code.lines().enumerate() {
                    let mut spans = vec![Span::styled(format!("{:>3} │ ", i + 1), theme.dim())];
                    spans.extend(highlight::highlight_line(l, theme));
                    lines.push(Line::from(spans));
                }
                if let Some(ex) = explain {
                    lines.push(Line::default());
                    for l in ex.lines() {
                        lines.push(Line::from(Span::styled(plain(l, unicode), theme.dim())));
                    }
                }
                lines.push(Line::default());
            }
            CBlock::Callout { kind, text } => {
                if !include_explanation {
                    continue;
                }
                let (glyph, style) = match kind {
                    CalloutKind::Note => (if unicode { "ℹ" } else { "i" }, theme.dim()),
                    CalloutKind::Tip => (if unicode { "★" } else { "*" }, theme.title()),
                    CalloutKind::Warning => (if unicode { "⚠" } else { "!" }, theme.warning()),
                };
                for l in text.lines() {
                    lines.push(Line::from(vec![
                        Span::styled(format!(" {glyph} "), style),
                        Span::styled(plain(l, unicode), theme.base()),
                    ]));
                }
                lines.push(Line::default());
            }
            CBlock::Demo {
                demo,
                name,
                steps,
                caption,
            } => {
                if include_explanation {
                    lines.extend(demo_lines_for(
                        *demo,
                        name,
                        steps,
                        caption,
                        view.demos.get(demo_serial).copied().unwrap_or(0),
                        theme,
                        unicode,
                        english,
                    ));
                    lines.push(Line::default());
                }
                demo_serial += 1;
            }
            CBlock::Quiz {
                question,
                options,
                correct,
                why,
            } => {
                if include_quiz {
                    let st = &view.quizzes[quiz_serial];
                    let qglyph = if unicode { "?" } else { "Q" };
                    for (i, l) in question.lines().enumerate() {
                        let prefix = if i == 0 {
                            format!(" {qglyph} ")
                        } else {
                            "   ".to_string()
                        };
                        lines.push(Line::from(vec![
                            Span::styled(prefix, theme.title().add_modifier(Modifier::BOLD)),
                            Span::styled(plain(l, unicode), theme.title()),
                        ]));
                    }
                    for (oi, opt) in options.iter().enumerate() {
                        let is_correct_option = oi == *correct;
                        let mark = if st.solved.is_some() && is_correct_option {
                            format!("   {}", if unicode { "✓" } else { "OK" })
                        } else {
                            format!("   {}", oi + 1)
                        };
                        let style = if st.solved.is_some() && oi == *correct {
                            theme.success()
                        } else if st.solved.is_some() {
                            theme.dim()
                        } else {
                            theme.base()
                        };
                        lines.push(Line::from(vec![
                            Span::styled(format!("{mark}  "), style),
                            Span::styled(plain(opt, unicode), style),
                        ]));
                    }
                    if st.solved.is_some() {
                        if let Some(w) = why {
                            lines.push(Line::from(Span::styled(
                                format!("   {}", t_for("lesson.why-head", english)),
                                theme.dim(),
                            )));
                            for l in w.lines() {
                                lines.push(Line::from(Span::styled(
                                    format!("   {}", plain(l, unicode)),
                                    theme.dim(),
                                )));
                            }
                        }
                    } else if st.attempts > 0 {
                        lines.push(Line::from(Span::styled(
                            format!("   {}", t_for("lesson.try-again", english)),
                            theme.warning(),
                        )));
                    } else {
                        lines.push(Line::from(Span::styled(
                            format!(
                                "   {}",
                                tf_for("lesson.answer-hint", english, &[&options.len()])
                            ),
                            theme.dim(),
                        )));
                    }
                    lines.push(Line::default());
                }
                quiz_serial += 1;
            }
        }
    }
    lines
}

/// Widget educativo conceitual (requisito 9): cada variante ilustra um
/// conceito passo a passo (`d`) e termina com o aviso explícito de que é
/// uma simplificação visual, não um depurador de runtime.
#[allow(dead_code)]
fn demo_lines(
    kind: DemoKind,
    name: &str,
    steps: &[String],
    caption: &Option<String>,
    step: usize,
    theme: &Theme,
    unicode: bool,
) -> Vec<Line<'static>> {
    demo_lines_for(kind, name, steps, caption, step, theme, unicode, false)
}

#[allow(clippy::too_many_arguments)]
fn demo_lines_for(
    kind: DemoKind,
    name: &str,
    steps: &[String],
    caption: &Option<String>,
    step: usize,
    theme: &Theme,
    unicode: bool,
    english: bool,
) -> Vec<Line<'static>> {
    let shown = steps
        .get(step.min(steps.len().saturating_sub(1)))
        .cloned()
        .unwrap_or_default();
    let (tl, tr, bl, br, vb, hr, arrow, loop_sym) = if unicode {
        ("┌", "┐", "└", "┘", "│", "─", "──►", "↻")
    } else {
        ("+", "+", "+", "+", "|", "-", "-->", "<->")
    };
    let step_lbl = if english { "step" } else { "passo" };
    let mut lines = Vec::new();
    match kind {
        DemoKind::Variables => {
            let head = if english {
                format!(" name: {name} ")
            } else {
                format!(" nome: {name} ")
            };
            let width = head.chars().count() + 2;
            lines.push(Line::from(Span::styled(
                format!("  {tl}{head}{tr}"),
                theme.accent_dim,
            )));
            lines.push(Line::from(vec![
                Span::styled(format!("  {vb} "), theme.accent_dim),
                Span::styled(
                    if english {
                        format!("value = {shown:<18}")
                    } else {
                        format!("valor = {shown:<18}")
                    },
                    theme.base(),
                ),
                Span::styled(vb.to_string(), theme.accent_dim),
            ]));
            lines.push(Line::from(Span::styled(
                format!(
                    "  {bl}{}{br}  {step_lbl} {}/{}",
                    "-".repeat(width),
                    step + 1,
                    steps.len()
                ),
                theme.accent_dim,
            )));
        }
        DemoKind::Loop => {
            lines.push(Line::from(Span::styled(
                if english {
                    format!(
                        "  {tl} loop: {name} {loop_sym} {tr}  {step_lbl} {}/{}",
                        step + 1,
                        steps.len()
                    )
                } else {
                    format!(
                        "  {tl} laço: {name} {loop_sym} {tr}  {step_lbl} {}/{}",
                        step + 1,
                        steps.len()
                    )
                },
                theme.accent_dim,
            )));
            lines.push(Line::from(vec![
                Span::styled(format!("  {vb} "), theme.accent_dim),
                Span::styled(
                    if english {
                        format!("iteration: {shown}")
                    } else {
                        format!("iteração: {shown}")
                    },
                    theme.base(),
                ),
            ]));
            lines.push(Line::from(Span::styled(
                if english {
                    format!("  {bl}{0}{0}{0} back to condition {loop_sym} {br}", hr)
                } else {
                    format!("  {bl}{0}{0}{0} volta à condição {loop_sym} {br}", hr)
                },
                theme.accent_dim,
            )));
        }
        DemoKind::Function => {
            lines.push(Line::from(Span::styled(
                if english {
                    format!(
                        "  {tl} function `{name}` {tr}  {step_lbl} {}/{}",
                        step + 1,
                        steps.len()
                    )
                } else {
                    format!(
                        "  {tl} função `{name}` {tr}  {step_lbl} {}/{}",
                        step + 1,
                        steps.len()
                    )
                },
                theme.accent_dim,
            )));
            lines.push(Line::from(vec![
                Span::styled(format!("  {vb} "), theme.accent_dim),
                Span::styled(
                    if english {
                        format!("flow: {arrow} [ {name} ] {arrow} {shown}")
                    } else {
                        format!("fluxo: {arrow} [ {name} ] {arrow} {shown}")
                    },
                    theme.base(),
                ),
            ]));
            lines.push(Line::from(Span::styled(
                format!("  {bl}{}{br}", hr.repeat(20)),
                theme.accent_dim,
            )));
        }
        DemoKind::StackHeap => {
            lines.push(Line::from(Span::styled(
                format!(
                    "  {tl} STACK {tr}  {arrow}  {tl} HEAP: {name} {tr}  {step_lbl} {}/{}",
                    step + 1,
                    steps.len()
                ),
                theme.accent_dim,
            )));
            lines.push(Line::from(vec![
                Span::styled(format!("  {vb} "), theme.accent_dim),
                Span::styled(shown, theme.base()),
            ]));
            lines.push(Line::from(Span::styled(
                format!("  {bl}{}{br}", hr.repeat(14)),
                theme.accent_dim,
            )));
        }
        DemoKind::Ownership => {
            lines.push(Line::from(Span::styled(
                if english {
                    format!(
                        "  {tl} ownership: {name} {tr}  {step_lbl} {}/{}",
                        step + 1,
                        steps.len()
                    )
                } else {
                    format!(
                        "  {tl} ownership (posse): {name} {tr}  {step_lbl} {}/{}",
                        step + 1,
                        steps.len()
                    )
                },
                theme.accent_dim,
            )));
            lines.push(Line::from(vec![
                Span::styled(format!("  {vb} "), theme.accent_dim),
                Span::styled(
                    if english {
                        format!("active owner {arrow} {shown}")
                    } else {
                        format!("dono ativo {arrow} {shown}")
                    },
                    theme.base(),
                ),
            ]));
            lines.push(Line::from(Span::styled(
                format!("  {bl}{}{br}", hr.repeat(24)),
                theme.accent_dim,
            )));
        }
        DemoKind::Borrowing => {
            lines.push(Line::from(Span::styled(
                if english {
                    format!(
                        "  {tl} borrowing: {name} {tr}  {step_lbl} {}/{}",
                        step + 1,
                        steps.len()
                    )
                } else {
                    format!(
                        "  {tl} borrowing (empréstimo): {name} {tr}  {step_lbl} {}/{}",
                        step + 1,
                        steps.len()
                    )
                },
                theme.accent_dim,
            )));
            lines.push(Line::from(vec![
                Span::styled(format!("  {vb} "), theme.accent_dim),
                Span::styled(
                    if english {
                        format!("access: {shown}")
                    } else {
                        format!("acesso: {shown}")
                    },
                    theme.base(),
                ),
            ]));
            lines.push(Line::from(Span::styled(
                format!("  {bl}{}{br}", hr.repeat(28)),
                theme.accent_dim,
            )));
        }
        DemoKind::Vector => {
            lines.push(Line::from(Span::styled(
                format!(
                    "  {tl} Vec<{name}> {tr}  {step_lbl} {}/{}",
                    step + 1,
                    steps.len()
                ),
                theme.accent_dim,
            )));
            lines.push(Line::from(vec![
                Span::styled(format!("  {vb} "), theme.accent_dim),
                Span::styled(
                    if english {
                        format!("elements: {shown}")
                    } else {
                        format!("elementos: {shown}")
                    },
                    theme.base(),
                ),
            ]));
            lines.push(Line::from(Span::styled(
                format!("  {bl}{}{br}", hr.repeat(18)),
                theme.accent_dim,
            )));
        }
    }
    if let Some(c) = caption {
        lines.push(Line::from(Span::styled(
            format!("   {}  (d)", plain(c, unicode)),
            theme.dim(),
        )));
    }
    lines.push(Line::from(Span::styled(
        t_for("demo.simplified", english),
        theme.dim(),
    )));
    lines
}

fn plain(s: &str, unicode: bool) -> String {
    if unicode {
        s.to_string()
    } else {
        s.replace('·', "*")
            .replace('→', "->")
            .replace('—', "-")
            .replace(['“', '”'], "\"")
            .replace('✓', "x")
            .replace('≡', "=")
            .replace('◐', "~")
            .replace('◦', "o")
            .replace('ℹ', "i")
            .replace('★', "*")
            .replace('⚠', "!")
            .replace('▚', "#")
    }
}

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let en = app.is_english();
    let theme = app.theme;
    let unicode = app.unicode;
    let Some(view) = app.lesson_view.clone() else {
        return;
    };
    let Some((lesson, ch_title, pos, related_titles)) = (|| {
        if let Some(custom) = view.custom_lesson.as_ref() {
            let entry = view
                .v3_entry_id
                .and_then(crate::curriculum::v3_blueprint::find_v3_entry);
            let ch_title = entry
                .map(|e| e.stage.label_for(en).to_string())
                .unwrap_or_else(|| {
                    if en {
                        "V3 Track".to_string()
                    } else {
                        "Trilha V3".to_string()
                    }
                });
            let pos = view
                .v3_entry_id
                .and_then(crate::curriculum::v3_blueprint::v3_entry_index)
                .map(|i| format!("{}/213", i + 1))
                .unwrap_or_else(|| "V3".to_string());
            let related = if let (Some(cur), Some(ent)) = (app.curriculum.as_ref(), entry) {
                let canon = crate::curriculum::v3_blueprint::canonical_lesson_for_v3_entry(ent);
                cur.exercises()
                    .filter(|ex| ex.lesson_ids.iter().any(|l| l == canon))
                    .map(|ex| ex.title.clone())
                    .collect()
            } else {
                Vec::new()
            };
            return Some((custom.clone(), ch_title, pos, related));
        }
        let cur = app.curriculum.as_ref()?;
        let ch = cur.chapters.get(view.chapter)?;
        let id = ch.lessons.get(view.lesson)?;
        let lesson = cur.lesson(id)?.clone();
        let pos = global_position(cur, &view);
        let related: Vec<String> = cur
            .exercises()
            .filter(|ex| ex.lesson_ids.contains(id))
            .map(|ex| ex.title.clone())
            .collect();
        Some((lesson, ch.title.clone(), pos, related))
    })() else {
        return;
    };

    let mut lesson = lesson;
    if !app.snapshot_mode
        && app.config.origin_language != crate::config::OriginLanguage::None
        && let Some(callout) = app
            .config
            .origin_language
            .polyglot_callout_for(&lesson.id, en)
    {
        let insert_at = usize::from(!lesson.blocks.is_empty());
        lesson.blocks.insert(
            insert_at,
            crate::curriculum::Block::Callout {
                kind: crate::curriculum::CalloutKind::Tip,
                text: callout,
            },
        );
    }

    let rows = Layout::vertical([
        Constraint::Length(if related_titles.is_empty() { 2 } else { 3 }), // cabeçalho
        Constraint::Min(3),                                                // corpo
        Constraint::Length(1),                                             // barra de progresso
    ])
    .split(area);

    // ---- cabeçalho ------------------------------------------------------
    let title = Line::from(vec![
        Span::styled(
            format!(" {} ", lesson.title),
            theme.title().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("· {} · ~{} min", ch_title, lesson.estimated_minutes),
            theme.dim(),
        ),
    ]);
    frame.render_widget(Paragraph::new(title), rows[0]);
    let nav_hint = Line::from(Span::styled(t_for("lesson.nav-hint", en), theme.dim()));
    frame.render_widget(
        nav_hint,
        Rect {
            y: rows[0].y + 1,
            ..rows[0]
        },
    );
    if !related_titles.is_empty() {
        frame.render_widget(
            Paragraph::new(if en {
                format!(
                    " e practice · {} exercise(s): {}",
                    related_titles.len(),
                    related_titles.join(", ")
                )
            } else {
                format!(
                    " e praticar · {} exercício(s): {}",
                    related_titles.len(),
                    related_titles.join(", ")
                )
            })
            .style(theme.title()),
            Rect {
                y: rows[0].y + 2,
                height: 1,
                ..rows[0]
            },
        );
    }

    // ---- corpo: TOPO explicação + BAIXO tarefa/mini-IDE + Ferris --------
    let body = rows[1];
    let has_split = body.height >= 12;
    let exp_lines = build_explanation_lines_for(&lesson, &view, &theme, unicode, en);
    let quiz_lines = build_quiz_lines_for(&lesson, &view, &theme, unicode, en);

    let (top_area, bottom_area_opt) = if has_split {
        let bottom_h = if view.practice_focused {
            (body.height / 2).max(11).min(body.height.saturating_sub(6))
        } else if !quiz_lines.is_empty() {
            10u16.min(body.height.saturating_sub(6))
        } else {
            7u16.min(body.height.saturating_sub(6))
        };
        let split =
            Layout::vertical([Constraint::Min(6), Constraint::Length(bottom_h)]).split(body);
        (split[0], Some(split[1]))
    } else {
        (body, None)
    };

    // Painel superior de explicação (alinhado à esquerda, com scroll real)
    let (scroll, max_scroll) = if let Some(bottom_area) = bottom_area_opt {
        let exp_block = Block::bordered()
            .title(if en {
                " Explanation · ↑↓ scroll "
            } else {
                " Explicação · ↑↓ rolar "
            })
            .border_style(theme.border_style(!view.practice_focused));
        let inner = exp_block.inner(top_area);
        let para = Paragraph::new(exp_lines).wrap(Wrap { trim: false });
        let total = para.line_count(inner.width.max(1));
        let max = total.saturating_sub(inner.height as usize);
        let scroll = sync_scroll(app, max);
        frame.render_widget(exp_block, top_area);
        frame.render_widget(para.scroll((scroll as u16, 0)), inner);
        draw_bottom_panel(
            frame,
            app,
            bottom_area,
            &view,
            quiz_lines,
            &related_titles,
            &theme,
            en,
        );
        (scroll, max)
    } else {
        // Terminais muito baixos (< 12 linhas no corpo): degrada sem cortes
        let full_lines = build_lines_for(&lesson, &view, &theme, unicode, en);
        let para = Paragraph::new(full_lines).wrap(Wrap { trim: false });
        let total = para.line_count(top_area.width.max(1));
        let max = total.saturating_sub(top_area.height as usize);
        let scroll = sync_scroll(app, max);
        frame.render_widget(para.scroll((scroll as u16, 0)), top_area);
        (scroll, max)
    };

    // Modal de consentimento se a prática embutida pediu `cargo run`
    if let Some(p) = app.lesson_view.as_ref().and_then(|v| v.practice.as_ref())
        && p.consent_pending
    {
        let r = centered_rect(area, 72, 9);
        frame.render_widget(Clear, r);
        frame.render_widget(
            Paragraph::new(if en {
                "Running this practice compiles and runs your local code with Cargo; this is NOT sandboxed. The process uses your user permissions.\n\nNothing has run yet. Enter accepts; Esc declines."
            } else {
                "Executar esta prática compila e executa seu código local com Cargo; isso NÃO é sandbox. O processo usa as permissões da sua conta.\n\nNada foi executado ainda. Enter aceita; Esc recusa."
            })
                .wrap(Wrap { trim: true })
                .block(
                    Block::bordered()
                        .title(if en {
                            " Consent required (F5) "
                        } else {
                            " Consentimento necessário (F5) "
                        })
                        .border_style(theme.border_style(true)),
                ),
            r,
        );
    }

    // ---- barra de progresso ---------------------------------------------
    let pct = if max_scroll == 0 {
        100
    } else {
        (scroll as u32 * 100 / max_scroll as u32).min(100)
    };
    let done = app
        .lesson_view
        .as_ref()
        .map(|v| v.is_finished())
        .unwrap_or_else(|| view.is_finished());
    let label = if done {
        tf_for("lesson.progress-done", en, &[&pos])
    } else {
        tf_for("lesson.progress", en, &[&pos, &pct])
    };
    let gauge = Gauge::default()
        .ratio(if done { 1.0 } else { pct as f64 / 100.0 })
        .label(label)
        .style(theme.dim())
        .gauge_style(Style::default().fg(theme.accent));
    frame.render_widget(gauge, rows[2]);
}

fn sync_scroll(app: &mut App, max_scroll: usize) -> usize {
    if let Some(v) = app.lesson_view.as_mut() {
        v.last_height = max_scroll;
        v.scroll = v.scroll.min(max_scroll);
        if v.scroll >= max_scroll {
            v.at_bottom = true;
        }
        v.scroll
    } else {
        0
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_bottom_panel(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    view: &LessonView,
    quiz_lines: Vec<Line<'static>>,
    related_titles: &[String],
    theme: &Theme,
    english: bool,
) {
    let show_ferris = area.width >= 68;
    let (task_area, ferris_area) = if show_ferris {
        let split = Layout::horizontal([Constraint::Min(38), Constraint::Length(26)]).split(area);
        (split[0], Some(split[1]))
    } else {
        (area, None)
    };

    if view.practice_focused
        && let Some(practice) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut())
    {
        draw_embedded_practice(frame, task_area, practice, theme, english);
    } else {
        draw_quiz_or_task_summary(
            frame,
            task_area,
            view,
            quiz_lines,
            related_titles,
            theme,
            english,
        );
    }

    if let Some(f_area) = ferris_area {
        draw_ferris_box(
            frame,
            f_area,
            view.ferris,
            view.practice.as_ref(),
            theme,
            english,
        );
    }
}

fn draw_quiz_or_task_summary(
    frame: &mut Frame,
    area: Rect,
    view: &LessonView,
    mut quiz_lines: Vec<Line<'static>>,
    related_titles: &[String],
    theme: &Theme,
    english: bool,
) {
    let title = if let Some(p) = &view.practice {
        if english {
            format!(" Task & Quiz · Tab open Mini-IDE ({}) ", p.spec.id)
        } else {
            format!(" Tarefa & Quiz · Tab abrir Mini-IDE ({}) ", p.spec.id)
        }
    } else if !quiz_lines.is_empty() {
        if english {
            " Review Quiz · 1-9 answer · → next ".to_string()
        } else {
            " Quiz de fixação · 1-9 responder · → avançar ".to_string()
        }
    } else if !related_titles.is_empty() {
        if english {
            " Related practice · e open exercises ".to_string()
        } else {
            " Prática relacionada · e abrir exercícios ".to_string()
        }
    } else if english {
        " Summary & Next step ".to_string()
    } else {
        " Resumo & Próximo passo ".to_string()
    };

    if quiz_lines.is_empty() {
        if let Some(p) = &view.practice {
            quiz_lines.push(Line::from(Span::styled(
                format!("{} — {}", p.spec.id, p.spec.prompt),
                theme.base(),
            )));
            quiz_lines.push(Line::from(Span::styled(
                if english {
                    "Press Tab to type in the embedded editor and F5 to run Cargo (100+100 XP with no hints)."
                } else {
                    "Pressione Tab para digitar no editor embutido e F5 para rodar Cargo (100+100 XP sem dica)."
                },
                theme.title(),
            )));
        } else if !related_titles.is_empty() {
            quiz_lines.push(Line::from(Span::styled(
                if english {
                    format!(
                        "Exercise(s) for this lesson: {}. Press `e` to practice with Cargo tests.",
                        related_titles.join(", ")
                    )
                } else {
                    format!(
                        "Exercício(s) desta lição: {}. Pressione `e` para praticar com testes Cargo.",
                        related_titles.join(", ")
                    )
                },
                theme.base(),
            )));
        } else {
            quiz_lines.push(Line::from(Span::styled(
                if english {
                    "Scroll to the end of the explanation above (`End`) and press `→` or `Enter` to continue."
                } else {
                    "Role até o fim da explicação acima (`End`) e pressione `→` ou `Enter` para avançar."
                },
                theme.base(),
            )));
        }
    } else if let Some(p) = &view.practice {
        quiz_lines.push(Line::from(Span::styled(
            if english {
                format!(
                    "→ Mini-IDE {}: press Tab to write and run with Cargo (`F5`).",
                    p.spec.id
                )
            } else {
                format!(
                    "→ Mini-IDE {}: pressione Tab para escrever e rodar com Cargo (`F5`).",
                    p.spec.id
                )
            },
            theme.title(),
        )));
    }

    frame.render_widget(
        Paragraph::new(quiz_lines).wrap(Wrap { trim: true }).block(
            Block::bordered().title(title).border_style(
                theme.border_style(!view.practice_focused && view.practice.is_none()),
            ),
        ),
        area,
    );
}

fn draw_embedded_practice(
    frame: &mut Frame,
    area: Rect,
    practice: &mut LessonPractice,
    theme: &Theme,
    english: bool,
) {
    let out_h = 5u16.min(area.height.saturating_sub(5));
    let panes = Layout::vertical([Constraint::Min(5), Constraint::Length(out_h)]).split(area);

    let editor_rect = Rect {
        x: panes[0].x.saturating_add(1),
        y: panes[0].y.saturating_add(1),
        width: panes[0].width.saturating_sub(2),
        height: panes[0].height.saturating_sub(2),
    };
    practice.editor.ensure_visible(
        editor_rect.height as usize,
        editor_rect.width.saturating_sub(6) as usize,
    );

    let editor = &practice.editor;
    let code_lines: Vec<Line<'static>> = (0..editor.buf.line_count())
        .map(|row| {
            let source = editor.buf.line_str(row);
            let mut spans = vec![Span::styled(format!("{:>3} │ ", row + 1), theme.dim())];
            let cursor_col = (editor.cursor.line == row).then_some(editor.cursor.col);
            let mut col = 0;
            for span in highlight::highlight_line(&source, theme) {
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
        .collect();

    let ed_title = format!(
        " {} · {} ",
        practice.spec.id,
        match (editor.dirty, english) {
            (true, true) => "unsaved · Ctrl+S · F5 run",
            (true, false) => "não salvo · Ctrl+S · F5 rodar",
            (false, true) => "F5 Cargo run · Ctrl+S save · Tab read",
            (false, false) => "F5 Cargo run · Ctrl+S salvar · Tab leitura",
        }
    );
    frame.render_widget(
        Paragraph::new(code_lines)
            .scroll((editor.scroll_row as u16, editor.scroll_col as u16))
            .block(
                Block::bordered()
                    .title(ed_title)
                    .border_style(theme.border_style(true)),
            ),
        panes[0],
    );

    let status_line = match practice.outcome {
        PracticeOutcome::NotRun => Line::from(vec![
            Span::styled(if english { "Goal: " } else { "Objetivo: " }, theme.title()),
            Span::styled(practice.spec.prompt, theme.base()),
        ]),
        PracticeOutcome::Running => Line::from(Span::styled(
            if english {
                "Running `cargo run`… please wait."
            } else {
                "Executando `cargo run`… aguarde."
            },
            theme.title(),
        )),
        PracticeOutcome::Passed => Line::from(Span::styled(
            if english {
                "✓ Passed Cargo checks! You can advance or keep experimenting."
            } else {
                "✓ Aprovado pelo Cargo! Pode avançar ou experimentar mais."
            },
            theme.success(),
        )),
        PracticeOutcome::CompileFailed
        | PracticeOutcome::RunFailed
        | PracticeOutcome::ValidationFailed
        | PracticeOutcome::TimedOut
        | PracticeOutcome::StartFailed => Line::from(Span::styled(
            if english {
                "! Check the output/error below or press F1 for a hint."
            } else {
                "! Confira a saída/erro abaixo ou pressione F1 para dica."
            },
            theme.warning(),
        )),
        PracticeOutcome::Cancelled => Line::from(Span::styled(
            if english {
                "Execution cancelled."
            } else {
                "Execução cancelada."
            },
            theme.dim(),
        )),
    };

    let mut body_lines = vec![status_line];
    if practice.outcome != PracticeOutcome::NotRun && !practice.output.is_empty() {
        let total = practice.output.len();
        let start = total.saturating_sub(practice.output_scroll + 2);
        for line in practice.output.iter().skip(start).take(3) {
            body_lines.push(Line::from(Span::styled(line.clone(), theme.base())));
        }
    } else if practice.shown_hints > 0 {
        let study_tag = if practice.hint_index == 3 {
            if english {
                " [Study/Review Mode]"
            } else {
                " [Modo Estudo/Revisão]"
            }
        } else {
            ""
        };
        body_lines.push(Line::from(Span::styled(
            format!(
                "{} {}/4{study_tag}: {}",
                if english { "Hint" } else { "Dica" },
                practice.hint_index + 1,
                practice.spec.hints[practice.hint_index]
            ),
            theme.dim(),
        )));
    }

    let hint_badge = match (practice.outcome == PracticeOutcome::NotRun, english) {
        (true, true) => format!(
            " Output & Hints · F1 ({}/4) · PgUp/PgDn ",
            practice.shown_hints
        ),
        (true, false) => format!(
            " Saída & Dicas · F1 ({}/4) · PgUp/PgDn ",
            practice.shown_hints
        ),
        (false, true) => format!(
            " Output & Hints {} · F1 ({}/4) · PgUp/PgDn ",
            practice.outcome.status_badge_for(true),
            practice.shown_hints
        ),
        (false, false) => format!(
            " Saída & Dicas {} · F1 ({}/4) · PgUp/PgDn ",
            practice.outcome.status_badge_for(false),
            practice.shown_hints
        ),
    };
    frame.render_widget(
        Paragraph::new(body_lines).wrap(Wrap { trim: true }).block(
            Block::bordered()
                .title(hint_badge)
                .border_style(theme.border_style(false)),
        ),
        panes[1],
    );
}

fn draw_ferris_box(
    frame: &mut Frame,
    area: Rect,
    mood: FerrisMood,
    practice: Option<&LessonPractice>,
    theme: &Theme,
    english: bool,
) {
    let mood_style = match mood {
        FerrisMood::Neutral => theme.title().add_modifier(Modifier::BOLD),
        FerrisMood::Happy => theme.success().add_modifier(Modifier::BOLD),
        FerrisMood::Encouraging | FerrisMood::Confused => {
            theme.warning().add_modifier(Modifier::BOLD)
        }
    };
    let mut lines = vec![
        Line::from(vec![
            Span::styled(format!("  {}  ", mood.ascii()), mood_style),
            Span::styled(mood.label_for(english), theme.base()),
        ]),
        Line::from(Span::styled(mood.caption_for(english), theme.dim())),
    ];
    if let Some(p) = practice {
        let (base, bonus) = crate::learn::calculate_v3_practice_xp(p.shown_hints, p.ai_used);
        let xp_text = match (bonus > 0, english) {
            (true, true) => format!("Possible XP: {base}+{bonus} (no hints)"),
            (true, false) => format!("XP possível: {base}+{bonus} (sem dicas)"),
            (false, true) => format!("Possible XP: {base} (hints: {})", p.shown_hints),
            (false, false) => format!("XP possível: {base} (dicas: {})", p.shown_hints),
        };
        lines.push(Line::from(Span::styled(xp_text, theme.dim())));
    }
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::bordered()
                .title(" Ferris ")
                .border_style(theme.border_style(false)),
        ),
        area,
    );
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width.saturating_sub(2));
    let h = height.min(area.height.saturating_sub(2));
    Rect {
        x: area.x + area.width.saturating_sub(w) / 2,
        y: area.y + area.height.saturating_sub(h) / 2,
        width: w,
        height: h,
    }
}

fn global_position(cur: &crate::curriculum::Curriculum, view: &LessonView) -> String {
    let order = cur.lesson_order();
    order
        .iter()
        .position(|(c, l, _)| *c == view.chapter && *l == view.lesson)
        .map(|p| format!("{}/{}", p + 1, order.len()))
        .unwrap_or_else(|| "?".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curriculum::Curriculum;

    fn cur() -> Curriculum {
        Curriculum::embedded().unwrap()
    }

    fn lines_of(chapter: usize, lesson_idx: usize) -> Vec<String> {
        let c = cur();
        let ch = &c.chapters[chapter];
        let l = c.lesson(&ch.lessons[lesson_idx]).unwrap();
        let v = LessonView::new(chapter, lesson_idx, l);
        let theme = Theme::for_id(crate::config::ThemeId::Cuia);
        build_lines(l, &v, &theme, true)
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect()
    }

    #[test]
    fn licao_com_codigo_aparece_realcado_com_numeracao() {
        let lines = lines_of(0, 3); // hello-world
        let joined = lines.join("\n");
        assert!(joined.contains("1 │ "), "faltou numeração: {joined}");
        assert!(joined.contains("fn main() {"), "faltou código: {joined}");
        assert!(joined.contains("src/main.rs"), "faltou nome do arquivo");
    }

    #[test]
    fn demo_de_variaveis_rende_caixa_com_passo_atual() {
        let lines = lines_of(0, 1); // como-um-programa-executa tem demo
        let joined = lines.join("\n");
        assert!(joined.contains("nome: passo"), "{joined}");
        assert!(joined.contains("valor = você escreve o código"));
        assert!(joined.contains("passo 1/4"));
        // e o rótulo honesto de simplificação (requisito 9)
        assert!(joined.contains("ilustração conceitual"));
    }

    #[test]
    fn quiz_mostra_dica_de_resposta_antes_de_tentar() {
        let lines = lines_of(0, 0); // tem quiz
        let joined = lines.join("\n");
        assert!(joined.contains("responda com as teclas 1 a 4"), "{joined}");
        assert!(joined.contains("Qual frase melhor descreve"));
    }

    #[test]
    fn texto_plano_tem_glifos_ascii_para_terminal_sem_unicode() {
        assert_eq!(plain("a · b → c — d", false), "a * b -> c - d");
        assert_eq!(plain("a · b", true), "a · b");
    }

    #[test]
    fn todos_os_demos_renderizam_com_rotulo_conceitual_e_ascii() {
        let theme = Theme::for_id(crate::config::ThemeId::Cuia);
        let steps = vec!["passo A".to_string(), "passo B".to_string()];
        let cap = Some("legenda".to_string());
        for kind in [
            DemoKind::Variables,
            DemoKind::Loop,
            DemoKind::Function,
            DemoKind::StackHeap,
            DemoKind::Ownership,
            DemoKind::Borrowing,
            DemoKind::Vector,
        ] {
            for unicode in [true, false] {
                let rendered = demo_lines(kind, "x", &steps, &cap, 1, &theme, unicode)
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(rendered.contains("passo 2/2"), "{kind:?}: {rendered}");
                assert!(
                    rendered.contains("ilustração conceitual"),
                    "{kind:?}: {rendered}"
                );
                if !unicode {
                    assert!(!rendered.contains('┌'), "{kind:?} vazou unicode");
                    assert!(!rendered.contains('─'), "{kind:?} vazou unicode");
                    assert!(!rendered.contains('►'), "{kind:?} vazou unicode");
                }
            }
        }
    }
}
