use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use ratatui::crossterm::style::Stylize;
use ratatui::prelude::Modifier;
use ratatui_interact::components::{ScrollableContent, TextArea, TextAreaStyle};
use ratatui_interact::prelude::ScrollableContentStyle;
use crate::app::{App, InteractArea};
use virtual_exec_core::parse;

pub(crate) fn ui(f: &mut Frame, app: &mut App) {
    // Clear click regions
    app.lock().unwrap().click_region_registry.clear();

    let area = f.area();

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Fill(8), // Repl history
            Constraint::Max(10), // Input
            Constraint::Max(1) // Status
        ])
        .split(area);

    // Repl buffer
    let content = app.lock().unwrap().repl_buffer.iter().map(|(k, v)| format!(">>> {}\n{}", k, v))
        .collect::<Vec<String>>()
        .join("\n")
        .split("\n")
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    if app.lock().unwrap().repl_buffer_state.is_at_bottom(chunks[0].height as usize) {
        app.lock().unwrap().repl_buffer_state.set_lines(content.iter().map(|s| s.to_string()).collect());
        app.lock().unwrap().repl_buffer_state.scroll_to_bottom(chunks[0].height as usize);
    } else {
        app.lock().unwrap().repl_buffer_state.set_lines(content.iter().map(|s| s.to_string()).collect());
    }
    let state = app.lock().unwrap().repl_buffer_state.clone();
    let title = ScrollableContent::new(&state)
        .style(
            ScrollableContentStyle::default()
                .text_style(Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD))

        );
    f.render_widget(title, chunks[0]);
    app.lock().unwrap().click_region_registry
        .register(chunks[0], InteractArea::ReplBufferArea);
    app.lock().unwrap().repl_buffer_height = chunks[0].height as usize;

    // TextArea
    let textarea_area = chunks[1];

    let style = TextAreaStyle::default()
        .focused_border(Color::Cyan)
        .cursor_fg(Color::Yellow)
        .line_number_fg(Color::DarkGray)
        .current_line_bg(Some(Color::Rgb(40, 40, 50)))
        .show_line_numbers(true);

    let textarea = TextArea::new()
        .label("Input")
        .placeholder("")
        .style(style);

    let render_result = textarea.render_stateful(f, textarea_area, &mut app.lock().unwrap().repl_input);
    app.lock().unwrap().click_region_registry
        .register(render_result.click_region.area, InteractArea::Textarea);

    // Status bar
    let status_text = format!(
        "{}:{} ({}) | ",
        app.lock().unwrap().repl_input.cursor_line + 1,
        app.lock().unwrap().repl_input.cursor_col + 1,
        app.lock().unwrap().repl_input.line_count(),

    );
    let status = Paragraph::new(Line::from(
        vec![status_text.into(),
             if parse(&app.lock().unwrap().repl_input.text()).is_ok() { Span::styled("✔ Valid", Style::default().green()) } else {
                 Span::styled("✖ Syntax Error", Style::default().red()) }
        ]))
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .block(Block::default());
    f.render_widget(status, chunks[2]);
}