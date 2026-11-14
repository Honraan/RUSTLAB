//! Navegador de curso: capítulos à esquerda, lições do selecionado à
//! direita. Capítulos planejados permanecem visíveis (honestidade sobre o
//! roadmap), sem entrar.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::app::App;
use crate::curriculum::ChapterStatus;
use crate::i18n::t;
use crate::learn::BrowserPane;
use crate::ui::titled_block;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let Some(cur) = app.curriculum.as_ref() else {
        frame.render_widget(
            Paragraph::new(Span::styled(t("learn.empty"), theme.warning())),
            area,
        );
        return;
    };

    if app.browser.v3_mode {
        draw_v3_browser(frame, app, area);
        return;
    }

    let cols = Layout::horizontal([Constraint::Length(38), Constraint::Min(24)]).split(area);

    // ---- capítulos ------------------------------------------------------
    let items: Vec<ListItem> = cur
        .chapters
        .iter()
        .map(|ch| {
            let done = ch
                .lessons
                .iter()
                .filter(|id| app.progress.completed.contains(id.as_str()))
                .count();
            let total = ch.lessons.len();
            let (icon, icon_style, tail) = match ch.status {
                ChapterStatus::Available if total == 0 => ("…", theme.dim(), String::new()),
                ChapterStatus::Available if done == total && total > 0 => {
                    ("✓", theme.success(), format!(" {done}/{total}"))
                }
                ChapterStatus::Available => ("≡", theme.title(), format!(" {done}/{total}")),
                ChapterStatus::Planned => {
                    ("◦", theme.dim(), format!(" · {}", t("browser.planned-tag")))
                }
            };
            let short = ch.title.split('—').next_back().unwrap_or(&ch.title).trim();
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {icon} "), icon_style),
                Span::styled(short.to_string(), theme.base()),
                Span::styled(tail, theme.dim()),
            ]))
        })
        .collect();

    let focused = app.browser.focus == BrowserPane::Chapters;
    let chapters_list = List::new(items)
        .highlight_symbol(if app.unicode { "▶ " } else { "> " })
        .highlight_style(theme.selection());
    let block = titled_block(t("browser.title"), theme, focused);
    let inner = block.inner(cols[0]);
    frame.render_widget(block, cols[0]);
    let mut state = ListState::default().with_selected(Some(app.browser.chapter_idx));
    frame.render_stateful_widget(chapters_list, inner, &mut state);

    // ---- lições do capítulo selecionado ---------------------------------
    let ch = &cur.chapters[app.browser.chapter_idx.min(cur.chapters.len() - 1)];
    let lesson_items: Vec<ListItem> = match ch.status {
        ChapterStatus::Planned => vec![
            ListItem::new(Line::from(Span::styled(
                format!("   {}", t("browser.planned-line1"),),
                theme.dim(),
            ))),
            ListItem::new(Line::from(Span::styled(
                format!(
                    "   {}",
                    t("browser.planned-line2").replacen(
                        "{}",
                        ch.planned_phase.unwrap_or(9).to_string().as_str(),
                        1
                    ),
                ),
                theme.dim(),
            ))),
        ],
        ChapterStatus::Available => ch
            .lessons
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let lesson = cur.lesson(id);
                let title = lesson
                    .map(|l| l.title.clone())
                    .unwrap_or_else(|| id.clone());
                let minutes = lesson.map(|l| l.estimated_minutes).unwrap_or(0);
                let done = app.progress.completed.contains(id.as_str());
                let read = app.progress.read.contains(id.as_str());
                let mark = if done {
                    Span::styled(if app.unicode { "✓ " } else { "[x] " }, theme.success())
                } else if read {
                    Span::styled(if app.unicode { "◐ " } else { "[~] " }, theme.warning())
                } else {
                    Span::styled(format!("{} ", i + 1), theme.dim())
                };
                ListItem::new(Line::from(vec![
                    mark,
                    Span::styled(title, theme.base()),
                    Span::styled(format!("   ~{minutes} min"), theme.dim()),
                ]))
            })
            .collect(),
    };

    let lfocused = app.browser.focus == BrowserPane::Lessons;
    let lessons_list = List::new(lesson_items)
        .highlight_symbol(if app.unicode { "▶ " } else { "> " })
        .highlight_style(theme.selection());
    let panel_title = format!("{} — {}", ch.title, t("browser.lessons"));
    let lblock = titled_block(&panel_title, theme, lfocused);
    let linner = lblock.inner(cols[1]);
    frame.render_widget(lblock, cols[1]);
    let mut lstate = ListState::default().with_selected(match ch.status {
        ChapterStatus::Available => Some(app.browser.lesson_idx),
        ChapterStatus::Planned => None,
    });
    frame.render_stateful_widget(lessons_list, linner, &mut lstate);
    // resumo do capítulo na última linha do painel direito
    if linner.height > 0 {
        let summary_area = Rect {
            x: linner.x,
            y: linner.bottom().saturating_sub(1),
            width: linner.width,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(Span::styled(ch.summary.clone(), theme.dim())),
            summary_area,
        );
    }
}

