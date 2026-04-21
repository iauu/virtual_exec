use crate::base::ValueKind;
use crate::builtin::*;

register_op_add!(VirInt, VirInt, ValueKind::Int);
register_op_add!(VirFloat, VirFloat, ValueKind::Float);
register_op_add!(VirInt, VirFloat, ValueKind::Float);
register_op_add!(VirFloat, VirInt, ValueKind::Float);

register_op_sub!(VirInt, VirInt, ValueKind::Int);
register_op_sub!(VirFloat, VirFloat, ValueKind::Float);
register_op_sub!(VirInt, VirFloat, ValueKind::Float);
register_op_sub!(VirFloat, VirInt, ValueKind::Float);

register_op_mul!(VirInt, VirInt, ValueKind::Int);
register_op_mul!(VirFloat, VirFloat, ValueKind::Float);
register_op_mul!(VirInt, VirFloat, ValueKind::Float);
register_op_mul!(VirFloat, VirInt, ValueKind::Float);

register_op_div!(VirInt, VirInt, ValueKind::Float);
register_op_div!(VirFloat, VirFloat, ValueKind::Float);
register_op_div!(VirInt, VirFloat, ValueKind::Float);
register_op_div!(VirFloat, VirInt, ValueKind::Float);

register_op_moduls!(VirInt, VirInt, ValueKind::Int);
register_op_moduls!(VirFloat, VirFloat, ValueKind::Float);
register_op_moduls!(VirInt, VirFloat, ValueKind::Float);
register_op_moduls!(VirFloat, VirInt, ValueKind::Float);

register_op_eq!(bool, bool, ValueKind::Bool, |a: bool, b: bool| Ok(a == b));
register_op_le!(bool, bool, ValueKind::Bool, |a: bool, b: bool| Ok(a <= b));
register_op_lt!(bool, bool, ValueKind::Bool, |a: bool, b: bool| Ok(!a & b));
register_op_ge!(bool, bool, ValueKind::Bool, |a: bool, b: bool| Ok(a >= b));
register_op_gt!(bool, bool, ValueKind::Bool, |a: bool, b: bool| Ok(a & !b));
register_op_ne!(bool, bool, ValueKind::Bool, |a: bool, b: bool| Ok(a != b));

register_op_eq!(
    VirInt,
    VirInt,
    ValueKind::Bool,
    |a: VirInt, b: VirInt| Ok(a.value == b.value)
);
register_op_eq!(
    VirInt,
    VirFloat,
    ValueKind::Bool,
    |a: VirInt, b: VirFloat| Ok(a.value as f64 == b.value)
);
register_op_eq!(
    VirFloat,
    VirInt,
    ValueKind::Bool,
    |a: VirFloat, b: VirInt| Ok(a.value == b.value as f64)
);
register_op_eq!(
    VirFloat,
    VirFloat,
    ValueKind::Bool,
    |a: VirFloat, b: VirFloat| Ok(a.value == b.value)
);

register_op_le!(
    VirInt,
    VirInt,
    ValueKind::Bool,
    |a: VirInt, b: VirInt| Ok(a.value <= b.value)
);
register_op_le!(
    VirInt,
    VirFloat,
    ValueKind::Bool,
    |a: VirInt, b: VirFloat| Ok(a.value as f64 <= b.value)
);
register_op_le!(
    VirFloat,
    VirInt,
    ValueKind::Bool,
    |a: VirFloat, b: VirInt| Ok(a.value <= b.value as f64)
);
register_op_le!(
    VirFloat,
    VirFloat,
    ValueKind::Bool,
    |a: VirFloat, b: VirFloat| Ok(a.value <= b.value)
);

register_op_ge!(
    VirInt,
    VirInt,
    ValueKind::Bool,
    |a: VirInt, b: VirInt| Ok(a.value >= b.value)
);
register_op_ge!(
    VirInt,
    VirFloat,
    ValueKind::Bool,
    |a: VirInt, b: VirFloat| Ok(a.value as f64 >= b.value)
);
register_op_ge!(
    VirFloat,
    VirInt,
    ValueKind::Bool,
    |a: VirFloat, b: VirInt| Ok(a.value >= b.value as f64)
);
register_op_ge!(
    VirFloat,
    VirFloat,
    ValueKind::Bool,
    |a: VirFloat, b: VirFloat| Ok(a.value >= b.value)
);

