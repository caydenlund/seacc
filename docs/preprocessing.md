# Preprocessing Directives

This text is adapted from the freely available April 12, 2011 ISO Committee Draft of the C language.

## Syntax

A _preprocessing directive_ consists of a line of preprocessor tokens, at the beginning of translation phase 4.
In particular, this means that escaped newlines and comments are removed and digraph/trigraph sequences have been replaced with the corresponding characters, but macros are not expanded.
- The first token of the sequence is `#`. It must be the first preprocessor token on the line, excluding whitespace.
- The last token is newline. It must be the only newline in the sequence.

## Conditional Inclusion

An integer constant expression may be used to conditionally compile the contents of a block of text.

Preprocessing directives of the following forms check whether the controlling constant expression evalutates to nonzero:
- `#`    `if`    _constant-expression_    _new-line_
- `#`    `elif`    _constant-expression_    _new-line_

Each directive's condition in a `#if`/`#elif` chain is checked in order.
If the condition evaluates to **0** (false), the group that it controls is skipped, and the next `#elif` condition is checked; only the first group whose control condition evaluates to nonzero (true) is processed.
If none of the conditions evaluates to true, and there is a `#else` directive, then the group controlled by the `#else` is processed; without an `#else`, all groups until the `#endif` are skipped.

### `defined` Keyword

Such expressions (i.e., the condition for conditional inclusion) may use the `defined` keyword as a unary operator expression:
- `defined`    _identifier_
- `defined`    `(`    _identifier_    `)`

This will evaluate to **1** if the identifier is currently defined as a macro name.

### `ifdef` / `ifndef` Directives

Preprocessing directives of the forms
- `#`    `ifdef`    _identifier_    _new-line_
- `#`    `ifndef`    _identifier_    _new-line_

check whether the identifier is or isn't currently defined as a macro name.
Their conditions are equivalent to
- `#if defined`    _identifier_
- `#if !defined`    _identifier_
respectively.

## Source File Inclusion

A `#include` directive will find a header or source file that can be processed by the implementation.
- `#`    `include`    `<`    _h-char-sequence_    `>`    _new-line_
  searches a sequence of implementation-defined places for a header identified by the h-char-sequence.
- `#`    `include`    `"`    _q-char-sequence_    `"`    _new-line_
  searches a sequence of implementation-defined places for a header identified by the q-char-sequence.
  If not found, it will be reprocessed as if it read `#include <`    _h-char-sequence_    `>`    _new-line_

## Macro Replacement

### Object-like Macros

A preprocessing directive of the following form defines an object-like macro.
- `#`    `define`    _identifer_    _replacement-list_    _new-line_

Each subsequent use of this macro name (the identifier) will be replaced by the replacement list of preprocessing tokens.

### Function-like Macros

A preprocessing directive of the following forms defines a function-like macro with parameters.
Note that _lparen_ is a left parenthesis that immediately follows the identifier, with no whitespace in between.
- `#`    `define`    _identifier_    _lparen_    _optional(identifier-list)_    `)`    _replacement-list_    _new-line_
- `#`    `define`    _identifier_    _lparen_    `...`    `)`    _replacement-list_    _new-line_
- `#`    `define`    _identifier_    _lparen_    _identifier-list_    `,`    `...`    `)`    _replacement-list_    _new-line_

Each subsequent use of this macro name (the identifier) followed by a `(` as the next preprocessing token introduces the sequence of preprocessing tokens that is replaced by the replacement list in the definition (including any substitution of parameters).

The number of arguments in the invocation must match the number of parameters in the definition.
For a macro definition with `...`, the `...` will match zero or more arguments, which may be accessed in the replacement list using the identifier `__VA_ARGS__`.

#### Argument Substitution

When a function-like macro is invoked:
1. Arguments are identified as the preprocessing token sequences separated by commas that are not nested within matching inner parentheses.
2. Within the replacement list, each instance of a parameter identifier is replaced by the corresponding argument token sequence.
3. If the parameter is preceded by `#` (the stringizing operator), the argument is converted to a string literal.
4. If two tokens are separated by `##` (the token-pasting operator), they are concatenated to form a single token.

### Rescanning and Further Replacement

After all parameters in the replacement list have been substituted, the resulting preprocessing token sequence is rescanned along with all subsequent tokens for more macro names to replace.
However, if the name of the macro being replaced is found during this scan, it is not replaced (preventing infinite recursion).

### Undefinition

A preprocessing directive of the following form will cause the specified identifier to no longer be defined as a macro name.
- `#`    `undef`    _identifier-name_    _new-line_

## Line Control

A preprocessing directive of the following forms causes the implementation to behave as if the following sequence of source lines begins with a source line that has a line number as specified by the digit sequence (interpreted as a decimal integer).
- `#`    `line`    _digit-sequence_    _new-line_
- `#`    `line`    _digit-sequence_    `"`    _optional(s-char-sequence)_    `"`    _new-line_
- `#`    `line`    _pp-tokens_    _new-line_
  ... such that _pp-tokens_ expands to one of the other 2 forms

The second form additionally causes the presumed name of the source file to be the s-char-sequence.
The digit sequence shall not specify zero nor a number greater than 2147483647.

## Error Directive

A preprocessing directive of the following form causes the implementation to produce a diagnostic message that includes the specified sequence of preprocessing tokens.
- `#`    `error`    _optional(pp-tokens)_    _new-line_

