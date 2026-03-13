pub enum StringLiteralEncoding {
    /// No prefix or prefix `u8` (corresponds to `char`)
    None,
    /// Wide character prefix `L` (corresponds to `wchar_t`)
    Wide,
    /// UTF-16 character prefix `u` (corresponds to `char16_t`)
    Utf16,
    /// UTF-32 character prefix `U` (corresponds to `char32_t`)
    Utf32,
}

pub struct StringLiteral(pub StringLiteralEncoding, pub String);
