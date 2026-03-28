# Lexical Elements

This text is adapted from the freely available April 12, 2011 ISO Committee Draft of the C language.


## Lexical elements

_token:_
  - _keyword_
  - _identifier_
  - _constant_
  - _string-literal_
  - _punctuator_

_preprocessing-token:_
  - _header-name_
  - _identifier_
  - _pp-number_
  - _character-constant_
  - _string-literal_
  - _punctuator_
  - each non-white-space character that cannot be one of the above


## Keywords

_keyword:_ one of
  - `auto`
  - `break`
  - `case`
  - `char`
  - `const`
  - `continue`
  - `default`
  - `do`
  - `double`
  - `else`
  - `enum`
  - `extern`
  - `float`
  - `for`
  - `goto`
  - `if`
  - `inline`
  - `int`
  - `long`
  - `register`
  - `restrict`
  - `return`
  - `short`
  - `signed`
  - `sizeof`
  - `static`
  - `struct`
  - `switch`
  - `typedef`
  - `union`
  - `unsigned`
  - `void`
  - `volatile`
  - `while`
  - `_Alignas`
  - `_Alignof`
  - `_Atomic`
  - `_Bool`
  - `_Complex`
  - `_Generic`
  - `_Imaginary`
  - `_Noreturn`
  - `_Static_assert`
  - `_Thread_local`


## Identifiers

_identifier:_
  - _identifier-nondigit_
  - _identifier_    _identifier-nondigit_
  - _identifier_    _digit_

_identifier-nondigit:_
  - _nondigit_
  - _universal-character-name_
  - other implementation-defined characters

_nondigit:_ one of
  - `_`
  - regex: `[a-zA-Z]`

_digit:_ one of
  - regex: `[0-9]`


## Universal character names

_universal-character-name:_
  - `\u`    _hex-quad_
  - `\U`    _hex-quad_    _hex-quad_

_hex-quad:_
  - _hexadecimal-digit_    _hexadecimal-digit_    _hexadecimal-digit_    _hexadecimal-digit_


## Constants

_constant:_
  - _integer-constant_
  - _floating-constant_
  - _enumeration-constant_
  - _character-constant_

_integer-constant:_
  - _decimal-constant_    _optional(integer-suffix)_
  - _octal-constant_    _optional(integer-suffix)_
  - _hexadecimal-constant_    _optional(integer-suffix)_

_decimal-constant:_
  - _nonzero-digit_
  - _decimal-constant_    _digit_

_octal-constant:_
  - `0`
  - _octal-constant_    _octal-digit_

_hexadecimal-constant:_
  - _hexadecimal-prefix_    _hexadecimal-digit_
  - _hexadecimal-constant_    _hexadecimal-digit_

_hexadecimal-prefix:_ one of
  - `0x`
  - `0X`

_nonzero-digit:_ one of
  - regex: `[1-9]`

_octal-digit:_ one of
  - regex: `[0-7]`

_hexadecimal-digit:_ one of
  - regex: `[0-9a-fA-F]`

_integer-suffix:_
  - _unsigned-suffix_    _optional(long-suffix)_
  - _unsigned-suffix_    _long-long-suffix_
  - _long-suffix_    _optional(unsigned-suffix)_
  - _long-long-suffix_    _optional(unsigned-suffix)_

_unsigned-suffix:_ one of
  - `u`
  - `U`

_long-suffix:_ one of
  - `l`
  - `L`

_long-long-suffix:_ one of
  - `ll`
  - `LL`

_floating-constant:_
  - _decimal-floating-constant_
  - _hexadecimal-floating-constant_

_decimal-floating-constant:_
  - _fractional-constant_    _optional(exponent-part)_    _optional(floating-suffix)_
  - _digit-sequence_    _exponent-part_    _optional(floating-suffix)_

_hexadecimal-floating-constant:_
  - _hexadecimal-prefix_    _hexadecimal-fractional-constant_    _binary-exponent-part_    _optional(floating-suffix)_
  - _hexadecimal-prefix_    _hexadecimal-digit-sequence_    _binary-exponent-part_    _optional(floating-suffix)_

_fractional-constant:_
  - _optional(digit-sequence)_    `.`    _digit-sequence_
  - _digit-sequence_    `.`

