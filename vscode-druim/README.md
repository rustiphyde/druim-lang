# Druim Language Support

Official Visual Studio Code language support for **Druim**, an actively developed programming language built around explicit structure, deterministic parsing, clear token boundaries, and intentional state semantics.

This extension registers Druim as a language in Visual Studio Code and provides syntax highlighting, structural editing, completions, hover documentation, signature help, folding, navigation, snippets, comments, explicit conversion-expression support, and the included Druim color theme.

> **Current extension version:** `0.1.4`  
> **Language file extension:** `.drm`

## Features

### Language Recognition

Files ending in `.drm` are recognized by Visual Studio Code as **Druim**.

The extension registers the language ID:

```text
druim
```

### Syntax Highlighting

Druim syntax highlighting covers the language's major lexical and structural forms, including:

- Source boundaries
- Blocks and block continuations
- Loops
- Functions
- Boxes
- Bags
- Comments
- Text interpolation
- Keywords
- Scope and identity modifiers
- Types
- Literals
- Explicit conversion expressions
- Core functions
- Statement operators
- Traversal operators
- Arithmetic operators
- Comparison operators
- Logical operators

### Structural Editing

The extension understands Druim's structural delimiters and assists with creating and removing them.

#### Boxes

Typing:

```druim
:[
```

creates an empty inline Box:

```druim
:[]:
```

with the cursor inside the collection.

Pressing **Enter** from the empty collection expands it:

```druim
:[

]:
```

Backspacing out of the untouched empty structure also removes the generated closing delimiter.

#### Bags

Typing:

```druim
:|
```

creates:

```druim
:||:
```

Pressing **Enter** expands the empty Bag:

```druim
:|

|:
```

Backspacing out of the untouched empty structure removes the generated closing delimiter.

#### Functions

Typing a Druim function structural opener uses VS Code's parenthesis pairing and expands the function into Druim's parameter/body form:

```druim
fn example :()(

):
```

The cursor is placed in the parameter section.

Backspacing out of an untouched generated function removes the generated remainder of the function structure.

#### Loops

Typing:

```druim
:<
```

generates Druim's three-section loop structure:

```druim
:<

>?<

>?<

>:
```

The sections are:

1. setup
2. condition
3. process

Backspacing from the untouched first setup position removes the generated loop structure.

#### Blocks

Typing a block opener at the required structural position generates:

```druim
:{

}:
```

Block continuation uses:

```druim
}{
```

A continuation remains part of the **same lexical block scope** rather than creating a nested block.

### Comments

Druim uses explicit opening and closing comment delimiters.

Single-line comment:

```druim
:- comment -:
```

Multiline comment:

```druim
:--
    comment
--:
```

The extension supports automatic comment construction and demotion between empty multiline and single-line forms.

Comments are not permitted inside function parameter lists or function call argument lists. The extension prevents comment construction in those regions, including through the standard comment command.

The standard VS Code comment shortcut is supported:

```text
Ctrl+/
```

through the command:

```text
Druim: Toggle Line Comment
```

### Code Completion

Completion items are provided for:

- Druim keywords
- Types
- Flag literals
- Explicit conversion expressions
- Core functions
- Statement operators
- Traversal operators
- Arithmetic operators
- Comparison operators
- Logical operators

Core functions insert a normal call form with the cursor inside the argument list.

### Hover Documentation

Hovering Druim syntax displays contextual documentation.

Richer hover documentation is currently available for conversion expressions, Core functions, traversal syntax, user-defined functions, parameters, and bindings, including relevant information such as:

- Signature
- Description
- Parameters
- Return type
- Behavioral rules
- Examples
- Diagnostics

### Signature Help

Signature help is available while writing:

- Druim conversion expressions
- Druim Core function calls
- User-defined Druim function calls

Core function signature help includes parameter descriptions, return information, behavioral notes, and examples.

### Go to Definition

Ctrl+Click / Go to Definition supports:

- User-defined functions
- Function parameters
- Druim bindings

Resolution accounts for Druim scope behavior rather than treating every matching identifier as a global symbol.

### Folding

Folding is supported for Druim structures including:

- Blocks
- Block segments
- Loops
- Loop sections
- Functions
- Boxes
- Bags
- Multiline comments

### Druim Theme

The extension includes the **Druim** dark color theme.

It is designed around the language's syntax categories, including distinct treatment for built-in Core functions.

To enable it:

1. Open the Command Palette.
2. Choose **Preferences: Color Theme**.
3. Select **Druim**.

---

# Druim Language Quick Reference

Druim is under active development. This section documents the currently established syntax represented by the language tooling.

## Source Boundary

A complete Druim source file is bounded by:

```druim
:-:-:

:-:-:
```

The same token opens and closes the source.

## Statement Terminator

Statements use:

```druim
;
```

Example:

```druim
value = 10;
```

## Bindings and State

### Define — `=`

Defines a new binding from an expression.

```druim
value = 10;
```

`=` establishes a binding; it is not mutation.

### Define Empty — `=;`

Defines a binding without an initial value expression.

```druim
value =;
```

### Mutate — `<<`

Changes the value stored in an existing visible binding identity.

```druim
value << 20;
```

### Copy — `:=`

Creates an independent binding from the current value of another binding.

```druim
copy := original;
```

### Bind — `:>`

Creates another name that shares the same underlying binding identity.

```druim
alias :> original;
```

### Guard — `?=`

Defines a target using guarded candidate expressions.

Candidate branches are separated by standalone `:` delimiters.

```druim
result ?= first : second : fallback;
```

Candidates are considered from left to right.

## Scope and Identity Modifiers

### `loc`

Applies local scope behavior.

```druim
loc value = 10;
```

### `glo`

Targets global scope.

```druim
glo value = 10;
```

### `stone`

Marks the resulting binding identity as immutable.

```druim
stone value = 10;
```

For ordinary modified statements, canonical modifier order is:

```text
stone → scope modifier → statement
```

Example:

```druim
stone loc value = 10;
```

`loc` and `glo` are mutually exclusive.

## Types

Druim currently defines these type keywords:

| Type | Meaning |
|---|---|
| `num` | Whole-number numeric type |
| `dec` | Decimal numeric type |
| `flag` | Boolean flag type |
| `text` | Text value type |
| `void` | Absence of a value |

## Explicit Type Conversion Expressions

Druim provides four dedicated conversion expressions:

```druim
num(expression)
dec(expression)
text(expression)
flag(expression)
```

These are language-level conversion expressions, not Core functions.

`void(...)` is invalid.

Druim does not perform implicit coercion. Conversion is explicit, and converting a value to its existing type returns the value unchanged.

### `num(expression)`

Converts a supported value to `num`.

When converting a `dec`, Druim rounds to the nearest integer. Exact `.5` ties round away from zero.

Numeric text must be strict: an optional sign, at least one digit, and no surrounding whitespace, exponent notation, or partial parsing.

### `dec(expression)`

Converts a supported value to `dec`.

Decimal text must be strict: an optional sign, at least one digit, and an optional decimal point followed by at least one digit.

### `text(expression)`

Converts a supported value to its canonical Druim text representation.

`void` converts to:

```text
void
```

### `flag(expression)`

Converts a supported value to its canonical truth value.

`void` converts to:

```druim
false
```

Boxes and Bags may be converted to `flag`.

Function values and Core functions are rejected as conversion inputs.

`void` cannot be converted to `num` or `dec`.

## Flag Literals

```druim
true
false
```

## Numeric Literals

Whole numbers:

```druim
0
42
123
```

Decimals require digits on both sides of the decimal point:

```druim
0.5
12.34
1.0
```

Forms such as these are not valid decimal literals:

```text
.5
1.
1..2
```

## Text Literals

Text literals use double quotes:

```druim
message = "Hello, Druim";
```

## Text Interpolation

A Druim expression can be embedded into text using:

```druim
:. expression .:
```

Example form:

```druim
message = "Value: :. value .:";
```

## Identifiers

Druim identifiers may contain ASCII letters, digits, and underscores.

Unlike many languages, an identifier may begin with a digit **as long as the complete identifier contains at least one non-digit character**.

Valid forms include:

```text
value
value2
1value
9lives
123abc
123_456
_thing
```

An all-digit sequence is a numeric literal rather than an identifier:

```text
123
```

---

# Structures

## Blocks

A block uses:

```druim
:{

}:
```

A continuation uses:

```druim
}{
```

Example shape:

```druim
:{
    first = 1;
}{
    second = 2;
}:
```

`}{` continues the **same lexical block scope**.

Nested blocks are not part of the current canonical block model.

## Loops

A loop has exactly three structural sections:

```druim
:<
    setup
>?<
    condition
>?<
    process
>:
```

The separators are:

```text
>?<
```

## Functions

A function declaration uses:

```druim
fn name :(parameters)(
    body
):
```

Example:

```druim
fn double :(value)(
    ret value * 2;
):
```

Parameters are parameter forms, not restricted to bare identifiers; parameter defaults are supported by the language model.

Comments are not permitted anywhere inside a function parameter list. From the opening `:(` through the parameter-closing `)(`, neither single-line (`:- ... -:`) nor multiline (`:-- ... --:`) comment syntax is valid.

### Return

Use:

```druim
ret expression;
```

inside a function.

## Function Calls

A function call uses:

```druim
name(arguments)
```

