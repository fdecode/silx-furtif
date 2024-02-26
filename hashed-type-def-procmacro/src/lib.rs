//!This is part of [**Silx**](https://github.com/fdecode/silx-furtif) project  
//! 
//! `hashed-type-def-procmacro` contains the implementation of procedural macros for deriving type hashcode  
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
 
#![recursion_limit = "128"]

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use quote::ToTokens;
use rand::Rng;
use syn::{ DataStruct, DataEnum, DataUnion, FieldsNamed, FieldsUnnamed, Variant, Field, };

#[doc(hidden)]
/// token stream prefix definition: essentially the FNV1a algorithms for starting or incrementing hash
fn prefix_token_stream_hash() -> proc_macro2::TokenStream {
    // prefix for hash computation
    quote! {
        const FNV_OFFSET_BASIS: u128 = 0x6c62272e07bb014262b821756295c58d;
        const FNV_PRIME: u128 = 0x0000000001000000000000000000013B;

        /// Build initial FNV1a hash code for a bytes sequence
        /// * `bytes: &[u8]` : bytes sequence to be hashed
        /// * Output: hash code
        #[inline(always)]
        pub const fn start_hash_fnv1a(bytes: &[u8]) -> u128 {
            add_hash_fnv1a(bytes, FNV_OFFSET_BASIS)
        }

        /// Build incremental FNV1a hash code for a bytes sequence and an initial hash code
        /// * `bytes: &[u8]` : bytes sequence to be hashed
        /// * `hash: u128` : initial hash code
        /// * Output: hash code
        #[inline(always)]
        pub const fn add_hash_fnv1a(bytes: &[u8], mut hash: u128,) -> u128 {
            let len = bytes.len();
            let mut u = 0;
            while u < len {
                hash ^= bytes[u] as u128;
                hash = hash.wrapping_mul(FNV_PRIME);
                u += 1;
            }
            hash
        }
    }
}

/// Boolean flag for debugging
const _DEBUG_ : bool = false;

#[proc_macro]
/// procedural macro for the definition of common hasher functions
pub fn build_hasher(_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(prefix_token_stream_hash())
}

#[proc_macro_derive(HashedTypeDef)]
/// procedural macro for deriving HashedTypeDef
pub fn hashed_type_def(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = parse_macro_input!(input as syn::DeriveInput);
    // Build the impl
    impl_hashed_type_def_param(quote!{()}, &ast)
}

#[proc_macro]
/// procedural macro for adding implementation of HashedTypeDef to existing type
/// * Input : fake type definition 
/// * Output: implementation code
pub fn add_hashed_type_def(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = parse_macro_input!(input as syn::DeriveInput);
    // Build the impl
    impl_hashed_type_def_param(quote!{()}, &ast)
}

/// procedural macro for adding implementation of `HashedTypeDef<Prefix>` to existing type; hash code is computed incrementally involving prefix type hash 
/// * Input : Prefix (a type) followed by fake redefinition of existing type; prefix and redefinition are separated by semicolon
/// * Output: implementation code
#[proc_macro]
pub fn add_hashed_type_def_param(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let mut prefix = vec![];
    let mut postfix = vec![];
    let mut found_semicolon = 0;
    // iterate the tokens and split the tokens between a prefix and a postfix
    for token in input {
        match (found_semicolon, token) {
            (_,proc_macro::TokenTree::Punct(punct)) if punct == ';' => { found_semicolon += 1; },
            (0,t) => { prefix.push(t); },
            (1,t) => { postfix.push(t); },
            _ => { panic!("Required: two token streams separated by semicolon"); }
        }
    }
    // get the prefix and postfix token streams
    let prefix = prefix.into_iter().collect::<proc_macro::TokenStream>();
    let postfix = postfix.into_iter().collect::<proc_macro::TokenStream>();
    // print some debugging information
    if _DEBUG_ {
        println!("prefix -> {prefix}");
        println!("postfix -> {postfix}");
    }
    // parse postfix
    let ast = parse_macro_input!(postfix as syn::DeriveInput);  
    // Build the implementation
    impl_hashed_type_def_param(prefix.into(), &ast)
}

