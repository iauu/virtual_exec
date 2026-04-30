pub use virtual_exec_macro::{fn_extern_wrap, fn_extern_wrap_async};

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