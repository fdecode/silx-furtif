use crate::multi_hashed_type_def;
use hashed_type_def_procmacro::{ add_hashed_type_def, HashedTypeDef, };
use super::HashedTypeDef;

/// tag for HashedTypeDef implementation of rend types
#[derive(HashedTypeDef)]
enum Rend {}

use rend::{ 
    // needed trait
    Primitive,
    // Structs
    BigEndian, LittleEndian, NativeEndian, 
};

// implementation of HashedTypeDef for rend structs
multi_hashed_type_def! {
    // Structs
    struct BigEndian<T: Primitive> { rend: Rend, };
    struct LittleEndian<T: Primitive> { rend: Rend, };
    struct NativeEndian<T: Primitive> { rend: Rend, };
}