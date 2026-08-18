<p align="center">
  <img src="./assets/Druim%20Logo%20Color.png" alt="Druim Logo" width="300">
</p>

# Druim

Druim is a deterministic, explicitly structured programming language under active development.

Its design emphasizes:

- Explicit structure over implicit behavior
- Deterministic parsing over convenience
- Clear token boundaries over inferred meaning
- Early diagnostics over silent coercion
- Intentional absence through `void`
- Distinct operators for distinct value relationships
- Explicit scope and binding lifetime
- Source-aware compiler diagnostics

This repository contains the reference compiler and evaluator implementation.

The authoritative definition of the language is maintained in:

[docs/druim-canon.md](./docs/druim-canon.md)

When the README, implementation, tests, comments, or prior discussion conflict with the canon, the canon is authoritative.

---

## Status

Druim is under active development.

The current Druim Canon defines stable language rules for:

- Source-file boundaries and `.drm` execution
- Single-line and multiline comments
- Comment restrictions inside function parameters and call arguments
- Lexical structure and longest-match rules
- Definition, mutation, copying, binding, and guarded selection
- Ordinary, local, global, and stone binding behavior
- Scope commitment through `loc Mutate` and `glo Mutate`
- Block and block-segment lifetime
- Function definitions, calls, defaults, returns, and scope modifiers
- Loop structure, persistent loop scope, nesting, and return propagation
- Truth evaluation
- Explicit type conversion expressions
- Text interpolation
- Print output
- Core functions
- Box and Bag collections
- Named and indexed traversal
- Missing-member behavior
- Invalid-selector diagnostics
- Command-line execution and source-aware diagnostics

Runtime semantics explicitly defined by the canon are authoritative. Additional type-system rules, library design, and undocumented behavior are still evolving.

Any behavior not defined by the canon should be treated as unsupported unless explicitly introduced by a later revision.

---

## Quick Start

Druim source files use the `.drm` extension and are enclosed by the canonical file boundary:

```druim
:-:-:

...program...

:-:-:
```

A complete Hello World program can define and call a function, interpolate a value into text, and print the result:

```druim
:-:-:

:{
    fn hello :()(
        name = "World";
        |> ("Hello :.name.:!");
    ):

    hello();
}:

:-:-:
```

Run a Druim source file with:

```text
druim hello.drm
```

Output:

```text
Hello World!
```

`|>` is line-oriented Print and appends a newline automatically.

---

## Canonical Source of Truth

The Druim Canon is versioned and revision-controlled.

Each revision supersedes:

- Earlier canon revisions
- Informal discussion
- Implementation drift
- Tests that encode obsolete behavior
- Source comments
- Personal recollection

This README is an overview. It does not independently define the language.

---

## Design Principles

Druim favors language forms that are structurally explicit and mechanically enforceable.

At the language level, Druim rejects:

- Ambiguous parsing
- Silent token consumption
- Undefined values
- Implicit fallback behavior
- Implicit scope requalification
- Selector coercion
- Invalid traversal being treated as missing data
- Definition being used as mutation

Every valid construct should be:

- Lexically unambiguous
- Syntactically complete
- Deterministically parsed
- Explicitly evaluable
- Diagnosable when invalid

---

## Compiler and Execution Pipeline

A `.drm` file executed by the Druim command follows this pipeline:

```text
source file
    ↓
lexer
    ↓
tokens
    ↓
file parser
    ↓
AST
    ↓
evaluator
    ↓
runtime output / diagnostics
```

Each stage has a distinct responsibility:

- The command-line entry point validates and reads the source file.
- The lexer recognizes atomic token forms and removes comments.
- The file parser validates complete `.drm` structure, including file boundaries.
- The AST represents parsed language constructs.
- The evaluator executes canonical runtime semantics.
- Diagnostics report invalid programs without silent recovery.

Complete source files use the file-level parser. Internal tests and tooling may use program/snippet parsing where appropriate.

---

## Command-Line Execution

Canonical invocation:

```text
druim <file.drm>
```

The Druim command:

