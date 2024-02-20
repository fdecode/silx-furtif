use crate::multi_hashed_type_def;
use hashed_type_def_procmacro::{ add_hashed_type_def, HashedTypeDef, };
use super::HashedTypeDef;

/// tag for HashedTypeDef implementation of nalgebra types
#[derive(HashedTypeDef)]
enum NAlgebra {}

use nalgebra::{
    // traits (necessary for the signatures)
    Dim, DimName, DimMin, DimMinimum, DimSub, DimDiff, DimNameAdd, DimNameSum, RawStorage, RawStorageMut,
    Scalar, RealField, ComplexField, SimdComplexField, geometry::TCategory, allocator::Allocator,
    // type (necessary for the signatures)
    U1,
    // Struct
    Complex,
    base::{
        ArrayStorage, EuclideanNorm, LpNorm, Matrix, UniformNorm, Unit, VecStorage, ViewStorage, ViewStorageMut, 
        constraint::ShapeConstraint, 
        coordinates::{
            IJKW, M2x2, M2x3, M2x4, M2x5, M2x6, M3x2, M3x3, M3x4, M3x5, M3x6, M4x2, M4x3, M4x4, M4x5, M4x6, M5x2, 
            M5x3, M5x4, M5x5, M5x6, M6x2, M6x3, M6x4, M6x5, M6x6, X, XY, XYZ, XYZW, XYZWA, XYZWAB
        }, default_allocator::DefaultAllocator, dimension::{Const, Dyn}, 
        iter::{
            ColumnIter, ColumnIterMut, MatrixIter, MatrixIterMut, RowIter, RowIterMut
        }, uninit::{Init, Uninit}
    },
    geometry::{
        DualQuaternion, Isometry, OPoint, Orthographic3, Perspective3, Quaternion, Reflection, Rotation, 
        Scale, Similarity, Transform, Translation,
    },
    linalg::{
        Bidiagonal, Cholesky, ColPivQR, FullPivLU, Hessenberg, LU, PermutationSequence, QR, SVD, Schur, 
        SymmetricEigen, SymmetricTridiagonal, UDU, givens::GivensRotation,
    },
    // Enum
    geometry::{ TAffine, TGeneral, TProjective },
};