fn draw_v3_browser(frame: &mut Frame, app: &App, area: Rect) {
    use crate::curriculum::v3_blueprint::{
        V3EntryKind, V3Stage, entries_for_stage, is_v3_entry_completed, is_v3_entry_unlocked,
        is_v3_stage_unlocked,
    };

    let theme = &app.theme;
    let preview = app.browser.v3_preview_unlocked;
    let cols = Layout::horizontal([Constraint::Length(38), Constraint::Min(24)]).split(area);

    let stage_items: Vec<ListItem> = V3Stage::ALL
        .iter()
        .map(|&stage| {
            let stage_entries = entries_for_stage(stage);
            let done = stage_entries
                .iter()
                .filter(|e| is_v3_entry_completed(&app.progress, e))
                .count();
            let total = stage_entries.len();
            let unlocked = is_v3_stage_unlocked(&app.progress, stage, preview);
            let (icon, icon_style) = if done == total && total > 0 {
                (if app.unicode { "✓" } else { "OK" }, theme.success())
            } else if unlocked {
                (if app.unicode { "≡" } else { "=" }, theme.title())
            } else {
                (if app.unicode { "×" } else { "L" }, theme.dim())
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {icon} "), icon_style),
                Span::styled(stage.label(), theme.base()),
                Span::styled(format!("  {done}/{total}"), theme.dim()),
            ]))
        })
        .collect();

    let stage_idx = app.browser.chapter_idx.min(V3Stage::ALL.len() - 1);
    let stage = V3Stage::ALL[stage_idx];
    let focused = app.browser.focus == BrowserPane::Chapters;
    let stage_title = if preview {
        "Trilha V3 (213) · Prévia Livre"
    } else {
        "Trilha V3 (213 lições)"
    };
    let sblock = titled_block(stage_title, theme, focused);
    let sinner = sblock.inner(cols[0]);
    frame.render_widget(sblock, cols[0]);
    let mut sstate = ListState::default().with_selected(Some(stage_idx));
    frame.render_stateful_widget(
        List::new(stage_items)
            .highlight_symbol(if app.unicode { "▶ " } else { "> " })
            .highlight_style(theme.selection()),
        sinner,
        &mut sstate,
    );

    let entries = entries_for_stage(stage);
    let entry_items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let done = is_v3_entry_completed(&app.progress, e);
            let unlocked = is_v3_entry_unlocked(&app.progress, e, preview);
            let read = app.progress.read.contains(e.id);
            let mark = if done {
                Span::styled(if app.unicode { "✓ " } else { "[x] " }, theme.success())
            } else if !unlocked {
                Span::styled(if app.unicode { "× " } else { "[L] " }, theme.dim())
            } else if read {
                Span::styled(if app.unicode { "◐ " } else { "[~] " }, theme.warning())
            } else {
                Span::styled(format!("{:>2} ", i + 1), theme.dim())
            };
            let badge = match e.kind {
                V3EntryKind::Theory => "[Teoria]",
                V3EntryKind::Practice => "[Prática Cargo]",
                V3EntryKind::CapstoneMilestone => "[Capstone]",
            };
            ListItem::new(Line::from(vec![
                mark,
                Span::styled(format!("[{}] {}", e.id, e.title), theme.base()),
                Span::styled(format!("  {badge}"), theme.dim()),
            ]))
        })
        .collect();

    let lfocused = app.browser.focus == BrowserPane::Lessons;
    let selected_li = app.browser.lesson_idx.min(entries.len().saturating_sub(1));
    let lblock = titled_block(&format!("{} — v clássica", stage.label()), theme, lfocused);
    let linner = lblock.inner(cols[1]);
    frame.render_widget(lblock, cols[1]);
    let mut lstate =
        ListState::default().with_selected((!entries.is_empty()).then_some(selected_li));
    frame.render_stateful_widget(
        List::new(entry_items)
            .highlight_symbol(if app.unicode { "▶ " } else { "> " })
            .highlight_style(theme.selection()),
        linner,
        &mut lstate,
    );
    if linner.height > 0 {
        let summary_text = entries
            .get(selected_li)
            .map(|e| format!("{}: {}", e.id, e.learning_goal))
            .unwrap_or_else(|| stage.subtitle().to_string());
        let summary_area = Rect {
            x: linner.x,
            y: linner.bottom().saturating_sub(1),
            width: linner.width,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(Span::styled(summary_text, theme.dim())),
            summary_area,
        );
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::app::App;
    use crate::audio::{AudioBackend, AudioReport};
    use crate::config::Config;
    use crate::curriculum::Curriculum;
    use crate::learn::PracticeOutcome;

    #[test]
    fn v3_browser_e_licao_paint_com_badge_e_dica4_renderizam() {
        let mut app = App::from_config(
            Config::default(),
            None,
            true,
            AudioReport {
                backend: AudioBackend::None,
                detail: String::new(),
            },
        );
        app.curriculum = Some(Curriculum::embedded().unwrap());
        assert!(app.browser.v3_mode);
        app.browser.v3_preview_unlocked = true;

        let mut term = Terminal::new(TestBackend::new(90, 30)).unwrap();
        term.draw(|f| super::draw(f, &app, f.area())).unwrap();
        let buf_text: String = term
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(buf_text.contains("O BÁSICO") || buf_text.contains("Trilha V3"));

        app.browser.v3_mode = false;
        term.draw(|f| super::draw(f, &app, f.area())).unwrap();
        let classic_text: String = term
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(classic_text.contains("Etapas do curso"));
        app.browser.v3_mode = true;

        app.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        crate::learn::open_v3_entry(&mut app, "PAINT-01");
        if let Some(view) = app.lesson_view.as_mut() {
            view.practice_focused = true;
            if let Some(p) = view.practice.as_mut() {
                p.outcome = PracticeOutcome::NotRun;
                p.shown_hints = 4;
                p.hint_index = 3;
            }
        }
        term.draw(|f| crate::ui::lesson::draw(f, &mut app, f.area()))
            .unwrap();
        let lesson_text: String = term
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(lesson_text.contains("Modo Estudo/Revisão"));
        assert!(lesson_text.contains("MINI PAINT"));

        if let Some(p) = app.lesson_view.as_mut().and_then(|v| v.practice.as_mut()) {
            p.outcome = PracticeOutcome::Passed;
        }
        term.draw(|f| crate::ui::lesson::draw(f, &mut app, f.area()))
            .unwrap();
        let passed_text: String = term
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(passed_text.contains("[Aprovado]"));
    }
}
