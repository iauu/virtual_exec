register_op_add!(i64, i64, i64);
register_op_add!(f64, f64, f64);
register_op_add!(i64, f64, f64, |a, b| Ok((a as f64) + b));
register_op_add!(f64, i64, f64, |a, b| Ok(a + (b as f64)));

register_op_sub!(i64, i64, i64);
register_op_sub!(f64, f64, f64);
register_op_sub!(i64, f64, f64, |a, b| Ok((a as f64) - b));
register_op_sub!(f64, i64, f64, |a, b| Ok(a - (b as f64)));

register_op_mul!(i64, i64, i64);
register_op_mul!(f64, f64, f64);
register_op_mul!(i64, f64, f64, |a, b| Ok((a as f64) * b));
register_op_mul!(f64, i64, f64, |a, b| Ok(a * (b as f64)));

register_op_div!(i64, i64, i64);
register_op_div!(f64, f64, f64);
register_op_div!(i64, f64, f64, |a, b| Ok((a as f64) / b));
register_op_div!(f64, i64, f64, |a, b| Ok(a / (b as f64)));

register_op_moduls!(i64, i64, i64);
register_op_moduls!(f64, f64, f64);
register_op_moduls!(i64, f64, f64, |a, b| Ok((a as f64) % b));
register_op_moduls!(f64, i64, f64, |a, b| Ok(a % (b as f64)));

register_op_eq!(bool, bool, bool, |a: bool, b: bool| Ok(a == b));
register_op_le!(bool, bool, bool, |a: bool, b: bool| Ok(a <= b));
register_op_lt!(bool, bool, bool, |a: bool, b: bool| Ok(!a & b));
register_op_ge!(bool, bool, bool, |a: bool, b: bool| Ok(a >= b));
register_op_gt!(bool, bool, bool, |a: bool, b: bool| Ok(a & !b));
register_op_ne!(bool, bool, bool, |a: bool, b: bool| Ok(a != b));

register_op_eq!(i64, i64, bool);
register_op_eq!(f64, f64, bool);
register_op_eq!(i64, f64, bool, |a, b| Ok(((a as f64) - b).abs() < f64::EPSILON));
register_op_eq!(f64, i64, bool, |b, a| Ok(((a as f64) - b).abs() < f64::EPSILON));
register_op_eq!((), (), bool);
register_op_eq!(bool, bool, bool);

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

register_op_ne!(i64, i64, bool);
register_op_ne!(f64, f64, bool);
register_op_ne!(i64, f64, bool, |a, b| Ok(((a as f64) - b).abs() >= f64::EPSILON));
register_op_ne!(f64, i64, bool, |b, a| Ok(((a as f64) - b).abs() >= f64::EPSILON));

register_op_bsl!(i64, i64, i64);
register_op_bsr!(i64, i64, i64);
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
register_op_neg!(i64, i64);
register_op_neg!(f64, f64);

register_op_add!(
    String,
    String,
    String,
    |a: String, b: String| Ok(format!("{}{}", a, b).to_string())
);