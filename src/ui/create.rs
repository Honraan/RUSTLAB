use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, List, ListItem, Paragraph, Wrap};

use crate::app::App;
use crate::config::{AiProvider, AiSettings};
use crate::create::{AgentRole, CreateState, CreateView, Focus};
use crate::execution::StreamKind;
use crate::i18n::bi;

struct ExecPanelState<'a> {
    running: bool,
    running_label: Option<&'a str>,
    input_open: bool,
}

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let en = app.is_english();
    let theme = app.theme;
    let config = app.config.clone();
    let running = app.exec.running;
    let running_label = app.exec.running_label.clone();
    let input_open = app.exec.accepts_input();
    let Some(state) = app.create.as_mut() else {
        return;
    };
    match state.view {
        CreateView::Start => draw_start(frame, &config, state, area, en),
        CreateView::Workspace => draw_workspace(
            frame,
            &theme,
            &config.ai,
            state,
            area,
            ExecPanelState {
                running,
                running_label: running_label.as_deref(),
                input_open,
            },
            en,
        ),
    }
    if state.consent_request.is_some() {
        draw_consent(frame, area, &theme, en);
    }
    if state.agent.consent_pending.is_some() {
        let popup = centered(area, 72, 10);
        let msg = bi(
            "O Agente está configurado para API remota compatível com OpenAI.\n\nEnviar esta pergunta, arquivo atual e diagnósticos para o provedor remoto?\n\nEnter autoriza (e salva para próximas) · Esc cancela e usa Guia local",
            "The Agent is configured for an OpenAI-compatible remote API.\n\nSend this question, the current file, and diagnostics to the remote provider?\n\nEnter authorizes (and saves for future requests) · Esc cancels and uses Local Guide",
            en,
        );
        frame.render_widget(ratatui::widgets::Clear, popup);
        frame.render_widget(
            Paragraph::new(msg)
                .style(theme.warning())
                .block(
                    Block::bordered()
                        .title(bi(
                            " Consentimento para API remota ",
                            " Remote API Consent ",
                            en,
                        ))
                        .border_style(theme.border_style(true)),
                )
                .wrap(Wrap { trim: true }),
            popup,
        );
    }
    if state.agent.model_picker.open {
        crate::ui::mentor::draw_model_picker_for(
            frame,
            &theme,
            &state.agent.model_picker,
            &config.ai.ollama_model,
            area,
            en,
        );
    }
    if let Some(prompt) = &state.prompt {
        let popup = centered(area, 72, 8);
        let title = match prompt.kind {
            crate::create::PromptKind::NewProject => bi(
                "Novo projeto Cargo (nome ou ~/pasta/nome · Tab preenche pasta)",
                "New Cargo project (name or ~/folder/name · Tab completes path)",
                en,
            ),
            crate::create::PromptKind::OpenPath => {
                bi("Abrir caminho do projeto", "Open project path", en)
            }
            crate::create::PromptKind::NewFile => bi(
                "Novo arquivo .rs (ex.: src/util.rs)",
                "New .rs file (e.g. src/util.rs)",
                en,
            ),
            crate::create::PromptKind::AddDependency => bi(
                "Adicionar dependência (nome ou nome@versão; pode usar rede)",
                "Add dependency (name or name@version; may use network)",
                en,
            ),
            crate::create::PromptKind::RemoveDependency => {
                bi("Remover dependência (nome)", "Remove dependency (name)", en)
            }
            crate::create::PromptKind::ProjectSearch => bi(
                "Buscar em todo o projeto (texto ou símbolo)",
                "Search across the entire project (text or symbol)",
                en,
            ),
            crate::create::PromptKind::SetProjectsDir => bi(
                "Pasta padrão para novos projetos (vazio = padrão)",
                "Default folder for new projects (empty = default)",
                en,
            ),
        };
        let footer = bi(
            "Enter confirma · Esc cancela",
            "Enter confirms · Esc cancels",
            en,
        );
        let msg = format!(
            "{title}\n> {}_\n{}\n{footer}",
            prompt.text,
            prompt.error.as_deref().unwrap_or("")
        );
        frame.render_widget(ratatui::widgets::Clear, popup);
        frame.render_widget(
            Paragraph::new(msg)
                .block(
                    Block::bordered()
                        .title(" RustLab ")
                        .border_style(theme.border_style(true)),
                )
                .wrap(Wrap { trim: true }),
            popup,
        );
    }
}

fn draw_start(
    frame: &mut Frame,
    config: &crate::config::Config,
    state: &CreateState,
    area: Rect,
    en: bool,
) {
    let theme = crate::theme::Theme::for_id(config.ui.theme);
    let mut options: Vec<String> = [
        bi("Criar projeto novo", "Create new project", en),
        bi(
            "Abrir projeto Cargo por caminho",
            "Open Cargo project by path",
            en,
        ),
        bi("Reabrir último projeto", "Reopen last project", en),
    ]
    .into_iter()
    .map(String::from)
    .collect();
    options.extend(
        config
            .workspace
            .recent_projects
            .iter()
            .take(8)
            .map(|p| format!("{}: {p}", bi("Recente", "Recent", en))),
    );
    let last =
        config
            .workspace
            .last_project
            .as_deref()
            .unwrap_or(bi("nenhum salvo", "none saved", en));
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, title)| {
            let detail = if i == 0 {
                match config.workspace.projects_dir.as_deref() {
                    Some(custom) => {
                        if en {
                            format!("  (folder: {custom} · d changes)")
                        } else {
                            format!("  (pasta: {custom} · d altera)")
                        }
                    }
                    None => bi(
                        "  (pasta padrão · d escolhe pasta)",
                        "  (default folder · d chooses folder)",
                        en,
                    )
                    .to_string(),
                }
            } else if i == 2 {
                format!("  ({last})")
            } else {
                String::new()
            };
            let style = if state.start_sel == i {
                theme.selection()
            } else {
                theme.base()
            };
            ListItem::new(Line::from(vec![
                Span::styled(if state.start_sel == i { "› " } else { "  " }, style),
                Span::styled(title.clone(), style),
                Span::styled(detail, theme.dim()),
            ]))
        })
        .collect();
    let block = Block::bordered()
        .title(bi(
            " Modo Criar · Projetos Cargo ",
            " Create Mode · Cargo Projects ",
            en,
        ))
        .border_style(theme.border_style(true));
    let status_height = u16::from(state.last_failure.is_some()) * 3;
    let chunks =
        Layout::vertical([Constraint::Min(5), Constraint::Length(status_height)]).split(area);
    let mut list_state =
        ratatui::widgets::ListState::default().with_selected(Some(state.start_sel));
    frame.render_stateful_widget(List::new(items).block(block), chunks[0], &mut list_state);
    if let Some(reason) = &state.last_failure {
        frame.render_widget(
            Paragraph::new(format!("{}: \n{reason}", bi("Erro", "Error", en)))
                .wrap(Wrap { trim: true })
                .style(theme.warning()),
            chunks[1],
        );
    }
}

