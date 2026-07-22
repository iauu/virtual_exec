mod app;
mod component;
mod r#override;
mod ui;

use crate::app::{AppState, CodeEvalState, FocusArea, InteractArea};
use crate::r#override::{OVERRIDE, PRINT_BUFFER};
use crate::ui::ui;
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
};
use ratatui_interact::events::{
        get_char, get_scroll, has_ctrl, is_backspace, is_delete, is_enter, is_left_click,
        is_tab,
    };
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use virtual_exec_core::sequential::compile::GetInstruction;
use virtual_exec_core::sequential::exec::State;
use virtual_exec_core::sequential::instructions::{InstForceOffset, Instruction};
use virtual_exec_core::{compile, parse};
use virtual_exec_macro::compile;
use virtual_exec_std::{BASIC, SYS};

/// Application state

pub enum ExecState {
    Nothing,
    RstInput(Option<String>),
    Exec,
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = Arc::new(Mutex::new(AppState::new(vec![
        BASIC.clone(),
        OVERRIDE.clone(),
        SYS.clone(),
    ])));
    app.lock().unwrap().machine.machine.state = Ok(State::Terminated);
    app.lock().unwrap().rollback.machine.state = Ok(State::Terminated);

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        let mut is_exec = ExecState::Nothing;

        let mut app_obj = app.lock().unwrap();

        let mut curr_exec = false;

        if let Ok(State::Ok) = app_obj.machine.machine.state {
            curr_exec = true;
            app_obj.machine.machine.lim = u64::MAX;
            let _ = app_obj.machine.sync_run_for(100);
            app_obj
                .eval_state
                .as_mut()
                .unwrap()
                .buffer
                .push_str(PRINT_BUFFER.lock().unwrap().as_str());
            PRINT_BUFFER.lock().unwrap().clear();
            if let Ok(State::Terminated) = app_obj.machine.machine.state {
                app_obj.rollback.machine = app_obj.machine.machine.clone();
                let code = app_obj.eval_state.as_ref().unwrap().code.clone();
                app_obj
                    .eval_state
                    .as_mut()
                    .unwrap()
                    .buffer
                    .push_str(PRINT_BUFFER.lock().unwrap().as_str());
                PRINT_BUFFER.lock().unwrap().clear();
                let buffer = app_obj.eval_state.as_ref().unwrap().buffer.clone();
                app_obj.repl_buffer.push((code, buffer));
                app_obj.eval_state = None;
                app_obj.idx = app_obj.repl_buffer.len();
            }
        }
        if let Err(e) = app_obj.machine.machine.state.clone() {
            app_obj.machine.machine = app_obj.rollback.machine.clone();
            let code = app_obj.eval_state.as_ref().unwrap().code.clone();
            app_obj
                .eval_state
                .as_mut()
                .unwrap()
                .buffer
                .push_str(PRINT_BUFFER.lock().unwrap().as_str());
            PRINT_BUFFER.lock().unwrap().clear();
            let mut buffer = app_obj.eval_state.as_ref().unwrap().buffer.clone();
            buffer.push_str(&format!("Error: {:?}", e));
            app_obj.repl_buffer.push((code, buffer));
            app_obj.eval_state = None;
            app_obj.idx = app_obj.repl_buffer.len();
        }

