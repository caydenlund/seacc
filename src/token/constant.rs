use super::Identifier;

mod character_constant;
pub use character_constant::*;

pub enum Constant {
    Integer(IntegerConstant),
    Floating(FloatingConstant),
    Enumeration(Identifier),
    Character(CharacterConstant),
}

pub struct IntegerConstant(pub i128, pub Option<IntegerSuffix>);

pub enum IntegerSuffix {
    Unsigned,         // `u` or `U`
    Long,             // `l` or `L`
    LongLong,         // `ll` or `LL`
    UnsignedLong,     // (`u` or `U`) + (`l` or `L`) or (`l` or `L`) + (`u` or `U`)
    UnsignedLongLong, // (`u` or `U`) + (`ll` or `LL`) or (`ll` or `LL`) + (`u` or `U`)
}

pub struct FloatingConstant(pub f64, pub Option<FloatingSuffix>);

pub enum FloatingSuffix {
    Float,  // `f` or `F`
    Double, // `d` or `D`
}