fn draw_workspace(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    ai: &AiSettings,
    state: &mut CreateState,
    area: Rect,
    exec: ExecPanelState<'_>,
    en: bool,
) {
    if !state.lib_menu_open {
        state.lib_modal_area = Rect::default();
    }
    if !state.build_menu_open {
        state.build_modal_area = Rect::default();
    }
    if !state.lab.open {
        state.lab_modal_area = Rect::default();
    }
    let top_parts = Layout::vertical([Constraint::Length(1), Constraint::Min(3)]).split(area);
    draw_workspace_top_bar(frame, theme, state, top_parts[0], en);
    let workspace_area = top_parts[1];

    let cols = if state.agent.open {
        Layout::horizontal([
            Constraint::Length(18),
            Constraint::Percentage(38),
            Constraint::Percentage(24),
            Constraint::Percentage(38),
        ])
        .split(workspace_area)
    } else {
        Layout::horizontal([
            Constraint::Length(22),
            Constraint::Percentage(58),
            Constraint::Percentage(42),
        ])
        .split(workspace_area)
    };
    let files: Vec<ListItem> = state
        .files
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let name = p
                .strip_prefix(
                    state
                        .project
                        .as_ref()
                        .map(|p| p.root.as_path())
                        .unwrap_or(p.as_path()),
                )
                .unwrap_or(p)
                .display()
                .to_string();
            let selected = i == state.selected_file;
            ListItem::new(format!("{}{}", if selected { "› " } else { "  " }, name)).style(
                if selected {
                    theme.selection()
                } else {
                    theme.base()
                },
            )
        })
        .collect();
    let file_list = List::new(files).block(
        Block::bordered()
            .title(bi(" Arquivos ", " Files ", en))
            .border_style(theme.border_style(state.focus == Focus::Files)),
    );
    let mut file_state =
        ratatui::widgets::ListState::default().with_selected(Some(state.selected_file));
    frame.render_stateful_widget(file_list, cols[0], &mut file_state);

    let mut editor_area = inner(cols[1]);
    state.editor_text_area = editor_area;
    let line_num_width = state.editor.buf.line_count().to_string().len().max(2) as u16 + 1;
    if state.editor.search.is_some() {
        editor_area.height = editor_area.height.saturating_sub(1);
    }
    let mut lines = Vec::new();
    let first = state.editor.scroll_row;
    let visible = editor_area.height as usize;
    let selection = state.editor.selection();
    for row in first..(first + visible).min(state.editor.buf.line_count()) {
        let source = state.editor.buf.line_str(row);
        let line_1based = row + 1;
        let (gutter_mark, gutter_style) = if state.lab.debugger.active_line() == Some(line_1based) {
            ("▶", theme.success())
        } else if state.lab.debugger.breakpoints.contains(&line_1based) {
            ("●", theme.error())
        } else if state.lab.lifetime_gutter_enabled
            && let Some((lt_mark, is_conflict, is_mut)) = state
                .lab
                .last_lifetimes
                .as_ref()
                .and_then(|r| r.gutter_marker_for_line(line_1based))
        {
            let st = if is_conflict {
                theme.error()
            } else if is_mut {
                theme.warning()
            } else {
                theme.title()
            };
            (lt_mark, st)
        } else {
            match state.diagnostic_level_for_open_line(line_1based as u32) {
                Some(crate::execution::DiagLevel::Error) => ("!", theme.error()),
                Some(crate::execution::DiagLevel::Warning) => ("*", theme.warning()),
                Some(crate::execution::DiagLevel::Note) => ("·", theme.title()),
                None => (" ", theme.dim()),
            }
        };
        let mut spans = vec![
            Span::styled(
                format!("{:>width$}", row + 1, width = line_num_width as usize - 1),
                theme.dim(),
            ),
            Span::styled(gutter_mark, gutter_style),
        ];
        let highlighted = crate::highlight::highlight_line(&source, theme);
        let selected_range = selection
            .filter(|(a, b)| row >= a.line && row <= b.line)
            .map(|(a, b)| {
                (
                    if row == a.line { a.col } else { 0 },
                    if row == b.line {
                        b.col
                    } else {
                        source.chars().count()
                    },
                )
            });
        let cursor_col = (state.focus == Focus::Editor && row == state.editor.cursor.line)
            .then_some(state.editor.cursor.col);
        let mut col = 0usize;
        for part in highlighted {
            for ch in part.content.chars() {
                let mut style = part.style;
                if selected_range.is_some_and(|(start, end)| col >= start && col < end) {
                    style = theme.selection();
                }
                if cursor_col == Some(col) {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                spans.push(Span::styled(ch.to_string(), style));
                col += 1;
            }
        }
        if cursor_col == Some(col) {
            spans.push(Span::styled(" ", theme.selection()));
        }
        lines.push(Line::from(spans));
    }
    let filename = state
        .editor
        .path
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| bi("sem arquivo", "no file", en).into());
    let dirty = if state.editor.dirty { " ●" } else { "" };
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .scroll((0, state.editor.scroll_col as u16))
            .block(
                Block::bordered()
                    .title(format!(" {filename}{dirty} "))
                    .border_style(theme.border_style(state.focus == Focus::Editor)),
            ),
        cols[1],
    );

    let right =
        Layout::vertical([Constraint::Percentage(78), Constraint::Percentage(22)]).split(cols[2]);
    let console_parts = Layout::vertical([
        Constraint::Min(2),
        Constraint::Length(if exec.input_open { 2 } else { 0 }),
    ])
    .split(right[0]);
    let console_lines: Vec<Line> = state
        .console
        .lines
        .iter()
        .map(|line| {
            let style = match line.kind {
                StreamKind::Stdout => theme.base(),
                StreamKind::Stderr => theme.error(),
                StreamKind::System => theme.dim(),
            };
            Line::styled(line.text.clone(), style)
        })
        .collect();
    let running_status = if exec.running {
        format!(
            " · {} {}",
            exec.running_label.unwrap_or("cargo"),
            bi("rodando", "running", en)
        )
    } else {
        String::new()
    };
    state.console.width = console_parts[0].width.saturating_sub(2).max(1);
    state.console.page_rows = console_parts[0].height.saturating_sub(2).max(1) as usize;
    let max_scroll = state.console.max_scroll();
    state.console.scroll_back = state.console.scroll_back.min(max_scroll);
    let offset = max_scroll
        .saturating_sub(state.console.scroll_back)
        .min(u16::MAX as usize) as u16;
    frame.render_widget(
        Paragraph::new(console_lines)
            .wrap(Wrap { trim: false })
            .scroll((offset, 0))
            .block(
                Block::bordered()
                    .title(format!(" Console{running_status} "))
                    .border_style(theme.border_style(state.focus == Focus::Console)),
            ),
        console_parts[0],
    );
    if exec.input_open {
        frame.render_widget(
            Paragraph::new(format!(
                "stdin > {}_\n{}",
                state.console_input,
                bi(
                    "Enter enviar · Ctrl+D EOF · Shift+Tab foco",
                    "Enter sends · Ctrl+D EOF · Shift+Tab focus",
                    en,
                )
            ))
            .style(theme.base()),
            console_parts[1],
        );
    }
    let diag_text = if state.diags.is_empty() {
        bi("Sem diagnósticos", "No diagnostics", en).to_string()
    } else {
        format!(
            "{} {}\n{}",
            state.diags.len(),
            bi("diagnóstico(s)", "diagnostic(s)", en),
            state
                .diags
                .iter()
                .take(4)
                .map(|d| format!("{:?}: {}", d.level, d.message))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    frame.render_widget(
        Paragraph::new(diag_text)
            .block(
                Block::bordered()
                    .title(format!(
                        " {} · {} ",
                        bi("Diagnósticos", "Diagnostics", en),
                        state.lsp_status
                    ))
                    .border_style(theme.border_style(false)),
            )
            .wrap(Wrap { trim: true }),
        right[1],
    );

    if state.agent.open {
        draw_agent_panel(frame, theme, ai, state, cols[3], en);
    }

    if let Some(search) = &state.editor.search {
        let bar = Rect {
            x: cols[1].x + 1,
            y: cols[1].y + cols[1].height.saturating_sub(2),
            width: cols[1].width.saturating_sub(2),
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(format!(
                "{}: {} {}",
                bi("buscar", "find", en),
                search.query,
                search.message.as_deref().unwrap_or("")
            ))
            .style(theme.selection()),
            bar,
        );
    }
    if state.focus == Focus::Editor && !state.completion.items.is_empty() {
        let popup = centered(area, 56, state.completion.items.len() as u16 + 4);
        let mut suggestions: Vec<Line> = state
            .completion
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                Line::styled(
                    format!(
                        "{} {item}",
                        if index == state.completion.selected {
                            ">"
                        } else {
                            " "
                        }
                    ),
                    if index == state.completion.selected {
                        theme.selection()
                    } else {
                        theme.base()
                    },
                )
            })
            .collect();
        suggestions.push(Line::from(bi(
            "↑↓ escolher · Enter/Tab aceitar · Esc fechar",
            "↑↓ choose · Enter/Tab accept · Esc close",
            en,
        )));
        suggestions.push(Line::styled(
            bi(
                "Sugestões locais por prefixo (sem análise de tipos)",
                "Local prefix suggestions (no type analysis)",
                en,
            ),
            theme.dim(),
        ));
        frame.render_widget(ratatui::widgets::Clear, popup);
        frame.render_widget(
            Paragraph::new(suggestions).block(
                Block::bordered()
                    .title(bi(
                        " Autocomplete · Ctrl+Espaço ",
                        " Autocomplete · Ctrl+Space ",
                        en,
                    ))
                    .border_style(theme.border_style(true)),
            ),
            popup,
        );
    } else if let Some(popup) = &state.doc_popup {
        let width = 68.min(cols[1].width.saturating_sub(2)).max(24);
        let height = if popup.url.is_some() { 7 } else { 6 }.min(area.height.saturating_sub(2));
        let box_rect = Rect {
            x: cols[1].x + 1,
            y: cols[1]
                .y
                .saturating_add(cols[1].height.saturating_sub(height + 1)),
            width,
            height,
        };
        let mut lines = vec![
            Line::from(Span::styled(popup.title.as_str(), theme.title())),
            Line::from(popup.body.as_str()),
        ];
        if let Some(url) = &popup.url {
            lines.push(Line::from(Span::styled(url.as_str(), theme.dim())));
        }
        frame.render_widget(ratatui::widgets::Clear, box_rect);
        frame.render_widget(
            Paragraph::new(lines).wrap(Wrap { trim: true }).block(
                Block::bordered()
                    .title(bi(" Ajuda · F1/Esc fechar ", " Help · F1/Esc close ", en))
                    .border_style(theme.border_style(true)),
            ),
            box_rect,
        );
    } else if let Some((symbol, x, y)) = state.hovered_doc.as_ref() {
        let code = state.editor.buf.to_string_lossy();
        let resolved = crate::doc_hint::resolve_symbol_doc_for(symbol, &code, en);
        let local_doc = state.workspace_index.find_doc(symbol);
        if resolved.is_some() || local_doc.is_some() {
            let (title, summary, url) = if let Some(doc) = local_doc {
                let u = resolved
                    .as_ref()
                    .map(|r| r.best_url_for(en).to_string())
                    .unwrap_or_else(|| {
                        bi(
                            "https://rust-br.github.io/rust-book-pt-br/",
                            "https://doc.rust-lang.org/book/",
                            en,
                        )
                        .to_string()
                    });
                (symbol.clone(), doc, u)
            } else {
                let r = resolved.expect("checked is_some above");
                let u = r.best_url_for(en).to_string();
                (r.title, r.summary, u)
            };
            let width = 70.min(workspace_area.width.saturating_sub(2)).max(24);
            let height = 7.min(workspace_area.height.saturating_sub(2)).max(4);
            let popup_x = if x.saturating_add(width)
                < workspace_area.x.saturating_add(workspace_area.width)
            {
                x.saturating_add(1)
            } else {
                x.saturating_sub(width)
            }
            .clamp(
                workspace_area.x,
                workspace_area
                    .x
                    .saturating_add(workspace_area.width.saturating_sub(width)),
            );
            let popup_y = if y.saturating_add(height)
                < workspace_area.y.saturating_add(workspace_area.height)
            {
                y.saturating_add(1)
            } else {
                y.saturating_sub(height)
            }
            .clamp(
                workspace_area.y,
                workspace_area
                    .y
                    .saturating_add(workspace_area.height.saturating_sub(height)),
            );
            let box_rect = Rect {
                x: popup_x,
                y: popup_y,
                width,
                height,
            };
            frame.render_widget(ratatui::widgets::Clear, box_rect);
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled(title, theme.title())),
                    Line::from(summary),
                    Line::from(Span::styled(
                        format!(
                            "{url} · {}",
                            bi("clique para abrir doc", "click to open docs", en)
                        ),
                        theme.dim(),
                    )),
                ])
                .wrap(Wrap { trim: true })
                .block(
                    Block::bordered()
                        .title(bi(
                            " Ajuda Rust · clique para documentação PT/EN ",
                            " Rust Help · click for EN/PT docs ",
                            en,
                        ))
                        .border_style(theme.border_style(true)),
                ),
                box_rect,
            );
        }
    }
    state.editor.ensure_visible(
        editor_area.height as usize,
        editor_area.width.saturating_sub(line_num_width) as usize,
    );
    if state.lib_menu_open {
        draw_libraries_modal(frame, theme, state, area, en);
    } else if state.build_menu_open {
        draw_build_modal(frame, theme, state, area, en);
    } else if state.lab.open {
        draw_lab_modal(frame, theme, state, area, en);
    }
}