        if (app_obj.focus) == FocusArea::TextArea {
            app_obj.repl_input.focused = true;
        } else {
            app_obj.repl_input.focused = false;
        }
        if curr_exec {
            match event::poll(Duration::from_millis(1)) {
                Ok(true) => {}
                Ok(false) => {
                    continue;
                }
                Err(e) => Err(e)?,
            };
        }
        let mut ctrl_c = false;
        match event::read()? {
            Event::Key(key) => {
                if is_enter(&key) {
                    if app_obj.can_compile
                        && app_obj.repl_input.cursor_line + 1 == app_obj.repl_input.line_count()
                        && app_obj.repl_input.cursor_col == app_obj.repl_input.current_line().len()
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
                    break;
                } else if key.code == KeyCode::Char('c') && has_ctrl(&key) {
                    if app_obj.first_ctrl_c {
                        break;
                    }
                    if app_obj.repl_input.is_empty() {
                        app_obj.first_ctrl_c = true;
                        ctrl_c = true;
                        is_exec = ExecState::RstInput(Some(
                            "Press Ctrl+C again, or press Ctrl+D to exit".to_string(),
                        ));
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
                    if (app_obj.repl_input.is_empty() || app_obj.switch_mode)
                        && app_obj.repl_buffer.len() > 0
                    {
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
                    if (app_obj.repl_input.is_empty() || app_obj.switch_mode)
                        && app_obj.repl_buffer.len() > 0
                    {
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
                    if let Some(area) = app_obj
                        .click_region_registry
                        .handle_click(mouse.column, mouse.row)
                    {
                        match area {
                            InteractArea::Textarea => {
                                app_obj.focus = FocusArea::TextArea;
                            }
                            InteractArea::ToggleDebugs => {
                                app_obj.show_debug.toggle();
                            }
                            InteractArea::ToggleVars => {
                                app_obj.show_vars.toggle();
                            }
                            _ => {}
                        }
                    }
                }
                if let Some(v) = get_scroll(&mouse) {
                    if let Some(area) = app_obj
                        .click_region_registry
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
                            }
                            InteractArea::ReplBufferArea => {
                                if v < 0 {
                                    app_obj.repl_buffer_state.scroll_up(v as usize);
                                } else {
                                    let height = app_obj.repl_buffer_height;
                                    app_obj.repl_buffer_state.scroll_down(v as usize, height);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        if !ctrl_c {
            app_obj.first_ctrl_c = false;
        }

        if let ExecState::Exec = is_exec
            && app_obj.can_compile
        {
            let code = app_obj.repl_input.text();
            app_obj.repl_input.set_text("");
            let inst = match parse(&code) {
                Ok(module) => module.inst(app_obj.machine.machine.instructions.len() as u64),
                Err(_) => {
                    let parsed = parse(&(code.clone() + ";")).unwrap();
                    let mut pre_inst = compile(&parsed);
                    let inst_last = pre_inst.pop();
                    match inst_last {
                        Some(Instruction::Pop) => {
                            pre_inst.push(Instruction::LoadName(Box::from("_r")));
                            pre_inst.push(Instruction::Swap);
                            pre_inst.push(Instruction::Assign);
                            let offset =
                                app_obj.machine.machine.instructions.len() + pre_inst.len();
                            let print_inst =
                                compile! {if _r != None {print(_r);}}.offset(offset as u64);
                            pre_inst.extend(print_inst);
                        }
                        Some(e) => {
                            pre_inst.push(e);
                        }
                        None => {
                            continue;
                        }
                    }
                    pre_inst
                }
            };
            let len = inst.len();
            app_obj.machine.machine.instructions.extend(inst);
            app_obj.eval_state = Some(CodeEvalState {
                code,
                inst_count: 0,
                buffer: String::new(),
            });
            if let Ok(State::Terminated) = app_obj.machine.machine.state
                && len > 0
            {
                app_obj.machine.machine.state = Ok(State::Ok);
            } else if let Ok(State::Terminated) = app_obj.machine.machine.state
                && len == 0
            {
                let code = app_obj.eval_state.as_ref().unwrap().code.clone();
                app_obj.repl_buffer.push((code, "".to_string()));
                app_obj.eval_state = None;
            }
            app_obj.machine.machine.lim = u64::MAX;
        } else if let ExecState::RstInput(opt) = is_exec {
            let txt = app_obj.repl_input.text();
            app_obj.repl_input.set_text("");
            app_obj
                .repl_buffer
                .push((txt, opt.unwrap_or("".to_string())));
            app_obj.idx = app_obj.repl_buffer.len();
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
