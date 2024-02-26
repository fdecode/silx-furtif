#![allow(non_camel_case_types)]

#[cfg(feature = "use_nalgebra")]
use nalgebra::{ Field, RealField, ComplexField, };
#[cfg(feature = "use_nalgebra")]
use simba::{
    scalar::SubsetOf, simd::{ SimdValue, PrimitiveSimdValue, },
};
use hashed_type_def::HashedTypeDef;

use approx::{ RelativeEq, UlpsEq, AbsDiffEq, };
use serde::{ Serialize as SerdeSerialize, Deserialize as SerdeDeserialize, };
#[cfg(feature = "be_silx")]
use rend::BigEndian;
#[cfg(not(feature = "be_silx"))]
use rend::LittleEndian;
use std::{ pin::Pin, fmt, fmt::{ Display, Debug, }, };
pub use std::{
    ops::{
        Add, AddAssign, Div, DivAssign, 
        Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
        Not, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Shl, ShlAssign, Shr, ShrAssign,
    },
    iter::{Sum, Product,},
    num::FpCategory, hash::{ Hash, Hasher, },
}; 
pub use num_traits::{ 
    Num, Bounded, Float, FloatConst, AsPrimitive, FromPrimitive, NumCast, ToPrimitive, One, Zero, PrimInt,
    CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedRem, CheckedShl, CheckedShr, CheckedSub, CheckedEuclid, Euclid,
    Inv, MulAdd, MulAddAssign, Saturating, SaturatingAdd, SaturatingMul, SaturatingSub, WrappingAdd, WrappingMul, WrappingNeg, 
    WrappingShl, WrappingShr, WrappingSub, Pow, Signed, Unsigned,
    one, zero, checked_pow, pow, abs, abs_sub, signum, bounds, cast, float, identities, int, ops, real, sign,
};

use silx_core::{ types::{DerefArch, DerefMutArch}, utils::ArchData, };
use super::{ 
    SlxFrom, SlxInto, macros::{ 
        impl_num_char_type, impl_inc_int_type, impl_inc_num_type, impl_inc_signed_num_type, impl_inc_int_char_type, impl_inc_float_type,
    },
};

// definitions of u8slx, i8slx, u16slx, u32slx, u64slx, u128slx, i16slx, i32slx, i64slx, i128slx, f32slx, f64slx, char_slx
pub type u8slx = u8;
pub type i8slx = i8;
impl_num_char_type! { // and first implementations
    u16slx-u16, u32slx-u32, u64slx-u64, u128slx-u128, i16slx-i16, i32slx-i32, i64slx-i64, i128slx-i128, f32slx-f32, f64slx-f64, char_slx-char,
}

impl_inc_int_type! { // implementations of operators specific to integer types
    u16slx-u16slx-u16slx, 
    u32slx-u32slx-u32slx, 
    u64slx-u64slx-u64slx, 
    u128slx-u128slx-u128slx, 
    i16slx-i16slx-i16slx, 
    i32slx-i32slx-i32slx, 
    i64slx-i64slx-i64slx, 
    i128slx-i128slx-i128slx, 
}

impl_inc_num_type! { // implementations specific to numeric types
    u16slx-u16, u32slx-u32, u64slx-u64, u128slx-u128, i16slx-i16, i32slx-i32, i64slx-i64, i128slx-i128, f32slx-f32, f64slx-f64, 
}

impl_inc_signed_num_type! { // implementations specific to signed types
    i16slx-i16, i32slx-i32, i64slx-i64, i128slx-i128, f32slx-f32, f64slx-f64, 
}

impl_inc_int_char_type! { // implementations specific to integer and char types
    u16slx-u16, u32slx-u32, u64slx-u64, u128slx-u128, 
    i16slx-i16, i32slx-i32, i64slx-i64, i128slx-i128, char_slx-char, 
}

impl_inc_float_type! { // implementations specific to float types
    f32slx-f32, f64slx-f64,
}