// implementation of HashedTypeDef for nalgebra structs and enums
multi_hashed_type_def! {
    // Struct
    struct Complex<T,> { nalgebra: NAlgebra, };
    struct ArrayStorage<T, const R: usize, const C: usize,> { nalgebra_base: NAlgebra, };
    struct EuclideanNorm { nalgebra_base: NAlgebra, };
    struct LpNorm { nalgebra_base: NAlgebra, };
    struct Matrix<T, R, C, S,> { nalgebra_base: NAlgebra, };
    struct UniformNorm { nalgebra_base: NAlgebra, };
    struct Unit<T,> { nalgebra_base: NAlgebra, };
    struct VecStorage<T, R: Dim, C: Dim,> { nalgebra_base: NAlgebra, };
    struct ViewStorage<'a, T, R: Dim, C: Dim, RStride: Dim, CStride: Dim,> { nalgebra_base: NAlgebra, };
    struct ViewStorageMut<'a, T, R: Dim, C: Dim, RStride: Dim, CStride: Dim,> { nalgebra_base: NAlgebra, };
    struct ShapeConstraint { nalgebra_base_constraint: NAlgebra, };
    struct IJKW<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M2x2<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M2x3<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M2x4<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M2x5<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M2x6<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M3x2<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M3x3<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M3x4<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M3x5<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M3x6<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M4x2<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M4x3<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M4x4<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M4x5<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M4x6<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M5x2<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M5x3<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M5x4<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M5x5<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M5x6<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M6x2<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M6x3<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M6x4<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M6x5<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct M6x6<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct X<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct XY<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct XYZ<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct XYZW<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct XYZWA<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct XYZWAB<T: Scalar,> { nalgebra_base_coordinates: NAlgebra, };
    struct DefaultAllocator { nalgebra_base_default__allocator: NAlgebra, };
    struct Const<const R: usize,> { nalgebra_base_dimension: NAlgebra, };
    struct Dyn { nalgebra_base_dimension: NAlgebra, };
    struct ColumnIter<'a, T, R: Dim, C: Dim, S: RawStorage<T, R, C,>,> { nalgebra_base_iter: NAlgebra, };
    struct ColumnIterMut<'a, T, R: Dim, C: Dim, S: RawStorageMut<T, R, C,>,> { nalgebra_base_iter: NAlgebra, };
    struct MatrixIter<'a, T, R: Dim, C: Dim, S: 'a + RawStorage<T, R, C,>,> { nalgebra_base_iter: NAlgebra, };
    struct MatrixIterMut<'a, T, R: Dim, C: Dim, S: 'a + RawStorageMut<T, R, C,>,> { nalgebra_base_iter: NAlgebra, };
    struct RowIter<'a, T, R: Dim, C: Dim, S: RawStorage<T, R, C,>,> { nalgebra_base_iter: NAlgebra, };
    struct RowIterMut<'a, T, R: Dim, C: Dim, S: RawStorageMut<T, R, C,>,> { nalgebra_base_iter: NAlgebra, };
    struct Init { nalgebra_base_uninit: NAlgebra, };
    struct Uninit { nalgebra_base_uninit: NAlgebra, };
    struct DualQuaternion<T,> { nalgebra_geometry: NAlgebra, };
    struct Isometry<T, R, const D: usize,> { nalgebra_geometry: NAlgebra, };
    struct Orthographic3<T,> { nalgebra_geometry: NAlgebra, };
    struct Perspective3<T,> { nalgebra_geometry: NAlgebra, };
    struct Quaternion<T,> { nalgebra_geometry: NAlgebra, };
    struct Reflection<T, D, S,> { nalgebra_geometry: NAlgebra, };
    struct Rotation<T, const D: usize,> { nalgebra_geometry: NAlgebra, };
    struct Scale<T, const D: usize,> { nalgebra_geometry: NAlgebra, };
    struct Similarity<T, R, const D: usize,> { nalgebra_geometry: NAlgebra, };
    struct Translation<T, const D: usize,> { nalgebra_geometry: NAlgebra, };
    struct GivensRotation<T: ComplexField,> { nalgebra_linalg_givens: NAlgebra, };
    struct OPoint<T: Scalar, D: DimName> where DefaultAllocator: Allocator<T, D> {
        nalgebra_geometry: NAlgebra,
    };
    struct Transform<T: RealField, C: TCategory, const D: usize> where Const<D>: DimNameAdd<U1>,
                    DefaultAllocator: Allocator<T, DimNameSum<Const<D>, U1>, DimNameSum<Const<D>, U1>>, {
        nalgebra_geometry: NAlgebra,
    };
    struct Bidiagonal<T: ComplexField, R: DimMin<C>, C: Dim> where DimMinimum<R, C>: DimSub<U1>,
            DefaultAllocator: Allocator<T, R, C> + Allocator<T, DimMinimum<R, C>> + Allocator<T, 
            DimDiff<DimMinimum<R, C>, U1>>, {
        nalgebra_linalg: NAlgebra,
    };
    pub struct Cholesky<T: SimdComplexField, D: Dim> where DefaultAllocator: Allocator<T, D, D>, {
        nalgebra_linalg: NAlgebra,
    };
    pub struct ColPivQR<T: ComplexField, R: DimMin<C>, C: Dim> where 
            DefaultAllocator: Allocator<T, R, C> + Allocator<T, DimMinimum<R, C>>
                + Allocator<(usize, usize), DimMinimum<R, C>>, {
        nalgebra_linalg: NAlgebra,
    };
    pub struct FullPivLU<T: ComplexField, R: DimMin<C>, C: Dim> where
            DefaultAllocator: Allocator<T, R, C> + Allocator<(usize, usize), DimMinimum<R, C>>, {
        nalgebra_linalg: NAlgebra,
    };
    pub struct Hessenberg<T: ComplexField, D: DimSub<U1>> where
            DefaultAllocator: Allocator<T, D, D> + Allocator<T, DimDiff<D, U1>>, {
        nalgebra_linalg: NAlgebra,
    };
    pub struct LU<T: ComplexField, R: DimMin<C>, C: Dim> where
            DefaultAllocator: Allocator<T, R, C> + Allocator<(usize, usize), DimMinimum<R, C>>, {
        nalgebra_linalg: NAlgebra,
    };
    pub struct PermutationSequence<D: Dim> where DefaultAllocator: Allocator<(usize, usize), D>, {
        nalgebra_linalg: NAlgebra,
    };
    pub struct QR<T: ComplexField, R: DimMin<C>, C: Dim> where
            DefaultAllocator: Allocator<T, R, C> + Allocator<T, DimMinimum<R, C>>, {
        nalgebra_linalg: NAlgebra,
    };
    pub struct SVD<T: ComplexField, R: DimMin<C>, C: Dim> where
            DefaultAllocator: Allocator<T, DimMinimum<R, C>, C> + Allocator<T, R, DimMinimum<R, C>> 
                + Allocator<T::RealField, DimMinimum<R, C>>, {
        nalgebra_linalg: NAlgebra,
    };
    pub struct Schur<T: ComplexField, D: Dim> where DefaultAllocator: Allocator<T, D, D>, {
        nalgebra_linalg: NAlgebra,
    };
    pub struct SymmetricEigen<T: ComplexField, D: Dim> where
            DefaultAllocator: Allocator<T, D, D> + Allocator<T::RealField, D>, {
        nalgebra_linalg: NAlgebra,
    };
    pub struct SymmetricTridiagonal<T: ComplexField, D: DimSub<U1>> where
            DefaultAllocator: Allocator<T, D, D> + Allocator<T, DimDiff<D, U1>>, {
        nalgebra_linalg: NAlgebra,
    };
    pub struct UDU<T: RealField, D: Dim> where DefaultAllocator: Allocator<T, D> + Allocator<T, D, D>, {
        nalgebra_linalg: NAlgebra,
    };
    // Enum
    enum TAffine { NalgebraGeometry(NAlgebra) };
    enum TGeneral { NalgebraGeometry(NAlgebra) };
    enum TProjective { NalgebraGeometry(NAlgebra) };
}

#[cfg(feature = "impl_nalgebra_sparse")]
use nalgebra::{
    // trait (necessary for the signatures)
    sparse::CsStorage,
    // Struct
    sparse::{CsCholesky, CsMatrix, CsVecStorage},
};

#[cfg(feature = "impl_nalgebra_sparse")]
// implementation of HashedTypeDef for nalgebra sparse structs
multi_hashed_type_def! {
    // Struct
    struct CsCholesky<T, D> where T: RealField, D: Dim,
            DefaultAllocator: Allocator<usize, D> + Allocator<T, D>, { 
        nalgebra_sparse: NAlgebra, 
    };
    struct CsMatrix<T, R, C, S> where T: Scalar, R: Dim, C: Dim, S: CsStorage<T, R, C>, { 
        nalgebra_sparse: NAlgebra, 
    };
    struct CsVecStorage<T, R, C> where T: Scalar, R: Dim, C: Dim, DefaultAllocator: Allocator<usize, C>, { 
        nalgebra_sparse: NAlgebra, 
    };
}
