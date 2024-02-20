use std::env::SplitPaths;

use hashed_type_def::HashedTypeDef;

fn fn_type_hash<'a>(_temp: &'a ()) -> u128 { <fn(u32,&'a str) -> f32>::TYPE_HASH_NATIVE }

#[test]
/// test some type hash
fn assert_good_hash() {
    assert_eq!(bool::TYPE_HASH_NATIVE,0x62eed0693fb12c800353f7db588bf3e6);
    assert_eq!(f64::TYPE_HASH_NATIVE,0x85bc8f27efb1355d88eb9d1a4cecd480);
    assert_eq!(u128::TYPE_HASH_NATIVE,0x54e2893415a6ed33045f91ef22133932);
    assert_eq!(std::sync::Weak::<f32>::TYPE_HASH_NATIVE,0x1899fd6d5383c2497034aa888d64fa71);
    assert_eq!(<(bool,f64,u128)>::TYPE_HASH_NATIVE,0x84bb33458ec7afbd0ddc75b47eee44d4);
    assert_eq!(SplitPaths::TYPE_HASH_NATIVE,0x2eaa10b34d64006544f325b30e58545a);
    assert_eq!(<(u32,&str)>::TYPE_HASH_NATIVE,0x70e392409e72055b9e51a0a40ba251ba);
    assert_eq!(<fn(u32,&'static str) -> f32>::TYPE_HASH_NATIVE,0x204246f6bde74f1ca9ae7fb48730d4a7);
    { let a = (); assert_eq!(fn_type_hash(&a),0x204246f6bde74f1ca9ae7fb48730d4a7); };
}
