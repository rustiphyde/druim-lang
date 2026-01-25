# Druim Canon (Living Document)

## Purpose

This document defines the **current canonical truths** of the Druim programming language.

It exists to capture what is *intentionally true right now* about Druim’s syntax, structure, and invariants, independent of any single implementation file or development session. Its purpose is continuity, not completeness.

Druim is under active development. As such, this document is **living**: it will evolve as design decisions are validated, revised, or replaced.

---

## Document Scope

This document describes:

- Structural rules that must be upheld across the language
- Lexer- and parser-level invariants
- Intentional constraints (what Druim explicitly allows and forbids)
- Semantics that are stable enough to rely on during development

This document does **not** aim to be:

- A tutorial
- A full formal grammar
- An exhaustive language reference
- A promise of future behavior

Details that are experimental, provisional, or unresolved should be recorded explicitly as such.

---

## Authority

When conflicts arise between:

- informal discussion
- comments in source files
- tests
- personal recollection

**This document is authoritative**, unless a newer revision explicitly supersedes it.

Code may temporarily diverge during development, but this document represents the intended direction and constraints of the language.

---

## Design Philosophy (High-Level)

Druim favors:

- Explicit structure over implicit behavior
- Deterministic parsing over convenience
- Clear token boundaries over inferred meaning
- Early, loud errors over silent coercion
- Syntax that reflects intent, not side effects

Features are added only when their semantics are well-defined and enforceable at the language level.

---

## Change Discipline

Changes to Druim should follow this order:

1. Update tokens and lexer rules
2. Update parser behavior
3. Update this canon to reflect the new truth
4. Update tests to enforce the canon

This document should describe **what is**, not speculative ideas or abandoned experiments.

---

## Reading Note

If you are reading this document to understand Druim:

- Assume all rules are intentional
- Assume omissions are deliberate
- Assume undefined behavior is disallowed unless stated otherwise

## Tokens and Lexical Invariants

This section defines the tokens recognized by Druim and the invariants enforced at the lexer level. These rules describe **what the lexer guarantees** before any parsing or semantic analysis occurs.

All tokens described here are **lexically atomic**: the lexer will never emit partial or ambiguous sequences for these forms.

---

## Identifiers and Literals

### Identifiers
- **Token**: Ident
- Identifiers begin with an ASCII letter, digit, or _
- Identifiers may contain ASCII letters, digits, or _
- Keywords are resolved at lexing time

### Numeric Literals
- **NumLit** — whole number literals
- **DecLit** — decimal literals (contain a single .)
- Numeric literals are not signed at the lexer level

