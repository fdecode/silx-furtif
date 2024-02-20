use nalgebra::{ Owned, Storage, Scalar, RawStorage, U1, Const, allocator::Allocator, DefaultAllocator, ArrayStorage, };
use super::ArchRefArrayStorageSlx;

// implement `nalgebra::Storage` for `ArchRefArrayStorageSlx`
unsafe impl<'a, T,const R: usize, const C: usize> Storage<T,Const<R>, Const<C>> for ArchRefArrayStorageSlx<'a,T,R,C> 
        where T: Scalar + rkyv::Archive<Archived = T>, DefaultAllocator: Allocator<T, Const<R>, Const<C>, Buffer = ArrayStorage<T,R,C>> {
    #[inline] fn into_owned(self) -> Owned<T, Const<R>, Const<C>> { self.clone_owned() }
    #[inline] fn clone_owned(&self) -> Owned<T, Const<R>, Const<C>> { ArrayStorage(self.0.0.clone()) }
}

// implement `nalgebra::RawStorage`  for `ArchRefArrayStorageSlx`
unsafe impl<'a, T,const R: usize, const C: usize> RawStorage<T,Const<R>, Const<C>> for ArchRefArrayStorageSlx<'a,T,R,C> where T: rkyv::Archive<Archived = T> {
    type RStride = U1; type CStride = Const<R>;
    #[inline] fn ptr(&self) -> *const T { self.0.0.as_ptr() as *const T }
    #[inline] fn shape(&self) -> (Const<R>, Const<C>) { (Const,Const) }
    #[inline] fn strides(&self) -> (Self::RStride, Self::CStride) { (Const,Const) }
    #[inline] fn is_contiguous(&self) -> bool { true }
    #[inline] unsafe fn as_slice_unchecked(&self) -> &[T] { std::slice::from_raw_parts(self.ptr(), R * C) }
}
