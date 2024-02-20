/// definition of numeric silx types
mod num; 
pub use num::{
    u8slx, u16slx, u32slx, u64slx, u128slx, i8slx, i16slx, i32slx, i64slx, i128slx, f32slx, f64slx, char_slx,
    Num, Bounded, Float, FloatConst, AsPrimitive, FromPrimitive, NumCast, ToPrimitive, One, Zero, PrimInt,
    CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedRem, CheckedShl, CheckedShr, CheckedSub, CheckedEuclid, Euclid,
    Inv, MulAdd, MulAddAssign, Saturating, SaturatingAdd, SaturatingMul, SaturatingSub, WrappingAdd, WrappingMul, WrappingNeg, 
    WrappingShl, WrappingShr, WrappingSub, Pow, Signed, Unsigned,
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign, Mul, MulAssign, Neg, Not, 
    Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign, Sum, Product, FpCategory,
    one, zero, checked_pow, pow, abs, abs_sub, signum, bounds, cast, float, identities, int, ops, real, sign,
}; // definitions and reimports
pub use silx_core::{
    types::{ WakeSlx, ArchToDerefMut, ArchToDeref, IntoSlx, SlxFrom, SlxInto, FromSlx, },
    utils::{ ArchSized, SlxData, },
}; // reimports

#[cfg(feature = "use_nalgebra")]
/// definition and implementation of silx types for nalgebra; the implementation is limited to statically sized array
pub mod nalgebra;

/// some useful macros for the crate
pub (crate) mod  macros;
