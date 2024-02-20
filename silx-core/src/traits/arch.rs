use crate::{ shared::utils::ArchData, utils::SlxData, NALLOC }; 
use rkyv::{ Serialize, ser::serializers::AllocSerializer, };

/// A trait helper for building a sized archive from a slx data
pub trait ArchSized {
    /// Type of the slx data
    type Archivable: SlxData + Serialize<AllocSerializer<NALLOC>>;

    /// Build the archive data from a slx data reference
    /// * Output: the archive data or an error
    fn arch_sized(&self) -> Result<ArchData<Self::Archivable>,String>;
}

impl<U> ArchSized for U where U: SlxData + Serialize<AllocSerializer<NALLOC>> {
    type Archivable = U;
    #[inline] fn arch_sized(&self) -> Result<ArchData<Self::Archivable>,String> { 
        ArchData::new_sized(self) 
    }
}