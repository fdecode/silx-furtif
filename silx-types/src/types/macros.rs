/// macro for implementation of traits specific to numbers and character
macro_rules! impl_num_char_type {
    (rkyv;$type:ty) => { // macro option implementing rkyv traits for slx primitives
        // implement `rkyv::Archive` for silx data
        impl rkyv::Archive for $type {
            type Archived = Self;
            type Resolver = ();
            #[inline] unsafe fn resolve(&self, _: usize, _: Self::Resolver, out: *mut Self::Archived) {
                out.write(*self);
            }
        }

        // implement `rkyv::Serialize` for silx data
        impl<S: rkyv::Fallible + ?Sized> rkyv::Serialize<S> for $type {
            #[inline] fn serialize(&self, _: &mut S) -> Result<Self::Resolver, S::Error> {
                Ok(())
            }
        }

        // implement `rkyv::Deserialize` for silx data
        impl<D: rkyv::Fallible + ?Sized> rkyv::Deserialize<$type, D> for rkyv::Archived<$type> {
            #[inline] fn deserialize(&self, _: &mut D) -> Result<$type, D::Error> {
                Ok(*self)
            }
        }
    };
    ($($I:ident-$T:ident,)*) => ($(
        #[cfg(not(feature = "be_silx"))]
        #[repr(transparent)]
        #[derive(Clone, Copy,)]
        #[derive(HashedTypeDef)]
        #[doc = "Silx primitive related to std primitive "]
        pub struct $I(LittleEndian<$T>);

        #[cfg(feature = "be_silx")]
        #[repr(transparent)]
        #[derive(Clone, Copy,)]
        #[derive(HashedTypeDef)]
        #[doc = "Silx primitive related to std primitive "]
        pub struct $I(BigEndian<$T>);

        // implement rkyv traits
        impl_num_char_type!{rkyv; $I}

        // specific implementation  of `$I`
        impl $I {
            /// Get native value of data
            #[inline] fn inner(self) -> $T { self.0.value() }

            #[cfg(not(feature = "be_silx"))]
            /// Create silx data from native value
            #[inline] pub fn new(t:$T) -> $I { Self(LittleEndian::<$T>::new(t)) }

            #[cfg(feature = "be_silx")]
            /// Create silx data from native value
            #[inline] pub fn new(t:$T) -> $I { Self(BigEndian::<$T>::new(t)) }
        }

        // implement IntoSlx for native value
        impl SlxFrom<$T> for $I {
            #[inline] fn slx_from(orig: $T) -> Self { Self::new(orig) }
        }
        
        // implement SlxInto for silx data
        impl SlxInto<$T> for $I {
            #[inline] fn unslx(self) -> $T { self.inner() }
        }

        // implement From for native value
        impl From<$T> for $I {
            #[inline] fn from(orig: $T) -> Self { Self::new(orig) }
        }
        
        // implement From for native value
        impl From<$I> for $T {
            #[inline] fn from(orig: $I) -> Self { orig.inner() }
        }
        
        // implement `DerefArch` on silx array storages
        // as a consequence is implemented `ArchToRef` for reference to archived silx data
        impl<'a> DerefArch<'a, $I> for &'a $I {
            #[inline] fn deref_arch(arch:  &'a ArchData<$I>) -> Result<Self,String> {
                let arc = arch.archive_ref()?;
                Ok(unsafe { std::mem::transmute(&arc.0) })
            }
        }

        // implement `DerefMutArch` on silx array storages
        // as a consequence is implemented `ArchToMut` for pinned mutable reference to archived silx data
        impl<'a> DerefMutArch<'a, $I> for &'a mut $I {
            #[inline] fn deref_mut_arch(arch:  Pin<&'a mut ArchData<$I>>) -> Result<Pin<Self>,String> {
                let arc = arch.archive_mut()?;
                Ok(unsafe { arc.map_unchecked_mut(|s| std::mem::transmute(&mut s.0)) } ) 
            }
        }

        // implement Debug for silx data
        impl Debug for $I {
            #[inline] fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                Debug::fmt(&self.0,f)
            }
        }

        // implement Display for silx data
        impl Display  for $I {
            #[inline] fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                Display::fmt(&self.0,f)
            }
        }
        
        // implement partial equality for silx data
        impl PartialEq for $I {
            #[inline] fn eq(&self, other: &$I) -> bool { self.0.eq(&other.0) }
        }

        // implement serde Serialize for silx data
        impl SerdeSerialize for $I {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
                self.inner().serialize(serializer)
            }            
        } 

        // implement serde Deserialize for silx data
        impl<'de> SerdeDeserialize<'de> for $I {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
                let inner = $T::deserialize(deserializer)?;
                Ok($I::new(inner))
            }
        } 
    )*)
}