- Accepts exactly one source-file path
- Requires the `.drm` extension
- Reports missing files explicitly
- Reports unreadable or invalid source files explicitly
- Exits successfully with status code `0`
- Exits non-zero on command-line, lexical, parser, or evaluator failure

Examples:

```text
druim hello.drm
druim C:\path\to\program.drm
```

---

## Source File Boundaries

Every complete `.drm` source file uses the same delimiter to open and close the program:

```druim
:-:-:

...program...

:-:-:
```

Rules:

- The first `:-:-:` opens the Druim program.
- The final `:-:-:` closes the Druim program.
- Executable source belongs between those boundaries.
- The opening boundary is the first structural token.
- The closing boundary is the final structural token.
- Nested or extra program boundaries are invalid.

The boundary is structural syntax, not a comment or runtime value.

---

## Comments

Comments are consumed by the lexer and do not appear in the AST.

### Single-Line Comments

```druim
:- this is a single-line comment -:
```

A single-line comment:

- Begins with `:-`
- Must end with `-:`
- May not cross a newline
- Produces a lexical diagnostic if its closing delimiter is missing

### Multiline Comments

```druim
:--
    this comment
    spans multiple lines
--:
```

A multiline comment:

- Begins with `:--`
- Must end with `--:`
- May span multiple lines
- Produces a lexical diagnostic if its closing delimiter is missing

Because the file boundary and comments share prefixes, longest-match rules require the lexer to recognize:

1. `:-:-:`
2. `:--`
3. `:-`

in the appropriate lexical context.

### Comment Placement Restrictions

Comments are not permitted anywhere inside a function parameter list.

From the opening `:(` through the parameter-closing `)(`, neither single-line nor multiline comment syntax is valid.

Comments are also not permitted anywhere inside a function call argument list.

From the opening `(` through its matching `)`, neither single-line nor multiline comment syntax is valid.

These restrictions are enforced lexically.

---

## Lexical Overview

Druim's lexer guarantees:

- Longest-match operator resolution
- Deterministic left-to-right token emission
- Atomic compound operators
- Explicit diagnostics for unexpected characters
- No token backtracking

### Identifiers

Identifiers:

- Use ASCII letters, digits, and `_`
- May begin with a digit
- Must contain at least one non-digit character

Examples:

```druim
a
user_name
9lives
123abc
123_456
```

All-digit sequences are numeric literals rather than identifiers.

### Numeric Literals

Druim supports integer and decimal literals.

```druim
0
42
123
0.5
12.34
```

Decimal literals require digits on both sides of the decimal point.

Invalid:

```druim
.
1.
.5
1..2
```

### Text Literals

Text literals are enclosed in double quotes:

```druim
"hello"
```

Unterminated text literals produce source-aware lexical diagnostics.

### Text Interpolation

Text literals may embed Druim expressions with `:.` and `.:`:

```druim
name = "Rusty";
message = "Hello :.name.:!";
```

The opening delimiter `:.` has interpolation meaning only inside a text literal. Normal Druim expression syntax applies until the matching `.:`.

Full expressions may be interpolated:

```druim
price = 10;
tax = 2;
total = "Total: :.price + tax.:";
```

Periods remain ordinary text unless they participate in the exact closing delimiter while interpolation is open:

```druim
version = "Version 1.2 is current.";
sentence = "Hello :.name.:. Welcome back.";
```

Canonical text conversion currently supports:

- `num`
- `dec`
- `flag`
- `text`
- `void`

Box, Bag, and function values do not yet have canonical interpolation text forms and produce a diagnostic if interpolated.

### Loop Structural Tokens

Druim loops use three lexically atomic structural tokens:

| Token | Name | Purpose |
|---|---|---|
| `:<` | LoopStart | Begins a loop |
| `>?<` | LoopSplit | Separates loop sections |
| `>:` | LoopEnd | Ends a loop |

A valid loop contains exactly two identical `>?<` separators. Their meaning is determined by position.

---

## Statement Boundaries

Druim statements must consume exactly one complete statement form.

Most statements end with `;`.

The `=;` operator is lexically atomic and includes its own terminator.