/// Internal use: generate code for adding implementation of `HashedTypeDef<Prefix>` to existing type
/// * `prefix: proc_macro2::TokenStream` : token stream of the prefix 
/// * `ast: &syn::DeriveInput` : data structure of fake redefinition of the existing type 
/// * Output: implementation code
fn impl_hashed_type_def_param(prefix: proc_macro2::TokenStream, ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    // get type elements
    let ident =  ast.ident.clone();
    let ident_string = ident.to_string();
    let ident_mod = syn::parse_str::<syn::Ident>( // build implementation `mod` name for current type
        &format!("_impl_hashed_type_def_{:x}",rand::thread_rng().gen::<u128>())
    ).expect("failed to parse automated generated name");
    // get type data
    let data = ast.data.clone();
    // get type generics
    let generics = ast.generics.clone();
    // get lifetimes
    let lifetimes = generics.lifetimes().collect::<Vec<_>>();
    let lifetimes_string = generics.lifetimes().map(
        |lt| lt.to_token_stream().to_string()
    ).collect::<Vec<_>>();
    let lifetimes_names = generics.lifetimes().map(|lt| {
        lt.lifetime.clone()
    }).collect::<Vec<_>>();
    // get type params
    let type_params = generics.type_params().collect::<Vec<_>>();
    let type_params_string = generics.type_params().map(
        |tp| tp.to_token_stream().to_string()
    ).collect::<Vec<_>>();
    let type_params_names = generics.type_params().into_iter().map(|tp| {
        tp.ident.clone()
    }).collect::<Vec<_>>();
    // get const params
    let const_params = generics.const_params().collect::<Vec<_>>();
    let const_params_string = generics.const_params().map(
        |cp| cp.to_token_stream().to_string()
    ).collect::<Vec<_>>();
    let const_params_values = generics.const_params().into_iter().map(|cp| {
        cp.ident.clone()
    }).collect::<Vec<_>>();
    // get where clauses
    let where_clause = match generics.where_clause.clone() {
        Some(wc) => wc.predicates.into_iter().collect::<Vec<_>>(), None => vec![],
    };
    let where_clause_string = where_clause.iter().map(
        |wc| wc.to_token_stream().to_string()
    ).collect::<Vec<_>>();
    // print some debugging information
    if _DEBUG_ {
        println!("--------ast.ident -> {:?}", ident);
        println!("--------ident_mod -> {ident_mod}");
        for lt in &lifetimes_string { println!("-LT- -> {}", lt); }
        for tp in &type_params_string { println!("-TP- -> {}", tp); }
        for cp in &const_params_string { println!("-CP- -> {}", cp); }
        for predicate in &where_clause_string { println!("-WC- -> {}", predicate); }
        for lid in &lifetimes_names {
            println!("-lifetime type names- -> {}", lid.to_token_stream().to_string());
        }
        for tid in &type_params_names {
            println!("-type names- -> {}", tid.to_token_stream().to_string());
        }
        for cid in &const_params_values {
            println!("-const params names- -> {}", cid.to_token_stream().to_string());
        }        
    }
    // prefix for hash computation (the same is used in core)
    let prelude = prefix_token_stream_hash();
    // get definition characteristic items
    let (
        data_type, // type of data (struck, enum, union)
        second_data_type, // secondary type of data, especially used to describe enum substructure
        idents, // list of idents of first level fields, if named, in struct, enum or union
        types, // flattened list of types of last level fields; last level is first level for struct or union, and is second level for enum
        second_idents, // flattened list of idents of second level fields, if named, in enum
        second_ident_shifts, // list of first level sizes for unflattening second_idents; populated for enum
        type_shifts, // list of first level sizes for unflattening types; populated for enum 
        // Nota: second_ident_shifts and type_shifts are not necessarily the same
    ) = match &data {
        // case of struct type
        syn::Data::Struct(DataStruct { fields, .. }) => { 
            let (data_type,(idents, types),) = match fields { // struct either is unit, has named fields, has unnamed fields
                syn::Fields::Named(FieldsNamed { named, .. }) => { ("<struct-named-proc>", // if it is named
                    // then gets the vectors of fields names (ident) and the vector of types
                    named.into_iter().cloned().map(|Field { ident, ty, .. } | (ident.unwrap().to_string(),ty)).unzip()
                ) },
                syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => { ("<struct-unnamed-proc>", // if it is unnamed
                    // then gets the vector of types (vector of fields names is empty)
                    (vec![], unnamed.into_iter().cloned().map(|Field { ty, .. } | ty).collect())
                ) },
                syn::Fields::Unit => ("<struct-unit-proc>", (vec![],vec![])), // if it is unit, both vectors are empty
            };
            // there is no secondary type of data
            let second_data_type = vec![];
            // secondary idents and shifts vectors are empty for struct types
            let (second_idents, second_ident_shifts, type_shifts) = ( 
                Vec::<String>::new(), Vec::<usize>::new(), Vec::<usize>::new(),
            );
            (data_type,second_data_type,idents,types,second_idents,second_ident_shifts,type_shifts,)
        },
        // case of enum type
        syn::Data::Enum(DataEnum { variants, .. }) => {
            // only one kind of data type tag, but there are secondary data type tags
            let data_type = "<enum-proc>";
            // idents vector is given here by the list of variants; fields vector is also obtained from these variants
            let (idents,fields,): (Vec<_>,Vec<_>) = variants.into_iter()
                .map(|Variant { ident, fields, .. }| (ident.to_string(),fields.clone())).unzip();
            // prepare to collect and compute idents, types and shifts
            let mut second_data_type = Vec::<&'static str>::with_capacity(idents.len());
            let mut second_ident_shifts = Vec::<usize>::with_capacity(idents.len()); 
            let mut type_shifts = Vec::<usize>::with_capacity(idents.len());
            let mut pre_second_idents = Vec::<Vec<String>>::with_capacity(idents.len());
            let mut pre_types = Vec::<Vec<syn::Type>>::with_capacity(idents.len());
            for field in fields {
                // collect and compute idents, types and shifts from each field
                match field { // field either is unit, has named subfields, has unnamed subfields
                    syn::Fields::Named(FieldsNamed { named, .. }) => { // if it is named
                        second_data_type.push("<field-named-proc>",); // set secondary tag as named
                        let (si,t): (Vec<_>,Vec<_>) = named.into_iter().map( // get collections of secondary idents and types
                            |Field { ident, ty, .. } | (ident.unwrap().to_string(),ty)
                        ).unzip();
                        // compute common shift and push them
                        let shift = t.len();
                        second_ident_shifts.push(shift); 
                        type_shifts.push(shift);
                        // push the collections of secondary idents and types
                        pre_second_idents.push(si);
                        pre_types.push(t);
                    },
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => { // if it is unnamed
                        second_data_type.push("<field-unnamed-proc>",); // set secondary tag as unnamed
                        // collect all types of the field
                        let t = unnamed.into_iter().map(|Field { ty, .. } | ty).collect::<Vec<_>>();
                        // collection of secondary idents for this case is empty; related shift is zero
                        pre_second_idents.push(vec![]);
                        second_ident_shifts.push(0); 
                        // push the collection of types for this case; type shift is the length of this collection
                        type_shifts.push(t.len());
                        pre_types.push(t);
                    },
                    syn::Fields::Unit => { // if it is unit
                        second_data_type.push("<field-unit-proc>",); // set secondary tag as unit
                        // collections of secondary idents and types for this case are empty
                        pre_second_idents.push(vec![]);
                        pre_types.push(vec![]);
                        // shifts are thus zero
                        second_ident_shifts.push(0); 
                        type_shifts.push(0);
                    },
                }
            }
            // build collections of second idents and of types by flattening
            let second_idents = pre_second_idents.into_iter().flat_map(|c|c).collect::<Vec<_>>();
            let types = pre_types.into_iter().flat_map(|c|c).collect::<Vec<_>>();
            (data_type,second_data_type,idents,types,second_idents,second_ident_shifts,type_shifts,)
        },
         // case of union type
        syn::Data::Union(DataUnion { fields: FieldsNamed { named, .. }, .. }) => {
            let data_type = "<union-proc>";
            // gets the vectors of fields names (ident) and the vector of types
            let (idents,types): (Vec<_>,Vec<_>) = named.into_iter().cloned().map(
                |Field { ident, ty, .. } | (ident.unwrap().to_string(),ty)
            ).unzip();            
            // there is no secondary type of data
            let second_data_type = vec![];
            let (second_idents, second_ident_shifts, type_shifts) = (
                Vec::<String>::new(), Vec::<usize>::new(), Vec::<usize>::new(),
            );
            (data_type,second_data_type,idents,types,second_idents,second_ident_shifts,type_shifts,)
        },
    };
    // get vectors lengths
    let second_data_type_len = second_data_type.len() as u128;
    let lifetimes_string_len = lifetimes_string.len() as u128;
    let type_params_string_len = type_params_string.len() as u128;
    let const_params_string_len = const_params_string.len() as u128;
    let where_clause_string_len = where_clause_string.len() as u128;
    let const_params_values_len = const_params_values.len() as u128;
    let idents_len = idents.len() as u128;
    let types_len = types.len() as u128;
    let second_idents_len = second_idents.len() as u128;
    let second_ident_shifts_len = second_ident_shifts.len() as u128;
    let type_shifts_len = type_shifts.len() as u128;
    // build implementation code
    proc_macro::TokenStream::from(quote! {
        // mod within which HashedTypeDef is implemented 
        mod #ident_mod {
            use super::*;
            // function definitions for FNV1a computation
            #prelude
            // implementing `HashedTypeDef<Prefix>` for existing type
            impl <#(#lifetimes,)*#(#type_params,)*#(#const_params,)*> HashedTypeDef<#prefix>
                    for #ident<#(#lifetimes_names,)*#(#type_params_names,)*#(#const_params_values,)*>
                    where #(#where_clause,)*#(#type_params_names: HashedTypeDef,)*#prefix: HashedTypeDef, {
                const TYPE_HASH_NATIVE: u128 =  {
                    // start hash with data type label
                    #[allow(unused_mut)] let mut hash = start_hash_fnv1a(#data_type.as_bytes()); 
                    // add secondary data type labels
                    hash = add_hash_fnv1a(&#second_data_type_len.to_le_bytes(),hash);
                    #(hash = add_hash_fnv1a(&#second_data_type.as_bytes(),hash);)* 
                    // add type ident to hash
                    hash = add_hash_fnv1a(&#ident_string.as_bytes(),hash); 
                    // add Prefix hash
                    hash = add_hash_fnv1a(&<#prefix>::TYPE_HASH_NATIVE.to_le_bytes(),hash); 
                    // add lifetime names
                    hash = add_hash_fnv1a(&#lifetimes_string_len.to_le_bytes(),hash);
                    #(hash = add_hash_fnv1a(&#lifetimes_string.as_bytes(),hash);)* 
                    // add types params names
                    hash = add_hash_fnv1a(&#type_params_string_len.to_le_bytes(),hash);
                    #(hash = add_hash_fnv1a(&#type_params_string.as_bytes(),hash);)* 
                    // add consts params names
                    hash = add_hash_fnv1a(&#const_params_string_len.to_le_bytes(),hash);
                    #(hash = add_hash_fnv1a(&#const_params_string.as_bytes(),hash);)* 
                    // add stringified where clause
                    hash = add_hash_fnv1a(&#where_clause_string_len.to_le_bytes(),hash);
                    #(hash = add_hash_fnv1a(&#where_clause_string.as_bytes(),hash);)* 
                    // add const values
                    hash = add_hash_fnv1a(&#const_params_values_len.to_le_bytes(),hash);
                    #(hash = add_hash_fnv1a(&(#const_params_values as u128).to_le_bytes(),hash);)* 
                    // add fields names
                    hash = add_hash_fnv1a(&#idents_len.to_le_bytes(),hash);
                    #(hash = add_hash_fnv1a(&#idents.as_bytes(),hash);)* 
                    // add fields types hashes
                    hash = add_hash_fnv1a(&#types_len.to_le_bytes(),hash);
                    #(hash = add_hash_fnv1a(&<#types>::TYPE_HASH_NATIVE.to_le_bytes(),hash);)*
                    // add secondary idents names
                    hash = add_hash_fnv1a(&#second_idents_len.to_le_bytes(),hash);
                    #(hash = add_hash_fnv1a(&#second_idents.as_bytes(),hash);)*
                    // add secondary idents shifts
                    hash = add_hash_fnv1a(&#second_ident_shifts_len.to_le_bytes(),hash);
                    #(hash = add_hash_fnv1a(&(#second_ident_shifts as u128).to_le_bytes(),hash);)*
                    // add type shifts
                    hash = add_hash_fnv1a(&#type_shifts_len.to_le_bytes(),hash);
                    #(hash = add_hash_fnv1a(&(#type_shifts as u128).to_le_bytes(),hash);)*
                    hash
                };
            }
        }
    })
}