pub(crate) use impl_num_char_type;

/// macro for incremental implementation of traits specific to integers
macro_rules! impl_inc_int_type {(
    $($I:ident-$R:ident-$O:ident,)*
) => (
    $(
        // implementation of bitwise Not
        impl Not for $I {
            type Output = $O;
            fn not(self) -> Self::Output {
                Self::Output::new(self.inner().not())
            }
        }
        // implementation of bitwise And
        impl BitAnd<$R> for $I {
            type Output = $O;
            fn bitand(self, rhs: $R) -> Self::Output {
                Self::Output::new(self.inner().bitand(rhs.inner()))
            }
        }
        // implementation of bitwise Or
        impl BitOr<$R> for $I {
            type Output = $O;
            fn bitor(self, rhs: $R) -> Self::Output {
                Self::Output::new(self.inner().bitor(rhs.inner()))
            }
        }
        // implementation of bitwise exclusive Or
        impl BitXor<$R> for $I {
            type Output = $O;
            fn bitxor(self, rhs: $R) -> Self::Output {
                Self::Output::new(self.inner().bitxor(rhs.inner()))
            }
        }
        // implementation of bits shift left
        impl Shl<$R> for $I {
            type Output = $O;
            fn shl(self, rhs: $R) -> Self::Output {
                Self::Output::new(self.inner().shl(rhs.inner()))
            }
        }
        // implementation of bits shift right
        impl Shr<$R> for $I {
            type Output = $O;
            fn shr(self, rhs: $R) -> Self::Output {
                Self::Output::new(self.inner().shr(rhs.inner()))
            }
        }
        // implementation of bitwise And with self assignment
        impl BitAndAssign<$R> for $I {
            fn bitand_assign(&mut self, rhs: $R) {
                *self = (*self).bitand(rhs)
            }
        }
        // implementation of bitwise Or with self assignment
        impl BitOrAssign<$R> for $I {
            fn bitor_assign(&mut self, rhs: $R) {
                *self = (*self).bitor(rhs)
            }
        }
        // implementation of bitwise exclusive Or with self assignment
        impl BitXorAssign<$R> for $I {
            fn bitxor_assign(&mut self, rhs: $R) {
                *self = (*self).bitxor(rhs)
            }
        }
        // implementation of bits shift left with self assignment
        impl ShlAssign<$R> for $I {
            fn shl_assign(&mut self, rhs: $R) {
                *self = (*self).shl(rhs)
            }
        }
        // implementation of bits shift right with self assignment
        impl ShrAssign<$R> for $I {
            fn shr_assign(&mut self, rhs: $R) {
                *self = (*self).shr(rhs)
            }
        }
    )*
)}

pub(crate) use impl_inc_int_type;

