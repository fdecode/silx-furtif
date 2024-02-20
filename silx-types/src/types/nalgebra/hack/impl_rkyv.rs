use super::{ ConstSlx, ArrayStorageSlx, };
use rkyv::{ out_field, Archive, Archived, Fallible, Serialize, Deserialize, };

//////////////////////////////////
// Implementation for ConstSlx
impl<const T: usize> Archive for ConstSlx<T> {
    type Archived = Self;
    type Resolver = ();
    #[inline] unsafe fn resolve(&self, _: usize, _: Self::Resolver, out: *mut Self::Archived) { out.write(*self); }
}

impl<S: Fallible + ?Sized, const T: usize> Serialize<S> for ConstSlx<T> {
    #[inline] fn serialize(&self, _: &mut S) -> Result<Self::Resolver, S::Error> { Ok(()) }
}

impl<D: Fallible + ?Sized, const T: usize> Deserialize<ConstSlx<T>, D> for Archived<ConstSlx<T>> {
    #[inline] fn deserialize(&self, _: &mut D) -> Result<ConstSlx<T>, D::Error> { Ok(*self) }
}

///////////////////////////////////////
// Implementation for ArrayStorageSlx

impl<T, const R: usize, const C: usize> Archive for ArrayStorageSlx<T,R,C> where T: Archive {
    type Archived = ArrayStorageSlx<Archived<T>,R,C>;
    type Resolver = <[[T; R]; C] as Archive>::Resolver;
    #[inline] unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        let (fp, fo) = out_field!(out.0);
        self.0.resolve(pos + fp, resolver, fo);
    }
}

impl<S: Fallible + ?Sized, T, const R: usize, const C: usize> Serialize<S> for ArrayStorageSlx<T,R,C> where T: Archive, [[T; R]; C]: Serialize<S> {
    #[inline] fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> { self.0.serialize(serializer) }
}

impl<D: Fallible + ?Sized, T, const R: usize, const C: usize> Deserialize<ArrayStorageSlx<T,R,C>, D> 
                    for Archived<ArrayStorageSlx::<T,R,C>> where T: Archive, [[Archived<T>; R]; C]: Deserialize<[[T; R]; C], D>, {
    #[inline] fn deserialize(&self, deserializer: &mut D) -> Result<ArrayStorageSlx<T,R,C>, D::Error> {
        Ok(ArrayStorageSlx(self.0.deserialize(deserializer)?))
    }
}