fn draw_workspace_top_bar(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
    en: bool,
) {
    state.top_lib_btn_area = Rect::default();
    state.top_build_btn_area = Rect::default();
    if area.height == 0 || area.width < 12 {
        return;
    }
    let lib_text = if state.lib_menu_open {
        bi(" [ ▲ Bibliotecas (F9) ] ", " [ ▲ Libraries (F9) ] ", en)
    } else {
        bi(" [ ▼ Bibliotecas (F9) ] ", " [ ▼ Libraries (F9) ] ", en)
    };
    let build_text = if state.build_menu_open {
        bi(
            " [ ▲ Compilar Linux/Win (F4) ] ",
            " [ ▲ Build Linux/Win (F4) ] ",
            en,
        )
    } else {
        bi(
            " [ Compilar Linux/Win (F4) ] ",
            " [ Build Linux/Win (F4) ] ",
            en,
        )
    };
    let lib_w = lib_text.chars().count() as u16;
    let build_w = build_text.chars().count() as u16;
    let gap = 2u16;
    let total_w = lib_w + gap + build_w;
    if total_w <= area.width {
        let start_x = area.x + (area.width - total_w) / 2;
        let lib_rect = Rect {
            x: start_x,
            y: area.y,
            width: lib_w,
            height: 1,
        };
        let build_rect = Rect {
            x: start_x + lib_w + gap,
            y: area.y,
            width: build_w,
            height: 1,
        };
        state.top_lib_btn_area = lib_rect;
        state.top_build_btn_area = build_rect;
        let lib_style = if state.lib_menu_open {
            theme.selection()
        } else {
            theme.title()
        };
        let build_style = if state.build_menu_open {
            theme.selection()
        } else {
            theme.title()
        };
        frame.render_widget(Paragraph::new(Span::styled(lib_text, lib_style)), lib_rect);
        frame.render_widget(
            Paragraph::new(Span::styled(build_text, build_style)),
            build_rect,
        );
    } else {
        let lib_rect = Rect {
            x: area.x + area.width.saturating_sub(lib_w.min(area.width)) / 2,
            y: area.y,
            width: lib_w.min(area.width),
            height: 1,
        };
        state.top_lib_btn_area = lib_rect;
        frame.render_widget(
            Paragraph::new(Span::styled(lib_text, theme.title())),
            lib_rect,
        );
    }
}

