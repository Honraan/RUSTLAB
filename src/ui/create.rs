use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, List, ListItem, Paragraph, Wrap};

use crate::app::App;
use crate::config::{AiProvider, AiSettings};
use crate::create::{AgentRole, CreateState, CreateView, Focus};
use crate::execution::StreamKind;

struct ExecPanelState<'a> {
    running: bool,
    running_label: Option<&'a str>,
    input_open: bool,
}

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme;
    let config = app.config.clone();
    let running = app.exec.running;
    let running_label = app.exec.running_label.clone();
    let input_open = app.exec.accepts_input();
    let Some(state) = app.create.as_mut() else {
        return;
    };
    match state.view {
        CreateView::Start => draw_start(frame, &config, state, area),
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
        ),
    }
    if state.consent_request.is_some() {
        draw_consent(frame, area, &theme);
    }
    if state.agent.consent_pending.is_some() {
        let popup = centered(area, 72, 10);
        let msg = "O Agente está configurado para API remota compatível com OpenAI.\n\nEnviar esta pergunta, arquivo atual e diagnósticos para o provedor remoto?\n\nEnter autoriza (e salva para próximas) · Esc cancela e usa Guia local";
        frame.render_widget(ratatui::widgets::Clear, popup);
        frame.render_widget(
            Paragraph::new(msg)
                .style(theme.warning())
                .block(
                    Block::bordered()
                        .title(" Consentimento para API remota ")
                        .border_style(theme.border_style(true)),
                )
                .wrap(Wrap { trim: true }),
            popup,
        );
    }
    if state.agent.model_picker.open {
        crate::ui::mentor::draw_model_picker(
            frame,
            &theme,
            &state.agent.model_picker,
            &config.ai.ollama_model,
            area,
        );
    }
    if let Some(prompt) = &state.prompt {
        let popup = centered(area, 72, 8);
        let title = match prompt.kind {
            crate::create::PromptKind::NewProject => {
                "Novo projeto Cargo (nome ou ~/pasta/nome · Tab preenche pasta)"
            }
            crate::create::PromptKind::OpenPath => "Abrir caminho do projeto",
            crate::create::PromptKind::NewFile => "Novo arquivo .rs (ex.: src/util.rs)",
            crate::create::PromptKind::AddDependency => {
                "Adicionar dependência (nome ou nome@versão; pode usar rede)"
            }
            crate::create::PromptKind::RemoveDependency => "Remover dependência (nome)",
            crate::create::PromptKind::ProjectSearch => {
                "Buscar em todo o projeto (texto ou símbolo)"
            }
            crate::create::PromptKind::SetProjectsDir => {
                "Pasta padrão para novos projetos (vazio = padrão)"
            }
        };
        let msg = format!(
            "{title}\n> {}_\n{}\nEnter confirma · Esc cancela",
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

fn draw_start(frame: &mut Frame, config: &crate::config::Config, state: &CreateState, area: Rect) {
    let theme = crate::theme::Theme::for_id(config.ui.theme);
    let mut options: Vec<String> = [
        "Criar projeto novo",
        "Abrir projeto Cargo por caminho",
        "Reabrir último projeto",
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
            .map(|p| format!("Recente: {p}")),
    );
    let last = config
        .workspace
        .last_project
        .as_deref()
        .unwrap_or("nenhum salvo");
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, title)| {
            let detail = if i == 0 {
                match config.workspace.projects_dir.as_deref() {
                    Some(custom) => format!("  (pasta: {custom} · d altera)"),
                    None => "  (pasta padrão · d escolhe pasta)".to_string(),
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
        .title(" Modo Criar · Projetos Cargo ")
        .border_style(theme.border_style(true));
    let status_height = u16::from(state.last_failure.is_some()) * 3;
    let chunks =
        Layout::vertical([Constraint::Min(5), Constraint::Length(status_height)]).split(area);
    let mut list_state =
        ratatui::widgets::ListState::default().with_selected(Some(state.start_sel));
    frame.render_stateful_widget(List::new(items).block(block), chunks[0], &mut list_state);
    if let Some(reason) = &state.last_failure {
        frame.render_widget(
            Paragraph::new(format!("Erro:\n{reason}"))
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
    draw_workspace_top_bar(frame, theme, state, top_parts[0]);
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
            .title(" Arquivos ")
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
        .unwrap_or_else(|| "sem arquivo".into());
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
        format!(" · {} rodando", exec.running_label.unwrap_or("cargo"))
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
                "stdin > {}_\nEnter enviar · Ctrl+D EOF · Shift+Tab foco",
                state.console_input
            ))
            .style(theme.base()),
            console_parts[1],
        );
    }
    let diag_text = if state.diags.is_empty() {
        "Sem diagnósticos".to_string()
    } else {
        format!(
            "{} diagnóstico(s)\n{}",
            state.diags.len(),
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
                    .title(format!(" Diagnósticos · {} ", state.lsp_status))
                    .border_style(theme.border_style(false)),
            )
            .wrap(Wrap { trim: true }),
        right[1],
    );

    if state.agent.open {
        draw_agent_panel(frame, theme, ai, state, cols[3]);
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
                "buscar: {} {}",
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
        suggestions.push(Line::from("↑↓ escolher · Enter/Tab aceitar · Esc fechar"));
        suggestions.push(Line::styled(
            "Sugestões locais por prefixo (sem análise de tipos)",
            theme.dim(),
        ));
        frame.render_widget(ratatui::widgets::Clear, popup);
        frame.render_widget(
            Paragraph::new(suggestions).block(
                Block::bordered()
                    .title(" Autocomplete · Ctrl+Espaço ")
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
                    .title(" Ajuda · F1/Esc fechar ")
                    .border_style(theme.border_style(true)),
            ),
            box_rect,
        );
    } else if let Some((symbol, x, y)) = state.hovered_doc.as_ref() {
        let code = state.editor.buf.to_string_lossy();
        let resolved = crate::doc_hint::resolve_symbol_doc(symbol, &code);
        let local_doc = state.workspace_index.find_doc(symbol);
        if resolved.is_some() || local_doc.is_some() {
            let (title, summary, url) = if let Some(doc) = local_doc {
                let u = resolved
                    .as_ref()
                    .map(|r| r.best_url().to_string())
                    .unwrap_or_else(|| "https://rust-br.github.io/rust-book-pt-br/".to_string());
                (symbol.clone(), doc, u)
            } else {
                let r = resolved.expect("checked is_some above");
                let u = r.best_url().to_string();
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
                        format!("{url} · clique para abrir doc"),
                        theme.dim(),
                    )),
                ])
                .wrap(Wrap { trim: true })
                .block(
                    Block::bordered()
                        .title(" Ajuda Rust · clique para documentação PT/EN ")
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
        draw_libraries_modal(frame, theme, state, area);
    } else if state.build_menu_open {
        draw_build_modal(frame, theme, state, area);
    } else if state.lab.open {
        draw_lab_modal(frame, theme, state, area);
    }
}

fn draw_workspace_top_bar(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
) {
    state.top_lib_btn_area = Rect::default();
    state.top_build_btn_area = Rect::default();
    if area.height == 0 || area.width < 12 {
        return;
    }
    let lib_text = if state.lib_menu_open {
        " [ ▲ Bibliotecas (F9) ] "
    } else {
        " [ ▼ Bibliotecas (F9) ] "
    };
    let build_text = if state.build_menu_open {
        " [ ▲ Compilar Linux/Win (F4) ] "
    } else {
        " [ Compilar Linux/Win (F4) ] "
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
) {
    let modal = centered(area, 82, 20);
    state.lib_modal_area = modal;
    frame.render_widget(ratatui::widgets::Clear, modal);
    frame.render_widget(
        Block::bordered()
            .title(" ▼ Bibliotecas Visuais, Efeitos & Mini Engine ")
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
            "Clique/Enter adiciona · T insere template · D abre docs · Esc fecha",
            theme.dim(),
        )),
        parts[0],
    );

    let cols = Layout::horizontal([Constraint::Length(28), Constraint::Min(18)]).split(parts[1]);
    let items: Vec<ListItem> = crate::doc_hint::EXTRA_LIBRARIES
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

    let selected = crate::doc_hint::EXTRA_LIBRARIES
        .get(state.lib_menu_sel)
        .unwrap_or(&crate::doc_hint::EXTRA_LIBRARIES[0]);
    let mut details = vec![
        Line::from(Span::styled(
            format!("{} · {}", selected.name, selected.category),
            theme.title(),
        )),
        Line::from(selected.summary),
        Line::default(),
        Line::from(vec![
            Span::styled("Como funciona: ", theme.title()),
            Span::raw(selected.docs_pt),
        ]),
        Line::from(vec![
            Span::styled("Inserção automática: ", theme.title()),
            Span::raw(selected.use_line),
        ]),
        Line::from(vec![
            Span::styled("Símbolos principais: ", theme.title()),
            Span::raw(selected.key_items),
        ]),
        Line::default(),
        Line::from(Span::styled("Exemplo rápido:", theme.dim())),
    ];
    for line in selected.example.lines() {
        details.push(Line::from(Span::styled(line, theme.success())));
    }
    frame.render_widget(
        Paragraph::new(details)
            .wrap(Wrap { trim: true })
            .block(Block::bordered().title(" Documentação da Biblioteca ")),
        cols[1],
    );

    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("[ Docs: {} ] (clique ou D)", selected.doc_url),
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
) {
    let modal = centered(area, 72, 11);
    state.build_modal_area = modal;
    frame.render_widget(ratatui::widgets::Clear, modal);
    frame.render_widget(
        Block::bordered()
            .title(" Compilar & Executar App (Linux / Windows / Bevy) ")
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
            "Escolha como compilar ou abrir seu projeto visual/CLI/Bevy:",
            theme.dim(),
        )),
        parts[0],
    );

    let options = [
        "Abrir em janela própria (GUI / Bevy / Mini Engine)",
        "Rodar em outro terminal (cargo run externo — ideal p/ Bevy)",
        "Compilar executável Linux release (target/release/...)",
        "Compilar executável Windows .exe (x86_64-pc-windows-gnu)",
        "Build padrão debug no workspace (cargo build)",
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
            "Enter ou clique confirma · ↑/↓ navega · Esc fecha",
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
) {
    let mut provider_label = match ai.provider {
        AiProvider::Ollama => format!("Ollama · {}", ai.ollama_model),
        AiProvider::Remote => format!("API · {}", ai.remote_model),
        AiProvider::Disabled => "Guia local".to_string(),
    };
    if state.agent_running() {
        provider_label.push_str(" · pensando…");
    }
    let focused = state.focus == Focus::Agent;
    let block = Block::bordered()
        .title(format!(" Agente · {provider_label} "))
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
            AgentRole::User => ("Você: ".to_string(), theme.title(), theme.base()),
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
                "★ Código pronto: Alt+Enter (ou /aplicar) aplica no editor (Ctrl+Z desfaz)",
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
            "Enter envia · Alt+O modelo · Alt+Enter aplicar · Esc editor",
            theme.dim(),
        )),
    ];
    frame.render_widget(
        Paragraph::new(input_lines).wrap(Wrap { trim: true }),
        parts[2],
    );
}

