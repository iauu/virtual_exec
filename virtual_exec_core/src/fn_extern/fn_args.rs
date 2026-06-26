use virtual_exec_type::HashMap;
use virtual_exec_type::mem::MemoryAllocator;
use crate::Machine;

#[derive(Clone)]
pub enum FnExternArg<'a> {
    Alloc(MemoryAllocator<'a>),
}

macro_rules! fn_extern_arg_type_construct {
    ($($name:ident),*) => {
        #[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Copy)]
        pub enum FnExternArgType {
            $($name,)*
        }

        impl FnExternArgType {
            pub fn err_string() -> &'static str {
                concat!("Expected one of "$(, "'", stringify!($name), "'")*)
            }

            pub fn from_ident(ident: &::proc_macro2::Ident) -> Option<$crate::fn_extern::fn_args::FnExternArgType> {
                match ident.to_string().as_ref() {
                    $(stringify!($name) => Some($crate::fn_extern::fn_args::FnExternArgType::$name),)*
                    _ => None
                }
            }
        }
    };
}

fn_extern_arg_type_construct!(Alloc);

pub struct LazyMapping<'a, 'b>(&'b mut Machine<'a>, HashMap<FnExternArgType, FnExternArg<'a>>);

impl<'a, 'b> LazyMapping<'a, 'b> {
    pub fn new(machine: &'b mut Machine<'a>) -> Self {
        Self(machine, HashMap::new())
    }

    pub fn construct(machine: &'b Machine<'a>, arg: FnExternArgType) -> FnExternArg<'a> {
        match arg {
            FnExternArgType::Alloc => {
                FnExternArg::Alloc(machine.alloc.clone())
            }
        }
    }

    pub fn get(&mut self, ty: FnExternArgType) -> FnExternArg<'a> {
        self.1.entry(ty).or_insert_with(|| LazyMapping::construct(self.0, ty)).clone()
    }
}