use hashed_type_def::HashedTypeDef;
use nalgebra::{ ArrayStorage, Const, SMatrix, };
use rkyv::Archive;
use super::{ ArrayStorageSlx, ConstSlx, SlxInto, };


///////////////////////////////////////////////////////
// Implementations of SlxInto into Const for ConstSlx
impl<const R: usize> SlxInto<Const<R>> for ConstSlx<R> {
    #[inline] fn unslx(self) -> Const<R> { Const::<R> }
}

/////////////////////////////////////////////////////////////////////
// Implementations of SlxInto into ArrayStorage for ArrayStorageSlx
// 
impl<T, const R: usize, const C: usize> SlxInto<ArrayStorage<T, R, C,>> for ArrayStorageSlx<T,R,C> where T: Archive + HashedTypeDef {
    #[inline] fn unslx(self) -> ArrayStorage<T, R, C,> { ArrayStorage { 0: self.0 } } // Damned! Label 0 does work!
}

////////////////////////////////////////////////////////////////
// Implementations of SlxInto into SMatrix for ArrayStorageSlx
// 
impl<T, const R: usize, const C: usize> SlxInto<SMatrix<T, R, C,>> for ArrayStorageSlx<T,R,C> where T: Archive + HashedTypeDef {
    #[inline] fn unslx(self) -> SMatrix<T, R, C,> { SMatrix::from_array_storage(self.unslx()) } // Damned! Label 0 does work!
}

