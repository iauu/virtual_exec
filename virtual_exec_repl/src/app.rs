use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use ratatui_interact::components::TextAreaState;
use ratatui_interact::prelude::ScrollableContentState;
use ratatui_interact::traits::ClickRegionRegistry;
use virtual_exec_core::fn_extern::MethodResolver;
use virtual_exec_core::Machine;


#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum InteractArea {
    None,
    ReplBufferArea,
    Textarea,
    Vars,
    DebugInst,
    DebugStack
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum FocusArea {
    TextArea
}

impl Into<InteractArea> for FocusArea {
    fn into(self) -> InteractArea {
        match self {
            FocusArea::TextArea => InteractArea::Textarea
        }
    }
}

pub struct AppState {
    pub machine: Machine<'static>,
    pub repl_buffer: Vec<(String, String)>,
    pub repl_input: TextAreaState,
    pub repl_buffer_state: ScrollableContentState,
    pub show_vars: bool,
    pub show_debug: bool,
    pub click_region_registry: ClickRegionRegistry<InteractArea>,
    pub first_ctrl_c: bool,
    pub focus: FocusArea,
    pub repl_buffer_height: usize
}

impl AppState {
    pub fn new(resolvers: Vec<MethodResolver>) -> Self {
        Self {
            machine: Machine::new(vec![], 2 << 28, 2 << 20, resolvers).unwrap(),
            repl_buffer: vec![],
            repl_input: TextAreaState::new(""),
            show_vars: false,
            show_debug: false,
            click_region_registry: ClickRegionRegistry::new(),
            repl_buffer_state: ScrollableContentState::empty(),
            first_ctrl_c: false,
            focus: FocusArea::TextArea,
            repl_buffer_height: 1,
        }
    }
}

pub type App = Arc<Mutex<AppState>>;