fn draw_libraries_modal(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
    en: bool,
) {
    let modal = centered(area, 82, 20);
    state.lib_modal_area = modal;
    frame.render_widget(ratatui::widgets::Clear, modal);
    frame.render_widget(
        Block::bordered()
            .title(bi(
                " ▼ Bibliotecas Visuais, Efeitos & Mini Engine ",
                " ▼ Visual, Effects & Mini-Engine Libraries ",
                en,
            ))
            .border_style(theme.border_style(true)),
        modal,
    );
    let inner_area = inner(modal);
    if inner_area.height < 6 || inner_area.width < 24 {
        return;
    }
    let parts = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(4),
        Constraint::Length(1),
    ])
    .split(inner_area);

    frame.render_widget(
        Paragraph::new(Span::styled(
            bi(
                "Clique/Enter adiciona · T insere template · D abre docs · Esc fecha",
                "Click/Enter adds · T inserts template · D opens docs · Esc closes",
                en,
            ),
            theme.dim(),
        )),
        parts[0],
    );

    let cols = Layout::horizontal([Constraint::Length(28), Constraint::Min(18)]).split(parts[1]);
    let libs = crate::doc_hint::extra_libraries(en);
    let items: Vec<ListItem> = libs
        .iter()
        .enumerate()
        .map(|(idx, lib)| {
            let marker = if idx == state.lib_menu_sel {
                "▶"
            } else {
                " "
            };
            let style = if idx == state.lib_menu_sel {
                theme.selection()
            } else {
                theme.base()
            };
            ListItem::new(Line::from(Span::styled(
                format!("{marker} {} ({})", lib.name, lib.version),
                style,
            )))
        })
        .collect();
    frame.render_widget(List::new(items), cols[0]);

    let selected = libs.get(state.lib_menu_sel).unwrap_or(&libs[0]);
    let mut details = vec![
        Line::from(Span::styled(
            format!("{} · {}", selected.name, selected.category),
            theme.title(),
        )),
        Line::from(selected.summary),
        Line::default(),
        Line::from(vec![
            Span::styled(bi("Como funciona: ", "How it works: ", en), theme.title()),
            Span::raw(selected.docs_pt),
        ]),
        Line::from(vec![
            Span::styled(
                bi("Inserção automática: ", "Auto import: ", en),
                theme.title(),
            ),
            Span::raw(selected.use_line),
        ]),
        Line::from(vec![
            Span::styled(
                bi("Símbolos principais: ", "Key symbols: ", en),
                theme.title(),
            ),
            Span::raw(selected.key_items),
        ]),
        Line::default(),
        Line::from(Span::styled(
            bi("Exemplo rápido:", "Quick example:", en),
            theme.dim(),
        )),
    ];
    for line in selected.example.lines() {
        details.push(Line::from(Span::styled(line, theme.success())));
    }
    frame.render_widget(
        Paragraph::new(details)
            .wrap(Wrap { trim: true })
            .block(Block::bordered().title(bi(
                " Documentação da Biblioteca ",
                " Library Documentation ",
                en,
            ))),
        cols[1],
    );

    frame.render_widget(
        Paragraph::new(Span::styled(
            format!(
                "[ Docs: {} ] ({})",
                selected.doc_url,
                bi("clique ou D", "click or D", en)
            ),
            theme.dim(),
        )),
        parts[2],
    );
}

fn draw_build_modal(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
    en: bool,
) {
    let modal = centered(area, 72, 11);
    state.build_modal_area = modal;
    frame.render_widget(ratatui::widgets::Clear, modal);
    frame.render_widget(
        Block::bordered()
            .title(bi(
                " Compilar & Executar App (Linux / Windows / Bevy) ",
                " Build & Run App (Linux / Windows / Bevy) ",
                en,
            ))
            .border_style(theme.border_style(true)),
        modal,
    );
    let inner_area = inner(modal);
    if inner_area.height < 6 || inner_area.width < 20 {
        return;
    }
    let parts = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .split(inner_area);

    frame.render_widget(
        Paragraph::new(Span::styled(
            bi(
                "Escolha como compilar ou abrir seu projeto visual/CLI/Bevy:",
                "Choose how to build or launch your visual/CLI/Bevy project:",
                en,
            ),
            theme.dim(),
        )),
        parts[0],
    );

    let options = [
        bi(
            "Abrir em janela própria (GUI / Bevy / Mini Engine)",
            "Open in separate window (GUI / Bevy / Mini Engine)",
            en,
        ),
        bi(
            "Rodar em outro terminal (cargo run externo — ideal p/ Bevy)",
            "Run in another terminal (external cargo run — ideal for Bevy)",
            en,
        ),
        bi(
            "Compilar executável Linux release (target/release/...)",
            "Build Linux release binary (target/release/...)",
            en,
        ),
        bi(
            "Compilar executável Windows .exe (x86_64-pc-windows-gnu)",
            "Cross-compile Windows .exe (x86_64-pc-windows-gnu)",
            en,
        ),
        bi(
            "Build padrão debug no workspace (cargo build)",
            "Standard workspace debug build (cargo build)",
            en,
        ),
    ];
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(idx, label)| {
            let marker = if idx == state.build_menu_sel {
                "▶"
            } else {
                " "
            };
            let style = if idx == state.build_menu_sel {
                theme.selection()
            } else {
                theme.base()
            };
            ListItem::new(Line::from(Span::styled(format!("{marker} {label}"), style)))
        })
        .collect();
    frame.render_widget(List::new(items), parts[1]);

    frame.render_widget(
        Paragraph::new(Span::styled(
            bi(
                "Enter ou clique confirma · ↑/↓ navega · Esc fecha",
                "Enter or click confirms · ↑/↓ navigates · Esc closes",
                en,
            ),
            theme.dim(),
        )),
        parts[2],
    );
}

fn draw_agent_panel(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    ai: &AiSettings,
    state: &mut CreateState,
    area: Rect,
    en: bool,
) {
    let mut provider_label = match ai.provider {
        AiProvider::Ollama => format!("Ollama · {}", ai.ollama_model),
        AiProvider::Remote => format!("API · {}", ai.remote_model),
        AiProvider::Disabled => bi("Guia local", "Local guide", en).to_string(),
    };
    if state.agent_running() {
        provider_label.push_str(bi(" · pensando…", " · thinking…", en));
    }
    let focused = state.focus == Focus::Agent;
    let block = Block::bordered()
        .title(format!(
            " {} · {provider_label} ",
            bi("Agente", "Agent", en)
        ))
        .border_style(theme.border_style(focused));
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    if inner_area.height < 4 || inner_area.width < 10 {
        return;
    }

    let banner_height = u16::from(state.agent.proposed_code.is_some());
    let parts = Layout::vertical([
        Constraint::Min(4),
        Constraint::Length(banner_height),
        Constraint::Length(3),
    ])
    .split(inner_area);

    let mut lines: Vec<Line> = Vec::new();
    for (idx, msg) in state.agent.messages.iter().enumerate() {
        if idx > 0 {
            lines.push(Line::default());
        }
        let (prefix, header_style, body_style) = match msg.role {
            AgentRole::User => (
                bi("Você: ", "You: ", en).to_string(),
                theme.title(),
                theme.base(),
            ),
            AgentRole::Agent => (format!("{}: ", msg.sender), theme.success(), theme.base()),
            AgentRole::System => (format!("{}: ", msg.sender), theme.dim(), theme.dim()),
        };
        let mut text_lines = msg.text.lines();
        if let Some(first) = text_lines.next() {
            lines.push(Line::from(vec![
                Span::styled(prefix, header_style),
                Span::styled(first, body_style),
            ]));
            for extra in text_lines {
                lines.push(Line::from(Span::styled(extra, body_style)));
            }
        } else {
            lines.push(Line::from(Span::styled(prefix, header_style)));
        }
    }
    let history = Paragraph::new(lines).wrap(Wrap { trim: false });
    let total_lines = history.line_count(parts[0].width.max(1)) as u16;
    let max_scroll = total_lines.saturating_sub(parts[0].height);
    let clamped_scroll = state.agent.scroll.min(max_scroll);
    frame.render_widget(history.scroll((clamped_scroll, 0)), parts[0]);
    state.agent.scroll = clamped_scroll;

    if state.agent.proposed_code.is_some() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                bi(
                    "★ Código pronto: Alt+Enter (ou /aplicar) aplica no editor (Ctrl+Z desfaz)",
                    "★ Code ready: Alt+Enter (or /apply) applies to editor (Ctrl+Z undoes)",
                    en,
                ),
                theme.success(),
            ))),
            parts[1],
        );
    }

    let input_lines = vec![
        Line::from(vec![
            Span::styled("> ", theme.title()),
            Span::styled(state.agent.input.as_str(), theme.base()),
            Span::styled("_", if focused { theme.title() } else { theme.dim() }),
        ]),
        Line::from(Span::styled(
            bi(
                "Enter envia · Alt+O modelo · Alt+Enter aplicar · Esc editor",
                "Enter sends · Alt+O model · Alt+Enter apply · Esc editor",
                en,
            ),
            theme.dim(),
        )),
    ];
    frame.render_widget(
        Paragraph::new(input_lines).wrap(Wrap { trim: true }),
        parts[2],
    );
}

