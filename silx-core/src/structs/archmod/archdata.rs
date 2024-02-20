use std::{
    marker::PhantomData,
    pin::Pin, 
};

use hashed_type_def::HashedTypeDef;
use rkyv::{ 
    Archive, Serialize, ser::serializers::AllocSerializer,  Deserialize,
};

use crate::NALLOC;
use super::ser_data::SerializedData;

/// A marker for data exchanged through silx channels
/// * This marker is automatically implemented by types implemeting `rkyv::Archive` and `HashedTypeDef`
pub trait SlxData: Archive + HashedTypeDef { }

impl<T> SlxData for T where T: Archive + HashedTypeDef { }

#[derive(Debug)]
/// Silx archive data
/// * `U`: the type of the data; needs to implement `SlxData`
pub struct ArchData<U: ?Sized> where U: SlxData {
    pub(crate) bytes: SerializedData,
    phantom: PhantomData<U>,
}

/// Pinned mutable reference to silx archive data
pub type PinArchData<'a, U> = Pin<&'a mut ArchData<U>>;

impl<U> ArchData<U> where U: SlxData {
    pub(crate) fn to_bytes(self) -> SerializedData { self.bytes }
    
    pub(super) fn from_bytes(bytes: SerializedData) -> Self { Self { bytes, phantom: PhantomData, } } // allowed only for internal use
} 
impl<U> ArchData<U> where U: SlxData {
    pub fn new_sized(data: &U) -> Result<Self, String> where U: Sized + Archive + Serialize<AllocSerializer<NALLOC>> {
        let bytes = SerializedData::new_sized(data)?;
        Ok(Self::from_bytes(bytes))
    }

    /// Get a pinned mutable reference to the archive
    /// * Output: the pinned mutable reference to the archive
    pub fn pinned(&mut self) -> PinArchData<U> where U: Unpin { Pin::new(self) }
    
    /// Get reference to the archived data within archive data (zero-copy)
    /// * Output: the reference or an error
    pub fn archive_ref(&self) -> Result<&<U as Archive>::Archived,String> where U: Archive {
        self.bytes.archive_ref::<U>()
    }

    /// Get pinned mutable reference to the archived data within archive data (zero-copy)
    /// * Output: the pinned mutable reference or an error
    pub fn archive_mut(self: Pin<&mut Self>) -> Result<Pin<&mut <U as Archive>::Archived>,String> where U: Archive {
        let bytes: Pin<&mut SerializedData> = unsafe { self.map_unchecked_mut(|s| &mut s.bytes) };
        bytes.archive_ref_mut::<U>()
    }

    /// Unarchive the archive data
    /// * Output: the data or an error
    pub fn unarchive(&self) -> Result<U,String> where U: Archive, U::Archived: Deserialize<U,rkyv::Infallible> {
        let ref_arc = self.bytes.archive_ref::<U>()?;
        Ok(match U::Archived::deserialize(ref_arc, &mut rkyv::Infallible){
             Ok(w) => w,
             Err(e) => return Err(format!("Failed to unarchive -> {e}")),
        })
    }
} 


impl<U> Clone for ArchData<U> where U: SlxData {
    fn clone(&self) -> Self { Self::from_bytes(self.bytes.clone()) }
}
