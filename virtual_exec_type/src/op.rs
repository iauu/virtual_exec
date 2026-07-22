use crate::error::TypeConversionError;
use crate::mem::MemoryAllocator;
use crate::mem::ValuePtr;

type BinaryOpFn = for<'ctx> fn(
    lhs: &ValuePtr<'ctx>,
    rhs: &ValuePtr<'ctx>,
    arena: &MemoryAllocator<'ctx>,
) -> Option<Result<ValuePtr<'ctx>, TypeConversionError>>;

type UnaryOpFn = for<'ctx> fn(
    rhs: &ValuePtr<'ctx>,
    arena: &MemoryAllocator<'ctx>,
) -> Option<Result<ValuePtr<'ctx>, TypeConversionError>>;

#[macro_export]
macro_rules! __binary_op_register {
    (
        $lhs_type:ty,
        $rhs_type:ty,
        $func:expr,
        $ret:ty,
        $impl_path:path
    ) => {
        const _: () = {
            use $crate::base::Upcast;
            fn _op_impl<'ctx>(
                lhs: &$crate::mem::ValuePtr<'ctx>,
                rhs: &$crate::mem::ValuePtr<'ctx>,
                arena: &$crate::mem::MemoryAllocator<'ctx>,
            ) -> Option<Result<$crate::mem::ValuePtr<'ctx>, $crate::error::TypeConversionError>>
            {
                let lhs_val = <$lhs_type as $crate::base::Downcast>::from_value(lhs.clone())?;
                let rhs_val = <$rhs_type as $crate::base::Downcast>::from_value(rhs.clone())?;
                fn checked<F>(f: F) -> F
                where
                    F: Fn(
                        $lhs_type,
                        $rhs_type,
                    )
                        -> ::core::result::Result<$ret, $crate::error::TypeConversionError>,
                {
                    f
                }
                let func = checked($func);
                Some(match func(lhs_val, rhs_val) {
                    Ok(result) => result
                        .from_value(arena)
                        .map_err(|e: $crate::error::MemoryError| e.into()),
                    Err(err) => Err(err),
                })
            }

            ::inventory::submit! {
                $impl_path { function: _op_impl }
            };
        };
    };
}

macro_rules! __binary_op_create {
    ($name:tt, $alt_name:tt, $op:tt) => {
        __binary_op_create!(@impl $name, @impl $alt_name, $op, $);
    };
    (@impl $name:tt, @impl $alt_name:tt, $op:tt, $d:tt) => {
        ::paste::paste!{
            pub struct [< Op $alt_name Impl>] {pub function: $crate::op::BinaryOpFn }
            ::inventory::collect!([< Op $alt_name Impl>]);

            pub fn [<err_op_ $name>]<'ctx>(lhs: $crate::mem::ValuePtr<'ctx>, rhs: $crate::mem::ValuePtr<'ctx>, arena: &$crate::mem::MemoryAllocator<'ctx>) -> ::core::result::Result<$crate::mem::ValuePtr<'ctx>, $crate::error::TypeConversionError> {
                for implementation in ::inventory::iter::<[<Op $alt_name Impl>]> {
                    if let ::core::option::Option::Some(result) = (implementation.function)(&lhs, &rhs, arena) {
                       return result;
                    }
                }
                return Err($crate::error::TypeConversionError::UndefinedOperatorMethodError)
            }

            #[macro_export]
            macro_rules! [<register_op_ $name>] {
                ($d lhs_type:ty, $d rhs_type:ty, $d ret:ty) => {
                    [<register_op_ $name>]!($d lhs_type, $d rhs_type, $d ret, |a, b| Ok(a $op b));
                };
                ($d lhs_type:ty, $d rhs_type:ty, $d ret:ty, $d func:expr) => {
                    $crate::__binary_op_register!($d lhs_type, $d rhs_type, $d func, $d ret, $crate::op::[<Op $alt_name Impl>]);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! __unary_op_register {
    (
        $rhs_type:ty,
        $func:expr,
        $ret:ty,
        $impl_path:path
    ) => {
        const _: () = {
            use $crate::base::Upcast;
            fn _op_impl<'ctx>(
                rhs: &$crate::mem::ValuePtr<'ctx>,
                arena: &$crate::mem::MemoryAllocator<'ctx>,
            ) -> Option<Result<$crate::mem::ValuePtr<'ctx>, $crate::error::TypeConversionError>>
            {
                let rhs_val = <$rhs_type as $crate::base::Downcast>::from_value(rhs.clone())?;
                fn checked<F>(f: F) -> F
                where
                    F: Fn(
                        $rhs_type,
                    )
                        -> ::core::result::Result<$ret, $crate::error::TypeConversionError>,
                {
                    f
                }
                let func = checked($func);
                Some(match func(rhs_val) {
                    Ok(result) => result
                        .from_value(arena)
                        .map_err(|e: $crate::error::MemoryError| e.into()),
                    Err(err) => Err(err),
                })
            }

            ::inventory::submit! {
                $impl_path { function: _op_impl }
            };
        };
    };
}

macro_rules! __unary_op_create {
    ($name:tt, $alt_name:tt, $op:tt) => {
        __unary_op_create!(@impl $name, @impl $alt_name, $op, $);
    };
    (@impl $name:tt, @impl $alt_name:tt, $op:tt, $d:tt) => {
        ::paste::paste!{
            pub struct [< Op $alt_name Impl>] {pub function: $crate::op::UnaryOpFn }
            ::inventory::collect!([< Op $alt_name Impl>]);
            pub fn [<err_op_ $name>]<'ctx>(rhs: $crate::mem::ValuePtr<'ctx>, arena: &$crate::mem::MemoryAllocator<'ctx>) -> ::core::result::Result<$crate::mem::ValuePtr<'ctx>, $crate::error::TypeConversionError> {
                for implementation in ::inventory::iter::<[<Op $alt_name Impl>]> {
                    if let ::core::option::Option::Some(result) = (implementation.function)(&rhs, arena) {
                       return result;
                    }
                }
                return Err($crate::error::TypeConversionError::UndefinedOperatorMethodError)
            }

            #[macro_export]
            macro_rules! [<register_op_ $name>] {
                ($d rhs_type:ty, $d ret:ty) => {
                    [<register_op_ $name>]!($d rhs_type, $d ret, |a| Ok($op a));
                };
                ($d rhs_type:ty, $d ret:ty, $d func:expr) => {
                    $crate::__unary_op_register!($d rhs_type, $d func, $d ret, $crate::op::[<Op $alt_name Impl>]);
                }
            }
        }
    };
}

__binary_op_create!(add, Add, +);
__binary_op_create!(sub, Sub, -);
__binary_op_create!(mul, Mul, *);
__binary_op_create!(div, Div, /);
__binary_op_create!(eq, Eq, ==);
__binary_op_create!(ge, Ge, >=);
__binary_op_create!(gt, Gt, >);
__binary_op_create!(le, Le, <=);
__binary_op_create!(lt, Lt, <);
__binary_op_create!(ne, Ne, !=);
__binary_op_create!(moduls, Mod, %);
__binary_op_create!(bsl, Bsl, <<);
__binary_op_create!(bsr, Bsr, >>);
__binary_op_create!(band, BitwiseAnd, &);
__binary_op_create!(bor, BitwiseOr, |);
__binary_op_create!(bxor, BitwiseXor, ^);
__binary_op_create!(and, And, &&);
__binary_op_create!(or, Or, ||);
__unary_op_create!(not, Not, !);
__unary_op_create!(pos, Pos, +);
__unary_op_create!(neg, Neg, -);
__unary_op_create!(bnot, BitwiseNot, ~);
