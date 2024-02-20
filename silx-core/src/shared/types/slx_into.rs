use crate::shared::utils::SlxData;

/// Into trait for converting slx data into original data
/// * This trait is generally used for implementing both `SlxInto` and `FromSlx`
/// * `O` : type of original data
pub trait SlxInto<O>: SlxData {
    /// Convert slx data into original data
    /// * Output: original data
    fn unslx(self) -> O;
}

/// From trait for converting slx data into original data
/// * `T` : type of slx data
pub trait FromSlx<T> where T: SlxData {
    /// Convert slx data into original data
    /// * `slx: T` : slx data
    /// * Output: original data
    fn from_slx(slx: T) -> Self;
}

impl<T,O> FromSlx<T> for O where T: SlxInto<O> {
    #[inline] fn from_slx(slx: T) -> Self { slx.unslx() }
}

