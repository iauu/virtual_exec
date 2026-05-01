use cfg_if::cfg_if;
use virtual_exec_extern::*;
use virtual_exec_type::vm_type::*;
use virtual_exec_core::Machine;
use virtual_exec_type::base::TypeCast;
use virtual_exec_type::error::ExecutionError;

#[fn_extern_wrap]
fn print<'a>(_: &mut Machine<'a>, str: Any<'a>) -> Result<None, Error> {
    if let Some(s) = str.as_string() {
        print!("{}", s);
    } else {
        print!("{}", str.lock_arc_blocking().to_string());
    }
    Ok(())
}

cfg_if!(
    if #[cfg(feature = "tokio-io")] {
        use tokio::io::{self, AsyncWriteExt};
    }
);

cfg_if!(
    if #[cfg(feature = "tokio-io")] {
        #[fn_extern_wrap_async]
        async fn print_async<'a>(_: &mut Machine<'a>, str: Any<'a>) -> Result<None, Error> {
            let mut stdout = io::stdout();
            if let Some(s) = str.as_string() {
                stdout.write_all(s.as_bytes()).await.map_err(|_| ExecutionError::GenericError)
            } else {
                stdout.write_all(str.lock_arc_blocking().to_string().as_bytes()).await.map_err(|_| ExecutionError::GenericError)
            }
        }
        
        extern_link!(Print, print, print_async, 1);
    } else {
        extern_link!(Print, print, 1);
    }

);

#[fn_extern_wrap]
fn println<'a>(_: &mut Machine<'a>, str: Any<'a>) -> Result<None, Error> {
    if let Some(s) = str.as_string() {
        println!("{}", s);
    } else {
        println!("{}", str.lock_arc_blocking().to_string());
    }
    Ok(())
}

cfg_if!(
    if #[cfg(feature = "tokio-io")] {
        #[fn_extern_wrap_async]
        async fn println_async<'a>(_: &mut Machine<'a>, str: Any<'a>) -> Result<None, Error> {
            let mut stdout = io::stdout();
            if let Some(s) = str.as_string() {
                stdout.write_all((s + "\n").as_bytes()).await.map_err(|_| ExecutionError::GenericError)
            } else {
                stdout.write_all((str.lock_arc_blocking().to_string() + "\n").as_bytes()).await.map_err(|_| ExecutionError::GenericError)
            }
        }
        
        extern_link!(PrintLn, println, println_async, 1);
    } else {
        extern_link!(PrintLn, println, 1);
    }

);