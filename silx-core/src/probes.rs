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
