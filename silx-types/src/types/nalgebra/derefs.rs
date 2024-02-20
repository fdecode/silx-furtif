use std::ops::{ Deref, DerefMut, };
use nalgebra::{ Matrix, Const, };
use super::{ ConstSlx, DerefMatrixSlx, ArchRefArrayStorageSlx, ArchMutArrayStorageSlx, };

// Implement `Deref` from `MatrixSlx` to Matrix given storage `ArchRefArrayStorageSlx`
impl<'a, T, const R: usize, const C: usize> Deref for DerefMatrixSlx<T,ConstSlx<R>,ConstSlx<C>,ArchRefArrayStorageSlx<'a, T,R,C>>
        where T: rkyv::Archive {
    type Target = Matrix<T,Const<R>,Const<C>,ArchRefArrayStorageSlx<'a, T,R,C>>;
    fn deref(&self) -> &Self::Target {
        unsafe{ std::mem::transmute(self) } 
    }
}
// Implement `Deref` from `MatrixSlx` to Matrix given storage `ArchMutArrayStorageSlx`
impl<'a, T, const R: usize, const C: usize> Deref for DerefMatrixSlx<T,ConstSlx<R>,ConstSlx<C>,ArchMutArrayStorageSlx<'a, T,R,C>> 
        where T: rkyv::Archive {
    type Target = Matrix<T,Const<R>,Const<C>,ArchMutArrayStorageSlx<'a, T,R,C>>;
    fn deref(&self) -> &Self::Target { unsafe{ std::mem::transmute(self) } }
}
// Implement `DerefMut` from `MatrixSlx` to Matrix given storage `ArchMutArrayStorageSlx`
impl<'a, T, const R: usize, const C: usize> DerefMut for DerefMatrixSlx<T,ConstSlx<R>,ConstSlx<C>,ArchMutArrayStorageSlx<'a, T,R,C>> 
        where T: rkyv::Archive {
    fn deref_mut(&mut self) -> &mut Self::Target { unsafe{ std::mem::transmute(self) } }
}