fn draw_consent(frame: &mut Frame, area: Rect, theme: &crate::theme::Theme, en: bool) {
    let popup = centered(area, 74, 12);
    let text = bi(
        "Execução local NÃO é sandbox.\nSeu código e dependências rodam com permissões da sua conta: podem acessar arquivos, rede e iniciar processos.\n\nF5 aceita entrada por linhas (não é terminal completo). Dependências podem usar a rede.\n\nNada foi executado ainda. Enter autoriza comandos futuros; Esc recusa.",
        "Local execution is NOT sandboxed.\nYour code and dependencies run with your account permissions: they may access files, network, and spawn processes.\n\nF5 reads line-by-line input (not a full terminal). Dependencies may access the network.\n\nNothing has run yet. Enter authorizes future commands; Esc declines.",
        en,
    );
    frame.render_widget(ratatui::widgets::Clear, popup);
    frame.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(theme.warning))
            .block(
                Block::bordered()
                    .title(bi(" Consentimento de execução ", " Execution Consent ", en))
                    .border_style(theme.border_style(true)),
            )
            .wrap(Wrap { trim: true }),
        popup,
    );
}

fn draw_lab_modal(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
    en: bool,
) {
    let modal = centered(area, 88, 24);
    state.lab_modal_area = modal;
    frame.render_widget(ratatui::widgets::Clear, modal);
    frame.render_widget(
        Block::bordered()
            .title(format!(
                " {} · {} ",
                crate::i18n::bi("Laboratório RustLab", "RustLab Lab", en),
                state.lab.tab.title_for(en)
            ))
            .border_style(theme.border_style(true)),
        modal,
    );
    let inner_area = inner(modal);
    if inner_area.height < 8 || inner_area.width < 30 {
        return;
    }
    let tab_bar_height = if inner_area.width < 82 { 2 } else { 1 };
    let parts = Layout::vertical([
        Constraint::Length(tab_bar_height),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .split(inner_area);

    let tab_spans: Vec<Span> = crate::lab_tools::LabToolTab::ALL
        .iter()
        .flat_map(|tab| {
            let active = *tab == state.lab.tab;
            let label = if active {
                format!("[{}]", tab.short_title_for(en))
            } else {
                tab.short_title_for(en).to_string()
            };
            let style = if active {
                theme.selection()
            } else {
                theme.dim()
            };
            vec![Span::styled(label, style), Span::raw(" ")]
        })
        .collect();
    frame.render_widget(
        Paragraph::new(Line::from(tab_spans)).wrap(Wrap { trim: false }),
        parts[0],
    );

    let footer_hint = match state.lab.tab {
        crate::lab_tools::LabToolTab::RaioXMemoria => {
            draw_lab_xray(frame, theme, state, parts[1], en);
            crate::i18n::bi(
                "R recalcula · 1-9/0/Tab troca ferramenta · PgUp/PgDn rola · Esc fecha",
                "R recalculate · 1-9/0/Tab switch tool · PgUp/PgDn scroll · Esc close",
                en,
            )
        }
        crate::lab_tools::LabToolTab::LenteCompilador => {
            draw_lab_lens(frame, theme, state, parts[1], en);
            crate::i18n::bi(
                "S alterna Modo 10 Anos (ELI10) · ↑↓ solução · Enter envia p/ Agente · Esc fecha",
                "S toggle 10-Year-Old Mode (ELI10) · ↑↓ fix · Enter send to Agent · Esc close",
                en,
            )
        }
        crate::lab_tools::LabToolTab::DebuggerVisual => {
            draw_lab_debugger(frame, theme, state, parts[1], en);
            crate::i18n::bi(
                "N/F10 over · S into · C bp · B ● · G gdb/lldb nativo · R reinicia · X para",
                "N/F10 over · S into · C bp · B ● · G native gdb/lldb · R restart · X stop",
                en,
            )
        }
        crate::lab_tools::LabToolTab::ConserteOCodigo => {
            draw_lab_quests(frame, theme, state, parts[1], en);
            crate::i18n::bi(
                "↑↓ escolhe missão · Enter carrega quebrado no editor · V valida conserto + XP",
                "↑↓ pick quest · Enter load broken code · V verify fix + XP",
                en,
            )
        }
        crate::lab_tools::LabToolTab::PlaygroundComunidade => {
            draw_lab_playground(frame, theme, state, parts[1], en);
            crate::i18n::bi(
                "O abre Playground · E exporta Markdown · ↑↓ abordagem · T aplica no editor",
                "O open Playground · E export Markdown · ↑↓ approach · T apply in editor",
                en,
            )
        }
        crate::lab_tools::LabToolTab::EcossistemasModernos => {
            draw_lab_ecosystems(frame, theme, state, parts[1], en);
            crate::i18n::bi(
                "↑↓ trilha · Enter/T insere template completo · D adiciona dep ao Cargo.toml",
                "↑↓ track · Enter/T insert full template · D add dep to Cargo.toml",
                en,
            )
        }
        crate::lab_tools::LabToolTab::MapaLifetimes => {
            draw_lab_lifetimes(frame, theme, state, parts[1], en);
            crate::i18n::bi(
                "L alterna marcadores ┣/┃/✘ na margem · R recalcula · PgUp/PgDn rola · Esc fecha",
                "L toggle ┣/┃/✘ gutter markers · R recalculate · PgUp/PgDn scroll · Esc close",
                en,
            )
        }
        crate::lab_tools::LabToolTab::LayoutMemoria => {
            draw_lab_layout(frame, theme, state, parts[1], en);
            crate::i18n::bi(
                "R recalcula · O/Enter envia ordem otimizada de campos ao Agente · Esc fecha",
                "R recalculate · O/Enter send optimized field order to Agent · Esc close",
                en,
            )
        }
        crate::lab_tools::LabToolTab::DesacucararMacros => {
            draw_lab_desugar(frame, theme, state, parts[1], en);
            crate::i18n::bi(
                "↑↓ escolhe macro/syntax sugar · Enter envia expansão ao Agente · Esc fecha",
                "↑↓ pick macro/syntax sugar · Enter send expansion to Agent · Esc close",
                en,
            )
        }
        crate::lab_tools::LabToolTab::PoliglotaIntuicao => {
            draw_lab_polyglot_intuition(frame, theme, state, parts[1], en);
            if state.lab.polyglot_mode_flashcards {
                crate::i18n::bi(
                    "T Dicionário · ↑↓ card · C Compila ✔ · E Dá Erro ✘ (+25 XP/SRS) · Esc fecha",
                    "T Dictionary · ↑↓ card · C Compiles ✔ · E Error ✘ (+25 XP/SRS) · Esc close",
                    en,
                )
            } else {
                crate::i18n::bi(
                    "T Flashcards · ↑↓ tópico · F filtro py/js/c++ · Enter carrega exemplo Rust",
                    "T Flashcards · ↑↓ topic · F filter py/js/c++ · Enter load Rust example",
                    en,
                )
            }
        }
    };

    frame.render_widget(
        Paragraph::new(Span::styled(footer_hint, theme.dim())),
        parts[2],
    );
}

fn render_scrolled_lines(frame: &mut Frame, lines: Vec<Line<'_>>, scroll: &mut u16, area: Rect) {
    let para = Paragraph::new(lines).wrap(Wrap { trim: true });
    let total = para.line_count(area.width.max(1)) as u16;
    let max_scroll = total.saturating_sub(area.height.max(1));
    *scroll = (*scroll).min(max_scroll);
    frame.render_widget(para.scroll((*scroll, 0)), area);
}

fn draw_lab_xray(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
    en: bool,
) {
    let report = state.lab.last_xray.clone().unwrap_or_else(|| {
        crate::lab_tools::MemoryXRayReport::analyze_for(&state.editor.buf.to_string_lossy(), en)
    });
    let lines: Vec<Line> = report
        .summary_lines_for(en)
        .into_iter()
        .map(|s| {
            let style = if s.contains("CONFLITO")
                || s.contains("CONFLICT")
                || s.contains("✗ movido")
                || s.contains("✗ moved")
            {
                theme.error()
            } else if s.contains("Zero-Cost") || s.contains("[✓ ativo]") || s.contains("[✓ active]")
            {
                theme.success()
            } else if s.starts_with("Medidor")
                || s.starts_with("Meter")
                || s.starts_with("Mapa Visual")
                || s.starts_with("Visual")
                || s.starts_with("Eventos")
                || s.starts_with("Events")
            {
                theme.title()
            } else {
                theme.base()
            };
            Line::from(Span::styled(s, style))
        })
        .collect();
    render_scrolled_lines(frame, lines, &mut state.lab.scroll, area);
}

fn draw_lab_lens(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
    en: bool,
) {
    let diag_text = state.diagnostics_text_summary();
    let code = state.editor.buf.to_string_lossy();
    let lens =
        state.lab.last_lens.clone().unwrap_or_else(|| {
            crate::lab_tools::SocraticErrorLens::analyze_for(&diag_text, &code, en)
        });
    let mut lines = vec![
        Line::from(Span::styled(
            format!("[{}] {}", lens.code, lens.title),
            theme.title(),
        )),
        Line::from(Span::styled(
            if state.lab.eli10_mode {
                format!(
                    "{}{}",
                    crate::i18n::bi("Modo 10 Anos (ELI10): ", "10-Year-Old Mode (ELI10): ", en),
                    lens.eli10_metaphor
                )
            } else {
                format!(
                    "{}{}",
                    crate::i18n::bi("Pergunta Socrática: ", "Socratic Question: ", en),
                    lens.socratic_question
                )
            },
            theme.warning(),
        )),
        Line::default(),
        Line::from(Span::styled(
            crate::i18n::bi(
                "Comparativo Antes → Depois (escolha 1..3):",
                "Before → After Comparison (pick 1..3):",
                en,
            ),
            theme.title(),
        )),
    ];
    for (idx, fix) in lens.fixes.iter().enumerate() {
        let sel = idx == state.lab.selected_fix;
        lines.push(Line::from(Span::styled(
            format!(
                "{} {}. {} — {}",
                if sel { "▶" } else { " " },
                idx + 1,
                fix.label,
                fix.explanation
            ),
            if sel { theme.selection() } else { theme.base() },
        )));
        lines.push(Line::from(vec![
            Span::styled(
                crate::i18n::bi("   Antes : ", "   Before: ", en),
                theme.error(),
            ),
            Span::raw(fix.before.replace('\n', " | ")),
        ]));
        lines.push(Line::from(vec![
            Span::styled(
                crate::i18n::bi("   Depois: ", "   After : ", en),
                theme.success(),
            ),
            Span::raw(fix.after.replace('\n', " | ")),
        ]));
    }
    render_scrolled_lines(frame, lines, &mut state.lab.scroll, area);
}

fn draw_lab_debugger(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
    en: bool,
) {
    let bps: Vec<String> = state
        .lab
        .debugger
        .breakpoints
        .iter()
        .map(|l| format!("L{l}"))
        .collect();
    let bp_text = if bps.is_empty() {
        crate::i18n::bi(
            "nenhum (Alt+B na linha do editor)",
            "none (Alt+B on editor line)",
            en,
        )
        .to_string()
    } else {
        bps.join(", ")
    };
    let status_lbl = if state.lab.debugger.active {
        crate::i18n::bi("ATIVO", "ACTIVE", en)
    } else {
        crate::i18n::bi("PARADO", "STOPPED", en)
    };
    let mut lines = vec![Line::from(Span::styled(
        format!(
            "{}: {status_lbl} · Breakpoints ●: {bp_text}",
            crate::i18n::bi("Estado", "State", en)
        ),
        theme.title(),
    ))];
    if let Some(step) = state.lab.debugger.current() {
        lines.push(Line::from(Span::styled(
            if en {
                format!(
                    "Step {}/{} ▶ Line {} in `{}`: {}",
                    step.step_index + 1,
                    state.lab.debugger.steps.len(),
                    step.line_1based,
                    step.func,
                    step.code_line
                )
            } else {
                format!(
                    "Passo {}/{} ▶ Linha {} em `{}`: {}",
                    step.step_index + 1,
                    state.lab.debugger.steps.len(),
                    step.line_1based,
                    step.func,
                    step.code_line
                )
            },
            theme.success(),
        )));
        lines.push(Line::from(Span::styled(
            format!(
                "{}: {} · Call Stack: {}",
                crate::i18n::bi("Nota", "Note", en),
                step.note,
                step.call_stack.join(" → ")
            ),
            theme.dim(),
        )));
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            crate::i18n::bi("Variáveis Locais:", "Local Variables:", en),
            theme.title(),
        )));
        if step.locals.is_empty() {
            lines.push(Line::from(crate::i18n::bi(
                "  (nenhuma variável inicializada neste ponto)",
                "  (no initialized variables at this point)",
                en,
            )));
        } else {
            for loc in &step.locals {
                lines.push(Line::from(format!(
                    "  {} [{} · {}] = {}  ({})",
                    loc.name,
                    loc.ty,
                    loc.region.label(),
                    loc.value,
                    loc.status
                )));
            }
        }
        if !step.stdout.is_empty() {
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                crate::i18n::bi("Saída capturada:", "Captured output:", en),
                theme.title(),
            )));
            for out in &step.stdout {
                lines.push(Line::from(format!("  > {out}")));
            }
        }
    } else {
        lines.push(Line::from(crate::i18n::bi(
            "Pressione N/F10 ou R para iniciar a execução passo a passo do arquivo aberto.",
            "Press N/F10 or R to start step-by-step execution of the open file.",
            en,
        )));
    }
    let probe = crate::lab_tools::detect_native_debugger_for(en);
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        format!(
            "{}: {} ({})",
            crate::i18n::bi(
                "Debugger Nativo (opcional [G])",
                "Native Debugger (optional [G])",
                en
            ),
            if probe.available {
                crate::i18n::bi("DISPONÍVEL", "AVAILABLE", en)
            } else {
                crate::i18n::bi("não encontrado no PATH", "not found in PATH", en)
            },
            probe.binary_name
        ),
        theme.title(),
    )));
    if let Some(native_msg) = &state.lab.native_debugger_status {
        lines.push(Line::from(Span::styled(native_msg.clone(), theme.dim())));
    } else if !probe.available {
        lines.push(Line::from(Span::styled(probe.install_hint, theme.dim())));
    }
    render_scrolled_lines(frame, lines, &mut state.lab.scroll, area);
}

