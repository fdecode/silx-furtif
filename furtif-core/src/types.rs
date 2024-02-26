#[cfg(not(feature = "silx-types"))]
#[allow(non_camel_case_types)]
/// Definition of `f64slx` as `silx_types::f64slx` or as `f64` if `silx-type` is disabled
pub type f64slx = f64;
#[cfg(feature = "silx-types")]
#[allow(non_camel_case_types)]
/// Definition of `f64slx` as `silx_types::f64slx` or as `f64` if `silx-type` is disabled
pub type f64slx = silx_types::f64slx;

#[cfg(not(feature = "silx-types"))] 
#[allow(non_camel_case_types)]
/// Definition of `u128slx` as `silx_types::u128slx` or as `u128` if `silx-type` is disabled
pub type u128slx = u128;
#[cfg(feature = "silx-types")]
#[allow(non_camel_case_types)]
/// Definition of `u128slx` as `silx_types::u128slx` or as `u128` if `silx-type` is disabled
pub type u128slx = silx_types::u128slx;

#[cfg(not(feature = "silx-types"))] 
#[allow(non_camel_case_types)]
/// Definition of `u32slx` as `silx_types::u32slx` or as `u32` if `silx-type` is disabled
pub type u32slx = u32;
#[cfg(feature = "silx-types")]
#[allow(non_camel_case_types)]
/// Definition of `u32slx` as `silx_types::u32slx` or as `u32` if `silx-type` is disabled
pub type u32slx = silx_types::u32slx;

/// Casting method from silx data to native data
pub trait SlxInto<O> {
    /// Cast from slx data; identity if `silx-type` is disabled 
    fn unslx(self) -> O;
} 
/// Casting method from native data to silx data
pub trait IntoSlx<T> {
    /// Cast to slx data; identity if `silx-type` is disabled 
    fn slx(self) -> T;
}
impl SlxInto<f64> for f64slx {
    #[cfg(not(feature = "silx-types"))]
    #[inline] fn unslx(self) -> f64 { self }
    #[cfg(feature = "silx-types")]
    #[inline] fn unslx(self) -> f64 { silx_types::SlxInto::unslx(self) }
}
impl SlxInto<u128> for u128slx {
    #[cfg(not(feature = "silx-types"))]
    #[inline] fn unslx(self) -> u128 { self }
    #[cfg(feature = "silx-types")]
    #[inline] fn unslx(self) -> u128 { silx_types::SlxInto::unslx(self) }
}
impl SlxInto<u32> for u32slx {
    #[cfg(not(feature = "silx-types"))]
    #[inline] fn unslx(self) -> u32 { self }
    #[cfg(feature = "silx-types")]
    #[inline] fn unslx(self) -> u32 { silx_types::SlxInto::unslx(self) }
}
impl IntoSlx<f64slx> for f64 {
    #[cfg(not(feature = "silx-types"))]
    #[inline] fn slx(self) -> f64slx { self }
    #[cfg(feature = "silx-types")]
    #[inline] fn slx(self) -> f64slx { silx_types::IntoSlx::slx(self) }
}
impl IntoSlx<u128slx> for u128 {
    #[cfg(not(feature = "silx-types"))]
    #[inline] fn slx(self) -> u128slx { self }
    #[cfg(feature = "silx-types")]
    #[inline] fn slx(self) -> u128slx { silx_types::IntoSlx::slx(self) }
}
impl IntoSlx<u32slx> for u32 {
    #[cfg(not(feature = "silx-types"))]
    #[inline] fn slx(self) -> u32slx { self }
    #[cfg(feature = "silx-types")]
    #[inline] fn slx(self) -> u32slx { silx_types::IntoSlx::slx(self) }
}
