//! This is part of [**Silx**](https://github.com/fdecode/silx-furtif) project    
//! 
//! `hashed-type-def-core` contains the core definition and implementations for deriving type hashcode 
//! 
//! # Content
//! The crate provides two traits `HashedTypeDef` and `HashedTypeUuid`
//! * the `HashedTypeDef` trait is the basis for defining a type's hashcode
//! * the `HashedTypeUuid` trait provides additional methods, such as UUID translation of the hashcode  
//! 
//! The regular way to implement `HashedTypeDef` is by using the procedural macro `#[derive(HashedTypeDef)]`  
//! 
//! The following example gives an overview of `hashed-type-def` features.
//! 
//! # Example of type hashcode implementation
//! ## Cargo.toml
//! ```toml
//! [package]
//! name = "silx_hashed-type-def_examples"
//! version = "0.1.1"
//! edition = "2021"
//! 
//! [dependencies]
//! uuid = "1.7.0"
//! hashed-type-def = { version = "^0.1.1", features = ["derive"] }
//! ```
//! ## main.rs
//! ```text
//! use std::fmt::Debug;
//! 
//! use hashed_type_def::{ HashedTypeDef, HashedTypeUuid, add_hashed_type_def_param, };
//! 
//! /// case 1: a named structure to be tested with procedural derivation
//! /// * procedural macro `#[derive(HashedTypeDef)]` is the regular way to implement `HashedTypeDef`
//! #[derive(Debug,HashedTypeDef)]
//! struct MyStruct<'a,'b,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'b, 'a: 'b {
//!     #[allow(dead_code)] un: &'a String,
//!     #[allow(dead_code)] deux: T,
//!     #[allow(dead_code)] trois: &'b U,
//! }
//! 
//! pub mod instance0 {
//!     use hashed_type_def::HashedTypeDef;
//!     use std::fmt::Debug;
//! 
//!     /// case 2: a structure with same definition and procedural derivation as case 1
//!     /// * type hash should be the same than case 1
//!     #[derive(Debug,HashedTypeDef)]
//!     pub struct MyStruct<'a,'b,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'b, 'a: 'b {
//!         #[allow(dead_code)] un: &'a String,
//!         #[allow(dead_code)] deux: T,
//!         #[allow(dead_code)] trois: &'b U,
//!     }
//! } 
//! 
//! pub mod instance1 {
//!     use hashed_type_def::{ add_hashed_type_def, HashedTypeDef, };
//!     use std::fmt::Debug;
//!     /// case 3: a structure with same definition as case 1 and post derivation obtained by macro `add_hashed_type_def!` processing on the same definition
//!     /// * type hash should be the same than case 1
//!     pub struct MyStruct<'a,'b,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'b, 'a: 'b {
//!         #[allow(dead_code)] un: &'a String,
//!         #[allow(dead_code)] deux: T,
//!         #[allow(dead_code)] trois: &'b U,
//!     }
//! 
//!     add_hashed_type_def!{
//!         pub struct MyStruct<'a,'b,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'b, 'a: 'b {
//!             un: &'a String,
//!             deux: T,
//!             trois: &'b U,
//!         }    
//!     }
//! } 
//! 
//! pub mod instance2 {
//!     use hashed_type_def::HashedTypeDef;
//!     use std::fmt::Debug;
//!     /// case 4: a structure with procedural derivation and same definition as case 1 except a swap on lifetime names
//!     /// * type hash should be different than case 1
//!     #[derive(Debug,HashedTypeDef)]
//!     pub struct MyStruct<'b,'a,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'a, 'b: 'a {
//!         #[allow(dead_code)] un: &'b String,
//!         #[allow(dead_code)] deux: T,
//!         #[allow(dead_code)] trois: &'a U,
//!     }
//! } 
//! 
//! pub mod instance3 {
//!     use hashed_type_def::{ add_hashed_type_def, HashedTypeDef, };
//!     use std::fmt::Debug;
//!     /// case 5: a structure with same definition as case 1 and post derivation obtained by macro `add_hashed_type_def!` processing on altered definition
//!     /// * type hash should be different than case 1
//!     pub struct MyStruct<'a,'b,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'b, 'a: 'b {
//!         #[allow(dead_code)] un: &'a String,
//!         #[allow(dead_code)] deux: T,
//!         #[allow(dead_code)] trois: &'b U,
//!     }
//! 
//!     add_hashed_type_def!{
//!         pub struct MyStruct<'a,'b,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'b, 'a: 'b {
//!             instance3_MyStruct: (),
//!             un: &'a String,
//!             deux: T,
//!             trois: &'b U,
//!         }    
//!     }
//! } 
//! 
//! pub mod instance4 {
//!     use hashed_type_def::HashedTypeDef;
//!     use std::fmt::Debug;
//!     /// case 6: a structure with procedural derivation and same definition as case 1 except conditions are moved to where clauses
//!     /// * type hash should be different than case 1
//!     #[derive(Debug,HashedTypeDef)]
//!     pub struct MyStruct<'a,'b,T,U,const C1: usize, const C2: bool> where 'a: 'b, T: Into<U> + 'b, U: Debug, {
//!         #[allow(dead_code)] un: &'a String,
//!         #[allow(dead_code)] deux: T,
//!         #[allow(dead_code)] trois: &'b U,
//!     }
//! } 
//! 
//! pub mod instance5 {
//!     use hashed_type_def::HashedTypeDef;
//!     use std::fmt::Debug;
//!     /// case 7: a structure with procedural derivation and same definition as case 1 except conditions are moved from where clauses
//!     /// * type hash should be different than case 1
//!     #[derive(Debug,HashedTypeDef)]
//!     pub struct MyStruct<'a: 'b,'b, T: Into<U> + 'b, U: Debug, const C1: usize, const C2: bool> {
//!         #[allow(dead_code)] un: &'a String,
//!         #[allow(dead_code)] deux: T,
//!         #[allow(dead_code)] trois: &'b U,
//!     }
//! } 
//! 
//! pub mod instance6 {
//!     use hashed_type_def::HashedTypeDef;
//!     use std::fmt::Debug;
//!     /// case 8: a structure with procedural derivation and same definition as case 1 except a conditions is removed from parameters list
//!     /// * type hash should be different than case 1    
//!     #[derive(Debug,HashedTypeDef)]
//!     pub struct MyStruct<'a,'b,T,U,const C1: usize, const C2: bool> where T: Into<U> + 'b, 'a: 'b {
//!         #[allow(dead_code)] un: &'a String,
//!         #[allow(dead_code)] deux: T,
//!         #[allow(dead_code)] trois: &'b U,
//!     }
//! } 
//! 
//! pub mod instance7 {
//!     use hashed_type_def::HashedTypeDef;
//!     use std::fmt::Debug;
//!     /// case 9: a structure with procedural derivation and same definition as case 1 except a condition is removed from where clauses
//!     /// * type hash should be different than case 1
//!     #[derive(Debug,HashedTypeDef)]
//!     pub struct MyStruct<'a,'b,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'b, {
//!         #[allow(dead_code)] un: &'a String,
//!         #[allow(dead_code)] deux: T,
//!         #[allow(dead_code)] trois: &'b U,
//!     }
//! } 
//! 
//! /// build type hash (Uuid format) of case 1 for specific lifetime
//! /// * type hash should not change
//! fn fn_type_hash<'a: 'b,'b>(_tmp1: &'a (), _tmp2: &'b ()) -> uuid::Uuid { MyStruct::<'a,'b,f32,f64,12,true>::type_uuid() }
//! 
//! /// case 10: a unnamed structure to be tested with procedural derivation
//! #[derive(Debug,HashedTypeDef)]
//! struct MyTupleStruct<T,U: Debug>(String,T,U,) where T: Into<U>;
//! 
//! /// case 11: a unit structure to be tested with procedural derivation
//! #[derive(Debug,HashedTypeDef)]
//! struct MyEmptyStruct;
//! 
//! /// case 12: enum to be tested with procedural derivation
//! #[derive(Debug,HashedTypeDef)]
//! enum MyEnum<T,U: Debug> where T: Into<U> {
//!     #[allow(dead_code)] Zero,
//!     #[allow(dead_code)] Un(String, T,),
//!     #[allow(dead_code)] Deux {
//!         double: f64,
//!         trois: U,
//!     },
//!     #[allow(dead_code)] Trois,
//! }
//! 
//! /// case 13: empty enum to be tested with procedural derivation
//! #[derive(Debug,HashedTypeDef)]
//! enum MyEmptyEnum { }
//! 
//! /// case 14: a struct to be tested with post derivation with parameter tag `(Option<u32>,[u128;24])`
//! struct AnotherStruct<T: Debug> where T: Default { #[allow(dead_code)] t: T }
//! 
//! add_hashed_type_def_param!((Option<u32>,[u128;24]); struct AnotherStruct<T: Debug> where T: Default { t: T });
//! 
//! /// Different cases of type hash derivation
//! fn main() {
//!     // case 1 with `true` parameter
//!     println!("MyStruct::<'static,'static,f32,f64,12,true>::type_uuid() -> {:x}", MyStruct::<'static,'static,f32,f64,12,true>::type_uuid());
//!     // case 1 with `true` parameter and different lifetime
//!     println!("MyStruct::<'a,'b,f32,f64,12,true>::type_uuid() -> {:x}", { let a = (); { let b = (); fn_type_hash(&a,&b) } });
//!     // case 1 with `false` parameter
//!     println!("MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
//!     // case 2 with `false` parameter
//!     println!("instance0::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance0::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
//!     // case 3 with `false` parameter
//!     println!("instance1::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance1::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
//!     // case 4 with `false` parameter
//!     println!("instance2::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance2::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
//!     // case 5 with `false` parameter
//!     println!("instance3::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance3::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
//!     // case 6 with `false` parameter
//!     println!("instance4::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance4::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
//!     // case 7 with `false` parameter
//!     println!("instance5::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance5::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
//!     // case 8 with `false` parameter
//!     println!("instance6::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance6::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
//!     // case 9 with `false` parameter
//!     println!("instance7::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance7::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
//!     // case 10
//!     println!("MyTupleStruct::<f32,f64>::type_uuid() -> {:x}", MyTupleStruct::<f32,f64>::type_uuid());
//!     // case 11
//!     println!("MyEmptyStruct::type_uuid() -> {:x}", MyEmptyStruct::type_uuid());
//!     // case 12
//!     println!("MyEnum::<f32,f64,>::type_uuid() -> {:x}", MyEnum::<f32,f64,>::type_uuid());
//!     // case 13
//!     println!("MyEmptyEnum::type_uuid() -> {:x}", MyEmptyEnum::type_uuid());
//!     // case 14
//!     println!("AnotherStruct::<Vec<u16>>::type_uuid() -> {:x}", AnotherStruct::<Vec<u16>>::type_uuid());
//! }
//! ```
//! ## Typical output
//! ```txt
//! MyStruct::<'static,'static,f32,f64,12,true>::type_uuid() -> bd84ac66-d0fa-5d1c-ae5e-25647a13dcb3
//! MyStruct::<'a,'b,f32,f64,12,true>::type_uuid() -> bd84ac66-d0fa-5d1c-ae5e-25647a13dcb3
//! MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> 1a8a7365-1c9c-afed-cca3-983399a91fd8
//! instance0::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> 1a8a7365-1c9c-afed-cca3-983399a91fd8
//! instance1::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> 1a8a7365-1c9c-afed-cca3-983399a91fd8
//! instance2::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> bbe982ff-fcad-5390-86f0-cce2e7dbae6b
//! instance3::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> 56d4f1b7-af31-d361-3afb-dc89e52c2ded
//! instance4::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> 394096e5-5187-edf4-ac77-3f6edc052b72
//! instance5::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> e7e26201-4095-31d1-bfa3-fd4b62abc938
//! instance6::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> 0bee6197-ef3e-a446-890a-c34705c30cdd
//! instance7::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> f8f3fcc7-e4a5-e021-e200-763b4cf9df7a
//! MyTupleStruct::<f32,f64>::type_uuid() -> ddd04a26-6807-0f27-67b2-227db8f17b75
//! MyEmptyStruct::type_uuid() -> 4ede75e3-1bf7-5298-ae87-0ee1d57a1357
//! MyEnum::<f32,f64,>::type_uuid() -> fcad4ca3-6cd0-6c7e-21ea-658a12369d9f
//! MyEmptyEnum::type_uuid() -> 9dec7519-c5f4-12b9-0509-b5b2ef1d521c
//! AnotherStruct::<Vec<u16>>::type_uuid() -> 933ae311-b69a-f600-7caa-030743cbb5e5
//! ```
//! 

