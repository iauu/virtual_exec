use cfg_if::cfg_if;
use virtual_exec_extern::*;
use virtual_exec_type::vm_type::*;
use virtual_exec_core::Machine;
use virtual_exec_type::error::ExecutionError;

#[fn_extern_wrap]
fn print<'a>(_: &mut Machine<'a>, str: Str) -> Result<None, Error> {
    print!("{}", str);
    Ok(())
}

cfg_if!(
    if #[cfg(feature = "tokio-io")] {
        use tokio::io::{self, AsyncWriteExt};
        #[fn_extern_wrap_async]
        async fn print_async<'a>(_: &mut Machine<'a>, str: Str) -> Result<None, Error> {
            let mut stdout = io::stdout();
            stdout.write_all(str.as_bytes()).await.map_err(|_| ExecutionError::GenericError)
        }
        
        extern_link!(Print, print, print_async, 1);
    } else {
        extern_link!(Print, print, 1);
    }

);