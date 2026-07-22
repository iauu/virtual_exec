macro_rules! func {
    ($name: ident) => {
        pub mod $name;
        #[allow(unused)]
        pub use $name::*;
    };
}

func!(print);
