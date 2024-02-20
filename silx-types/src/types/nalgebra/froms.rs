use std::{ pin::Pin, ops::Deref, marker::PhantomData, };
use rkyv::{Archive, Archived, };
use nalgebra::{ Matrix, Const, };
use super::{ 
    ConstSlx, DerefMatrixSlx, ArrayStorageSlx, ArchRefArrayStorageSlx, ArchMutArrayStorageSlx,
};

// implementing from for ArchRefArrayStorageSlx
impl<'a, T, const R: usize, const C: usize> From<&'a Archived<ArrayStorageSlx<T,R,C>>> for ArchRefArrayStorageSlx<'a, T,R,C> where T: Archive {
    fn from(archived: &'a Archived<ArrayStorageSlx<T,R,C>>) -> Self { Self::new(archived) }
}

// implementing froms for ArchMutArrayStorageSlx
impl<'a, T, const R: usize, const C: usize> From<&'a mut Archived<ArrayStorageSlx<T,R,C>>> for ArchMutArrayStorageSlx<'a,T,R,C> where T: Archive {
    fn from(archived: &'a mut Archived<ArrayStorageSlx<T,R,C>>) -> Self { Self::new(archived) }
}

// implementing froms for Matrix
impl<T,const R: usize,const C:usize> From<&ArrayStorageSlx<Archived<T>,R,C>>
            for &Matrix<T,Const<R>,Const<C>,ArrayStorageSlx<Archived<T>,R,C>> where T: Archive {
    fn from(data: &ArrayStorageSlx<Archived<T>,R,C>) -> Self { 
        unsafe{ std::mem::transmute(data) } 
    }
}
impl<T,const R: usize,const C:usize> From<Pin<&mut ArrayStorageSlx<Archived<T>,R,C>>>
            for Pin<&mut Matrix<T,Const<R>,Const<C>,ArrayStorageSlx<Archived<T>,R,C>>> where T: Archive {
    fn from(data: Pin<&mut ArrayStorageSlx<Archived<T>,R,C>>) -> Self { 
        unsafe{ std::mem::transmute(data) } 
    }
}

// implementing froms for Matrix deref type
impl<'a, T,const R: usize,const C: usize> From<&'a Archived<ArrayStorageSlx<T,R,C>>> 
        for DerefMatrixSlx<T,ConstSlx<R>,ConstSlx<C>,ArchRefArrayStorageSlx<'a, T,R,C>> where T: Archive {
    fn from(archived: &'a Archived<ArrayStorageSlx<T,R,C>>) -> Self { 
        let data = ArchRefArrayStorageSlx::new(archived); Self { data, _phantoms: PhantomData } 
    }
}
// implementing froms for Matrix derefmut type
impl<'a, T,const R: usize,const C: usize> From<&'a mut Archived<ArrayStorageSlx<T,R,C>>>
        for DerefMatrixSlx<T,ConstSlx<R>,ConstSlx<C>,ArchMutArrayStorageSlx<'a,T,R,C>> where T: Archive {
    fn from(archived: &'a mut Archived<ArrayStorageSlx<T,R,C>>) -> Self { 
        let data = ArchMutArrayStorageSlx::new(archived); Self { data, _phantoms: PhantomData }
    }
}
// implementing froms for pinned Matrix derefmut type
impl<'a, T,const R: usize,const C: usize> From<Pin<&'a mut Archived<ArrayStorageSlx<T,R,C>>>>
        for Pin<DerefMatrixSlx<T,ConstSlx<R>,ConstSlx<C>,ArchMutArrayStorageSlx<'a,T,R,C>>> 
        where T: Archive + Unpin, DerefMatrixSlx<T,ConstSlx<R>,ConstSlx<C>,ArchMutArrayStorageSlx<'a,T,R,C>>: Deref, 
              <DerefMatrixSlx<T,ConstSlx<R>,ConstSlx<C>,ArchMutArrayStorageSlx<'a,T,R,C>> as Deref>::Target: Unpin, Archived<ArrayStorageSlx<T,R,C>>: Unpin, {
    fn from(archived: Pin<&'a mut Archived<ArrayStorageSlx<T,R,C>>>) -> Self {
        let archived = archived.get_mut();
        let mat = From::from(archived);
        Pin::new(mat)
    }
}

