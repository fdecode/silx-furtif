#[cfg(not(feature = "default"))] #[doc(hidden)]
pub const _DEFAULT: bool = false;
#[cfg(feature = "default")] #[doc(hidden)]
pub const _DEFAULT: bool = true;

#[cfg(not(feature = "verbose1"))] #[doc(hidden)]
pub const _VERBOSE1: bool = false;
#[cfg(feature = "verbose1")] #[doc(hidden)]
pub const _VERBOSE1: bool = true;

#[cfg(not(feature = "verbose2"))] #[doc(hidden)]
pub const _VERBOSE2: bool = false;
#[cfg(feature = "verbose2")] #[doc(hidden)]
pub const _VERBOSE2: bool = true;

#[cfg(not(feature = "verbose3"))] #[doc(hidden)]
pub const _VERBOSE3: bool = false;
#[cfg(feature = "verbose3")] #[doc(hidden)]
pub const _VERBOSE3: bool = true;

#[cfg(not(feature = "verbose4"))] #[doc(hidden)]
pub const _VERBOSE4: bool = false;
#[cfg(feature = "verbose4")] #[doc(hidden)]
pub const _VERBOSE4: bool = true;

#[cfg(not(feature = "hashed-type-def-procmacro"))] #[doc(hidden)]
pub const _HASHED_TYPE_DEF_PROCMACRO: bool = false;
#[cfg(feature = "hashed-type-def-procmacro")] #[doc(hidden)]
pub const _HASHED_TYPE_DEF_PROCMACRO: bool = true;

#[cfg(not(feature = "impl_nalgebra_sparse"))] #[doc(hidden)]
pub const _IMPL_NALGEBRA_SPARSE: bool = false;
#[cfg(feature = "impl_nalgebra_sparse")] #[doc(hidden)]
pub const _IMPL_NALGEBRA_SPARSE: bool = true;

#[cfg(not(feature = "impl_nalgebra"))] #[doc(hidden)]
pub const _IMPL_NALGEBRA: bool = false;
#[cfg(feature = "impl_nalgebra")] #[doc(hidden)]
pub const _IMPL_NALGEBRA: bool = true;

#[cfg(not(feature = "impl_rend"))] #[doc(hidden)]
pub const _IMPL_REND: bool = false;
#[cfg(feature = "impl_rend")] #[doc(hidden)]
pub const _IMPL_REND: bool = true;

#[cfg(not(feature = "derive"))] #[doc(hidden)]
pub const _DERIVE: bool = false;
#[cfg(feature = "derive")] #[doc(hidden)]
pub const _DERIVE: bool = true;