fn draw_lab_quests(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
    en: bool,
) {
    let cols = Layout::horizontal([Constraint::Length(34), Constraint::Min(20)]).split(area);
    let quests = crate::lab_tools::troubleshooting_quests(en);
    let items: Vec<ListItem> = quests
        .iter()
        .enumerate()
        .map(|(i, q)| {
            let done = if state.lab.completed_quests.contains(q.id) {
                "✓"
            } else {
                " "
            };
            let sel = i == state.lab.quest_sel;
            ListItem::new(Line::from(Span::styled(
                format!("{} [{done}] {}", if sel { "▶" } else { " " }, q.title),
                if sel { theme.selection() } else { theme.base() },
            )))
        })
        .collect();
    frame.render_widget(
        List::new(items).block(Block::bordered().title(crate::i18n::bi(
            " Desafios (8) ",
            " Challenges (8) ",
            en,
        ))),
        cols[0],
    );
    let q = quests[state.lab.quest_sel.min(quests.len() - 1)];
    let mut detail = vec![
        Line::from(Span::styled(
            if en {
                format!(
                    "{} · Tag: {} · Par: {} lines · Max Heap: {}",
                    q.id, q.error_tag, q.par_lines, q.max_heap_allocs
                )
            } else {
                format!(
                    "{} · Tag: {} · Par: {} linhas · Max Heap: {}",
                    q.id, q.error_tag, q.par_lines, q.max_heap_allocs
                )
            },
            theme.title(),
        )),
        Line::from(q.description),
        Line::default(),
        Line::from(Span::styled(
            format!("{}{}", crate::i18n::bi("Dica: ", "Hint: ", en), q.hint),
            theme.dim(),
        )),
    ];
    if let Some(fb) = &state.lab.last_quest_feedback {
        detail.push(Line::default());
        detail.push(Line::from(Span::styled(fb.as_str(), theme.success())));
    }
    frame.render_widget(
        Paragraph::new(detail)
            .wrap(Wrap { trim: true })
            .block(Block::bordered().title(crate::i18n::bi(
                " Detalhes da Missão ",
                " Mission Details ",
                en,
            ))),
        cols[1],
    );
}