Valid:

```druim
a = 12;
b := a;
c :> a;
d ?= first : second;
count << count + 1;
|> ("Hello World!");
empty =;
```

Invalid:

```druim
a = 12 13;
a := b c;
a :> b :> c;
a ?= x : y z;
a << 12 13;
```

The parser may not accept a valid prefix while silently ignoring unexpected tokens that remain in the same statement.

---

## Statement Operators

Druim uses separate operators for separate value relationships.

| Operator | Name | Purpose |
|---|---|---|
| `=` | Define | Evaluates one complete expression and defines a new target |
| `=;` | DefineEmpty | Defines a new target as `void` |
| `:=` | Copy | Copies the current value of an existing identifier into a fresh identity |
| `:>` | Bind | Creates shared identity with an existing identifier |
| `?=` | Guard | Selects the first truthy branch or defines `void` |
| `<<` | Mutate | Changes the value held by an existing mutable identity |
| `|>` | Print | Emits one expression as line-oriented output |

Statement operators cannot be chained inside one statement.

### Define

```druim
a = 12;
b = 12 + 13;
c = user::profile;
```

Define creates a **new** target binding.

The target name must not already be visible. Define does not redefine or mutate an existing binding.

A bare identifier should use Copy or Bind when those forms express the intended relationship.

### DefineEmpty

```druim
value =;
```

Equivalent in meaning to:

```druim
value = void;
```

DefineEmpty also requires a new target name.

### Mutate

```druim
count = 10;
count << count + 1;
```

Mutate changes the value of an existing mutable binding identity.

It:

- Requires an existing target
- Evaluates exactly one complete expression
- Never creates a normal binding
- Cannot mutate a stone identity

Because Bind shares identity, mutation through any alias is visible through every name bound to that identity.

### Copy

```druim
snapshot := source;
```

Copy creates a fresh, independent identity containing the current value of `source`.

Future mutations of `source` do not affect `snapshot`.

### Bind

```druim
alias :> source;
```

Bind makes both names refer to the same underlying identity.

Future mutations through either name are visible through the other.

An alias may not be established with a lifetime longer than the identity it aliases.

### Guard

```druim
selected ?= primary : secondary : fallback;
```

Guard evaluates branches from left to right. The first branch whose value converts to `true` becomes the target value.

If no branch converts to `true`, the target becomes `void`.

Every written branch is conditional. The final written branch is not an unconditional fallback.

Guard defines a new target and therefore cannot reuse an already-visible target name.

### Print

```druim
|> ("Hello World!");
```

Print syntax is:

```druim
|> (expression);
```

The parentheses are part of Print syntax; they are not ordinary expression-grouping parentheses.

Print:

- Evaluates one expression
- Converts the supported value to text
- Emits that text
- Appends a newline automatically

Interpolation composes directly with Print:

```druim
name = "World";
|> ("Hello :.name.:!");
```

Canonical printable values currently include `num`, `dec`, `flag`, `text`, and `void`.

---

## Scope Modifiers

Druim has two explicit scope modifiers:

- `loc`
- `glo`

They are mutually exclusive.

Ordinary statement modifier order is:

```text
[stone] [loc | glo] statement
```

Valid examples:

```druim
loc value = 10;
glo value = 10;
stone value = 10;
stone loc value = 10;
stone glo value = 10;
```

Invalid ordering:

```druim
loc stone value = 10;
glo stone value = 10;
loc glo value = 10;
glo loc value = 10;
```

### `loc`

For target-defining forms, `loc` establishes a new target with the applicable local lifetime.

A local target **may not shadow or reuse an already-visible name**.

```druim
source = 10;

:{
    loc snapshot := source;
}:
```

The source may resolve from an enclosing scope, but `snapshot` must be a new name.

`loc Mutate` is the intentional localization exception:

```druim
count = 10;

:{
    loc count << count + 3;
}:
```

The visible identity is localized for the applicable local lifetime, and mutation does not escape that boundary.

### `glo`

`glo` establishes a new target in global scope regardless of where the statement executes.

```druim
:{
    glo value = 10;
}:
```