use hashed_type_def_procmacro::build_hasher;

mod macros;
mod impl_basis;
mod impl_uuid;
#[cfg(feature = "impl_nalgebra")]
mod impl_nalgebra;
#[cfg(feature = "impl_rend")]
mod impl_rend;

/// Type hash code definition; derive macro for this trait is based on 128bit FNV-1a computed from the type definition
pub trait HashedTypeDef<REF = ()> {
    /// native hash computation
    const TYPE_HASH_NATIVE: u128; 
    /// hash encoded to little endianess
    const TYPE_HASH_LE: u128 = {Self::TYPE_HASH_NATIVE.to_le()}; 
    /// hash encoded to big endianess
    const TYPE_HASH_BE: u128 = {Self::TYPE_HASH_NATIVE.to_be()}; 
}

// definition of methods `start_hash_fnv1a(bytes: &[u8]) -> u128` and `pub const fn add_hash_fnv1a(bytes: &[u8], mut hash: u128,) -> u128`
build_hasher!();

/// Uuid code derived from type hash code
pub trait HashedTypeUuid {
    /// return native type hash
    #[inline] fn type_hash_native<REF>() -> u128 where Self: HashedTypeDef<REF> {
        <Self as HashedTypeDef<REF>>::TYPE_HASH_NATIVE
    }
    
    /// return little endianess type hash
    #[inline] fn type_hash_le<REF>() -> u128 where Self: HashedTypeDef<REF> {
        <Self as HashedTypeDef<REF>>::TYPE_HASH_LE
    }
    
    /// return big endianess type hash
    #[inline] fn type_hash_be<REF>() -> u128 where Self: HashedTypeDef<REF> {
        <Self as HashedTypeDef<REF>>::TYPE_HASH_BE
    }
    
    /// return uuid derived from type hash
    #[inline] fn type_uuid<REF>() -> uuid::Uuid where Self: HashedTypeDef<REF> {
        uuid::Uuid::from_u128(Self::type_hash_native::<REF>()) 
    }

    /// return uuid hyphenated string
    #[inline] fn type_uuid_hyphenated<REF>() -> String where Self: HashedTypeDef<REF> {
        Self::type_uuid().as_hyphenated().to_string()
    }

    /// return native type hash from instance
    #[inline] fn self_type_hash_native<REF>(&self) -> u128 where Self: HashedTypeDef<REF> {
        Self::type_hash_native::<REF>()
    }
    
    /// return little endianess type hash from instance
    #[inline] fn self_type_hash_le<REF>(&self) -> u128 where Self: HashedTypeDef<REF> {
        Self::type_hash_le::<REF>()
    }
    
    /// return big endianess type hash from instance
    #[inline] fn self_type_hash_be<REF>(&self) -> u128 where Self: HashedTypeDef<REF> {
        Self::type_hash_be::<REF>()
    }
    
    /// return type hash-derived uuid from instance
    #[inline] fn self_type_uuid<REF>(&self) -> uuid::Uuid where Self: HashedTypeDef<REF> {
        Self::type_uuid::<REF>()
    }

    /// return uuid hyphenated string from instance
    #[inline] fn self_type_uuid_hyphenated<REF>(&self) -> String where Self: HashedTypeDef<REF> {
        Self::type_uuid_hyphenated::<REF>()
    }
}
    
impl<T> HashedTypeUuid for T { }


#[doc(hidden)]
/// Probes for testing features activation
pub mod probes;