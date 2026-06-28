use alloc::sync::Arc;
use virtual_exec_type::HashMap;
use virtual_exec_type::mem::MemoryAllocator;
use crate::Machine;
use async_lock::Mutex;


macro_rules! fn_extern_arg_type_construct {
    ($(($name:ident, $t:ty)),*) => {
        ::paste::paste!(
            #[derive(Clone)]
            pub enum FnExternArg<'a, 'b> where 'a : 'b {
                $($name($t),)*
            }

            impl<'a, 'b>  FnExternArg<'a, 'b> {
                $(
                    #[allow(non_snake_case)]
                    pub fn [< unwrap_ $name >](self) -> $t {
                        match self {
                            Self::$name(v) => v,
                            _ => panic!("Expected enum variant {}", stringify!($name))
                        }
                    }
                )*
            }

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
            );
    };
}

fn_extern_arg_type_construct!((Alloc, MemoryAllocator<'a>), (Machine, Arc<Mutex<&'b mut Machine<'a>>>));

pub struct LazyMapping<'a, 'b>(Arc<Mutex<&'b mut Machine<'a>>>, HashMap<FnExternArgType, FnExternArg<'a, 'b>>);

impl<'a, 'b> LazyMapping<'a, 'b> {
    pub fn new(machine: &'b mut Machine<'a>) -> Self {
        Self(Arc::new(Mutex::new(machine)), HashMap::new())
    }

    pub fn construct(machine: Arc<Mutex<&'b mut Machine<'a>>>, arg: FnExternArgType) -> FnExternArg<'a, 'b> {
        match arg {
            FnExternArgType::Alloc => {
                FnExternArg::Alloc(machine.lock_blocking().alloc.clone())
            },
            FnExternArgType::Machine => {
                FnExternArg::Machine(machine)
            },
        }
    }

    pub fn get(&mut self, ty: FnExternArgType) -> FnExternArg<'a, 'b> {
        self.1.entry(ty).or_insert_with(|| LazyMapping::construct(Arc::clone(&self.0), ty)).clone()
    }
}