fn draw_lab_playground(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
    en: bool,
) {
    let code = state.editor.buf.to_string_lossy();
    let url = crate::lab_tools::build_playground_url(&code);
    let sols = crate::lab_tools::community_solutions_for_lang(&code, en);
    let mut lines = vec![
        Line::from(Span::styled(
            crate::i18n::bi(
                "Rust Playground Link Gerado:",
                "Generated Rust Playground Link:",
                en,
            ),
            theme.title(),
        )),
        Line::from(Span::styled(url, theme.dim())),
        Line::from(format!(
            "{}: {}",
            crate::i18n::bi("Exportação local", "Local export", en),
            state
                .lab
                .last_export_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| {
                    crate::i18n::bi(
                        "Pressione E para gerar COMPARTILHAR_RUSTLAB.md",
                        "Press E to generate SHARE_RUSTLAB.md",
                        en,
                    )
                    .to_string()
                })
        )),
        Line::default(),
        Line::from(Span::styled(
            crate::i18n::bi(
                "Soluções Idiomáticas da Comunidade:",
                "Idiomatic Community Solutions:",
                en,
            ),
            theme.title(),
        )),
    ];
    for (idx, sol) in sols.iter().enumerate() {
        let sel = idx == state.lab.community_sel;
        lines.push(Line::from(Span::styled(
            format!(
                "{} {} [{}]",
                if sel { "▶" } else { " " },
                sol.name,
                sol.style_badge
            ),
            if sel { theme.selection() } else { theme.base() },
        )));
        lines.push(Line::from(format!("   {}", sol.pros)));
    }
    let picked = sols[state.lab.community_sel.min(2)];
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        crate::i18n::bi(
            "Prévia da abordagem selecionada:",
            "Preview of selected approach:",
            en,
        ),
        theme.dim(),
    )));
    for l in picked.code.lines().take(7) {
        lines.push(Line::from(Span::styled(format!("  {l}"), theme.success())));
    }
    render_scrolled_lines(frame, lines, &mut state.lab.scroll, area);
}

fn draw_lab_ecosystems(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
    en: bool,
) {
    let cols = Layout::horizontal([Constraint::Length(36), Constraint::Min(20)]).split(area);
    let ecos = crate::lab_tools::modern_ecosystems(en);
    let items: Vec<ListItem> = ecos
        .iter()
        .enumerate()
        .map(|(i, eco)| {
            let sel = i == state.lab.ecosystem_sel;
            ListItem::new(Line::from(Span::styled(
                format!(
                    "{} {} ({})",
                    if sel { "▶" } else { " " },
                    eco.name,
                    eco.dep_name
                ),
                if sel { theme.selection() } else { theme.base() },
            )))
        })
        .collect();
    frame.render_widget(
        List::new(items).block(Block::bordered().title(crate::i18n::bi(
            " Trilhas Modernas ",
            " Modern Tracks ",
            en,
        ))),
        cols[0],
    );
    let eco = ecos[state.lab.ecosystem_sel.min(ecos.len() - 1)];
    let mut lines = vec![
        Line::from(Span::styled(
            format!("{} · {}", eco.name, eco.ecosystem),
            theme.title(),
        )),
        Line::from(eco.summary),
        Line::default(),
        Line::from(eco.architecture_notes),
        Line::default(),
        Line::from(Span::styled(eco.run_instructions, theme.success())),
    ];
    for preview in eco.template_code.lines().take(5) {
        lines.push(Line::from(Span::styled(
            format!("  {preview}"),
            theme.dim(),
        )));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(Block::bordered().title(crate::i18n::bi(
                " Arquitetura & Template ",
                " Architecture & Template ",
                en,
            ))),
        cols[1],
    );
}

fn draw_lab_lifetimes(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
    en: bool,
) {
    let report = state.lab.last_lifetimes.clone().unwrap_or_else(|| {
        crate::lab_tools::LifetimeVisualizerReport::analyze_for(
            &state.editor.buf.to_string_lossy(),
            en,
        )
    });
    let mut lines = vec![Line::from(Span::styled(
        format!(
            "{}: {}  ({})",
            crate::i18n::bi("Marcadores na margem [L]", "Gutter markers [L]", en),
            if state.lab.lifetime_gutter_enabled {
                crate::i18n::bi("ATIVADOS", "ENABLED", en)
            } else {
                crate::i18n::bi("desligados", "off", en)
            },
            crate::i18n::bi(
                "┣ nascimento · ┃ vivo · ✘ conflito",
                "┣ birth · ┃ alive · ✘ conflict",
                en
            )
        ),
        theme.title(),
    ))];
    lines.push(Line::default());
    for raw in report.summary_lines_for(en) {
        let style = if raw.contains("✘") || raw.contains("Conflitos") || raw.contains("Conflicts")
        {
            theme.error()
        } else if raw.contains("✓") || raw.contains("&'a") {
            theme.success()
        } else if raw.contains("&'b mut") {
            theme.warning()
        } else if raw.starts_with("Linha do Tempo")
            || raw.starts_with("Timeline")
            || raw.starts_with("Lifetimes")
        {
            theme.title()
        } else {
            theme.base()
        };
        lines.push(Line::from(Span::styled(raw, style)));
    }
    render_scrolled_lines(frame, lines, &mut state.lab.scroll, area);
}

fn draw_lab_layout(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
    en: bool,
) {
    let report = state.lab.last_layout.clone().unwrap_or_else(|| {
        crate::lab_tools::MemoryLayoutReport::analyze_for(&state.editor.buf.to_string_lossy(), en)
    });
    let mut lines = Vec::new();
    if report.user_types.is_empty() {
        lines.push(Line::from(Span::styled(
            crate::i18n::bi(
                "Nenhuma `struct` com campos encontrada no arquivo aberto.",
                "No `struct` with fields found in the open file.",
                en,
            ),
            theme.dim(),
        )));
    } else {
        for st in &report.user_types {
            let mode = if st.is_repr_c {
                crate::i18n::bi("#[repr(C)] (ordem fixa)", "#[repr(C)] (fixed order)", en)
            } else {
                crate::i18n::bi(
                    "#[repr(Rust)] (compilador pode reordenar)",
                    "#[repr(Rust)] (compiler may reorder)",
                    en,
                )
            };
            lines.push(Line::from(Span::styled(
                if en {
                    format!(
                        "struct {} · {} · size_of = {}B · align_of = {}B · Padding = {}B · Opt = {}B (-{}B)",
                        st.name,
                        mode,
                        st.declared_size,
                        st.declared_align,
                        st.total_padding,
                        st.optimal_size,
                        st.bytes_saved
                    )
                } else {
                    format!(
                        "struct {} · {} · size_of = {}B · align_of = {}B · Padding = {}B · Ótimo = {}B (-{}B)",
                        st.name,
                        mode,
                        st.declared_size,
                        st.declared_align,
                        st.total_padding,
                        st.optimal_size,
                        st.bytes_saved
                    )
                },
                theme.title(),
            )));
            for f in &st.fields {
                lines.push(Line::from(format!(
                    "  +{:<3}B  {:<12}: {:<16} size={}B align={}B  ({}: {}B)",
                    f.offset,
                    f.name,
                    f.ty,
                    f.size,
                    f.align,
                    crate::i18n::bi("padding antes", "padding before", en),
                    f.padding_before
                )));
            }
            lines.push(Line::default());
        }
    }
    lines.push(Line::from(Span::styled(
        crate::i18n::bi(
            "Null Pointer Optimization (NPO) & Tipos da Biblioteca Padrão:",
            "Null Pointer Optimization (NPO) & Standard Library Types:",
            en,
        ),
        theme.title(),
    )));
    for fact in &report.npo_facts {
        lines.push(Line::from(vec![
            Span::styled(
                format!(
                    "  {:<28} {:>2}B (align {}B) · ",
                    fact.type_expr, fact.size_bytes, fact.align_bytes
                ),
                theme.success(),
            ),
            Span::raw(fact.note),
        ]));
    }
    render_scrolled_lines(frame, lines, &mut state.lab.scroll, area);
}

