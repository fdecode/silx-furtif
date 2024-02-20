use rkyv::{Archive, Deserialize, Serialize};
use hashed_type_def::HashedTypeDef;
use std::fmt::Debug;

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug))]
#[derive(HashedTypeDef)]
/// Awakening message
pub struct WakeSlx;
