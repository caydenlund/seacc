use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CharacterConstantPrefix {
    /// No prefix (corresponds to `int`)
    None,
    /// Wide character prefix `L` (corresponds to `wchar_t`)
    Wide,
    /// UTF-16 character prefix `u` (corresponds to `char16_t`)
    Utf16,
    /// UTF-32 character prefix `U` (corresponds to `char32_t`)
    Utf32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CharacterConstant(char, CharacterConstantPrefix);

impl CharacterConstant {
    /// Creates a new `CharacterConstant` from the given text
    ///
    /// # Errors
    /// Returns an error if `raw_char_sequence` is not a valid character constant.
    pub fn new(raw_char_sequence: &str) -> Result<Self, CharacterConstantError> {
        let err = || CharacterConstantError(raw_char_sequence.into());

        let (prefix_type, content) = match raw_char_sequence.chars().next() {
            Some('L') => (CharacterConstantPrefix::Wide, &raw_char_sequence[1..]),
            Some('u') => (CharacterConstantPrefix::Utf16, &raw_char_sequence[1..]),
            Some('U') => (CharacterConstantPrefix::Utf32, &raw_char_sequence[1..]),
            _ => (CharacterConstantPrefix::None, raw_char_sequence),
        };

        let content = content
            .strip_prefix('\'')
            .ok_or_else(err)?
            .strip_suffix('\'')
            .ok_or_else(err)?;

        if content.is_empty() {
            return Err(err());
        }

        let mut chars = content.chars();
        let first = chars.next().ok_or_else(err)?;

        let result = if first == '\\' {
            // Escape sequence
            let next = chars.next().ok_or_else(err)?;
            match next {
                '\'' => '\'',
                '"' => '"',
                '?' => '?',
                '\\' => '\\',
                'a' => '\x07',
                'b' => '\x08',
                'f' => '\x0C',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'v' => '\x0B',
                'x' => {
                    // Hexadecimal escape
                    let mut hex_str = String::new();
                    while let Some(next_char) = chars.as_str().chars().next() {
                        if next_char.is_ascii_hexdigit() {
                            hex_str.push(next_char);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if hex_str.is_empty() {
                        return Err(err());
                    }
                    let value = u32::from_str_radix(&hex_str, 16).map_err(|_| err())?;
                    char::from_u32(value).ok_or_else(err)?
                }
                c if c.is_digit(8) => {
                    // Octal escape
                    let mut octal_str = String::from(c);
                    for _ in 0..2 {
                        if let Some(next_char) = chars.as_str().chars().next() {
                            if next_char.is_digit(8) {
                                octal_str.push(next_char);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                    let value = u32::from_str_radix(&octal_str, 8).map_err(|_| err())?;
                    char::from_u32(value).ok_or_else(err)?
                }
                _ => return Err(err()),
            }
        } else if first == '\n' {
            return Err(err());
        } else {
            first
        };

        // Ensure there's only one character
        if chars.next().is_some() {
            return Err(err());
        }

        Ok(Self(result, prefix_type))
    }

    /// Creates a new `CharacterConstant` with the given prefix and contents
    ///
    /// # Errors
    /// Returns an error if `ch` is not valid for the given `prefix`.
    pub fn from_parts(
        ch: char,
        prefix: CharacterConstantPrefix,
    ) -> Result<Self, CharacterConstantError> {
        // u'...' uses `char16_t`, which can only hold BMP characters (U+0000–U+FFFF).
        // L'...' uses `wchar_t` (platform-defined size; no further validation here).
        // U'...' uses `char32_t`, which accepts any Unicode scalar — already guaranteed by `char`.
        if matches!(prefix, CharacterConstantPrefix::Utf16) && u32::from(ch) > 0xFFFF {
            return Err(CharacterConstantError(format!("u'{ch}'")));
        }
        Ok(Self(ch, prefix))
    }

    #[must_use]
    pub const fn into_char(self) -> char {
        self.0
    }
}

impl From<CharacterConstant> for char {
    fn from(val: CharacterConstant) -> Self {
        val.into_char()
    }
}

impl TryFrom<&str> for CharacterConstant {
    type Error = CharacterConstantError;

    fn try_from(raw_char_sequence: &str) -> Result<Self, Self::Error> {
        Self::new(raw_char_sequence)
    }
}

impl From<char> for CharacterConstant {
    fn from(ch: char) -> Self {
        Self(ch, CharacterConstantPrefix::None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CharacterConstantError(String);

impl Display for CharacterConstantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid character constant: {}", self.0)
    }
}

impl Error for CharacterConstantError {}
