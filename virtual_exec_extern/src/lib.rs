mod example;

pub use virtual_exec_macro::{fn_extern_wrap, fn_extern_wrap_async};

#[macro_export]
macro_rules! extern_link {
    ($name:ident, $sync_fn:expr) => {
        struct $name<'a> {
            _scoped: ::std::marker::PhantomData<&'a ()>
        }

        impl<'a> ::virtual_exec_core::fn_extern::FnExternConstruct for $name<'a> {
            fn new() -> Self {
                Self {
                    _scoped: ::std::marker::PhantomData,
                }
            }
        }
        impl<'a> ::virtual_exec_core::fn_extern::FnExtern<'a> for $name<'a> {

            fn fn_extern_sync(
                &self,
                machine: &mut ::virtual_exec_core::Machine<'a>,
                values: ::std::vec::Vec<::virtual_exec_type::mem::ValuePtr<'a>>
            ) -> Result<::virtual_exec_type::mem::ValuePtr<'a>, ::virtual_exec_type::error::ExecutionError> {
                $sync_fn(machine, values)
            }
        }
    };
    ($name:ident, $sync_fn:expr, $async_fn:expr) => {
        struct $name<'a> {
            _scoped: ::std::marker::PhantomData<&'a ()>
        }

        impl<'a> ::virtual_exec_core::fn_extern::FnExternConstruct for $name<'a> {
            fn new() -> Self {
                Self {
                    _scope: ::std::marker::PhantomData,
                }
            }
        }
        #[::async_trait::async_trait]
        impl<'a> ::virtual_exec_core::fn_extern::FnExtern<'a> for $name<'a> {

            fn fn_extern_sync(
                &self,
                machine: &mut ::virtual_exec_core::Machine<'a>,
                values: ::std::vec::Vec<::virtual_exec_type::mem::ValuePtr<'a>>
            ) -> Result<::virtual_exec_type::mem::ValuePtr<'a>, ::virtual_exec_type::error::ExecutionError> {
                $sync_fn(machine, values)
            }

            async fn fn_extern_async(
                &self,
                machine: &mut ::virtual_exec_core::Machine<'a>,
                values: ::std::vec::Vec<::virtual_exec_type::mem::ValuePtr<'a>>
            ) -> Result<::virtual_exec_type::mem::ValuePtr<'a>, ::virtual_exec_type::error::ExecutionError> {
                $async_fn(machine, values).await
            }
        }
    };
}