## Pragma Directive

A preprocessing directive of the following form causes the implementation to behave in an implementation-defined manner.
- `#`    `pragma`    _optional(pp-tokens)_    _new-line_
  such that preprocessing token `STDC` doesn't immediately follow `pragma`

Any pragma that is not recognized by the implementation is ignored.

### Standard Pragmas

The following pragmas are defined by the standard, where _on-off-switch_ is one of `ON`, `OFF`, or `DEFAULT`:
- `#`    `pragma`    `STDC`    `FP_CONTRACT`    _on-off-switch_    _new-line_
  Controls whether expressions may be contracted (e.g., allowing fused multiply-add operations).
- `#`    `pragma`    `STDC`    `FENV_ACCESS`    _on-off-switch_    _new-line_
  Informs the implementation whether the program intends to access the floating-point environment.
- `#`    `pragma`    `STDC`    `CX_LIMITED_RANGE`    _on-off-switch_    _new-line_
  Informs the implementation that the usual mathematical formulas for complex multiply, divide, and absolute value are acceptable (may cause inaccuracy in edge cases).

## Null Directive

The following directive has no effect:
- `#`    _new-line_

## Predefined Macro Names

The following macro names are predefined by the implementation.
All of these stay constant throughout the translation unit, except `__FILE__` and `__LINE__`.

None of the predefined macro names, nor the identifier `defined`, shall be the subject of a `#define` or `#undef` preprocessing directive.
Any other predefined macro names shall begin with a leading underscore followed by an uppercase letter or a second underscore.

The implementation shall not predefine the macro `__cplusplus`, nor shall any standard header define it.

### Mandatory

The following macro names shall be defined by the implementation:

- `__DATE__`
  The date of translation of the preprocessing translation unit as a character string literal of the form `"Mmm dd yyyy"`, where the names of the months are the same as those generated by the `asctime` function, and the first character of `dd` is a space character if the value is less than 10.

- `__FILE__`
  The presumed name of the current source file (a character string literal).

- `__LINE__`
  The presumed line number (within the current source file) of the current source line (an integer constant).

- `__STDC__`
  The integer constant **1**, intended to indicate a conforming implementation.

- `__STDC_HOSTED__`
  The integer constant **1** if the implementation is a hosted implementation, or the integer constant **0** if it is not.

- `__STDC_VERSION__`
  The integer constant `201112L`, indicating the version of the C standard supported.

- `__TIME__`
  The time of translation of the preprocessing translation unit as a character string literal of the form `"hh:mm:ss"` as in the time generated by the `asctime` function.

### Environment

The following macro names are conditionally defined by the implementation:

- `__STDC_ISO_10646__`
  An integer constant of the form `yyyymmL` (e.g., `201312L`).
  If defined, it indicates that values of type `wchar_t` are the coded representations of the characters defined by ISO/IEC 10646, along with all amendments and technical corrigenda as of the specified year and month.

- `__STDC_MB_MIGHT_NEQ_WC__`
  The integer constant **1**, defined if and only if the members of the basic character set might not have the same encoding in both the execution character set and the execution wide-character set.

- `__STDC_UTF_16__`
  The integer constant **1**, defined if and only if values of type `char16_t` are UTF-16 encoded.

- `__STDC_UTF_32__`
  The integer constant **1**, defined if and only if values of type `char32_t` are UTF-32 encoded.

### Conditional Features

The following macro names are conditionally defined by the implementation to indicate the presence or absence of certain features:

- `__STDC_ANALYZABLE__`
  The integer constant **1**, defined if and only if the implementation conforms to the specifications in Annex L (Analyzability).

- `__STDC_IEC_559__`
  The integer constant **1**, defined if and only if the implementation conforms to the IEC 60559 floating-point specification.

- `__STDC_IEC_559_COMPLEX__`
  The integer constant **1**, defined if and only if the implementation conforms to the IEC 60559-compatible complex arithmetic specification.

- `__STDC_LIB_EXT1__`
  The integer constant `201112L`, defined by the implementation if it supports the extensions defined in Annex K (Bounds-checking interfaces).

- `__STDC_NO_ATOMICS__`
  The integer constant **1**, defined if and only if the implementation does not support atomic types (including the `_Atomic` type qualifier) and the `<stdatomic.h>` header.

- `__STDC_NO_COMPLEX__`
  The integer constant **1**, defined if and only if the implementation does not support complex types and the `<complex.h>` header.

- `__STDC_NO_THREADS__`
  The integer constant **1**, defined if and only if the implementation does not support the `<threads.h>` header.

- `__STDC_NO_VLA__`
  The integer constant **1**, defined if and only if the implementation does not support variable length arrays.

## Pragma Operator

A unary operator expression of the form:
- `_Pragma`    `(`    _string-literal_    `)`

is processed as follows: The string literal is destringized by deleting the leading and trailing double-quotes, replacing each escape sequence `\"` by a double-quote, and replacing each escape sequence `\\` by a single backslash.
The resulting sequence of characters is processed through translation phase 3 to produce preprocessing tokens that are executed as if they were the pp-tokens in a pragma directive.

The original four preprocessing tokens in the unary operator expression are removed.
This operator allows pragma directives to be generated through macro expansion, since a pragma directive cannot be produced by normal macro expansion.
