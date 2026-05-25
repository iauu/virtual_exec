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

pub enum ExecState {
    Nothing,
    RstInput(Option<String>),
    Exec
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = Arc::new(Mutex::new(AppState::new(vec![SYS.clone(), BASIC.clone()])));
    

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        let mut is_exec = ExecState::Nothing;

        let mut app_obj = app.lock().unwrap();

        if (app_obj.focus) == FocusArea::TextArea {
            app_obj.repl_input.focused = true;
        } else {
            app_obj.repl_input.focused = false;
        }
        
        match event::read()? {
            Event::Key(key) => {
                if is_enter(&key) {
                    if
                        app_obj.can_compile &&
                        app_obj.repl_input.cursor_line + 1 == app_obj.repl_input.line_count() &&
                        app_obj.repl_input.cursor_col == app_obj.repl_input.current_line().len()
                     {
                        is_exec = ExecState::Exec;
                    } else {
                        app_obj.repl_input.insert_newline();
                    }
                } else if is_tab(&key) {
                    app_obj.repl_input.insert_tab();
                } else if is_backspace(&key) {
                    app_obj.repl_input.delete_char_backward();
                } else if is_delete(&key) {
                    app_obj.repl_input.delete_char_forward();
                } else if key.code == KeyCode::Char('d') && has_ctrl(&key) {
                    break
                } else if key.code == KeyCode::Char('c') && has_ctrl(&key) {
                    if app_obj.repl_input.is_empty() {
                        app_obj.first_ctrl_c = true;
                        is_exec = ExecState::RstInput(Some("Press Ctrl+C again, or press Ctrl+D to exit".to_string()));
                    } else {
                        is_exec = ExecState::RstInput(None);
                    }
                } else if key.code == KeyCode::Left {
                    if has_ctrl(&key) {
                        app_obj.repl_input.move_word_left();
                    } else {
                        app_obj.repl_input.move_left();
                    }
                } else if key.code == KeyCode::Right {
                    if has_ctrl(&key) {
                        app_obj.repl_input.move_word_right();
                    } else {
                        app_obj.repl_input.move_right();
                    }
                } else if key.code == KeyCode::Up {
                    if (app_obj.repl_input.is_empty() || app_obj.switch_mode) && app_obj.repl_buffer.len() > 0 {
                        app_obj.switch_mode = true;
                        if app_obj.idx > 0 {
                            app_obj.idx -= 1;
                        }
                        if app_obj.idx > app_obj.repl_buffer.len() - 1 {
                            app_obj.idx = app_obj.repl_buffer.len() - 1;
                        }
                        let text = app_obj.repl_buffer[app_obj.idx].0.clone();
                        app_obj.repl_input.set_text(text);
                    } else {
                        app_obj.repl_input.move_up();
                    }
                } else if key.code == KeyCode::Down {
                    if (app_obj.repl_input.is_empty() || app_obj.switch_mode)  && app_obj.repl_buffer.len() > 0 {
                        app_obj.switch_mode = true;
                        if app_obj.idx < app_obj.repl_buffer.len() - 1 {
                            app_obj.idx += 1;
                        } else {
                            app_obj.idx = app_obj.repl_buffer.len() - 1;
                        }
                        let text = app_obj.repl_buffer[app_obj.idx].0.clone();
                        app_obj.repl_input.set_text(text);
                    } else {
                        app_obj.repl_input.move_down();
                    }
                } else if key.code == KeyCode::PageUp {
                    app_obj.repl_input.move_page_up();
                } else if key.code == KeyCode::PageDown {
                    app_obj.repl_input.move_page_down();
                } else if let Some(c) = get_char(&key) {
                    app_obj.switch_mode = false;
                    app_obj.repl_input.insert_char(c);
                }
            }
            Event::Mouse(mouse) => {
                if is_left_click(&mouse) {
                    if let Some(area) = app_obj.click_region_registry
                        .handle_click(mouse.column, mouse.row)
                    {
                        match area {
                            InteractArea::Textarea => {
                                app_obj.focus = FocusArea::TextArea;
                            },
                            _ => {}
                        }
                    }
                }
                if let Some(v) = get_scroll(&mouse) {
                    if let Some(area) = app_obj.click_region_registry
                        .handle_click(mouse.column, mouse.row)
                    {
                        match area {
                            InteractArea::Textarea => {
                               if v < 0 {
                                   for _ in 0..(-v) {
                                       app_obj.repl_input.scroll_up();
                                   }
                               } else {
                                   for _ in 0..v {
                                       app_obj.repl_input.scroll_down();
                                   }
                               }
                            },
                            InteractArea::ReplBufferArea => {
                                if v < 0 {
                                    app_obj.repl_buffer_state.scroll_up(v as usize);
                                } else {
                                    let height = app_obj.repl_buffer_height;
                                    app_obj.repl_buffer_state.scroll_down(v as usize, height);
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
