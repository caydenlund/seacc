use super::Identifier;

mod character_constant;
pub use character_constant::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Integer(IntegerConstant),
    Floating(FloatingConstant),
    Enumeration(Identifier),
    Character(CharacterConstant),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegerConstant(pub i128, pub Option<IntegerSuffix>);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IntegerSuffix {
    Unsigned,         // `u` or `U`
    Long,             // `l` or `L`
    LongLong,         // `ll` or `LL`
    UnsignedLong,     // (`u` or `U`) + (`l` or `L`) or (`l` or `L`) + (`u` or `U`)
    UnsignedLongLong, // (`u` or `U`) + (`ll` or `LL`) or (`ll` or `LL`) + (`u` or `U`)
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatingConstant(pub f64, pub Option<FloatingSuffix>);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FloatingSuffix {
    Float,  // `f` or `F`
    Double, // `d` or `D`
}
