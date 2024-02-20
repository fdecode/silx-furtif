use std::{ pin::Pin, ops::{ Deref, DerefMut, }, };

use crate::utils::{ArchData, SlxData};


/// Convert slx archive to dereferencing
/// * `D` : type of dereferencing
pub trait ArchToDeref<D> where D: Deref{
    /// Convert slx archive to dereferencing
    /// * Output: dereferencing or error
    fn arch_deref(self) -> Result<D,String>;
}

/// Convert slx archive to mutable dereferencing 
/// * `D` : type of mutable dereferencing
pub trait ArchToDerefMut<D> where D: Deref + DerefMut {
    /// Convert slx archive to pinned mutable dereferencing
    /// * Output: pinned mutable dereferencing or error
    fn arch_deref_mut(self) -> Result<Pin<D>,String>;
}

/// Convert slx archive to dereferencing
/// * This trait is generally used for implementing both `DerefArch` and `ArchToRef`
/// * `T` : type of slx data
pub trait DerefArch<'a,T>: Sized + Deref where T: SlxData {
    /// Convert slx archive to dereferencing
    /// * `arch:  &'a ArchData<T>` : reference to slx archive
    /// * Output: dereferencing or error
    fn deref_arch(arch:  &'a ArchData<T>) -> Result<Self,String>;
}

/// Convert slx archive to mutable dereferencing
/// * This trait is generally used for implementing both `DerefMutArch` and `ArchToMut`
/// * `T` : type of slx data
pub trait DerefMutArch<'a,T>: Sized + Deref + DerefMut where T: SlxData {
    /// Convert slx archive to pinned mutable dereferencing
    /// * `arch:  Pin<&'a mut ArchData<T>>` : pinned mutable reference to slx archive
    /// * Output: pinned mutable dereferencing or error
    fn deref_mut_arch(arch:  Pin<&'a mut ArchData<T>>) -> Result<Pin<Self>,String>;
}

impl<'a,T,D> ArchToDeref<D> for &'a ArchData<T> where T: SlxData, D: DerefArch<'a,T> {
    fn arch_deref(self) -> Result<D,String> {
        D::deref_arch(self)
    }
}

impl<'a,D,T> ArchToDerefMut<D> for Pin<&'a mut ArchData<T>> where T: SlxData, D: DerefMutArch<'a,T> {
    fn arch_deref_mut(self) -> Result<Pin<D>,String> {
        D::deref_mut_arch(self)
    }
}

