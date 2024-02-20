/// Macro: type hash implementation for primitives
#[macro_export]
macro_rules! proc_hashed_type_def_primitive {
    ($($ty: ident,)*) => { $(add_hashed_type_def!( struct $ty { primitive: ()});)* };
}

/// Macro: type hash implementation for structures, enum, or unions
#[macro_export]
macro_rules! multi_hashed_type_def {
    ($($line: stmt;)*) => { $(add_hashed_type_def!($line);)* };
}

/// Macro: type hash implementation for tuples
#[macro_export]
macro_rules! impl_hashed_type_def_tuple {
    (($($T: ident,)*)) => {
        impl <$($T: HashedTypeDef),*> HashedTypeDef for ($($T,)*) {
            const TYPE_HASH_NATIVE: u128 =  {
                #[allow(unused_mut)] let mut hash = start_hash_fnv1a(b"<tuple>");
                $(
                    hash = add_hash_fnv1a(& $T::TYPE_HASH_NATIVE.to_le_bytes(),hash);
                )*                
                hash
            };
        }
    };
}

/// Macro: type hash implementation for functions
#[macro_export]
macro_rules! impl_hashed_type_def_fn {
    (fn ($($T: ident,)*) -> $R: ident) => {
        impl <$($T: HashedTypeDef,)* $R: HashedTypeDef> HashedTypeDef for fn($($T,)*) -> $R {
            const TYPE_HASH_NATIVE: u128 =  {
                #[allow(unused_mut)] let mut hash = start_hash_fnv1a(b"<fn>");
                $(
                    hash = add_hash_fnv1a(& $T::TYPE_HASH_NATIVE.to_le_bytes(),hash);
                )*                
                hash = add_hash_fnv1a(& $R::TYPE_HASH_NATIVE.to_le_bytes(),hash);
                hash
            };
        }
    };
}
