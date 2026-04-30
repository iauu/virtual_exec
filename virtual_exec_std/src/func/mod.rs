
macro_rules! func {
    ($name: ident) => {
        pub mod $name;
        #[allow(unused)]
        pub use $name::*;
    };
}

func!(push_array);
func!(pop_array);
func!(arr_get_from_idx);
func!(create_array);
func!(arr_get_len);