### Text Literals
- **TextLit**
- Enclosed in double quotes (")
- Unterminated text literals are a lexical error

---

## Keywords (Types)

The following identifiers are lexed as **type keywords** when matched exactly:

- num   → KwNum
- dec   → KwDec
- text  → KwText
- void  → KwVoid

These keywords represent literal or type-level concepts.

## Keywords (Control and Scope)

The following identifiers are lexed as **control or scope keywords**:

- fn   → KwFn      (function definition)
- ret  → KwRet     (function return)
- loc  → KwLoc     (body-local binding)

These keywords affect control flow or scope and are not expressions.

## Identifiers

Identifiers are unquoted names used to refer to declared values, bindings, and targets within the language.
All other identifier strings that do not match a keyword exactly are emitted as Ident.

### Lexical Form

An identifier is a contiguous sequence of ASCII alphanumeric characters (A–Z, a–z, 0–9) and underscores (_).

Identifiers **may begin with a digit**.

However, an identifier **must contain at least one non-digit character**.  
A sequence consisting entirely of digits is **not** an identifier and is treated as a numeric literal.

### Valid Identifiers

```druim
a
abc
a1
1a
9lives
123abc
123_456
_foo
```


### Invalid Identifiers

```druim
1
123
000
```


> Identifiers are not quoted. Quoted text represents string literals and is not used for naming.

### Distinction from Numeric Literals

- A sequence composed only of digits is lexed as a numeric literal.
- A sequence containing digits and at least one non-digit character is lexed as an identifier.
- Decimal literals are recognized separately and are not identifiers.

This distinction is purely lexical and does not, by itself, imply validity in all syntactic positions.

---

## Numeric Literals

Numeric literals represent literal numeric values written directly in source code.

### Lexical Form

Druim recognizes two forms of numeric literals:

- **Integer literals**
- **Decimal literals**

Numeric literals are unquoted.

### Integer Literals

An integer literal is a contiguous sequence of one or more ASCII digits (0–9).

```druim
0
1
42
123
000
```


All-digit sequences are lexed as integer literals unless recognized as decimal literals.

### Decimal Literals

A decimal literal consists of:

- one or more ASCII digits  
- followed by a single dot (.)  
- followed by one or more ASCII digits

```druim
0.1
1.0
12.34
000.5
6.0007
```


Decimal literals must contain digits on **both sides** of the dot.

### Invalid Numeric Forms

The following forms are not valid numeric literals:

```druim
.
1.
.5
1..2
```


Such sequences result in a lexical error.

### Distinction from Identifiers

A numeric literal consists **only** of digits, or digits with a single decimal point.

Any alphanumeric sequence that contains at least one non-digit character (such as a letter or underscore) is **not** a numeric literal and is lexed as an identifier.

```druim
123 // numeric literal
123.456 // decimal literal
123abc // identifier
123_456 // identifier
```

This distinction is purely lexical and does not imply validity in all syntactic positions.

## Define Operators

### Define
- = → Define
- Assigns a value to the left-hand side

### Define Empty
- =; → DefineEmpty
- Lexically atomic
- Explicitly defines the left-hand side as an empty value
- This is a complete define statement

## Truth Evaluation (Flags)

Druim does **not** have implicit truthiness in the C/JS sense.  
All conditional evaluation is **explicit, deterministic, and total**.

### Flag Type
- flag is the boolean type in Druim.
- A flag may only ever be true or false.

### Truth Coercion Rules
When a value is *explicitly evaluated* as a flag, the following rules apply:

- flag(true) → true
- flag(false) → false
- 0 → false
- 0.0 → false
- Any non-zero num → true
- Any non-zero dec → true
- Any text value → true
- void → false

No other values are permitted to participate in truth evaluation.

### Undefined Values
- **Undefined does not exist in Druim.**
- Any attempt to reference an undeclared or uninitialized identifier **must raise a diagnostic**.
- There is no silent fallback, null propagation, or implicit defaulting.

### Empty Definition
- x =; is valid syntax and is equivalent to x = void;
- void represents the absence of a value and always evaluates to false when coerced to flag.

### Design Guarantee
Every truth evaluation in Druim:
- Is explicitly defined
- Produces a valid flag
- Or fails with a diagnostic

There is no third state.

---

## Block Operators

Druim uses explicit block operators. Each block family has a **start**, **end**, and **chain** token. These tokens are always matched before any single-character operators.

### Statement Blocks
- :{ → BlockStmtStart
- }: → BlockStmtEnd
- }{ → BlockStmtChain

### Expression Blocks
- :[ → BlockExprStart
- ]: → BlockExprEnd
- ][ → BlockExprChain

### Function Blocks
- :( → BlockFuncStart
- ): → BlockFuncEnd
- )( → BlockFuncChain

### Branch Blocks
- :| → BlockBranchStart
- |: → BlockBranchEnd
- || → BlockBranchChain

### Array Blocks
- :< → BlockArrayStart
- >: → BlockArrayEnd
- >< → BlockArrayChain

**Invariant:**  
Block chain tokens (}{, ][, )(, ><) are only produced by the lexer and cannot be synthesized from individual characters.

## Blocks and Scope

Druim uses multiple block forms. Each form has a fixed delimiter family and a fixed semantic role.

### Block Statements

Block Statements contain statements and establish lexical scope.

The delimiter family is:

```druim
:{
    stmt;
}{
    stmt;
```

Rules:

- :{ starts a new block-statement scope.
- }{ continues the same scope (block chaining does not nest).
- }: ends the scope.
- Exactly one scope exists per statement-block chain.

### Block Expressions

Block Expressions contain expressions only and do not establish scope.

The delimiter family is:

```druim
:[ expr ][ expr ]:
```

Rules:

- Each segment between :[ and ]: must parse as an expression.
- Chained segments are evaluated left to right.
- The last segment yields the value of the whole block expression.
- No new scope is created by a block expression.

### Functions

A function definition is an expression that produces a callable value.

Syntax:

```druim
fn my_function :(a, b)( body1 )( body2 ):
```

Rules:

- A function definition must use fn, a snake_case identifier, exactly one parameter block, and at least one body block.
- Parameters are plain identifiers.
- Default parameter values are reserved and currently not part of the grammar unless explicitly implemented.

#### Function Scope

A function introduces a function-local scope when the function is invoked.

- Parameters are defined in the function scope at call entry.
- All chained bodies share the same function scope.
- Names defined in an earlier body are visible to later bodies by default.

Example:

```druim
fn example :(x)(
    y = x * 2;
)(
    ret y + 1;
):
```

#### Body-Local Scope with loc

Druim supports body-local restriction inside a function body via loc.

- loc introduces a sub-scope that exists only for the remainder of the current body.
- Names declared with loc are not visible to later bodies in the chain.

Example:

```druim
fn example :(x)(
    loc y = x * 2;
)(
    ret x;
):
```

### Evaluator Scope Responsibilities

The evaluator must implement scope handling with these guarantees:

- On :{ push a new lexical scope for the statement-block chain.
- On }{ do not push or pop; continue executing in the current lexical scope.
- On }: pop the lexical scope created by the matching :{.
- On function call, push a function scope, bind parameters, execute bodies in order, then pop the function scope.
- On loc within a function body, push a body-local scope for that body and pop it before leaving that body.

### Canonical Guarantee

- Block Statements establish lexical scope.
- Block Expressions do not establish scope.
- Function calls establish function scope.
- loc restricts visibility to a single function body.

This behavior is stable and locked.
## Logical Operators

Logical operators are **compound tokens only**. Single-character logical symbols are not valid.

- &? → And
- |? → Or
- !? → Not

Bare &, |, and ! are not legal tokens.

---

## Comparison Operators

- == → Eq
- != → Ne
- <  → Lt
- <= → Le
- >  → Gt
- >= → Ge

**Invariant:**  
Compound comparison operators are always matched before single-character < or >.

---

## Arithmetic Operators

- + → Add
- - → Sub
- * → Mul
- / → Div
- % → Mod

---

## Flow and Direction Operators

- |> → Pipe
- -> → ArrowR
- <- → ArrowL

---

## Colon Family Operators

The colon (:) introduces multiple structural operators. Longest matches are always preferred.

- :: → Has
- := → Bind
- :? → Present
- :> → Cast
- :  → Colon

## The :: Has Operator

The :: operator in Druim is called the **Has operator**.

It answers a simple, human question:

> “Does this thing contain that thing — and if so, give it to me.”

It is **not assignment**, **not mutation**, and **not scope creation**.  
It is a **safe access and propagation operator** that always evaluates to a value.

---

### Core Meaning

A :: B means:

> If **A has B**, evaluate to **B**.  
> If **A does not have B**, evaluate to **void**.

There are **no errors**, **no exceptions**, and **no implicit truthiness** introduced by this operator.  
Failure is represented explicitly as void.

---

### Why This Exists

In many languages, accessing something that doesn’t exist:

- Throws an error
- Returns undefined
- Requires special syntax or keywords
- Forces defensive boilerplate

Druim does none of that.

The Has operator lets you **ask for something without assuming it exists**.

---

### Basic Example (Real-World)

Imagine a user container that *may or may not* have a profile.

```druim
user::profile
```

- If user has a profile, the expression evaluates to that profile
- If not, the expression evaluates to void

No crash. No undefined. No branching required.

---

### Definition via Expression

Because :: is an **expression**, it can be used anywhere a value is expected.

```druim
a = user::profile;
```

This means:

- If user has profile, define a as that profile
- Otherwise, define a as void

This is **definition**, not assignment (<-).

---

### Chaining Behavior (Critical)

The Has operator is **chainable**.

```druim
a = user::profile::email;
```

This evaluates left to right:

1. If user has profile
2. And profile has email
3. Then define a as email
4. Otherwise, define a as void

At **any point** in the chain, failure collapses the entire expression to void.

This makes deep access safe by default.

---

### Use in Conditionals (Without New Keywords)

Because :: evaluates to a value, and because truth is explicit in Druim, it can be used directly in guards and conditionals.

```druim
x ?= user::profile::email;
```

This means:

- If user::profile::email evaluates to a truthy value, assign it to x
- Otherwise, x becomes void

No temporary variables.
No special syntax.
No extra keywords.

---

### What :: Works With

The Has operator works uniformly with any **container-like structure**, including:

- Block Statements
- Block Expressions
- Block Functions
- Block Arrays
- Branch blocks
- Any named or structured value

If the left side can *contain named values*, :: can query it.

---

### What :: Is Not

- It is **not assignment**
- It is **not mutation**
- It does **not create scope**
- It does **not throw errors**
- It does **not imply truth**

It only answers:

> “Does this have that — yes or void.”

---

### Design Philosophy

The Has operator exists to:

- Eliminate undefined
- Make absence explicit
- Allow safe deep access
- Reduce boilerplate
- Preserve composability
- Keep failure non-fatal and inspectable

In Druim, **absence is data**, and :: is how you ask for it safely.


## Bind (:=)

The := operator establishes a **value binding** between two identifiers.

In human terms:

> “Take the current value of that thing and give me my own copy of it.”

Bind copies the **current resolved value** of an existing identifier into a new name, **without linking their futures**.

This is **not reference aliasing**.

---

### Core Meaning

```druim
a := b;
```
Means:

- b **must already exist**
- a receives the **current value** of b
- a and b are **independent after binding**
- Future mutations of b do **not** affect a
- No expressions are evaluated
- No fallback logic is applied

---

### What Bind *Is*

Bind is a **value snapshot operator**.

It answers the question:

> “What is this value *right now*, and let me work with it independently.”

---

### What Bind *Does*

- Copies the current value of an existing identifier
- Produces a **new, independent value**
- Requires the right-hand side to be an identifier
- Requires the identifier to already be defined
- Allows safe manipulation without altering the source

---

### What Bind *Does Not Do*

- Does not evaluate expressions
- Does not perform conditional logic
- Does not track future changes
- Does not alias or link identities
- Does not provide fallback behavior

---

### Comparison With Other Operators

#### Define (=)

```druim
a = expr;
```

- Evaluates expr
- Produces a new value
- Defines a

#### Guard (?=)

```druim
a ?= x : y : z;
```

- Evaluates expressions
- Applies truth rules
- Selects the first truthy value or void
- Defines a as the result

#### Bind (:=)

```druim
a := b;
```

- Evaluates nothing
- Copies the current value of b
- Produces a new, independent value
- Freezes the value at bind-time

---

### Real-World Example

Imagine a configuration that must remain stable:

```druim
config = :{
    retries = 3;
}:
```

You want to experiment locally without touching the original:

```druim
testConfig := config;
testConfig.retries = 5;
```

Result:

- testConfig.retries → 5
- config.retries → still 3

This behavior is **intentional**.

---

### Why Bind Exists

Bind enables:

- Safe experimentation
- Temporary manipulation
- Snapshotting values
- Explicit intent without side effects

Without Bind, developers are forced to choose between:
- Recomputing (=)
- Conditional logic (?=)
- Or accidental mutation

Bind fills this gap cleanly.

---

### Design Principle

- = defines
- ?= decides
- := copies

Each operator has **one job**.


## Guard (?= / :)

The Guard operator provides conditional assignment without introducing statements, blocks, or control-flow keywords.

It evaluates expressions using explicit boolean (flag) semantics and always resolves to a defined value.

---

### Basic Form

```druim  
x ?= y;  
```

Semantics:

- `y` is evaluated and converted to a `flag`
- If `flag(y)` is `true` → `x = y;`
- If `flag(y)` is `false` → `x = void;`

This form implicitly falls back to `void`.

Equivalent to:

```druim  
x ?= y : void;  
```

---

### Guard With Fallback

```druim  
x ?= y : z;  
```

Semantics:

- If `flag(y)` is `true` → `x = y;`
- Otherwise → `x = z;`

Both branches are expressions.  
The result is always a defined value.

---

### Guard Chaining

Guards may be chained to express ordered conditional resolution.

```druim  
x ?= y : z : v;  
```

Semantics:

- If flag(y) is true → x = y;
- Else if flag(z) is true → x = z;
- Else → x = v;
- Else → x = void;

Rules:

- **?=** appears exactly once, immediately after the target identifier
- **:** is the only fallback separator
- Every segment after ?= is an expression
- **?=** requires at least one branch expression
- This syntax is invalid:

     ```druim
     a ?=;
     ```
     
- Fallbacks are unbound in count
- void is the implicit terminal fallback of every guard
- Evaluation proceeds left-to-right
- The first truthy branch wins
- If all guard and fallback expressions evaluate to false target is assigned **void**
- **void** always evaluates to a false **flag**

---

### Truth Evaluation

Guard conditions use **explicit truth evaluation**, not implicit truthiness.

| Type  | Truth Rule |
|------|------------|
| flag | true / false |
| num  | 0 → false, non-zero → true |
| dec  | 0.0 → false, non-zero → true |
| text | empty → false, non-empty → true |
| void  | always false |

There is no undefined value in Druim.

---

### Guarantees

- Guard never produces undefined
- All assignments resolve deterministically
- void represents intentional absence, not missing state
- Guard is an expression-level construct, not a statement or block

---

## Punctuation

- ( → LParen
- ) → RParen
- , → Comma
- ; → Semicolon

---

## General Lexical Rules

- Whitespace is ignored except as a separator
- Longest-match wins for all operators
- Tokens are emitted left-to-right with no backtracking
- Any unexpected character produces a LexError::UnexpectedChar
- End of input produces a final Eof` token

The lexer is responsible only for structure and atomicity.  
All semantic meaning is deferred to later compilation stages.