The target survives the local structure.

`glo` changes target placement, not source lookup. A local value may therefore be copied into a fresh global identity:

```druim
:{
    local_source = 10;
    glo snapshot := local_source;
}:
```

For an ordinary existing identity, `glo Mutate` commits that identity to global scope.

```druim
value = 2;

:{
    glo value << value + 2;
}:
```

After the mutation, `value` is globally committed and contains `4`.

Scope commitment is directional:

- Ordinary → Local through `loc Mutate`
- Ordinary → Global through `glo Mutate`
- Local → Global is invalid
- Global → Local is invalid
- Reapplying the same scope commitment is valid unless another rule prevents the mutation

---

## Stone Bindings

`stone` marks a binding identity immutable.

```druim
stone source = 10;
```

Mutation is then invalid:

```druim
source << 20;
```

Stone belongs to the underlying identity.

Because Bind shares identity:

```druim
stone source = 10;
alias :> source;
```

both names refer to the same stone identity.

Copy creates a new identity, so stone does not propagate automatically:

```druim
stone source = 10;
copy := source;
copy << 20;
```

An explicitly immutable copy is:

```druim
stone copy := source;
```

---

## Truth Evaluation

Druim does not use implicit C- or JavaScript-style truthiness.

Truth conversion is explicitly defined by type.

| Value | Result |
|---|---|
| `flag(true)` | `true` |
| `flag(false)` | `false` |
| `0` | `false` |
| Non-zero `num` | `true` |
| `0.0` | `false` |
| Non-zero `dec` | `true` |
| Empty or whitespace-only `text` | `false` |
| Other `text` | `true` |
| `void` | `false` |
| Empty Box | `false` |
| Non-empty Box | `true` |
| Empty Bag | `false` |
| Non-empty Bag | `true` |

Only values with canonical truth-conversion rules may participate in truth evaluation.

---

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

Druim does not perform implicit coercion. Conversion must be explicit.

Converting a value to its existing type returns that value unchanged.

### `num(expression)`

`num(...)` converts a supported value to `num`.

When converting from `dec`, Druim rounds to the nearest integer. Exact `.5` ties round away from zero.

Numeric text conversion is strict:

- An optional sign is allowed
- At least one digit is required
- A decimal point, when present, must be followed by at least one digit
- Surrounding whitespace is not trimmed
- Exponent notation is not accepted
- Partial parsing is not accepted

### `dec(expression)`

`dec(...)` converts a supported value to `dec`.

Text conversion follows the same strict numeric-text rules.

### `text(expression)`

`text(...)` converts a supported value using Druim's canonical textual representation.

`void` converts to:

```text
void
```

Box, Bag, function, and Core-function values are not serialized by `text(...)`.

### `flag(expression)`

`flag(...)` applies Druim's canonical truth conversion.

`void` converts to `false`.

Boxes and Bags may be converted to `flag` according to whether they are empty or non-empty.

Function and Core-function values cannot be converted to `flag`.

`void` cannot be converted to `num` or `dec`.

---

## `void` and Undefined Values

`void` represents intentional absence.

It is a defined Druim value, not an error state.

```druim
empty =;
```

When explicitly evaluated as a flag, `void` becomes `false`.

Druim has no undefined value. Referencing an undeclared identifier must produce a diagnostic.

---

## Blocks and Scope

Druim blocks exist to establish lexical scope.

A block chain uses:

- `:{` to begin a block scope
- `}{` to continue the same block chain
- `}:` to end the block scope

Example:

```druim
:{
    a = 1;
}{
    b = 2;
}:
```

A block chain creates one lexical scope, but `loc` lifetimes are segment-local.

A block segment is the source region between `:{` or `}{` and the next `}{` or `}:`.

Blocks:

- Do not evaluate to values
- Cannot be nested
- May contain statements and function definitions
- Preserve ordinary bindings across chained segments
- Discard segment-local targets and localized identity state when crossing `}{`

---

## Functions

A function definition produces a callable value.

```druim
fn multiply :(left, right)(
    ret left * right;
):
```

A valid function definition:

