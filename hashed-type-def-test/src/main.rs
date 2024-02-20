use std::env::SplitPaths;

use hashed_type_def::HashedTypeDef;

fn print_fn_type_hash<'a>(_temp: &'a ()) {
    println!("<fn(u32,&'a str) -> f32>::TYPE_HASH_NATIVE -> {:x}", <fn(u32,&'a str) -> f32>::TYPE_HASH_NATIVE);
}

/// compute some type hash
fn main() {
    println!("bool::TYPE_HASH_NATIVE -> {:x}", bool::TYPE_HASH_NATIVE);
    println!("f64::TYPE_HASH_NATIVE -> {:x}", f64::TYPE_HASH_NATIVE);
    println!("u128::TYPE_HASH_NATIVE -> {:x}", u128::TYPE_HASH_NATIVE);
    println!("std::sync::Weak::<f32>::TYPE_HASH_NATIVE -> {:x}", std::sync::Weak::<f32>::TYPE_HASH_NATIVE);
    println!("<(bool,f64,u128)>::TYPE_HASH_NATIVE -> {:x}", <(bool,f64,u128)>::TYPE_HASH_NATIVE);
    println!("SplitPaths::TYPE_HASH_NATIVE -> {:x}", SplitPaths::TYPE_HASH_NATIVE);
    println!("<(u32,&str)>::TYPE_HASH_NATIVE -> {:x}", <(u32,&str)>::TYPE_HASH_NATIVE);
    // following cases show that type hash of a reference does not depends on lifetime of this reference
    println!("<fn(u32,&'static str) -> f32>::TYPE_HASH_NATIVE -> {:x}", <fn(u32,&'static str) -> f32>::TYPE_HASH_NATIVE);
    { let a = (); print_fn_type_hash(&a); }
}