/// macro for incremental implementation of traits specific to numbers
macro_rules! impl_inc_num_type {
    // macro for implementing ToPrimitive methods
    (toprimitive;$($F:ident->$T:ident,)*) => ($(
        #[inline] fn $F(&self) -> Option<$T> { self.inner().$F() }
    )*);
    // macro for implementing FromPrimitive methods
    (fromprimitive;$($F:ident<-$T:ident,)*) => ($(
        #[inline] fn $F(n:$T) -> Option<Self> { FromPrimitive::$F(n).map(Self::new) }
    )*);
    // macro for implementting ToPrimitive and FromPrimitive
    (primitive;$I:ident) => (
        // implement ToPrimitive for silx data
        impl ToPrimitive for $I { 
            impl_inc_num_type!{toprimitive;
                to_i64->i64,to_isize->isize,to_i8->i8,to_i16->i16,to_i32->i32,
                to_u64->u64,to_usize->usize,to_u8->u8,to_u16->u16,to_u32->u32,
                to_f64->f64,to_f32->f32,
            }
        }
        // implement FromPrimitive for silx data
        impl FromPrimitive for $I {
            impl_inc_num_type!{fromprimitive;
                from_i64<-i64,from_isize<-isize,from_i8<-i8,from_i16<-i16,from_i32<-i32,
                from_u64<-u64,from_usize<-usize,from_u8<-u8,from_u16<-u16,from_u32<-u32,
                from_f64<-f64,from_f32<-f32,
            }
        }
    );
    ($($I:ident-$T:ident,)*) => ($( // for each pair of idents, related to silx data and primitive respectively,
        // implement NumCast for silx data
        impl NumCast for $I { 
            #[inline] fn from<T: ToPrimitive>(n: T) -> Option<Self> { (NumCast::from(n)).map(Self::new) }
        }
        // implement Zero for silx data        
        impl Zero for $I {
            #[inline] fn zero() -> Self { Self::new(Zero::zero()) }
            #[inline] fn is_zero(&self) -> bool { self.inner().is_zero() }
        }
        // implement One for silx data        
        impl One for $I {
            #[inline] fn one() -> Self { Self::new(One::one()) }
        }
        // implement Add for silx data        
        impl Add for $I {
            type Output = Self;
            #[inline] fn add(self, other: Self) -> Self { Self::new(self.inner() + other.inner()) }
        }
        // implement Sub for silx data
        impl Sub for $I {
            type Output = Self;        
            #[inline] fn sub(self, other: Self) -> Self { Self::new(self.inner() - other.inner()) }
        }
        // implement Mul for silx data        
        impl Mul for $I {
            type Output = Self;
            #[inline] fn mul(self, other: Self) -> Self { Self::new(self.inner() * other.inner()) }
        }
        // implement Div for silx data
        impl Div for $I {
            type Output = Self;
            #[inline] fn div(self, other: Self) -> Self { Self::new(self.inner() / other.inner()) }
        }
        // implement Rem for silx data        
        impl Rem for $I {
            type Output = Self;        
            #[inline] fn rem(self, other: Self) -> Self { Self::new(self.inner() % other.inner()) }
        }
        // implement AddAssign for silx data        
        impl AddAssign for $I {
            #[inline] fn add_assign(&mut self, rhs: Self) { *self = *self + rhs }
        }
        // implement DivAssign for silx data
        impl DivAssign for $I {
            #[inline] fn div_assign(&mut self, rhs: Self) { *self = *self / rhs }
        }
        // implement MulAssign for silx data
        impl MulAssign for $I {
            #[inline] fn mul_assign(&mut self, rhs: Self) { *self = *self * rhs }
        }
        // implement RemAssign for silx data
        impl RemAssign for $I {
            #[inline] fn rem_assign(&mut self, rhs: Self) { *self = *self % rhs }
        }
        // implement SubAssign for silx data
        impl SubAssign for $I {
            #[inline] fn sub_assign(&mut self, rhs: Self) { *self = *self - rhs }
        }
        // implement Num for silx data
        impl Num for $I {
            type FromStrRadixErr = <$T as Num>::FromStrRadixErr;
            #[inline] fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> { 
                Ok(Self::new(Num::from_str_radix(str,radix)?))
            }
        }
        // implement Sum for silx data
        impl Sum<$I> for $I {
            #[inline] fn sum<I>(iter: I) -> Self where I: Iterator<Item = Self> { iter.fold(Zero::zero(), |x,y| x+y) } 
        }
        // implement Product for silx data        
        impl Product<$I> for $I {
            #[inline] fn product<I>(iter: I) -> Self where I: Iterator<Item = Self> { iter.fold(One::one(), |x,y| x*y) } 
        }
        // implement ToPrimitive and FromPrimitive for silx data
        impl_inc_num_type!{primitive; $I }
        // implement AsPrimitive for silx data
        impl<T> AsPrimitive<T> for $I where T: 'static + Copy, $T: AsPrimitive<T> {
            #[inline] fn as_(self) -> T { self.inner().as_() }
        }
    )*)
}