- Uses `fn`
- Uses a `snake_case` identifier
- Contains exactly one parameter block
- Contains exactly one body block
- Defines parameters in function-local scope
- May not reuse an already-visible function name
- May not be nested inside another function

Parameters may be plain identifiers or may include defaults using `=` without a semicolon:

```druim
fn scale :(value, factor = 2)(
    ret value * factor;
):
```

Required and defaulted parameters may appear in any order.

Comments are not permitted inside the parameter list. The restriction begins at `:(` and ends at the parameter-closing `)(`.

Function calls use parentheses and comma-separated expressions:

```druim
scale(12)
scale(12, 4)
```

Comments are not permitted inside a function call argument list. The restriction begins at the call's opening `(` and ends at its matching `)`.

If a function completes without `ret`, it returns `void`.

### Function Scope Modifiers

Valid function forms are:

```druim
fn helper :()(ret 1;):

loc fn helper :()(ret 1;):

glo fn helper :()(ret 1;):
```

- `fn` establishes the function in the ordinary executing scope.
- `loc fn` gives the function binding the applicable local lifetime.
- `glo fn` establishes the function binding globally.

`stone` does not apply to function definitions because functions are already structurally non-mutable definitions.

Invalid:

```druim
stone fn helper :()(ret 1;):
stone loc fn helper :()(ret 1;):
stone glo fn helper :()(ret 1;):
```

---

## Loops

A Druim loop is a structural statement with three ordered sections:

```druim
:< setup
>?< condition
>?< process
>:
```

The sections are:

- **setup** — zero or more valid statements or nested loops, executed once
- **condition** — exactly one complete expression, evaluated before every iteration
- **process** — zero or more valid statements or nested loops, executed while the condition evaluates to `true`

Example:

```druim
:< index = 0;
>?< index < 3
>?< index << index + 1;
>:
```

### Structural Rules

A valid loop:

- Begins with exactly one `:<`
- Contains exactly two `>?<` separators
- Ends with exactly one `>:`
- Contains exactly one complete condition expression
- Is a statement and does not evaluate to a value

### Execution and Scope

Each loop creates exactly one persistent lexical scope for the entire loop execution.

- Setup executes once.
- The condition runs before every iteration.
- The same loop scope remains active across iterations.
- Loop-owned bindings disappear on exit.
- Nested loops create persistent child scopes.

Target-defining operators such as Define, DefineEmpty, Copy, Bind, and Guard still require a **new target name** inside loops.

To change an already-visible binding, use Mutate:

```druim
count = 0;

:<
>?< count < 3
>?< count << count + 1;
>:
```

`loc Mutate` localizes an outer identity into the loop scope instead of mutating the outer identity.

A `ret` inside a loop propagates to the enclosing function after the loop scope is removed.

---

## Core Functions

Core functions are callable operations supplied directly by Druim.

They use normal function-call syntax but are distinct from source-defined functions and from type conversion expressions.

The current canonical text Core functions are:

- `rise(text)` — converts text to uppercase
- `fall(text)` — converts text to lowercase
- `cut(text, start, [end])` — returns a character-based substring
- `size(text)` — returns the character count
- `fuse(text, text, ...)` — concatenates two or more text values
- `cap(text)` — uppercases the first character and preserves the remainder

Core functions perform no implicit coercion.

Text concatenation uses `fuse`; arithmetic `+` remains arithmetic-only.

---

## Collections

Druim currently defines two canonical collection types:

- **Box** — ordered and indexed
- **Bag** — named and unordered

Both may contain arbitrary Druim values, including nested Box and Bag values.

---

## Box

A Box is an ordered, zero-indexed collection.

### Declaration

```druim
numbers = :[
    10,
    20,
    30
]:;
```

Rules:

- Values are separated by commas.
- A Box may contain zero or more values.
- A trailing comma is not permitted.
- Each value must be one complete expression.
- Insertion order is preserved.
- Duplicate values are allowed.

### Indexed Traversal

Get retrieves a value by index:

```druim
first = numbers::[0];
```

Has checks whether an index exists:

