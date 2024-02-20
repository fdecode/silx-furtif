use std::pin::Pin;

use rkyv::{ 
    util::{ archived_value, archived_value_mut, },
    Archive, Serialize, ser::{ Serializer, serializers::AllocSerializer, }, AlignedVec, 
};
use crate::NALLOC;
use super::archannel::{ SerializedDataQuerySender as BinQySender, SerializedDataOneshot as Oneshot, SerializedDataOneshotReceiver as OReceiver, };


#[derive(Debug, Clone, Copy,)]
/// Enum defining serialized data type
pub (crate) enum SerializedDataType { Sized, Undefined, }

#[derive(Debug, Clone,)]
/// Serialized data; the serialization is done by rkyv
pub struct SerializedData {
    pub(crate) data_type: SerializedDataType,
    pub(crate) root: u32,
    pub(crate) data: AlignedVec,    
}

impl SerializedData {
    #[inline]
    pub (crate) fn undefined() -> Self { Self { data_type: SerializedDataType::Undefined, root: 0, data: AlignedVec::new(), } }

    #[inline]
    pub (crate) fn is_undefined(&self) -> bool { match self.data_type { SerializedDataType::Undefined => true, _ => false, }  }

    #[inline]
    pub (crate) fn sized_from(root: u32, data: AlignedVec,) -> Self { Self { data_type: SerializedDataType::Sized, root, data, } }

    #[inline]
    pub (crate) fn new_sized<U>(u: &U) -> Result<Self,String> where U: Archive + Serialize<AllocSerializer<NALLOC>> {
        let (root, data) = {
            let mut serial = AllocSerializer::<NALLOC>::default();
            match serial.serialize_value(u) { 
                Err(e) => return Err(format!("failed to serialize: {}", e)), Ok(root) => (root,serial.into_serializer().into_inner()), 
            }
        };
        let root = match TryFrom::try_from(root) { Err(e) => return Err(format!("out of bounds: {}", e)), Ok(root) => root, };
        Ok(Self::sized_from(root, data,)) 
    }

    #[inline]
    pub (crate) fn archive_ref<U>(&self) -> Result<&<U as Archive>::Archived,String> where U: Archive {
        match self.data_type {
            SerializedDataType::Sized => Ok(unsafe { archived_value::<U>(&self.data, self.root as usize) }),
            SerializedDataType::Undefined => Err(format!("Command is invalid for undefined data")),
        }
    }

    #[inline]
    pub (crate) fn archive_ref_mut<U>(self: Pin<&mut Self>) -> Result<Pin<&mut <U as Archive>::Archived>,String> where U: Archive {
        match self.data_type {
            SerializedDataType::Sized => Ok({
                let root = self.root as usize;
                unsafe { archived_value_mut::<U>(self.map_unchecked_mut(|s| {
                    let tab: &mut[u8] = &mut s.data; tab
                }), root) }
            }),
            SerializedDataType::Undefined => Err(format!("Command is invalid for undefined data")),
        }
    }

    #[inline]
    pub (crate) async fn send(self, sender: &BinQySender) -> Result<OReceiver,String> { // send a query from this self
        let (osender,oreceiver) = Oneshot::channel();
        match sender.send((self,osender)).await {
            Ok(()) => Ok(oreceiver), Err(_) => Err(format!("MsgFromMaster: failed to send message")),
        }
    }
}