register_op_lt!(
    VirInt,
    VirInt,
    ValueKind::Bool,
    |a: VirInt, b: VirInt| Ok(a.value < b.value)
);
register_op_lt!(
    VirInt,
    VirFloat,
    ValueKind::Bool,
    |a: VirInt, b: VirFloat| Ok((a.value as f64) < b.value)
);
register_op_lt!(
    VirFloat,
    VirInt,
    ValueKind::Bool,
    |a: VirFloat, b: VirInt| Ok(a.value < b.value as f64)
);
register_op_lt!(
    VirFloat,
    VirFloat,
    ValueKind::Bool,
    |a: VirFloat, b: VirFloat| Ok(a.value < b.value)
);

register_op_gt!(
    VirInt,
    VirInt,
    ValueKind::Bool,
    |a: VirInt, b: VirInt| Ok(a.value > b.value)
);
register_op_gt!(
    VirInt,
    VirFloat,
    ValueKind::Bool,
    |a: VirInt, b: VirFloat| Ok(a.value as f64 > b.value)
);
register_op_gt!(
    VirFloat,
    VirInt,
    ValueKind::Bool,
    |a: VirFloat, b: VirInt| Ok(a.value > b.value as f64)
);
register_op_gt!(
    VirFloat,
    VirFloat,
    ValueKind::Bool,
    |a: VirFloat, b: VirFloat| Ok(a.value > b.value)
);

register_op_ne!(
    VirInt,
    VirInt,
    ValueKind::Bool,
    |a: VirInt, b: VirInt| Ok(a.value != b.value)
);
register_op_ne!(
    VirInt,
    VirFloat,
    ValueKind::Bool,
    |a: VirInt, b: VirFloat| Ok(a.value as f64 != b.value)
);
register_op_ne!(
    VirFloat,
    VirInt,
    ValueKind::Bool,
    |a: VirFloat, b: VirInt| Ok(a.value != b.value as f64)
);
register_op_ne!(
    VirFloat,
    VirFloat,
    ValueKind::Bool,
    |a: VirFloat, b: VirFloat| Ok(a.value != b.value)
);

register_op_bsl!(
    VirInt,
    VirInt,
    ValueKind::Int,
    |a: VirInt, b: VirInt| Ok(VirInt::new(a.value << b.value))
);
register_op_bsr!(
    VirInt,
    VirInt,
    ValueKind::Int,
    |a: VirInt, b: VirInt| Ok(VirInt::new(a.value >> b.value))
);
register_op_band!(
    VirInt,
    VirInt,
    ValueKind::Int,
    |a: VirInt, b: VirInt| Ok(VirInt::new(a.value & b.value))
);
register_op_bor!(
    VirInt,
    VirInt,
    ValueKind::Int,
    |a: VirInt, b: VirInt| Ok(VirInt::new(a.value | b.value))
);
register_op_and!(bool, bool, ValueKind::Bool, |a: bool, b: bool| Ok(a && b));
register_op_or!(bool, bool, ValueKind::Bool, |a: bool, b: bool| Ok(a || b));
register_op_bxor!(bool, bool, ValueKind::Bool, |a: bool, b: bool| Ok(a ^ b));
// register_op_bsl!(VirFloat, VirInt, ValueKind::Float, |a: VirFloat, b: VirInt| Ok(VirFloat::new(a.value << b.value)));
// register_op_bsr!(VirFloat, VirInt, ValueKind::Float, |a: VirFloat, b: VirInt| Ok(VirFloat::new(a.value >> b.value)));
// Best attempt would be using i64::pow, but overflow would be a big issue

register_op_not!(bool, ValueKind::Bool, |a: bool| Ok(!a));
register_op_pos!(VirInt, ValueKind::Int, |a: VirInt| Ok(VirInt::new(
    a.value
)));
register_op_pos!(VirFloat, ValueKind::Float, |a: VirFloat| Ok(
    VirFloat::new(a.value)
));
register_op_neg!(VirInt, ValueKind::Int, |a: VirInt| Ok(VirInt::new(
    -a.value
)));
register_op_neg!(VirFloat, ValueKind::Float, |a: VirFloat| Ok(
    VirFloat::new(-a.value)
));

register_op_add!(
    String,
    String,
    ValueKind::String,
    |a: String, b: String| Ok(format!("{}{}", a, b).to_string())
);
