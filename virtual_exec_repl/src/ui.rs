use crate::app::{App, InteractArea};
use ratatui::crossterm::style::Stylize;
use ratatui::prelude::Modifier;
use ratatui::widgets::Padding;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use ratatui_interact::components::{
    Button, ButtonVariant, ScrollableContent, TextArea, TextAreaStyle,
};
use ratatui_interact::prelude::{ButtonState, ScrollableContentStyle};
use virtual_exec_core::parse;

pub(crate) fn ui(f: &mut Frame, app: &mut App) {
    // Clear click regions
    let mut app = app.lock().unwrap();
    app.click_region_registry.clear();

    let area = f.area();

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Fill(8), // Repl history
            Constraint::Max(10), // Input
            Constraint::Max(1),  // Status
        ])
        .split(area);

    // Repl buffer
    let mut content = app
        .repl_buffer
        .iter()
        .map(|(k, v)| {
            if v.is_empty() {
                format!(">>> {}", k)
            } else {
                format!(">>> {}\n{}", k, v)
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
        .split("\n")
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    if let Some(eval_state) = app.eval_state.clone() {
        content.push(format!("\n>>> {}\n{}", eval_state.code, eval_state.buffer));
    }
    if app
        .repl_buffer_state
        .is_at_bottom(chunks[0].height as usize)
    {
        app.repl_buffer_state
            .set_lines(content.iter().map(|s| s.to_string()).collect());
        app.repl_buffer_state
            .scroll_to_bottom(chunks[0].height as usize);
    } else {
        app.repl_buffer_state
            .set_lines(content.iter().map(|s| s.to_string()).collect());
    }
    let state = app.repl_buffer_state.clone();
    let title = ScrollableContent::new(&state).style(
        ScrollableContentStyle::default().text_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    );
    f.render_widget(title, chunks[0]);
    app.click_region_registry
        .register(chunks[0], InteractArea::ReplBufferArea);
    app.repl_buffer_height = chunks[0].height as usize;

    // TextArea
    let textarea_area = chunks[1];

    let style = TextAreaStyle::default()
        .focused_border(Color::Cyan)
        .cursor_fg(Color::Yellow)
        .line_number_fg(Color::DarkGray)
        .current_line_bg(Some(Color::Rgb(40, 40, 50)))
        .show_line_numbers(true);

    let textarea = TextArea::new().label("Input").placeholder("").style(style);

    let render_result = textarea.render_stateful(f, textarea_area, &mut app.repl_input);
    app.click_region_registry
        .register(render_result.click_region.area, InteractArea::Textarea);

    // Status bar
    let status_text = format!(
        "{}:{} ({}) | ",
        app.repl_input.cursor_line + 1,
        app.repl_input.cursor_col + 1,
        app.repl_input.line_count(),
    );
    let mut can_compile = parse(&app.repl_input.text()).is_ok();
    if !can_compile {
        can_compile = parse(&(app.repl_input.text() + ";")).is_ok();
    }
    app.can_compile = can_compile;

    let status_bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([
            Constraint::Length(25),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[2]);

    let status = Paragraph::new(Line::from(vec![
        status_text.into(),
        if can_compile {
            Span::styled("✔ Valid", Style::default().green())
        } else {
            Span::styled("✖ Syntax Error", Style::default().red())
        },
    ]))
    .style(Style::default().fg(Color::White).bg(Color::Reset))
    .block(Block::default());
    f.render_widget(status, status_bar_chunks[0]);

    let b1 = Button::new(&"Debug Panel", &app.show_debug).variant(ButtonVariant::Toggle);
    f.render_widget(b1, status_bar_chunks[1]);
    app.click_region_registry
        .register(status_bar_chunks[1], InteractArea::ToggleDebugs);

    let b2 = Button::new(&"Vars Panel", &app.show_vars).variant(ButtonVariant::Toggle);
    f.render_widget(b2, status_bar_chunks[2]);
    app.click_region_registry
        .register(status_bar_chunks[2], InteractArea::ToggleVars);
}