```druim
has_third = numbers:?[2];
```

Box index expressions must evaluate to a non-negative `num`.

- Existing index with Get → contained value
- Missing valid index with Get → `void`
- Existing index with Has → `true`
- Missing valid index with Has → `false`
- Negative index → diagnostic
- Non-numeric index → diagnostic
- Named selector against a Box → diagnostic

---

## Bag

A Bag is a named collection with no exposed positional ordering.

### Declaration

```druim
player = :|
    name: "Rusty",
    level: 42,
    active: true
|:;
```

Rules:

- Entries are separated by commas.
- A Bag may contain zero or more entries.
- A trailing comma is not permitted.
- Each entry uses `name: expression`.
- Entry names must be unique.
- Entry order is not preserved or exposed.

### Named Traversal

Get retrieves a named entry:

```druim
name = player::name;
```

Has checks whether a named entry exists:

```druim
has_level = player:?level;
```

- Existing name with Get → contained value
- Missing valid name with Get → `void`
- Existing name with Has → `true`
- Missing valid name with Has → `false`
- Indexed selector against a Bag → diagnostic

---

## Nested Traversal

Box and Bag values may be nested arbitrarily.

```druim
world = :|
    player: :|
        name: "Rusty"
    |:,
    inventory: :[
        "Sword",
        "Shield"
    ]:
|:;
```

Examples:

```druim
player_name = world::player::name;
first_item = world::inventory::[0];
has_second_item = world::inventory:?[1];
```

Traversal evaluates left to right.

Get may continue through retrieved container values. Has is terminal because it evaluates to a `flag`.

---

## Get and Has

Druim uses one traversal model for both named and indexed containers.

### Get (`::`)

Get retrieves a member:

```druim
container::member
container::[index]
```

A valid selector that does not identify an existing member evaluates to `void`.

### Has (`:?`)

Has checks whether a member exists:

```druim
container:?member
container:?[index]
```

A valid selector that does not identify an existing member evaluates to `false`.

### Missing Data vs. Invalid Access

Druim distinguishes absence from misuse.

- Missing valid member → `void` or `false`
- Unsupported selector form → diagnostic
- Invalid selector value → diagnostic

Missing data is non-fatal. Invalid traversal is not silently converted into absence.

---

## Logical Operators

Druim logical operators are:

- `&&` — AND
- `||` — OR
- `!` — NOT

Bare `&` and `|` are invalid tokens.

---

## Arithmetic and Comparison

Arithmetic operators:

```text
+  -  *  /  %
```

Comparison operators:

```text
==  !=  <  <=  >  >=
```

Compound operators, including `<<`, are matched before their single-character prefixes.

---

## Diagnostics

Druim favors early, explicit, source-aware diagnostics.

Lexer, parser, and evaluator errors use the shared diagnostic-rendering system when the failure refers to source code.

A source diagnostic identifies:

- The error
- The source line
- The relevant span
- A caret marker
- Help text when applicable

Example:

```text
error: unterminated text literal
--> line 3, column 11
 |
3 | message = "Hello World!
 |           ^
help: Druim expected `"` to close this text literal.
```

Canonical lexical diagnostic categories currently include:

- Unexpected characters
- Unterminated text literals
- Unterminated text interpolation
- Unterminated single-line comments
- Unterminated multiline comments
- Comments used inside function parameter or argument lists

Parser and evaluator failures use the same rendering system.

Command-line failures that occur before source compilation—such as a missing argument, invalid extension, or missing path—are reported directly because no source span exists yet.

Druim does not expose raw Rust debug representations as its normal user-facing error format.

---

## Development Discipline

Language changes should be made deliberately across the affected layers.

Depending on the feature, that may include:

1. Tokens and lexer rules
2. AST representation
3. Parser behavior
4. Evaluator/runtime behavior
5. Canon
6. Tests
7. User-facing documentation

The canon defines intended behavior. The implementation, tests, and documentation must converge on it.

---

## Final Note

Druim is designed for deterministic structure, explicit intent, and inspectable behavior.

The language prefers one clear meaning over multiple convenient interpretations.