pub(crate) use impl_inc_num_type;

/// macro for incremental implementation of traits specific to signed numbers
macro_rules! impl_inc_signed_num_type {(
    $($I:ident-$T:ident,)*
) => (
    $(
        // implement Neg for silx data
        impl Neg for $I {
            type Output = Self;
            #[inline] fn neg(self) -> Self { Self::new(-self.inner()) }
        }

        // implement Signed for silx data
        impl Signed for $I {
            fn abs(&self) -> Self { Self::new(Signed::abs(&self.inner())) }
            fn abs_sub(&self, other: &Self) -> Self { Self::new(Signed::abs_sub(&self.inner(),&other.inner())) }
            fn signum(&self) -> Self { Self::new(Signed::signum(&self.inner())) }
            fn is_positive(&self) -> bool { Signed::is_positive(&self.inner()) }
            fn is_negative(&self) -> bool { Signed::is_negative(&self.inner()) }
        }
    )*
)}

pub(crate) use impl_inc_signed_num_type;

/// macro for incremental implementation of traits specific to integers and character
macro_rules! impl_inc_int_char_type {( $($I:ident-$P:ident,)* ) => (
    $(
        // implement Ord for silx data
        impl Ord for $I {
            #[inline] fn cmp(&self, other: &$I) -> std::cmp::Ordering { self.inner().cmp(&other.inner()) }
        }
        // implement PartialOrd for silx data
        impl PartialOrd for $I {
            #[inline] fn partial_cmp(&self, other: &$I) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
        }
        // implement Eq for silx data
        impl Eq for $I { }
        // implement Hash for silx data
        impl Hash for $I {
            fn hash<H>(&self, state: &mut H) where H: Hasher {
                self.0.hash(state)
            }
        }
    )*
)}

pub(crate) use impl_inc_int_char_type;

