use std::pin::Pin;
use hashed_type_def::HashedTypeDef;
use silx_core::types::{DerefArch, DerefMutArch};
use super::{ 
    ArchData, ConstSlx, DerefMatrixSlx, ArrayStorageSlx,
    ArchRefArrayStorageSlx, ArchMutArrayStorageSlx, 
};
// implement `DerefArch` for `DerefMatrixSlx<ArchRefArrayStorageSlx>` on silx array storages
// as a consequence is implemented `ArchToRef` for reference to archived silx array storages
impl<'a, T, const R: usize, const C: usize> DerefArch<'a, ArrayStorageSlx<T,R,C>> 
            for DerefMatrixSlx<T,ConstSlx<R>,ConstSlx<C>,ArchRefArrayStorageSlx<'a,T,R,C>>  where T: 'a + rkyv::Archive<Archived = T> + HashedTypeDef {
    #[inline] fn deref_arch(arch:  &'a ArchData<ArrayStorageSlx<T,R,C>>) -> Result<Self,String> {
        Ok(From::from(arch.archive_ref()?))
    }
}

// implement `DerefMutArch` for `DerefMatrixSlx<ArchMutArrayStorageSlx>` on silx array storages
// as a consequence is implemented `ArchToMut` for pinned mutable reference to archived silx array storages
impl<'a, T, const R: usize, const C: usize> DerefMutArch<'a, ArrayStorageSlx<T,R,C>> 
            for DerefMatrixSlx<T,ConstSlx<R>,ConstSlx<C>,ArchMutArrayStorageSlx<'a,T,R,C>> where T: 'a + Unpin + rkyv::Archive<Archived = T> + HashedTypeDef {
    #[inline] fn deref_mut_arch(arch:  Pin<&'a mut ArchData<ArrayStorageSlx<T,R,C>>>) -> Result<Pin<Self>,String> {
        Ok(From::from(arch.archive_mut()?)) 
    }
}