fn draw_lab_desugar(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
    en: bool,
) {
    let report = state.lab.last_desugar.clone().unwrap_or_else(|| {
        crate::lab_tools::DesugarReport::analyze_for(&state.editor.buf.to_string_lossy(), en)
    });
    let cols = Layout::horizontal([Constraint::Length(34), Constraint::Min(20)]).split(area);
    let sel = state
        .lab
        .desugar_sel
        .min(report.items.len().saturating_sub(1));
    let list_items: Vec<ListItem> = report
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let line_tag = item
                .line_1based
                .map(|l| format!("L{l} "))
                .unwrap_or_default();
            ListItem::new(Line::from(Span::styled(
                format!(
                    "{} {line_tag}{}",
                    if i == sel { "▶" } else { " " },
                    item.title
                ),
                if i == sel {
                    theme.selection()
                } else {
                    theme.base()
                },
            )))
        })
        .collect();
    frame.render_widget(
        List::new(list_items).block(Block::bordered().title(if report.detected_from_editor {
            crate::i18n::bi(" Detectados no Editor ", " Detected in Editor ", en)
        } else {
            crate::i18n::bi(" Exemplos Clássicos ", " Classic Examples ", en)
        })),
        cols[0],
    );
    if let Some(item) = report.items.get(sel) {
        let mut detail = vec![
            Line::from(Span::styled(
                crate::i18n::bi(
                    "Você escreve (Syntax Sugar / Macro):",
                    "You write (Syntax Sugar / Macro):",
                    en,
                ),
                theme.title(),
            )),
            Line::from(Span::styled(
                format!("  {}", item.sugar_snippet),
                theme.warning(),
            )),
            Line::default(),
            Line::from(Span::styled(
                crate::i18n::bi(
                    "O compilador expande por baixo dos panos:",
                    "Compiler expands under the hood:",
                    en,
                ),
                theme.title(),
            )),
        ];
        for l in item.expanded_rust.lines() {
            detail.push(Line::from(Span::styled(format!("  {l}"), theme.success())));
        }
        detail.push(Line::default());
        detail.push(Line::from(item.explanation));
        frame.render_widget(
            Paragraph::new(detail).wrap(Wrap { trim: true }).block(
                Block::bordered().title(crate::i18n::bi(
                    " Código Expandido ",
                    " Expanded Code ",
                    en,
                )),
            ),
            cols[1],
        );
    }
}

fn draw_lab_polyglot_intuition(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
    en: bool,
) {
    if state.lab.polyglot_mode_flashcards {
        let cards = if en {
            &crate::lab_tools::INTUITION_FLASHCARDS_EN[..]
        } else {
            &crate::lab_tools::INTUITION_FLASHCARDS[..]
        };
        let sel = state.lab.flashcard_sel.min(cards.len() - 1);
        let card = cards[sel];
        let cols = Layout::horizontal([Constraint::Length(32), Constraint::Min(20)]).split(area);
        let list_items: Vec<ListItem> = cards
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let done = if state.lab.completed_flashcards.contains(c.id) {
                    "✓"
                } else {
                    " "
                };
                ListItem::new(Line::from(Span::styled(
                    format!(
                        "{} [{done}] {} · {}",
                        if i == sel { "▶" } else { " " },
                        c.id,
                        c.title
                    ),
                    if i == sel {
                        theme.selection()
                    } else {
                        theme.base()
                    },
                )))
            })
            .collect();
        frame.render_widget(
            List::new(list_items).block(Block::bordered().title(format!(
                " Flashcards ({}/{}) ",
                state.lab.completed_flashcards.len(),
                cards.len()
            ))),
            cols[0],
        );
        let mut right = vec![
            Line::from(Span::styled(
                if en {
                    format!(
                        "{} · Topic: {} — Does this code compile or give a Borrow Error?",
                        card.id, card.topic
                    )
                } else {
                    format!(
                        "{} · Tema: {} — Esse código compila ou dá Borrow Error?",
                        card.id, card.topic
                    )
                },
                theme.title(),
            )),
            Line::default(),
        ];
        for l in card.snippet.lines() {
            right.push(Line::from(Span::styled(format!("  {l}"), theme.warning())));
        }
        right.push(Line::default());
        right.push(Line::from(Span::styled(
            crate::i18n::bi(
                "Responda: [C] Compila ✔   ou   [E] Dá Erro de Compilação/Borrow ✘",
                "Answer: [C] Compiles ✔   or   [E] Compiler/Borrow Error ✘",
                en,
            ),
            theme.base(),
        )));
        if let Some((ok, msg)) = &state.lab.flashcard_feedback {
            right.push(Line::default());
            right.push(Line::from(Span::styled(
                msg.clone(),
                if *ok { theme.success() } else { theme.error() },
            )));
        }
        frame.render_widget(
            Paragraph::new(right)
                .wrap(Wrap { trim: true })
                .block(Block::bordered().title(crate::i18n::bi(
                    " Desafio Rápido (+25 XP + SRS) ",
                    " Quick Challenge (+25 XP + SRS) ",
                    en,
                ))),
            cols[1],
        );
        return;
    }

    let entries = crate::lab_tools::polyglot_dictionary(en);
    let sel = state.lab.polyglot_sel.min(entries.len() - 1);
    let cols = Layout::horizontal([Constraint::Length(34), Constraint::Min(20)]).split(area);
    let list_items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            ListItem::new(Line::from(Span::styled(
                format!(
                    "{} {} · {}",
                    if i == sel { "▶" } else { " " },
                    e.id,
                    e.concept
                ),
                if i == sel {
                    theme.selection()
                } else {
                    theme.base()
                },
            )))
        })
        .collect();
    frame.render_widget(
        List::new(list_items).block(Block::bordered().title(crate::i18n::bi(
            " Dicionário Poliglota ",
            " Polyglot Dictionary ",
            en,
        ))),
        cols[0],
    );
    let entry = entries[sel];
    let filter_name = match state.lab.polyglot_lang_filter {
        1 => "Python",
        2 => "JavaScript/TS",
        3 => "C++",
        _ => "Python + JS + C++",
    };
    let mut right = vec![Line::from(Span::styled(
        if en {
            format!(
                "{} · Comparison ({filter_name}) [F filters]:",
                entry.concept
            )
        } else {
            format!("{} · Comparação ({filter_name}) [F filtra]:", entry.concept)
        },
        theme.title(),
    ))];
    if matches!(state.lab.polyglot_lang_filter, 0 | 1) {
        right.push(Line::from(vec![
            Span::styled("  Python : ", theme.dim()),
            Span::raw(entry.from_python),
        ]));
    }
    if matches!(state.lab.polyglot_lang_filter, 0 | 2) {
        right.push(Line::from(vec![
            Span::styled("  JS/TS  : ", theme.dim()),
            Span::raw(entry.from_js),
        ]));
    }
    if matches!(state.lab.polyglot_lang_filter, 0 | 3) {
        right.push(Line::from(vec![
            Span::styled("  C++    : ", theme.dim()),
            Span::raw(entry.from_cpp),
        ]));
    }
    right.push(Line::from(vec![
        Span::styled("  Rust   : ", theme.success()),
        Span::styled(entry.idiomatic_rust, theme.success()),
    ]));
    right.push(Line::default());
    right.push(Line::from(entry.why_rust_differs));
    right.push(Line::default());
    for l in entry.rust_code.lines().take(5) {
        right.push(Line::from(Span::styled(format!("  {l}"), theme.dim())));
    }
    frame.render_widget(
        Paragraph::new(right)
            .wrap(Wrap { trim: true })
            .block(Block::bordered().title(crate::i18n::bi(
                " Enter carrega exemplo Rust ",
                " Enter loads Rust example ",
                en,
            ))),
        cols[1],
    );
}

fn inner(r: Rect) -> Rect {
    Rect {
        x: r.x.saturating_add(1),
        y: r.y.saturating_add(1),
        width: r.width.saturating_sub(2),
        height: r.height.saturating_sub(2),
    }
}
fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width.saturating_sub(2));
    let h = height.min(area.height.saturating_sub(2));
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn renderiza_create_e_lab_em_ingles_sem_panico() {
        let mut app = crate::app::App::for_snapshot();
        app.config.language = "en-US".to_string();
        app.create = Some(crate::create::CreateState::new_for(true));

        let backend = TestBackend::new(120, 36);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                super::draw(frame, &mut app, area);
            })
            .unwrap();

        let buf = terminal.backend().buffer();
        let text: String = buf
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<Vec<_>>()
            .join("");
        assert!(text.contains("Create Mode") || text.contains("Projects"));
    }
}
