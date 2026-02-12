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
    Unsigned, // `u` or `U`
    Long,     // `l` or `L`
    LongLong, // `ll` or `LL`
}

pub struct FloatingConstant(pub f64, pub Option<FloatingSuffix>);

pub enum FloatingSuffix {
    Float,  // `f` or `F`
    Double, // `d` or `D`
}
