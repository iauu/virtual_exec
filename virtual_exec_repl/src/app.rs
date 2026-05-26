use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use ratatui_interact::components::TextAreaState;
use ratatui_interact::prelude::{ButtonState, ScrollableContentState};
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
    DebugStack,
    ToggleVars,
    ToggleDebugs
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

#[derive(Clone)]
pub struct  CodeEvalState {
    pub code: String,
    pub inst_count: usize,
    pub buffer: String,
}

pub struct AppState {
    pub machine: Machine<'static>,
    pub rollback: Machine<'static>,
    pub repl_buffer: Vec<(String, String)>,
    pub repl_input: TextAreaState,
    pub repl_buffer_state: ScrollableContentState,
    pub show_vars: ButtonState,
    pub show_debug: ButtonState,
    pub click_region_registry: ClickRegionRegistry<InteractArea>,
    pub first_ctrl_c: bool,
    pub focus: FocusArea,
    pub repl_buffer_height: usize,
    pub can_compile: bool,
    pub idx: usize,
    pub switch_mode: bool,
    pub eval_state: Option<CodeEvalState>,
}

impl AppState {
    pub fn new(resolvers: Vec<MethodResolver>) -> Self {
        let machine = Machine::new(vec![], 2 << 32, u64::MAX, resolvers).unwrap();
        let rollback = machine.clone();
        Self {
            machine,
            rollback,
            repl_buffer: vec![],
            repl_input: TextAreaState::new(""),
            show_vars: ButtonState::toggled(false),
            show_debug: ButtonState::toggled(false),
            click_region_registry: ClickRegionRegistry::new(),
            repl_buffer_state: ScrollableContentState::empty(),
            first_ctrl_c: false,
            focus: FocusArea::TextArea,
            repl_buffer_height: 1,
            can_compile: false,
            idx: 0,
            switch_mode: true,
            eval_state: None,
        }
    }
}

pub type App = Arc<Mutex<AppState>>;