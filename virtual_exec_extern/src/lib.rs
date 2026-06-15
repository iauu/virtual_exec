pub use virtual_exec_macro::{fn_extern_wrap, fn_extern_wrap_async};

extern crate alloc;

#[macro_export]
macro_rules! extern_link {
    ($name:ident, $sync_fn:expr, $size:expr) => {
        pub struct $name {
        }

        impl ::virtual_exec_core::fn_extern::FnExternConstruct for $name {
            fn new() -> Self {
                Self {
                }
            }
        }
        impl<'async_trait> ::virtual_exec_core::fn_extern::FnExtern for $name {

            fn fn_extern_sync<'a>(
                &self,
                machine: &mut ::virtual_exec_core::Machine<'a>,
                values: ::std::vec::Vec<::virtual_exec_type::mem::ValuePtr<'a>>
            ) -> Result<::virtual_exec_type::mem::ValuePtr<'a>, ::virtual_exec_type::error::ExecutionError> {
                $sync_fn(machine, values)
            }
            
            fn get_size(&self) -> usize {
                $size
            }
        }
    };
    ($name:ident, $sync_fn:expr, $async_fn:expr, $size:expr) => {
        pub struct $name {
        }

        impl ::virtual_exec_core::fn_extern::FnExternConstruct for $name {
            fn new() -> Self {
                Self {
                }
            }
        }
        #[::async_trait::async_trait]
        impl ::virtual_exec_core::fn_extern::FnExtern for $name {

            fn fn_extern_sync<'a>(
                &self,
                machine: &mut ::virtual_exec_core::Machine<'a>,
                values: ::std::vec::Vec<::virtual_exec_type::mem::ValuePtr<'a>>
            ) -> Result<::virtual_exec_type::mem::ValuePtr<'a>, ::virtual_exec_type::error::ExecutionError> {
                $sync_fn(machine, values)
            }

            async fn fn_extern_async<'a>(
                &self,
                machine: &mut ::virtual_exec_core::Machine<'a>,
                values: ::std::vec::Vec<::virtual_exec_type::mem::ValuePtr<'a>>
            ) -> Result<::virtual_exec_type::mem::ValuePtr<'a>, ::virtual_exec_type::error::ExecutionError> where
    'a : 'async_trait {
                $async_fn(machine, values).await
            }
            
            fn get_size(&self) -> usize {
                $size
            }
        }
    };
}

#[macro_export]
macro_rules! add_item {
    ($map:expr, $name:expr, $item:ident) => {
        {
            use ::virtual_exec_core::fn_extern::FnExternConstruct;
            $map.insert($name.to_string(), ::std::sync::Arc::new($item::new()));
        };
    };
}


pub mod __private {
    pub use alloc::sync::Arc;
    pub use alloc::string::String;
}

#[macro_export]
macro_rules! resolve {
    ($(($name:expr, $item:ident)),*) => {

        {
            let mut map: ::virtual_exec_type::HashMap<$crate::__private::String, $crate::__private::Arc<dyn ::virtual_exec_core::fn_extern::FnExtern + ::core::marker::Send + ::core::marker::Sync>> = ::virtual_exec_type::HashMap::new();
            $($crate::add_item!(map, $name, $item);)*
            ::virtual_exec_core::fn_extern::MethodResolver::new(
                map
            )
        }
    };
}