use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use ratatui_interact::components::TextAreaState;
use ratatui_interact::prelude::ScrollableContentState;
use ratatui_interact::traits::ClickRegionRegistry;
use virtual_exec_core::fn_extern::MethodResolver;
use virtual_exec_core::Machine;


#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Selected {
    Textarea,
    Vars,
    Debug,
}

pub struct AppState {
    pub machine: Machine<'static>,
    pub repl_buffer: HashMap<String, String>,
    pub repl_input: TextAreaState,
    pub scrollable_content_state: ScrollableContentState,
    pub show_vars: bool,
    pub show_debug: bool,
    pub click_region_registry: ClickRegionRegistry<()>
}

impl AppState {
    pub fn new(resolvers: Vec<MethodResolver>) -> Self {
        Self {
            machine: Machine::new(vec![], 2 << 28, 2 << 20, resolvers).unwrap(),
            repl_buffer: HashMap::new(),
            repl_input: TextAreaState::new(""),
            show_vars: false,
            show_debug: false,
            click_region_registry: ClickRegionRegistry::new(),
            scrollable_content_state: ScrollableContentState::empty()
        }
    }
}

pub type App = Arc<Mutex<AppState>>;