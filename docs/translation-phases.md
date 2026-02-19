# Translation Phases

This text is lifted from the freely available April 12, 2011 ISO Committee Draft of the C language.

The precedence among the syntax rules of translation is specified by the following phases.

## 1.

Physical source file multibyte characters are mapped, in an implementation-defined manner, to the source character set (introducing new-line characters for end-of-line indicators) if necessary.
Trigraph sequences are replaced by corresponding single-character internal representations.

## 2.

Each instance of a backslash character (`\`) immediately followed by a new-line character is deleted, splicing physical source lines to form logical source lines.
Only the last backslash on any physical source line shall be eligible for being part of such a splice.
A source file that is not empty shall end in a new-line character, which shall not be immediately preceded by a backslash character before any such splicing takes place.

## 3.

The source file is decomposed into preprocessing tokens and sequences of white-space characters (including comments).
A source file shall not end in a partial preprocessing token or in a partial comment.
Each comment is replaced by one space character.
New-line characters are retained.
Whether each nonempty sequence of white-space characters other than new-line is retained or replaced by one space character is implementation-defined.

## 4.

Preprocessing directives are executed, macro invocations are expanded, and `_Pragma` unary operator expressions are executed.
If a character sequence that matches the syntax of a universal character name is produced by token concatenation, the behavior is undefined.
A `#include` preprocessing directive causes the named header or source file to be processed from phase 1 through phase 4, recursively.
All preprocessing directives are then deleted.

## 5.

Each source character set member and escape sequence in character constants and string literals is converted to the corresponding member of the execution character set; if there is no corresponding member, it is converted to an implementation-defined member other than the null (wide) character.

## 6.

Adjacent string literal tokens are concatenated.

## 7.

White-space characters separating tokens are no longer significant.
Each preprocessing token is converted into a token.
The resulting tokens are syntactically and semantically analyzed and translated as a translation unit.

## 8.

All external object and function references are resolved.
Library components are linked to satisfy external references to functions and objects not defined in the current translation.
All such translator output is collected into a program image which contains information needed for execution in its execution environment.
