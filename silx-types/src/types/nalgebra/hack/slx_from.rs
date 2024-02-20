use hashed_type_def::HashedTypeDef;
use nalgebra::{ ArrayStorage, Const, SMatrix, };
use rkyv::Archive;
use super::{ ArrayStorageSlx, ConstSlx, SlxFrom, };


///////////////////////////////////////////////////////
// Implementations of SlxFrom from Const for ConstSlx
impl<const R: usize> SlxFrom<Const<R>> for ConstSlx<R> {
    #[inline] fn slx_from(_: Const<R>) -> Self { ConstSlx::<R> }
}

////////////////////////////////////////////////////////////////////
// Implementations of SlxFrom from ArrayStorage for ArrayStorageSlx

impl<T, const R: usize, const C: usize> SlxFrom<ArrayStorage<T,R,C>> for ArrayStorageSlx<T, R, C,> where T: Archive + HashedTypeDef {
    #[inline] fn slx_from(orig: ArrayStorage<T, R, C,>) -> Self { ArrayStorageSlx { 0: orig.0 } } // Damned! Label 0 does work!
}

///////////////////////////////////////////////////////////////
// Implementations of SlxFrom from SMatrix for ArrayStorageSlx
impl<T, const R: usize, const C: usize> SlxFrom<SMatrix<T,R,C>> for ArrayStorageSlx<T, R, C,> where T: Archive + HashedTypeDef {
    fn slx_from(orig: SMatrix<T,R,C>) -> Self { SlxFrom::slx_from(orig.data) }
}