fn draw_consent(frame: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
    let popup = centered(area, 74, 12);
    let text = "Execução local NÃO é sandbox.\nSeu código e dependências rodam com permissões da sua conta: podem acessar arquivos, rede e iniciar processos.\n\nF5 aceita entrada por linhas (não é terminal completo). Dependências podem usar a rede.\n\nNada foi executado ainda. Enter autoriza comandos futuros; Esc recusa.";
    frame.render_widget(ratatui::widgets::Clear, popup);
    frame.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(theme.warning))
            .block(
                Block::bordered()
                    .title(" Consentimento de execução ")
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
) {
    let modal = centered(area, 88, 24);
    state.lab_modal_area = modal;
    frame.render_widget(ratatui::widgets::Clear, modal);
    frame.render_widget(
        Block::bordered()
            .title(format!(" Laboratório RustLab · {} ", state.lab.tab.title()))
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
                format!("[{}]", tab.short_title())
            } else {
                tab.short_title().to_string()
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
            draw_lab_xray(frame, theme, state, parts[1]);
            "R recalcula · 1-9/0/Tab troca ferramenta · PgUp/PgDn rola · Esc fecha"
        }
        crate::lab_tools::LabToolTab::LenteCompilador => {
            draw_lab_lens(frame, theme, state, parts[1]);
            "S alterna Modo 10 Anos (ELI10) · ↑↓ solução · Enter envia p/ Agente · Esc fecha"
        }
        crate::lab_tools::LabToolTab::DebuggerVisual => {
            draw_lab_debugger(frame, theme, state, parts[1]);
            "N/F10 over · S into · C bp · B ● · G gdb/lldb nativo · R reinicia · X para"
        }
        crate::lab_tools::LabToolTab::ConserteOCodigo => {
            draw_lab_quests(frame, theme, state, parts[1]);
            "↑↓ escolhe missão · Enter carrega quebrado no editor · V valida conserto + XP"
        }
        crate::lab_tools::LabToolTab::PlaygroundComunidade => {
            draw_lab_playground(frame, theme, state, parts[1]);
            "O abre Playground · E exporta Markdown · ↑↓ abordagem · T aplica no editor"
        }
        crate::lab_tools::LabToolTab::EcossistemasModernos => {
            draw_lab_ecosystems(frame, theme, state, parts[1]);
            "↑↓ trilha · Enter/T insere template completo · D adiciona dep ao Cargo.toml"
        }
        crate::lab_tools::LabToolTab::MapaLifetimes => {
            draw_lab_lifetimes(frame, theme, state, parts[1]);
            "L alterna marcadores ┣/┃/✘ na margem · R recalcula · PgUp/PgDn rola · Esc fecha"
        }
        crate::lab_tools::LabToolTab::LayoutMemoria => {
            draw_lab_layout(frame, theme, state, parts[1]);
            "R recalcula · O/Enter envia ordem otimizada de campos ao Agente · Esc fecha"
        }
        crate::lab_tools::LabToolTab::DesacucararMacros => {
            draw_lab_desugar(frame, theme, state, parts[1]);
            "↑↓ escolhe macro/syntax sugar · Enter envia expansão ao Agente · Esc fecha"
        }
        crate::lab_tools::LabToolTab::PoliglotaIntuicao => {
            draw_lab_polyglot_intuition(frame, theme, state, parts[1]);
            if state.lab.polyglot_mode_flashcards {
                "T Dicionário · ↑↓ card · C Compila ✔ · E Dá Erro ✘ (+25 XP/SRS) · Esc fecha"
            } else {
                "T Flashcards · ↑↓ tópico · F filtro py/js/c++ · Enter carrega exemplo Rust"
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
) {
    let report = state.lab.last_xray.clone().unwrap_or_else(|| {
        crate::lab_tools::MemoryXRayReport::analyze(&state.editor.buf.to_string_lossy())
    });
    let lines: Vec<Line> = report
        .summary_lines()
        .into_iter()
        .map(|s| {
            let style = if s.contains("CONFLITO") || s.contains("✗ movido") {
                theme.error()
            } else if s.contains("Zero-Cost") || s.contains("[✓ ativo]") {
                theme.success()
            } else if s.starts_with("Medidor")
                || s.starts_with("Mapa Visual")
                || s.starts_with("Eventos")
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
) {
    let diag_text = state.diagnostics_text_summary();
    let code = state.editor.buf.to_string_lossy();
    let lens = state
        .lab
        .last_lens
        .clone()
        .unwrap_or_else(|| crate::lab_tools::SocraticErrorLens::analyze(&diag_text, &code));
    let mut lines = vec![
        Line::from(Span::styled(
            format!("[{}] {}", lens.code, lens.title),
            theme.title(),
        )),
        Line::from(Span::styled(
            if state.lab.eli10_mode {
                format!("Modo 10 Anos (ELI10): {}", lens.eli10_metaphor)
            } else {
                format!("Pergunta Socrática: {}", lens.socratic_question)
            },
            theme.warning(),
        )),
        Line::default(),
        Line::from(Span::styled(
            "Comparativo Antes → Depois (escolha 1..3):",
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
            Span::styled("   Antes : ", theme.error()),
            Span::raw(fix.before.replace('\n', " | ")),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   Depois: ", theme.success()),
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
) {
    let bps: Vec<String> = state
        .lab
        .debugger
        .breakpoints
        .iter()
        .map(|l| format!("L{l}"))
        .collect();
    let bp_text = if bps.is_empty() {
        "nenhum (Alt+B na linha do editor)".to_string()
    } else {
        bps.join(", ")
    };
    let mut lines = vec![Line::from(Span::styled(
        format!(
            "Estado: {} · Breakpoints ●: {bp_text}",
            if state.lab.debugger.active {
                "ATIVO"
            } else {
                "PARADO"
            }
        ),
        theme.title(),
    ))];
    if let Some(step) = state.lab.debugger.current() {
        lines.push(Line::from(Span::styled(
            format!(
                "Passo {}/{} ▶ Linha {} em `{}`: {}",
                step.step_index + 1,
                state.lab.debugger.steps.len(),
                step.line_1based,
                step.func,
                step.code_line
            ),
            theme.success(),
        )));
        lines.push(Line::from(Span::styled(
            format!(
                "Nota: {} · Call Stack: {}",
                step.note,
                step.call_stack.join(" → ")
            ),
            theme.dim(),
        )));
        lines.push(Line::default());
        lines.push(Line::from(Span::styled("Variáveis Locais:", theme.title())));
        if step.locals.is_empty() {
            lines.push(Line::from("  (nenhuma variável inicializada neste ponto)"));
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
            lines.push(Line::from(Span::styled("Saída capturada:", theme.title())));
            for out in &step.stdout {
                lines.push(Line::from(format!("  > {out}")));
            }
        }
    } else {
        lines.push(Line::from(
            "Pressione N/F10 ou R para iniciar a execução passo a passo do arquivo aberto.",
        ));
    }
    let probe = crate::lab_tools::detect_native_debugger();
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        format!(
            "Debugger Nativo (opcional [G]): {} ({})",
            if probe.available {
                "DISPONÍVEL"
            } else {
                "não encontrado no PATH"
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
) {
    let cols = Layout::horizontal([Constraint::Length(34), Constraint::Min(20)]).split(area);
    let items: Vec<ListItem> = crate::lab_tools::TROUBLESHOOTING_QUESTS
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
        List::new(items).block(Block::bordered().title(" Desafios (8) ")),
        cols[0],
    );
    let q = crate::lab_tools::TROUBLESHOOTING_QUESTS[state
        .lab
        .quest_sel
        .min(crate::lab_tools::TROUBLESHOOTING_QUESTS.len() - 1)];
    let mut detail = vec![
        Line::from(Span::styled(
            format!(
                "{} · Tag: {} · Par: {} linhas · Max Heap: {}",
                q.id, q.error_tag, q.par_lines, q.max_heap_allocs
            ),
            theme.title(),
        )),
        Line::from(q.description),
        Line::default(),
        Line::from(Span::styled(format!("Dica: {}", q.hint), theme.dim())),
    ];
    if let Some(fb) = &state.lab.last_quest_feedback {
        detail.push(Line::default());
        detail.push(Line::from(Span::styled(fb.as_str(), theme.success())));
    }
    frame.render_widget(
        Paragraph::new(detail)
            .wrap(Wrap { trim: true })
            .block(Block::bordered().title(" Detalhes da Missão ")),
        cols[1],
    );
}

fn draw_lab_playground(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
) {
    let code = state.editor.buf.to_string_lossy();
    let url = crate::lab_tools::build_playground_url(&code);
    let sols = crate::lab_tools::community_solutions_for(&code);
    let mut lines = vec![
        Line::from(Span::styled("Rust Playground Link Gerado:", theme.title())),
        Line::from(Span::styled(url, theme.dim())),
        Line::from(format!(
            "Exportação local: {}",
            state
                .lab
                .last_export_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "Pressione E para gerar COMPARTILHAR_RUSTLAB.md".to_string())
        )),
        Line::default(),
        Line::from(Span::styled(
            "Soluções Idiomáticas da Comunidade:",
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
        "Prévia da abordagem selecionada:",
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
) {
    let cols = Layout::horizontal([Constraint::Length(36), Constraint::Min(20)]).split(area);
    let items: Vec<ListItem> = crate::lab_tools::MODERN_ECOSYSTEMS
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
        List::new(items).block(Block::bordered().title(" Trilhas Modernas ")),
        cols[0],
    );
    let eco = crate::lab_tools::MODERN_ECOSYSTEMS[state
        .lab
        .ecosystem_sel
        .min(crate::lab_tools::MODERN_ECOSYSTEMS.len() - 1)];
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
            .block(Block::bordered().title(" Arquitetura & Template ")),
        cols[1],
    );
}

fn draw_lab_lifetimes(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
) {
    let report = state.lab.last_lifetimes.clone().unwrap_or_else(|| {
        crate::lab_tools::LifetimeVisualizerReport::analyze(&state.editor.buf.to_string_lossy())
    });
    let mut lines = vec![Line::from(Span::styled(
        format!(
            "Marcadores na margem [L]: {}  (┣ nascimento · ┃ vivo · ✘ conflito)",
            if state.lab.lifetime_gutter_enabled {
                "ATIVADOS"
            } else {
                "desligados"
            }
        ),
        theme.title(),
    ))];
    lines.push(Line::default());
    for raw in report.summary_lines() {
        let style = if raw.contains("✘") || raw.contains("Conflitos") {
            theme.error()
        } else if raw.contains("✓") || raw.contains("&'a") {
            theme.success()
        } else if raw.contains("&'b mut") {
            theme.warning()
        } else if raw.starts_with("Linha do Tempo") || raw.starts_with("Lifetimes") {
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
) {
    let report = state.lab.last_layout.clone().unwrap_or_else(|| {
        crate::lab_tools::MemoryLayoutReport::analyze(&state.editor.buf.to_string_lossy())
    });
    let mut lines = Vec::new();
    if report.user_types.is_empty() {
        lines.push(Line::from(Span::styled(
            "Nenhuma `struct` com campos encontrada no arquivo aberto.",
            theme.dim(),
        )));
    } else {
        for st in &report.user_types {
            let mode = if st.is_repr_c {
                "#[repr(C)] (ordem fixa)"
            } else {
                "#[repr(Rust)] (compilador pode reordenar)"
            };
            lines.push(Line::from(Span::styled(
                format!(
                    "struct {} · {} · size_of = {}B · align_of = {}B · Padding = {}B · Ótimo = {}B (-{}B)",
                    st.name,
                    mode,
                    st.declared_size,
                    st.declared_align,
                    st.total_padding,
                    st.optimal_size,
                    st.bytes_saved
                ),
                theme.title(),
            )));
            for f in &st.fields {
                lines.push(Line::from(format!(
                    "  +{:<3}B  {:<12}: {:<16} size={}B align={}B  (padding antes: {}B)",
                    f.offset, f.name, f.ty, f.size, f.align, f.padding_before
                )));
            }
            lines.push(Line::default());
        }
    }
    lines.push(Line::from(Span::styled(
        "Null Pointer Optimization (NPO) & Tipos da Biblioteca Padrão:",
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
) {
    let report = state.lab.last_desugar.clone().unwrap_or_else(|| {
        crate::lab_tools::DesugarReport::analyze(&state.editor.buf.to_string_lossy())
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
            " Detectados no Editor "
        } else {
            " Exemplos Clássicos "
        })),
        cols[0],
    );
    if let Some(item) = report.items.get(sel) {
        let mut detail = vec![
            Line::from(Span::styled(
                "Você escreve (Syntax Sugar / Macro):",
                theme.title(),
            )),
            Line::from(Span::styled(
                format!("  {}", item.sugar_snippet),
                theme.warning(),
            )),
            Line::default(),
            Line::from(Span::styled(
                "O compilador expande por baixo dos panos:",
                theme.title(),
            )),
        ];
        for l in item.expanded_rust.lines() {
            detail.push(Line::from(Span::styled(format!("  {l}"), theme.success())));
        }
        detail.push(Line::default());
        detail.push(Line::from(item.explanation));
        frame.render_widget(
            Paragraph::new(detail)
                .wrap(Wrap { trim: true })
                .block(Block::bordered().title(" Código Expandido ")),
            cols[1],
        );
    }
}

fn draw_lab_polyglot_intuition(
    frame: &mut Frame,
    theme: &crate::theme::Theme,
    state: &mut CreateState,
    area: Rect,
) {
    if state.lab.polyglot_mode_flashcards {
        let cards = crate::lab_tools::INTUITION_FLASHCARDS;
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
                format!(
                    "{} · Tema: {} — Esse código compila ou dá Borrow Error?",
                    card.id, card.topic
                ),
                theme.title(),
            )),
            Line::default(),
        ];
        for l in card.snippet.lines() {
            right.push(Line::from(Span::styled(format!("  {l}"), theme.warning())));
        }
        right.push(Line::default());
        right.push(Line::from(Span::styled(
            "Responda: [C] Compila ✔   ou   [E] Dá Erro de Compilação/Borrow ✘",
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
                .block(Block::bordered().title(" Desafio Rápido (+25 XP + SRS) ")),
            cols[1],
        );
        return;
    }

    let entries = crate::lab_tools::POLYGLOT_DICTIONARY;
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
        List::new(list_items).block(Block::bordered().title(" Dicionário Poliglota ")),
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
        format!("{} · Comparação ({filter_name}) [F filtra]:", entry.concept),
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
            .block(Block::bordered().title(" Enter carrega exemplo Rust ")),
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
