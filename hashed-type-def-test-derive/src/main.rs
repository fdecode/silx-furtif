use std::fmt::Debug;

use hashed_type_def::{ HashedTypeDef, HashedTypeUuid, add_hashed_type_def_param, };

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

/// build type hash (Uuid format) of case 1 for specific lifetime
/// * type hash should not change
fn fn_type_hash<'a: 'b,'b>(_tmp1: &'a (), _tmp2: &'b ()) -> uuid::Uuid { MyStruct::<'a,'b,f32,f64,12,true>::type_uuid() }

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

fn main() {
    // case 1 with `true` parameter
    println!("MyStruct::<'static,'static,f32,f64,12,true>::type_uuid() -> {:x}", MyStruct::<'static,'static,f32,f64,12,true>::type_uuid());
    // case 1 with `true` parameter and different lifetime
    println!("MyStruct::<'a,'b,f32,f64,12,true>::type_uuid() -> {:x}", { let a = (); { let b = (); fn_type_hash(&a,&b) } });
    // case 1 with `false` parameter
    println!("MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
    // case 2 with `false` parameter
    println!("instance0::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance0::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
    // case 3 with `false` parameter
    println!("instance1::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance1::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
    // case 4 with `false` parameter
    println!("instance2::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance2::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
    // case 5 with `false` parameter
    println!("instance3::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance3::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
    // case 6 with `false` parameter
    println!("instance4::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance4::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
    // case 7 with `false` parameter
    println!("instance5::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance5::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
    // case 8 with `false` parameter
    println!("instance6::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance6::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
    // case 9 with `false` parameter
    println!("instance7::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid() -> {:x}", instance7::MyStruct::<'static,'static,f32,f64,12,false>::type_uuid());
    // case 10
    println!("MyTupleStruct::<f32,f64>::type_uuid() -> {:x}", MyTupleStruct::<f32,f64>::type_uuid());
    // case 11
    println!("MyEmptyStruct::type_uuid() -> {:x}", MyEmptyStruct::type_uuid());
    // case 12
    println!("MyEnum::<f32,f64,>::type_uuid() -> {:x}", MyEnum::<f32,f64,>::type_uuid());
    // case 13
    println!("MyEmptyEnum::type_uuid() -> {:x}", MyEmptyEnum::type_uuid());
    // case 14
    println!("AnotherStruct::<Vec<u16>>::type_uuid() -> {:x}", AnotherStruct::<Vec<u16>>::type_uuid());
}
