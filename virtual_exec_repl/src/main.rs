mod app;
mod component;
mod ui;

use std::io;
use std::sync::{Arc, Mutex};
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use ratatui_interact::{
    components::{TabConfig, TextArea, TextAreaState, TextAreaStyle},
    events::{
        get_char, has_ctrl, is_backspace, is_close_key, is_ctrl_a, is_ctrl_e, is_ctrl_k, is_ctrl_u,
        is_ctrl_w, is_delete, is_end, is_enter, is_home, is_left_click, is_tab, get_scroll
    },
    traits::ClickRegionRegistry,
};

use crate::app::{App, AppState, FocusArea, InteractArea};
use virtual_exec_std::{SYS, BASIC};
use crate::ui::ui;

/// Application state

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = Arc::new(Mutex::new(AppState::new(vec![SYS.clone(), BASIC.clone()])));

    if (app.lock().unwrap().focus) == FocusArea::TextArea {
        app.lock().unwrap().repl_input.focused = true;
    } else {
        app.lock().unwrap().repl_input.focused = false;
    }

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        match event::read()? {
            Event::Key(key) => {
                // if is_close_key(&key) {
                //     app.should_quit = true;
                // } else if is_enter(&key) {
                //     app.textarea.insert_newline();
                // } else if is_tab(&key) {
                //     app.textarea.insert_tab();
                // } else if is_backspace(&key) {
                //     app.textarea.delete_char_backward();
                // } else if is_delete(&key) {
                //     app.textarea.delete_char_forward();
                // } else if is_home(&key) {
                //     if has_ctrl(&key) {
                //         app.textarea.move_to_start();
                //     } else {
                //         app.textarea.move_line_start();
                //     }
                // } else if is_end(&key) {
                //     if has_ctrl(&key) {
                //         app.textarea.move_to_end();
                //     } else {
                //         app.textarea.move_line_end();
                //     }
                // } else if is_ctrl_a(&key) {
                //     app.textarea.move_line_start();
                // } else if is_ctrl_e(&key) {
                //     app.textarea.move_line_end();
                // } else if is_ctrl_u(&key) {
                //     app.textarea.delete_to_line_start();
                // } else if is_ctrl_k(&key) {
                //     app.textarea.delete_to_line_end();
                // } else if is_ctrl_w(&key) {
                //     app.textarea.delete_word_backward();
                // } else if key.code == KeyCode::Char('d') && has_ctrl(&key) {
                //     app.textarea.delete_line();
                // } else if key.code == KeyCode::Char('l') && has_ctrl(&key) {
                //     // Toggle line numbers with Ctrl+L
                //     app.show_line_numbers = !app.show_line_numbers;
                // } else if key.code == KeyCode::Left {
                //     if has_ctrl(&key) {
                //         app.textarea.move_word_left();
                //     } else {
                //         app.textarea.move_left();
                //     }
                // } else if key.code == KeyCode::Right {
                //     if has_ctrl(&key) {
                //         app.textarea.move_word_right();
                //     } else {
                //         app.textarea.move_right();
                //     }
                // } else if key.code == KeyCode::Up {
                //     app.textarea.move_up();
                // } else if key.code == KeyCode::Down {
                //     app.textarea.move_down();
                // } else if key.code == KeyCode::PageUp {
                //     app.textarea.move_page_up();
                // } else if key.code == KeyCode::PageDown {
                //     app.textarea.move_page_down();
                // } else if let Some(c) = get_char(&key) {
                //     app.textarea.insert_char(c);
                // }
            }
            Event::Mouse(mouse) => {
                if is_left_click(&mouse) {
                    if let Some(area) = app
                        .lock().unwrap().click_region_registry
                        .handle_click(mouse.column, mouse.row)
                    {
                        match area {
                            InteractArea::Textarea => {
                                app.lock().unwrap().focus = FocusArea::TextArea;
                            },
                            _ => {}
                        }
                    }
                }
                if let Some(v) = get_scroll(&mouse) {
                    if let Some(area) = app
                        .lock().unwrap().click_region_registry
                        .handle_click(mouse.column, mouse.row)
                    {
                        match area {
                            InteractArea::Textarea => {
                               if v < 0 {
                                   for _ in 0..(-v) {
                                       app.lock().unwrap().repl_input.scroll_up();
                                   }
                               } else {
                                   for _ in 0..v {
                                       app.lock().unwrap().repl_input.scroll_down();
                                   }
                               }
                            },
                            InteractArea::ReplBufferArea => {
                                if v < 0 {
                                    app.lock().unwrap().repl_buffer_state.scroll_up(v as usize);
                                } else {
                                    let height = app.lock().unwrap().repl_buffer_height;
                                    app.lock().unwrap().repl_buffer_state.scroll_down(v as usize, height);
                                }
                            },
                            _ => {}
                        }
                    }
                }

            }
            _ => {}
        }

        // if app.should_quit {
        //     break;
        // }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
