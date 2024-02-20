use crate::shared::utils::SlxData;

/// From trait for converting original data into slx data
/// * This trait is generally used for implementing both `SlxFrom` and `IntoSlx`
/// * `O` : type of original data
pub trait SlxFrom<O>: SlxData {
    /// Convert original data into slx data
    /// * `orig: O` : original data
    /// * Output: slx data
    fn slx_from(orig: O) -> Self;
}

/// Into trait for converting original data into slx data
/// * `T` : type of slx data
pub trait IntoSlx<T> where T: SlxData {
    /// Convert original data into slx data
    /// * Output: slx data
    fn slx(self) -> T;
}

impl<T,O> IntoSlx<T> for O where T: SlxFrom<O> {
    #[inline] fn slx(self) -> T { SlxFrom::<O>::slx_from(self) }
}
