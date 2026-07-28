<p align="center">
  <img src="./assets/Druim%20Logo%20Color%2064x64.png" alt="Druim Logo" width="300">
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

This repository contains the reference compiler implementation.

The authoritative definition of the language is maintained in:

[docs/druim-canon.md](./docs/druim-canon.md)

When the README, implementation, tests, comments, or prior discussion conflict with the canon, the canon is authoritative.

---

## Status

Druim is under active development.

The current canonical revision defines stable language rules for:

- Lexical structure
- Statement boundaries
- Definition, copying, binding, and guarded selection
- Blocks and scope
- Function structure
- Truth evaluation
- Box and Bag collections
- Named and indexed traversal
- Missing-member behavior
- Invalid-selector diagnostics

Runtime semantics explicitly defined by the canon are authoritative. Type-system rules, library design, and undocumented runtime behavior are still evolving.

Any behavior not defined by the canon should be treated as unsupported unless explicitly introduced by a later revision.

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
- Implicit scope creation
- Selector coercion
- Invalid traversal being treated as missing data

Every valid construct should be:

- Lexically unambiguous
- Syntactically complete
- Deterministically parsed
- Explicitly evaluable
- Diagnosable when invalid

---

## Compiler Pipeline

The reference implementation is organized as a staged compiler and evaluator pipeline:

```text
token → lexer → parser → AST → evaluator → diagnostics
```

Each stage has a distinct responsibility:

- The lexer recognizes atomic token forms.
- The parser validates complete structural forms.
- The AST represents parsed language constructs.
- The evaluator executes canonical runtime semantics.
- Diagnostics report invalid programs without silent recovery.

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

Druim supports whole-number and decimal literals.

```druim
0
42
123
0.5
12.34
```

Decimal literals require digits on both sides of the decimal point.

Invalid forms include:

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

Unterminated text literals produce lexical diagnostics.

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
empty =;
```

Invalid:

```druim
a = 12 13;
a := b c;
a :> b :> c;
a ?= x : y z;
```

The parser may not accept a valid prefix while silently ignoring unexpected tokens that remain in the same statement.

---

## Statement Operators

Druim uses separate operators for separate value relationships.

| Operator | Name | Purpose |
|---|---|---|
| `=` | Define | Evaluates one complete expression and defines the target |
| `=;` | DefineEmpty | Defines the target as `void` |
| `:=` | Copy | Copies the current value of an existing identifier |
| `:>` | Bind | Creates shared identity with an existing identifier |
| `?=` | Guard | Selects the first truthy branch or defines `void` |

Statement operators cannot be chained inside one statement.

### Define

```druim
a = 12;
b = 12 + 13;
c = user::profile;
```

The left-hand side must be one identifier. The right-hand side must be one complete expression.

A bare identifier should use Copy or Bind when those forms express the intended relationship.

### DefineEmpty

```druim
value =;
```

This is equivalent to:

```druim
value = void;
```

### Copy

```druim
snapshot := source;
```

Copy takes the current resolved value of `source` and gives `snapshot` an independent value.

Future changes to `source` do not affect `snapshot`.

### Bind

```druim
alias :> source;
```

Bind makes both identifiers refer to the same underlying identity.

Future changes through either name are visible through the other.

### Guard

```druim
selected ?= primary : secondary : fallback;
```

Guard evaluates branches from left to right. The first branch whose value converts to `true` becomes the target value.

If no branch converts to `true`, the target becomes `void`.

Every written branch is conditional. The final written branch is not an unconditional fallback.

---

## Truth Evaluation

Druim does not use implicit C- or JavaScript-style truthiness.

Truth conversion is explicit and defined by type.

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

## `void` and Undefined Values

`void` represents intentional absence.

It is a defined Druim value, not an error state.

```druim
empty =;
```

When explicitly evaluated as a flag, `void` becomes `false`.

Druim has no undefined value. Referencing an undeclared or uninitialized identifier must produce a diagnostic.

---

## Blocks and Scope

Druim blocks exist to establish lexical scope.

A block chain uses:

- `:{` to begin a block scope
- `}{` to continue the same block scope
- `}:` to end the block scope

Example:

```druim
:{
    a = 1;
}{
    b = 2;
}:
```

A block chain creates exactly one lexical scope.

Blocks:

- Do not evaluate to values
- Do not create new scopes between chained segments
- Cannot be nested
- Exist only to control visibility and lifetime

The `loc` modifier may restrict a binding to a single block segment where supported.

---

## Functions

A function definition produces a callable value.

```druim
fn multiply :(left, right)(
    ret left * right;
):
```

A valid function definition:

- Uses the `fn` keyword
- Uses a snake_case identifier
- Contains exactly one parameter block
- Contains exactly one body block
- Defines parameters in function-local scope

Parameters may be plain identifiers or valid parameter forms with defaults.

Example:

```druim
fn scale :(value, factor = 2;)(
    ret value * factor;
):
```

Function calls use parentheses and comma-separated argument expressions:

```druim
scale(12)
scale(12, 4)
```

If a function finishes without executing `ret`, it returns `void`.

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

Computed indexes are valid:

```druim
index = 1;
value = numbers::[index];
```

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

```druim
world::player:?name
```

Once `:?` is evaluated, traversal ends.

---

## Get and Has

Druim uses one traversal model for both named and indexed containers.

### Get (`::`)

Get retrieves a member.

```druim
container::member
container::[index]
```

A valid selector that does not identify an existing member evaluates to `void`.

An invalid selector form or selector value produces a diagnostic.

### Has (`:?`)

Has checks whether a member exists.

```druim
container:?member
container:?[index]
```

A valid selector that does not identify an existing member evaluates to `false`.

An invalid selector form or selector value produces a diagnostic.

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

Compound operators are matched before their single-character prefixes.

---

## Diagnostics

Druim favors early, explicit diagnostics.

Diagnostics are required for conditions including:

- Unexpected characters
- Unterminated literals
- Invalid numeric forms
- Incomplete statements
- Unexpected tokens before a statement terminator
- Chained statement operators
- Undeclared or uninitialized identifiers
- Invalid Box indexes
- Selector forms unsupported by the traversed collection

Druim does not silently reinterpret invalid programs as missing data.

---

## Development Discipline

Language changes should be made in this order:

1. Update tokens and lexer rules.
2. Update parser behavior.
3. Update the canon.
4. Update tests.

When runtime behavior is affected, the evaluator must also be updated to match the canon.

The canon defines intended behavior. The implementation and tests must converge on it.

---

## Final Note

Druim is designed for deterministic structure, explicit intent, and inspectable behavior.

The language prefers one clear meaning over multiple convenient interpretations.
