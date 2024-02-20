use std::fmt::Debug;

use hashed_type_def::{ HashedTypeDef, add_hashed_type_def_param, };


/// case 1: a named structure to be tested with procedural derivation
#[derive(Debug,HashedTypeDef)]
struct MyStruct<'a,'b,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'b, 'a: 'b {
    #[allow(dead_code)] un: &'a String,
    #[allow(dead_code)] deux: T,
    #[allow(dead_code)] trois: &'b U,
}

pub mod instance0 {
    use hashed_type_def::HashedTypeDef;
    use std::fmt::Debug;

    /// case 2: a structure with same definition and procedural derivation as case 1
    /// * type hash should be the same than case 1
    #[derive(Debug,HashedTypeDef)]
    pub struct MyStruct<'a,'b,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'b, 'a: 'b {
        #[allow(dead_code)] un: &'a String,
        #[allow(dead_code)] deux: T,
        #[allow(dead_code)] trois: &'b U,
    }
} 

pub mod instance1 {
    use hashed_type_def::{ add_hashed_type_def, HashedTypeDef, };
    use std::fmt::Debug;
    /// case 3: a structure with same definition as case 1 and post derivation obtained by macro `add_hashed_type_def!` processing on the same definition
    /// * type hash should be the same than case 1
    pub struct MyStruct<'a,'b,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'b, 'a: 'b {
        #[allow(dead_code)] un: &'a String,
        #[allow(dead_code)] deux: T,
        #[allow(dead_code)] trois: &'b U,
    }

    add_hashed_type_def!{
        pub struct MyStruct<'a,'b,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'b, 'a: 'b {
            un: &'a String,
            deux: T,
            trois: &'b U,
        }    
    }
} 

pub mod instance2 {
    use hashed_type_def::HashedTypeDef;
    use std::fmt::Debug;
    /// case 4: a structure with procedural derivation and same definition as case 1 except a swap on lifetime names
    /// * type hash should be different than case 1
    #[derive(Debug,HashedTypeDef)]
    pub struct MyStruct<'b,'a,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'a, 'b: 'a {
        #[allow(dead_code)] un: &'b String,
        #[allow(dead_code)] deux: T,
        #[allow(dead_code)] trois: &'a U,
    }
} 

pub mod instance3 {
    use hashed_type_def::{ add_hashed_type_def, HashedTypeDef, };
    use std::fmt::Debug;
    /// case 5: a structure with same definition as case 1 and post derivation obtained by macro `add_hashed_type_def!` processing on altered definition
    /// * type hash should be different than case 1
    pub struct MyStruct<'a,'b,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'b, 'a: 'b {
        #[allow(dead_code)] un: &'a String,
        #[allow(dead_code)] deux: T,
        #[allow(dead_code)] trois: &'b U,
    }

    add_hashed_type_def!{
        pub struct MyStruct<'a,'b,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'b, 'a: 'b {
            instance3_MyStruct: (),
            un: &'a String,
            deux: T,
            trois: &'b U,
        }    
    }
} 

pub mod instance4 {
    use hashed_type_def::HashedTypeDef;
    use std::fmt::Debug;
    /// case 6: a structure with procedural derivation and same definition as case 1 except conditions are moved to where clauses
    /// * type hash should be different than case 1
    #[derive(Debug,HashedTypeDef)]
    pub struct MyStruct<'a,'b,T,U,const C1: usize, const C2: bool> where 'a: 'b, T: Into<U> + 'b, U: Debug, {
        #[allow(dead_code)] un: &'a String,
        #[allow(dead_code)] deux: T,
        #[allow(dead_code)] trois: &'b U,
    }
} 

pub mod instance5 {
    use hashed_type_def::HashedTypeDef;
    use std::fmt::Debug;
    /// case 7: a structure with procedural derivation and same definition as case 1 except conditions are moved from where clauses
    /// * type hash should be different than case 1
    #[derive(Debug,HashedTypeDef)]
    pub struct MyStruct<'a: 'b,'b, T: Into<U> + 'b, U: Debug, const C1: usize, const C2: bool> {
        #[allow(dead_code)] un: &'a String,
        #[allow(dead_code)] deux: T,
        #[allow(dead_code)] trois: &'b U,
    }
} 

pub mod instance6 {
    use hashed_type_def::HashedTypeDef;
    use std::fmt::Debug;
    /// case 8: a structure with procedural derivation and same definition as case 1 except a conditions is removed from parameters list
    /// * type hash should be different than case 1    
    #[derive(Debug,HashedTypeDef)]
    pub struct MyStruct<'a,'b,T,U,const C1: usize, const C2: bool> where T: Into<U> + 'b, 'a: 'b {
        #[allow(dead_code)] un: &'a String,
        #[allow(dead_code)] deux: T,
        #[allow(dead_code)] trois: &'b U,
    }
} 

pub mod instance7 {
    use hashed_type_def::HashedTypeDef;
    use std::fmt::Debug;
    /// case 9: a structure with procedural derivation and same definition as case 1 except a condition is removed from where clauses
    /// * type hash should be different than case 1
    #[derive(Debug,HashedTypeDef)]
    pub struct MyStruct<'a,'b,T,U: Debug,const C1: usize, const C2: bool> where T: Into<U> + 'b, {
        #[allow(dead_code)] un: &'a String,
        #[allow(dead_code)] deux: T,
        #[allow(dead_code)] trois: &'b U,
    }
} 

