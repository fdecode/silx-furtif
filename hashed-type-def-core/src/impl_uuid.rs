use crate::multi_hashed_type_def;
use hashed_type_def_procmacro::{ add_hashed_type_def, HashedTypeDef, };
use super::HashedTypeDef;

/// tag for HashedTypeDef implementation of uuid types
#[derive(HashedTypeDef)]
enum UUID {}

use uuid::{
    // Structs
    Builder, Error, Uuid,
    fmt::{ Braced, Hyphenated, Simple, Urn, },
    timestamp::{ Timestamp, context::{ 
//        Context, // Not implemented: features 'v1' & 'v6' are not considered
        NoContext, 
    }, },
    // Enums
    Variant, Version,
};

// implementation of HashedTypeDef for uuid structs
multi_hashed_type_def! {
    // Struct
    struct Builder { uuid: UUID, };
    struct Error { uuid: UUID, };
    struct Uuid { uuid: UUID, };
    struct Braced { uuid_fmt: UUID, };
    struct Hyphenated { uuid_fmt: UUID, };
    struct Simple { uuid_fmt: UUID, };
    struct Urn { uuid_fmt: UUID, };
    struct Timestamp { uuid_timestamp: UUID, };
 //    // Context is not implemented: features 'v1' & 'v6' are not considered
 //    struct Context { uuid_timestamp_context: UUID, };
    struct NoContext { uuid_timestamp_context: UUID, };
    // Enums
    enum Variant { Uuid(UUID) };
    enum Version { Uuid(UUID) };
}