/// macro for incremental implementation of traits specific to floats
macro_rules! impl_inc_float_type {
    (func_0_to_self;$($F:ident,)*) => ( // Map Float 0-native->native function to 0-silx->silx
        $(
            #[inline] fn $F() -> Self { Self::new( Float::$F() ) }
        )*
    );
    (func_1_to_self;$($F:ident,)*) => ( // Map Float 1-native->native function to 1-silx->silx
        $(
            #[inline] fn $F(self) -> Self { Self::new( Float::$F(self.inner()) ) }
        )*
    );
    (func_1_to_bool;$($F:ident,)*) => ( // Map Float 1-native->bool function to 1-silx->bool
        $(
            #[inline] fn $F(self) -> bool { Float::$F(self.inner()) }
        )*
    );
    (func_2_to_self;$($F:ident,)*) => ( // Map Float 2-native->native function to 2-silx->silx
        $(
            #[inline] fn $F(self,other: Self) -> Self { Self::new( Float::$F(self.inner(),other.inner()) ) }
        )*
    );
    (rf_func_0_to_self;$($F:ident,)*) => ( // Map RealField 0-native->native function to 0-silx->silx
        $(
            #[inline] fn $F() -> Self { Self::new( RealField::$F() ) }
        )*
    );
    (rf_func_0_to_oself;$($F:ident,)*) => ( // Map RealField 0-native->Option<native> function to 0-silx->Option<silx>
        $(
            #[inline] fn $F() -> Option<Self> { RealField::$F().map(Self::new) }
        )*
    );
    (rf_func_1_to_bool;$($F:ident,)*) => ( // Map RealField 1-native->bool function to 1-silx->bool
        $(
            #[inline] fn $F(&self) -> bool { RealField::$F(&self.inner()) }
        )*
    );
    (rf_func_2_to_self;$($F:ident,)*) => ( // Map RealField 2-native->native function to 2-silx->silx
        $(
            #[inline] fn $F(self,other: Self) -> Self { Self::new( RealField::$F(self.inner(),other.inner()) ) }
        )*
    );   
    (cf_func_1_to_self;$($F:ident,)*) => ( // Map ComplexField 1-native->native function to 1-silx->silx
        $(
            #[inline] fn $F(self) -> Self { Self::new(ComplexField::$F(self.inner())) }
        )*
    );
    (cf_func_2_to_self;$($F:ident,)*) => ( // Map ComplexField 2-native->native function to 2-silx->silx
        $(
            #[inline] fn $F(self,other: Self) -> Self { Self::new(ComplexField::$F(self.inner(),other.inner()) ) }
        )*
    );
    ($($I:ident-$T:ident,)*) => ($(
        // implement AbsDiffEq for silx data
        impl AbsDiffEq for $I {
            type Epsilon = Self;
            #[inline] fn default_epsilon() -> Self::Epsilon { Self::new(<$T as AbsDiffEq>::default_epsilon()) }
            #[inline] fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
                AbsDiffEq::abs_diff_eq(&self.inner(),&other.inner(),epsilon.inner())
            }
        }
        // implement RelativeEq for silx data
        impl RelativeEq for $I {
            #[inline] fn default_max_relative() -> Self::Epsilon { Self::new(<$T as RelativeEq>::default_max_relative()) }
            #[inline] fn relative_eq(&self, other: &Self, epsilon: Self::Epsilon,  max_relative: Self::Epsilon) -> bool {
                RelativeEq::relative_eq(&self.inner(), &other.inner(), epsilon.inner(),  max_relative.inner())
            }
        }
        // implement UlpsEq for silx data
        impl UlpsEq for $I {
            #[inline] fn default_max_ulps() -> u32 { <$T as UlpsEq>::default_max_ulps() }
            #[inline] fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
                UlpsEq::ulps_eq(&self.inner(), &other.inner(), epsilon.inner(), max_ulps)
            }
        }
        // implement SubsetOf of silx data for silx data
        #[cfg(feature = "use_nalgebra")]
        impl SubsetOf<$I> for $I {
            #[inline] fn to_superset(&self) -> $I { *self }
            #[inline] fn from_superset_unchecked(element: &$I) -> $I { *element }
            #[inline] fn is_in_subset(_: &$I) -> bool { true }
        }
        // implement SubsetOf of silx data for f64
        #[cfg(feature = "use_nalgebra")]
        impl SubsetOf<$I> for f64 {
            #[inline] fn to_superset(&self) -> $I { $I::new((*self) as $T) }
            #[inline] fn from_superset_unchecked(element: &$I) -> f64 { element.inner() as f64 }
            #[inline] fn is_in_subset(_: &$I) -> bool { true }
        }
        // implement PrimitiveSimdValue for silx data
        #[cfg(feature = "use_nalgebra")]
        impl PrimitiveSimdValue for $I { }
        // implement SimdValue for silx data
        #[cfg(feature = "use_nalgebra")]
        impl SimdValue for $I {
            type Element = $I;
            type SimdBool = bool;
            #[inline(always)] fn lanes() -> usize { 1 }
            #[inline(always)] fn splat(val: Self::Element) -> Self { val }
            #[inline(always)] fn extract(&self, _: usize) -> Self::Element { *self }
            #[inline(always)]  unsafe fn extract_unchecked(&self, _: usize) -> Self::Element { *self }
            #[inline(always)] fn replace(&mut self, _: usize, val: Self::Element) { *self = val }
            #[inline(always)] unsafe fn replace_unchecked(&mut self, _: usize, val: Self::Element) { *self = val }
            #[inline(always)] fn select(self, cond: Self::SimdBool, other: Self) -> Self { if cond { self } else { other } }
        }
        // implement Field for silx data
        #[cfg(feature = "use_nalgebra")]
        impl Field for $I { }
        // implement ComplexField for silx data
        #[cfg(feature = "use_nalgebra")]
        impl ComplexField for $I {
            type RealField = $I;
            // implement methods of type 1-silx->silx
            impl_inc_float_type!{cf_func_1_to_self;
                real, imaginary, modulus, modulus_squared, argument, norm1, floor, ceil, round, trunc, fract, abs, recip, conjugate, sin, 
                cos, tan, asin, acos, atan, sinh, cosh, tanh, asinh, acosh, atanh, log2, log10, ln, ln_1p, sqrt, exp, exp2, exp_m1, cbrt,
            }
            // implement methods of type 2-silx->silx
            impl_inc_float_type!{cf_func_2_to_self; powf, powc, log, scale, unscale, hypot, }

            #[inline] fn from_real(re: Self::RealField) -> Self { Self::new(ComplexField::from_real(re.inner())) }
            #[inline] fn powi(self, n: i32) -> Self { Self::new(ComplexField::powi(self.inner(), n)) }
            #[inline] fn sin_cos(self) -> (Self, Self) { let (s,c) = ComplexField::sin_cos(self.inner()); (Self::new(s),Self::new(c)) }
            #[inline] fn mul_add(self, a: Self, b: Self) -> Self { Self::new(ComplexField::mul_add(self.inner(), a.inner(), b.inner())) }
            #[inline] fn is_finite(&self) -> bool { ComplexField::is_finite(&self.inner()) }
            #[inline] fn try_sqrt(self) -> Option<Self> { ComplexField::try_sqrt(self.inner()).map(Self::new) }        
        }
        // implement RealField for silx data
        #[cfg(feature = "use_nalgebra")]
        impl RealField for $I {
            // implement methods of type 1-silx->silx
            impl_inc_float_type!{rf_func_1_to_bool; is_sign_positive, is_sign_negative, }
            // implement methods of type 2-silx->silx
            impl_inc_float_type!{rf_func_2_to_self; copysign, max, min, atan2, }
            // implement methods of type 0-silx->Option<silx>
            impl_inc_float_type!{rf_func_0_to_oself; min_value, max_value, }
            // implement methods of type 0-silx->silx
            impl_inc_float_type!{rf_func_0_to_self; 
                pi, two_pi, frac_pi_2, frac_pi_3, frac_pi_4, frac_pi_6, frac_pi_8, frac_1_pi, frac_2_pi, frac_2_sqrt_pi, 
                e, log2_e, log10_e, ln_2, ln_10, 
            }
            fn clamp(self, min: Self, max: Self) -> Self { Self::new(RealField::clamp(self.inner(),min.inner(),max.inner())) }
        }
        // implement PartialOrd for silx data
        impl PartialOrd for $I {
            #[inline] fn partial_cmp(&self, other: &$I) -> Option<std::cmp::Ordering> {  self.inner().partial_cmp(&other.inner()) }
        }
        // implement Float for silx data
        impl Float for $I {
            // implement methods of type 0-silx->silx
            impl_inc_float_type!{func_0_to_self;
                nan, infinity, neg_infinity, neg_zero, min_value, min_positive_value, max_value, epsilon,
            }
            // implement methods of type 1-silx->bool
            impl_inc_float_type!{func_1_to_bool;
                is_nan, is_infinite, is_finite, is_normal, is_sign_positive, is_sign_negative,
            }
            // implement methods of type 1-silx->silx
            impl_inc_float_type!{func_1_to_self;
                floor, ceil, round, trunc, fract, abs, signum, recip, sqrt, exp, exp2, ln, log2, log10, cbrt, sin, cos, tan,
                asin, acos, atan, exp_m1, ln_1p, sinh, cosh, tanh, asinh, acosh, atanh, to_degrees, to_radians,
            }
            // implement methods of type 2-silx->silx
            impl_inc_float_type!{func_2_to_self;
                atan2, powf, log, max, min, abs_sub, hypot,
            }
            #[inline] fn classify(self) -> FpCategory { Float::classify(self.inner()) }
            #[inline] fn mul_add(self, a: Self, b: Self) -> Self { Self::new( Float::mul_add(self.inner(),a.inner(),b.inner()) ) }
            #[inline] fn powi(self, n: i32) -> Self { Self::new( Float::powi(self.inner(),n) ) }
            #[inline] fn sin_cos(self) -> (Self, Self) { let (s,c) = Float::sin_cos(self.inner()); (Self::new(s),Self::new(c)) }
            #[inline] fn integer_decode(self) -> (u64,i16,i8) { Float::integer_decode(self.inner()) }
        }
    )*);
}       

pub(crate) use impl_inc_float_type;