/// build type hash of case 1 for specific lifetime
/// * type hash should not change
fn fn_type_hash<'a: 'b,'b>(_tmp1: &'a (), _tmp2: &'b ()) -> u128 { MyStruct::<'a,'b,f32,f64,12,true>::TYPE_HASH_NATIVE }

/// case 10: a unnamed structure to be tested with procedural derivation
#[derive(Debug,HashedTypeDef)]
struct MyTupleStruct<T,U: Debug>(String,T,U,) where T: Into<U>;

/// case 11: a unit structure to be tested with procedural derivation
#[derive(Debug,HashedTypeDef)]
struct MyEmptyStruct;

/// case 12: enum to be tested with procedural derivation
#[derive(Debug,HashedTypeDef)]
enum MyEnum<T,U: Debug> where T: Into<U> {
    #[allow(dead_code)] Zero,
    #[allow(dead_code)] Un(String, T,),
    #[allow(dead_code)] Deux {
        double: f64,
        trois: U,
    },
    #[allow(dead_code)] Trois,
}

/// case 13: empty enum to be tested with procedural derivation
#[derive(Debug,HashedTypeDef)]
enum MyEmptyEnum { }

/// case 14: a struct to be tested with post derivation with parameter tag `(Option<u32>,[u128;24])`
struct AnotherStruct<T: Debug> where T: Default { #[allow(dead_code)] t: T }

add_hashed_type_def_param!((Option<u32>,[u128;24]); struct AnotherStruct<T: Debug> where T: Default { t: T });

#[test]
fn assert_good_hash() {
    // case 1 with `true` parameter
    assert_eq!(MyStruct::<'static,'static,f32,f64,12,true>::TYPE_HASH_NATIVE,0xbd84ac66d0fa5d1cae5e25647a13dcb3);
    // case 1 with `true` parameter and different lifetime
    assert_eq!({ let a = (); { let b = (); fn_type_hash(&a,&b) } },0xbd84ac66d0fa5d1cae5e25647a13dcb3);
    // case 1 with `false` parameter
    assert_eq!(MyStruct::<'static,'static,f32,f64,12,false>::TYPE_HASH_NATIVE,0x1a8a73651c9cafedcca3983399a91fd8);
    // case 2 with `false` parameter
    assert_eq!(instance0::MyStruct::<'static,'static,f32,f64,12,false>::TYPE_HASH_NATIVE,0x1a8a73651c9cafedcca3983399a91fd8);
    // case 3 with `false` parameter
    assert_eq!(instance1::MyStruct::<'static,'static,f32,f64,12,false>::TYPE_HASH_NATIVE,0x1a8a73651c9cafedcca3983399a91fd8);
    // case 4 with `false` parameter
    assert_eq!(instance2::MyStruct::<'static,'static,f32,f64,12,false>::TYPE_HASH_NATIVE,0xbbe982fffcad539086f0cce2e7dbae6b);
    // case 5 with `false` parameter
    assert_eq!(instance3::MyStruct::<'static,'static,f32,f64,12,false>::TYPE_HASH_NATIVE,0x56d4f1b7af31d3613afbdc89e52c2ded);
    // case 6 with `false` parameter
    assert_eq!(instance4::MyStruct::<'static,'static,f32,f64,12,false>::TYPE_HASH_NATIVE,0x394096e55187edf4ac773f6edc052b72);
    // case 7 with `false` parameter
    assert_eq!(instance5::MyStruct::<'static,'static,f32,f64,12,false>::TYPE_HASH_NATIVE,0xe7e26201409531d1bfa3fd4b62abc938);
    // case 8 with `false` parameter
    assert_eq!(instance6::MyStruct::<'static,'static,f32,f64,12,false>::TYPE_HASH_NATIVE,0xbee6197ef3ea446890ac34705c30cdd);
    // case 9 with `false` parameter
    assert_eq!(instance7::MyStruct::<'static,'static,f32,f64,12,false>::TYPE_HASH_NATIVE,0xf8f3fcc7e4a5e021e200763b4cf9df7a);
    // case 10
    assert_eq!(MyTupleStruct::<f32,f64>::TYPE_HASH_NATIVE,0xddd04a2668070f2767b2227db8f17b75);
    // case 11
    assert_eq!(MyEmptyStruct::TYPE_HASH_NATIVE,0x4ede75e31bf75298ae870ee1d57a1357);
    // case 12
    assert_eq!(MyEnum::<f32,f64,>::TYPE_HASH_NATIVE,0xfcad4ca36cd06c7e21ea658a12369d9f);
    // case 13
    assert_eq!(MyEmptyEnum::TYPE_HASH_NATIVE,0x9dec7519c5f412b90509b5b2ef1d521c);
    // case 14
    assert_eq!(AnotherStruct::<Vec<u16>>::TYPE_HASH_NATIVE,0x933ae311b69af6007caa030743cbb5e5);
}
