/// various implementations
/// * implementrations of rkyv traits for `ConstSlx` and for `ArrayStorageSlx`
/// * implementrations of `RawStorage`, `RawMutStorage`, `Storage` for `ArrayStorageSlx`
mod hack;
/// From implementations
mod froms;
/// Implement `Deref` (resp. `DerefMut`) from `ArrayStorageSlx` to Matrix given storage `ArchRefArrayStorageSlx` (resp. `ArchMutArrayStorageSlx`)
mod derefs;
/// implement `ArchToRef` and `ArchToMut` for archived silx array storages
mod arch_tools;

use crate::f64slx;

use super::{ IntoSlx, SlxInto, SlxFrom, ArchToDerefMut, ArchToDeref, };
use hashed_type_def::HashedTypeDef;
use silx_core::utils::{ ArchData, ArchSized, };

use std::{ pin::Pin, fmt::Debug, marker::PhantomData, };

use rkyv::Archived;
use nalgebra::{ matrix, Const, Dim, DimName, IsContiguous, Matrix, };

/// Silx constant dimension
#[derive(Debug, Clone, Copy, PartialEq,)]
#[derive(HashedTypeDef)]
pub struct ConstSlx<const R: usize>;

/// Silx 1-dimension
pub type U1Slx = ConstSlx<1>;

/// Silx array storage
#[derive(Debug, Clone, Copy, PartialEq,)]
#[derive(HashedTypeDef)]
#[repr(transparent)]
pub struct ArrayStorageSlx<T, const R: usize, const C: usize>(pub [[T; R]; C]);

/// array storage derived from archive reference of silx array storage; it is assumed that `T` is a simple structure which archive to itself
#[repr(transparent)]
pub struct ArchRefArrayStorageSlx<'a, T, const R: usize, const C: usize>(&'a Archived<ArrayStorageSlx<T,R,C>>) where T: 'a + rkyv::Archive;

/// mutable array storage derived from archive mutable reference of silx array storage; it is assumed that `T` is a simple structure which archive to itself
#[repr(transparent)]
pub struct ArchMutArrayStorageSlx<'a, T, const R: usize, const C: usize>(&'a mut Archived<ArrayStorageSlx<T,R,C>>) where T: 'a + rkyv::Archive;

/// Silx object for implementing Matrix deref
#[repr(C)]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize, Debug, Clone, Copy,)]
#[derive(HashedTypeDef)]
pub struct DerefMatrixSlx<T,R,C,S> where R: Dim, C: Dim, {
    data: S,
    _phantoms: PhantomData<(T, R, C)>,
}

/// Silx marker for storage
pub trait StorageSlx<R: Dim, C: Dim,> {}

// ////////////////////////
// Some implementations

// implementation of `Dim` for `ConstSlx`
unsafe impl<const T: usize> Dim for ConstSlx<T> {
    fn try_to_usize() -> Option<usize> { Some(T) }
    fn value(&self) -> usize { T }
    fn from_usize(dim: usize) -> Self { assert_eq!(dim, T); Self }
}
// implementation of `DimName` for `ConstSlx`
impl<const T: usize> DimName for ConstSlx<T> {
    const USIZE: usize = T;
    #[inline] fn name() -> Self { Self }
    #[inline] fn dim() -> usize { T }
}
impl<'a, T, const R: usize, const C: usize> ArchRefArrayStorageSlx<'a, T,R,C> where T: rkyv::Archive {
    /// Constructor for ArchRefArrayStorageSlx
    fn new(archive: &'a Archived<ArrayStorageSlx<T,R,C>>) -> Self { Self(archive) }
}
impl<'a, T, const R: usize, const C: usize> ArchMutArrayStorageSlx<'a, T,R,C> where T: rkyv::Archive {
    /// Constructor for ArchMutArrayStorageSlx
    fn new(archive: &'a mut Archived<ArrayStorageSlx<T,R,C>>) -> Self  { Self(archive) }
}

impl<T: Clone,const R: usize, const C: usize> StorageSlx<Const<R>, Const<C>> for ArrayStorageSlx<T,R,C> {}
unsafe impl<T: Clone, const R: usize, const C: usize> IsContiguous for ArrayStorageSlx<T, R, C> {}

/// Experimentations on silx matrices archives
pub fn exp_silx_matrix() {
    use hashed_type_def::HashedTypeUuid;
    // matrix to be processed
    let matrix: Matrix<f64slx,_,_,_> = matrix![
        1f64.slx(),     2f64.slx(),     3f64.slx(),     4f64.slx();
        5f64.slx(),     6f64.slx(),     7f64.slx(),     8f64.slx();
        10f64.slx(),    10f64.slx(),    11f64.slx(),    12f64.slx()
    ];
    println!("ArrayStorage:");
    // build slx storage for the matrix: 
    // * matrix storage is the only thing which is transmitted between processes
    // * matrix storage entirely characterize the SMatrix
    let arst: ArrayStorageSlx<_,3,4> = matrix.slx(); // the precision on the slx type is generally necessary
    println!("arst -> {:?}",arst);
    println!("arst::full_id() -> {}", arst.self_type_uuid());
    // build the archived slx data
    let mut arch = arst.arch_sized().expect("failed to serialize");
    { // some test for accessing the marchived matrix data by reference
      // * unlike method `unarchive()`, method `arch_ref()` does not need deserialization and is 'zero-copy'
        // get the matrix reference (actually a value which deref to the matrix) from the archive (zero-copy)
        let rmatrixslx: DerefMatrixSlx<_,_,_,_> = arch.arch_deref().expect("failure");
        // get the reference by dereferencing
        let rmatrix = & * rmatrixslx;
        println!("rmatrix -> {}", rmatrix);
        // compute matrice transpose  times matrice
        let mtm = rmatrix.transpose() * rmatrix;
        println!("mtm -> {:?}", mtm);
        println!();
    }
    { // some test for accessing the marchived matrix data by mutable reference
      // * unlike method `unarchive()`, method `arch_mut()` does not need deserialization and is 'zero-copy'
        // get the matrix mutable reference (actually a value which derefmut to the matrix) from the archive (zero-copy)
        // * rkyv makes necessary the use of pinned mutable reference
        let mut rmatrixslx: Pin<DerefMatrixSlx<_,_,_,_>> = Pin::new(&mut arch).arch_deref_mut().expect("failure");
        // get the mutable reference by dereferencing
        let rmatrix: &mut Matrix<f64slx,_,_,_> = &mut * rmatrixslx;
        // change left-top value of the matrix
        rmatrix[(0,0)] = (-1f64).slx();
        println!("rmatrix -> {}", rmatrix);
        // compute matrice transpose  times matrice
        let mtm = rmatrix.transpose() * &*rmatrix;
        println!("mtm -> {}", mtm);
        println!();
    }
}
