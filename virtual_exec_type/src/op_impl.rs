use crate::error::TypeConversionError;
use crate::vm_type::Error;
use alloc::format;
use alloc::string::{String, ToString};

register_op_add!(i64, i64, i64, |a: i64, b: i64| Ok(a.wrapping_add(b)));
register_op_add!(f64, f64, f64);
register_op_add!(i64, f64, f64, |a, b| Ok((a as f64) + b));
register_op_add!(f64, i64, f64, |a, b| Ok(a + (b as f64)));

register_op_sub!(i64, i64, i64, |a: i64, b: i64| Ok(a.wrapping_sub(b)));
register_op_sub!(f64, f64, f64);
register_op_sub!(i64, f64, f64, |a, b| Ok((a as f64) - b));
register_op_sub!(f64, i64, f64, |a, b| Ok(a - (b as f64)));

register_op_mul!(i64, i64, i64, |a: i64, b: i64| Ok(a.wrapping_mul(b)));
register_op_mul!(f64, f64, f64);
register_op_mul!(i64, f64, f64, |a, b| Ok((a as f64) * b));
register_op_mul!(f64, i64, f64, |a, b| Ok(a * (b as f64)));

register_op_div!(i64, i64, f64, |a: i64, b: i64| if b == 0 {
    Err(TypeConversionError::DivideByZeroError)
} else {
    Ok((a as f64) / (b as f64))
});
register_op_div!(f64, f64, f64, |a: f64, b: f64| if b == 0.0 {
    Err(TypeConversionError::DivideByZeroError)
} else {
    Ok(a / b)
});
register_op_div!(i64, f64, f64, |a: i64, b: f64| if b == 0.0 {
    Err(TypeConversionError::DivideByZeroError)
} else {
    Ok((a as f64) / b)
});
register_op_div!(f64, i64, f64, |a: f64, b: i64| if b == 0 {
    Err(TypeConversionError::DivideByZeroError)
} else {
    Ok(a / (b as f64))
});

register_op_moduls!(i64, i64, i64, |a: i64, b: i64| if b == 0 {
    Err(TypeConversionError::DivideByZeroError)
} else {
    Ok(a.wrapping_rem(b))
});
register_op_moduls!(f64, f64, f64, |a: f64, b: f64| if b == 0.0 {
    Err(TypeConversionError::DivideByZeroError)
} else {
    Ok(a % b)
});
register_op_moduls!(i64, f64, f64, |a: i64, b: f64| if b == 0.0 {
    Err(TypeConversionError::DivideByZeroError)
} else {
    Ok((a as f64) % b)
});
register_op_moduls!(f64, i64, f64, |a: f64, b: i64| if b == 0 {
    Err(TypeConversionError::DivideByZeroError)
} else {
    Ok(a % (b as f64))
});

register_op_eq!(bool, bool, bool, |a: bool, b: bool| Ok(a == b));
register_op_le!(bool, bool, bool, |a: bool, b: bool| Ok(a <= b));
register_op_lt!(bool, bool, bool, |a: bool, b: bool| Ok(!a & b));
register_op_ge!(bool, bool, bool, |a: bool, b: bool| Ok(a >= b));
register_op_gt!(bool, bool, bool, |a: bool, b: bool| Ok(a & !b));
register_op_ne!(bool, bool, bool, |a: bool, b: bool| Ok(a != b));

register_op_eq!(i64, f64, bool, |a, b| Ok(
    ((a as f64) - b).abs() < f64::EPSILON
));
register_op_eq!(f64, i64, bool, |b, a| Ok(
    ((a as f64) - b).abs() < f64::EPSILON
));

register_op_le!(i64, i64, bool);
register_op_le!(f64, f64, bool);
register_op_le!(i64, f64, bool, |a, b| Ok((a as f64) <= b));
register_op_le!(f64, i64, bool, |a, b| Ok(a <= (b as f64)));

register_op_ge!(i64, i64, bool);
register_op_ge!(f64, f64, bool);
register_op_ge!(i64, f64, bool, |a, b| Ok((a as f64) >= b));
register_op_ge!(f64, i64, bool, |a, b| Ok(a >= (b as f64)));

register_op_lt!(i64, i64, bool);
register_op_lt!(f64, f64, bool);
register_op_lt!(i64, f64, bool, |a, b| Ok((a as f64) < b));
register_op_lt!(f64, i64, bool, |a, b| Ok(a < (b as f64)));

register_op_gt!(i64, i64, bool);
register_op_gt!(f64, f64, bool);
register_op_gt!(i64, f64, bool, |a, b| Ok((a as f64) > b));
register_op_gt!(f64, i64, bool, |a, b| Ok(a > (b as f64)));

register_op_ne!(i64, f64, bool, |a, b| Ok(
    ((a as f64) - b).abs() >= f64::EPSILON
));
register_op_ne!(f64, i64, bool, |b, a| Ok(
    ((a as f64) - b).abs() >= f64::EPSILON
));

register_op_bsl!(i64, i64, i64, |a: i64, b: i64| Ok(a.wrapping_shl(b as u32)));
register_op_bsr!(i64, i64, i64, |a: i64, b: i64| Ok(a.wrapping_shr(b as u32)));
register_op_band!(i64, i64, i64);
register_op_bor!(i64, i64, i64);
register_op_and!(bool, bool, bool, |a: bool, b: bool| Ok(a && b));
register_op_or!(bool, bool, bool, |a: bool, b: bool| Ok(a || b));
register_op_bor!(bool, bool, bool, |a: bool, b: bool| Ok(a ^ b));
// register_op_bsl!(VirFloat, VirInt, |a: VirFloat, b: VirInt| Ok(VirFloat::new(a.value << b.value)));
// register_op_bsr!(VirFloat, VirInt, |a: VirFloat, b: VirInt| Ok(VirFloat::new(a.value >> b.value)));
// Best attempt would be using i64::pow, but overflow would be a big issue

register_op_not!(bool, bool, |a: bool| Ok(!a));
register_op_pos!(i64, i64, |a| Ok(a));
register_op_pos!(f64, f64, |a| Ok(a));
register_op_neg!(i64, i64, |a: i64| Ok(a.wrapping_neg()));
register_op_neg!(f64, f64);

register_op_add!(String, String, String, |a: String, b: String| Ok(format!(
    "{}{}",
    a, b
)
.to_string()));

macro_rules! auto_diff_type_op {
    ($(($lhs:ty, $rhs: ty)),+) => {
        $(
            register_op_eq!($lhs, $rhs, bool, |_, _| Ok(false));
            register_op_ne!($lhs, $rhs, bool, |_, _| Ok(true));
            register_op_eq!($rhs, $lhs, bool, |_, _| Ok(false));
            register_op_ne!($rhs, $lhs, bool, |_, _| Ok(true));
        )*
    };
}

macro_rules! auto_same_type_eq {
    ($($t:ty),+) => {
        $(
            register_op_eq!($t, $t, bool);
            register_op_ne!($t, $t, bool);
        )*
    };
}

auto_same_type_eq!((), bool, f64, i64, Error, String);

auto_diff_type_op!(
    ((), bool),
    ((), f64),
    ((), i64),
    ((), Error),
    ((), String),
    (bool, f64),
    (bool, i64),
    (bool, Error),
    (bool, String),
    (f64, Error),
    (f64, String),
    (i64, Error),
    (i64, String),
    (String, Error)
);
