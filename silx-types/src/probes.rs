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

#[cfg(not(feature = "nalgebra"))] #[doc(hidden)]
pub const _NALGEBRA: bool = false;
#[cfg(feature = "nalgebra")] #[doc(hidden)]
pub const _NALGEBRA: bool = true;

#[cfg(not(feature = "use_nalgebra"))] #[doc(hidden)]
pub const _USE_NALGEBRA: bool = false;
#[cfg(feature = "use_nalgebra")] #[doc(hidden)]
pub const _USE_NALGEBRA: bool = true;

#[cfg(not(feature = "be_silx"))] #[doc(hidden)]
pub const _BE_SILX: bool = false;
#[cfg(feature = "be_silx")] #[doc(hidden)]
pub const _BE_SILX: bool = true;