_exponent-part:_
  - `e`    _optional(sign)_    _digit-sequence_
  - `E`    _optional(sign)_    _digit-sequence_

_sign:_ one of
  - `+`
  - `-`

_digit-sequence:_
  - _digit_
  - _digit-sequence_     _digit_

_hexadecimal-fractional-constant:_
  - _optional(hexadecimal-digit-sequence)_    `.`    _hexadecimal-digit-sequence_
  - _optional(hexadecimal-digit-sequence)_    `.`

_binary-exponent-part:_
  - `p`    _optional(sign)_    _digit-sequence_
  - `P`    _optional(sign)_    _digit-sequence_

_hexadecimal-digit-sequence:_
  - _hexadecimal-digit_
  - _hexadecimal-digit-sequence_    _hexadecimal-digit_

_floating-suffix:_ one of
  - `f`
  - `l` 
  - `F`
  - `L` 

_enumeration-constant:_
  - _identifier_

_character-constant:_
  - `'`    _c-char-sequence_    `'`
  - `L'`    _c-char-sequence_    `'`
  - `u'`    _c-char-sequence_    `'`
  - `U'`    _c-char-sequence_    `'`

_c-char-sequence:_
  - _c-char_
  - _c-char-sequence_    _c-char_

_c-char:_
  - any member of the source character set except the single-quote `'`, backslash `\`, or new-line character
  - _escape-sequence_

_escape-sequence:_
  - _simple-escape-sequence_
  - _octal-escape-sequence_
  - _hexadecimal-escape-sequence_
  - _universal-character-name_

_simple-escape-sequence:_ one of
  - `\'`
  - `\"`
  - `\?`
  - `\\`
  - `\a`
  - `\b`
  - `\f`
  - `\n`
  - `\r`
  - `\t`
  - `\v`

_octal-escape-sequence:_
  - `\`    _octal-digit_
  - `\`    _octal-digit_    _octal-digit_
  - `\`    _octal-digit_    _octal-digit_    _octal-digit_

_hexadecimal-escape-sequence:_
  - `\x`    _hexadecimal-digit_
  - _hexadecimal-escape-sequence_    _hexadecimal-digit_


## String literals

_string-literal:_
  - _encoding-prefixopt_    `"`    _s-char-sequenceopt_    `"`

_encoding-prefix:_
  - `u8`
  - `u`
  - `U`
  - `L`

_s-char-sequence:_
  - _s-char_
  - _s-char-sequence_    _s-char_

_s-char:_
  - any member of the source character set except the double-quote `"`, backslash `\`, or new-line character
  - _escape-sequence_


## Punctuators

_punctuator:_ one of
  - `[`
  - `]`
  - `(`
  - `)`
  - `{`
  - `}`
  - `.`
  - `->`
  - `++`
  - `--`
  - `&`
  - `*`
  - `+`
  - `-`
  - `~`
  - `!`
  - `/`
  - `%`
  - `<<`
  - `>>`
  - `<`
  - `>`
  - `<=`
  - `>=`
  - `?`
  - `:`
  - `;`
  - `...`
  - `=`
  - `*=`
  - `/=`
  - `%=`
  - `+=`
  - `-=`
  - `<<=`
  - `,`
  - `#`
  - `##`
  - `<:`
  - `:>`
  - `<%`
  - `%>`
  - `%:`
  - `%:%:`


## Header names

_header-name:_
  - `<`    _h-char-sequence_    `>`
  - `"`    _q-char-sequence_    `"`

_h-char-sequence:_
  - _h-char_
  - _h-char-sequence_    _h-char_

_h-char:_
  - any member of the source character set except the new-line character and `>`

_q-char-sequence:_
  - _q-char_
  - _q-char-sequence_    _q-char_

_q-char:_
  - any member of the source character set except the new-line character and `"`


## Preprocessing numbers

_pp-number:_
  - _digit_
  - `.`    _digit_
  - _pp-number_    _digit_
  - _pp-number_    _identifier-nondigit_
  - _pp-number_    `e`    _sign_
  - _pp-number_    `E`    _sign_
  - _pp-number_    `p`    _sign_
  - _pp-number_    `P`    _sign_
  - _pp-number_    `.`