Comments are not permitted anywhere inside a function call argument list. From the opening `(` through its matching `)`, neither single-line (`:- ... -:`) nor multiline (`:-- ... --:`) comment syntax is valid.

## Boxes

A Box is an ordered collection:

```druim
values = :[
    10,
    20,
    30
]:;
```

Boxes use indexed traversal.

Indexes are zero-based.

## Bags

A Bag is a named collection:

```druim
player = :|
    name: "Aryn",
    score: 10
|:;
```

Bags use named traversal.

---

# Traversal

Druim distinguishes retrieval from existence testing.

## Get — `::`

Retrieves a member from a traversable value.

### Bag member

```druim
player::name
```

### Box index

```druim
values::[0]
```

### Text index

```druim
word::[0]
```

For a **valid selector** whose member does not exist, Get evaluates to:

```druim
void
```

Get may be chained when the retrieved value is itself traversable.

Example:

```druim
player::inventory::[0]
```

## Has — `:?`

Tests whether a valid member exists and returns a `flag`.

### Named member

```druim
player:?name
```

### Indexed member

```druim
values:?[0]
```

### Text index

```druim
word:?[0]
```

A valid missing member or out-of-range index returns:

```druim
false
```

Has is terminal because its result is a flag.

It may follow a Get traversal:

```druim
player::inventory:?[2]
```

## Indexed Selectors

Indexed traversal uses:

```druim
[index]
```

The index must be a non-negative `num`.

Boxes and text support indexed selectors.

Bags do not.

Text indexing is character-based rather than byte-based.

Invalid selector forms produce a diagnostic rather than being treated as a normal missing member.

---

# Core Functions

Core functions are callable operations supplied directly by Druim.

## `rise(text)`

Converts all characters in a text value to uppercase.

```druim
result = rise("Druim");
```

Result:

```text
DRUIM
```

## `fall(text)`

Converts all characters in a text value to lowercase.

```druim
result = fall("Druim");
```

Result:

```text
druim
```

## `cap(text)`

Uppercases the first Unicode character of a text value and leaves the remainder unchanged.

```druim
result = cap("druim");
```

Result:

```text
Druim
```

An empty text value remains empty.

## `cut(text, start, [end])`

Returns a substring of a text value.

- `start` is inclusive.
- `end` is optional.
- `end` is exclusive.

Example:

```druim
result = cut("Druim", 1, 4);
```

Result:

```text
rui
```

## `size(text)`

Returns the character count of a text value.

```druim
result = size("Druim");
```

Result:

```text
5
```

Text length is based on characters rather than raw byte length.

## `fuse(text, text, ...)`

Concatenates two or more text values.

```druim
result = fuse("Dru", "im");
```

Result:

```text
Druim
```

`fuse` requires at least two text arguments.

Text concatenation is intentionally handled by `fuse`; arithmetic `+` is not the text concatenation operator.

---

# Operators

## Arithmetic

```text
+   addition
-   subtraction / unary negation where valid
*   multiplication
/   division
%   modulo
```

`+` is arithmetic-only.

## Comparison

```text
==  equal
!=  not equal
<   less than
<=  less than or equal
>   greater than
>=  greater than or equal
```

## Logical

```text
&&  logical AND
||  logical OR
!   logical NOT
```

## Print — `|>`

Prints an expression using Druim's text conversion and appends a newline.

```druim
|> ("Hello, Druim");
```

---

# Editor Commands

## Toggle Comment

Command:

```text
Druim: Toggle Line Comment
```

Default keybinding:

```text
Ctrl+/
```

The command understands Druim's explicit comment delimiters rather than applying JavaScript/C-style comments.

The command does nothing when the selection begins or ends inside a function parameter list or function call argument list, because comments are invalid in those regions.

---

# Installation

## Visual Studio Marketplace

Once published, install **Druim Language Support** from the Visual Studio Marketplace or directly from the Extensions view inside Visual Studio Code.

Search for:

```text
Druim Language Support
```

## Install from VSIX

A packaged `.vsix` can also be installed manually:

1. Open Visual Studio Code.
2. Open the Extensions view.
3. Open the `...` menu.
4. Choose **Install from VSIX...**
5. Select the Druim `.vsix` package.

---

# Development

Repository:

https://github.com/rustiphyde/druim-lang

The VS Code extension lives in the Druim project under:

```text
vscode-druim/
```

To package the extension locally:

```bash
vsce package
```

---

# Release Status

Druim and its editor tooling are under active development.

The `0.1.x` extension series should be considered an early public language-support release. Existing tooling is usable, but additional documentation, diagnostics, semantic awareness, editor assistance, and language features will continue to arrive in later versions.

Extension releases can evolve independently while remaining compatible with the registered `druim` language mode.

---

# License

MIT License

Copyright (c) 2026 Rusty Hoppins

See the included `LICENSE` file for the full license text.
