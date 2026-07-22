# Druim Canon (Living Document)

## Canon Revision Baseline
- Revision ID: DRUIM-CANON-R003
- Status: Current
- Effective Date: 2026-07-22
- Authoritative Scope: Global
- Supersedes: DRUIM-CANON-R002
- Notes: This revision formalizes statement-boundary invariants for Define, DefineEmpty, Copy, Bind, and Guard; clarifies complete-expression requirements; and resolves inconsistencies in the documented truth and Guard rules.


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
- flag  → KwFlag
- void  → KwVoid

These keywords represent literal or type-level concepts.

## Keywords (Control and Scope)

The following identifiers are lexed as **control or scope keywords**:

- fn   → KwFn      (function definition)
- ret  → KwRet     (function return)
- loc  → KwLoc     (local scope)

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

## Statement Structure and Boundaries

Druim statements are structurally complete forms terminated by a semicolon unless their syntax includes an atomic terminator.

### Statement Terminators

The semicolon (`;`) terminates a statement.

The `=;` DefineEmpty operator is lexically atomic and includes the statement terminator as part of the operator token.

### Complete Consumption

A valid statement must consume every token belonging to that statement.

After the statement's final identifier, value, or expression has been parsed, the next token must be the terminating semicolon unless the statement uses the atomic `=;` operator.

Unexpected tokens may not appear between the completed statement body and its terminator.

Invalid:

```druim
a := b c;
a :> b c;
a = 12 13;
a ?= 12 13;
a ?= 12 : 13 14;
```

A parser must never silently discard or consume an unexpected token as though it were the statement terminator.

### Statement-Operator Chaining

Statement operators may not be chained within one statement.

The statement operators are:

• = — Define
• =; — DefineEmpty
• := — Copy
• :> — Bind
• ?= — Guard

Invalid:

```druim
a = 12 = 13;
a := b := c;
a :> b :> c;
a = 12 :> b;
a ?= b := c;
```
Each operation must be written as a separate statement.

## Local Modifier

The **loc** keyword may appear at most once and only at the beginning of a statement form that supports local scope.

Valid:

```druim
loc a = 12;
loc a =;
loc a := b;
loc a :> b;
loc a ?= b : c;
```

Invalid:
```druim
loc loc a = 12;
a loc = 12;
```

## Parser Boundary Invariant

A successful parser routine must leave the parser positioned immediately after the complete construct it parsed.

A statement parser must therefore:

1. Parse the complete statement body.
2. Verify that no unexpected tokens remain within that statement.
3. Consume the statement terminator.
4. Leave the next statement untouched.

A parser may not report success after parsing only a valid prefix of an otherwise invalid statement.

This captures the exact invariant exposed by the Copy, Bind, Define, and Guard tests.

## Define Operators

### Define (`=`)
- The Define operator evaluates exactly one complete expression and defines the target identifier as the resulting value.

```druim
a = 12;
b = 12 + 13;
c = user::profile;
```
Rules:

• The left-hand side must be exactly one identifier.
• The right-hand side must contain exactly one complete expression.
• The right-hand side may not be empty.
• A bare identifier may not be used as the entire right-hand side when Copy (:=) or Bind (:>) expresses the intended operation.
• No unexpected tokens may remain between the expression and the terminating semicolon.
• Define may not be chained with another statement operator.

Invalid:

```druim
12 = a;
a =;
a = b;
a = 12 13;
a = 12 :> b;
```

To copy the current value of an existing identifier, use Copy:

```druim
a := b;
```

To establish shared identity with an existing identifier, use Bind:

```druim
a :> b;
```


### Define Empty (=;)

- The DefineEmpty operator explicitly defines the target identifier as void.

```druim
a =;
```

It is equivalent in meaning to:
```druim
a = void;
```

Rules
• **=;** is one lexically atomic token.
• The left-hand side must be exactly one identifier.
• **=;** completes the statement itself.
• No separate semicolon follows it.
• DefineEmpty may not be chained with another statement operator.

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

## Blocks and Scope

In Druim, blocks exist solely to establish lexical scope.
Blocks do not produce values and do not restrict what may appear inside them beyond general syntactic validity.

### Blocks

Blocks establish a scope boundary. They may contain any top-level construct that is meaningful in a scoped context.

The delimiter family is:

```druim
:{
    …
}{
    …
}:
```

### Block Structure Rules

- `:{` begins a new block scope.
- `}{` continues the same block chain.
- `}:` ends the block scope.
- Block chaining does **not** introduce nesting.
- Exactly one lexical scope exists per block chain.
- Individual bindings may be restricted to a single block segment via the `loc` keyword.
- Blocks do not evaluate to a value.
- Blocks exist only to control name visibility and lifetime.

### Block Nesting

Nested blocks are **not allowed** in Druim.

A `:{ ... }:` block may not appear inside another block.

The only valid way to extend a block is through **block chaining** using `}{`.

Invalid:

```druim
:{
    :{
        x = 1;
    }:
}:
```

Valid:

```druim
:{
    x = 1;
}{
    y = 2;
}:
```

### Block Contents

Blocks may contain, in any valid order:

- statements
- function definitions
- any construct that is syntactically meaningful at block level

Blocks impose no additional restrictions beyond general syntactic validity.

Standalone expressions that have no structural effect (e.g. `1 + 2`) are rejected by the grammar, not by block semantics.

---

## Functions

A function definition is an expression that produces a callable value.

Function definitions are treated as structural declarations and may appear as standalone forms.
Other expressions may not appear standalone unless they have a defined structural role.

### Function Delimiters

Functions use the delimiter family:

```druim
:( parameters )( body ):
```

- `:(` begins the parameter block.
- `)(` separates the parameter block from the body block.
- `):` ends the function definition.

These delimiters are structural and must appear exactly once each in a valid function definition.

### Syntax

```druim
fn my_function :(a, b)( body ):
```

### Rules

- A function definition must use `fn`, followed by a `snake_case` identifier.
- Exactly **one parameter block** and **one body block** must be present.
- Each parameter must be a valid parameter form.
- A parameter may be a plain identifier.
- A parameter may include a default value using the Define form.
- Parameter defaults use `=` and must contain exactly one complete expression.
- Copy, Bind, Guard, and DefineEmpty are not valid parameter-default forms unless a later canon revision explicitly permits them.
- The body contains a sequence of valid statements.

Example with a default parameter:

```druim
fn my_func :(w, x = 12;)(ret w * x;):
```

### Function Scope

A function introduces a **function-local scope** when the function is invoked.

- Parameters are defined in the function scope at call entry.
- Bindings created inside the function body exist only for the duration of the call.

Example:

```druim
fn example :(x)(
    y = x * 2;
    ret y;
):
```

### Evaluator Scope Responsibilities

The evaluator must implement scope handling with these guarantees:

- On :{ push a new lexical scope for the block chain.
- On }{ do not push or pop; continue executing in the current lexical scope.
- On }: pop the lexical scope created by the matching `:{`.
- On function call, push a function scope, bind parameters, then pop the function scope.

### Canonical Guarantee

- Blocks establish lexical scope.
- Function body establishes function scope.
- loc restricts visibility to a single block segment within a chain.

This behavior is stable and locked.

## Logical Operators

The binary logical operators are compound tokens. Their single-character forms are not valid.

- **&&** → And
- **||** → Or
- **!** → Not

Bare `&`, `|`, are not legal tokens.

---

## Comparison Operators

- `==` → Eq
- `!=` → Ne
- `<`  → Lt
- `<=` → Le
- `>`  → Gt
- `>=` → Ge

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
- := → Copy
- :? → Present
- :> → Bind
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

- Functions
- Arrays
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


## Copy (:=)

The := operator performs a **value copy** between two identifiers.

In human terms:

> “Take the current value of that thing and give me my own copy of it.”

Copy copies the **current resolved value** of an existing identifier into a new name, **without linking their futures**.

This is **not reference aliasing**.

---

### Core Meaning

```druim
a := b;
```
Means:

- b **must already exist**
- a receives the **current value** of b
- a and b are **independent after copying**
- Future mutations of b do **not** affect a
- No expressions are evaluated
- No fallback logic is applied

---

### Syntactic Form

Copy has exactly this form:

```druim
target := source;
```

Rules:

• **target** must be exactly one identifier.
• **source** must be exactly one identifier.
• No expression may appear on either side.
• No additional tokens may appear before the semicolon.
• Copy may not be chained with another statement operator.

Invalid:

```druim
a := 12;
a := b + c;
a := b c;
a := b := c;
a := b :> c;
```

### What Copy *Is*

Copy is a **value snapshot operator**.

It answers the question:

> “What is this value *right now*, and let me work with it independently.”

---

### What Copy *Does*

- Copies the current value of an existing identifier
- Produces a **new, independent value**
- Requires the right-hand side to be an identifier
- Requires the identifier to already be defined
- Allows safe manipulation without altering the source

---

### What Copy *Does Not Do*

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

#### Bind (`:>`)

```druim
a :> b;
```

- Evaluates nothing
- Creates shared identity
- Future changes propagate across all bound identifiers

#### Copy (:=)

```druim
a := b;
```

- Evaluates nothing
- Copies the current value of b
- Produces a new, independent value
- Freezes the value at copy-time

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

### Why Copy Exists

Copy enables:

- Safe experimentation
- Temporary manipulation
- Snapshotting values
- Explicit intent without side effects

Without Copy, developers are forced to choose between:
- Recomputing (=)
- Conditional logic (?=)
- Or accidental mutation

Copy fills this gap cleanly.

---

### Design Principle

- = defines
- ?= decides
- := copies
- :> binds

Each operator has **one job**.

---

## Bind (`:>`)

The `:>` operator establishes a **live identity binding** between two identifiers.

In human terms:

> “Make this name refer to the same thing as that name.”

Bind connects two identifiers to the **same underlying value identity**, such that future mutations through either name are visible to the other.

This is **reference aliasing**.

---

### Core Meaning

```druim
a :> b;
```

Means:

- `b` **must already exist**
- `a` becomes an alias of `b`
- `a` and `b` refer to the same underlying identity
- Future mutations of either identifier affect the shared value
- No expressions are evaluated
- No fallback logic is applied

Bind does not create a new value. It links identities.

---

### Syntactic Form

Bind has exactly this form:

```druim
target :> source;
```

Rules:

• **target** must be exactly one identifier.
• **source** must be exactly one identifier.
• No expression may appear on either side.
• No additional tokens may appear before the semicolon.
• Bind may not be chained with another statement operator.

Invalid:
```druim
a :> 12;
a :> b + c;
a :> b c;
a :> b :> c;
a :> b := c;
```

### What Bind *Is*

Bind is an **identity-linking operator**.

It answers the question:

> "Make these two names refer to the same value."

Bind does not snapshot the value.  
It establishes shared identity.

---

### What Bind *Does*

- Links two identifiers to the same underlying value
- Requires the right-hand side to be an identifier
- Requires the identifier to already be defined
- Propagates future mutations across all bound names
- Does not evaluate expressions
- Does not create a new value

---

### What Bind *Does Not Do*

- Does not copy values
- Does not evaluate expressions
- Does not perform conditional logic
- Does not freeze or snapshot state
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

#### Copy (:=)

```druim
a := b;
```

- Snapshots current value of b
- Produces independent value
- Future changes do not propagate

#### Bind (`:>`)

```druim
a :> b;
```

- Evaluates nothing
- Creates shared identity
- Future changes propagate across all bound identifiers

---

### Real-World Example

```druim
a = 10;
b :> a;
a = 20;
```

Result:

- `b` evaluates to `20`

Because `a` and `b` refer to the same identity.

---

### Why Bind Exists

Bind enables:

- Intentional aliasing
- Shared state by design
- Identity-based programming
- Explicit linkage between names

Without Bind, developers are forced to choose between:
- Snapshotting (`:=`)
- Re-defining (`=`)
- Or duplicating state manually

Bind provides explicit identity sharing.

---

### Design Principle

- = defines
- ?= decides
- := copies
- :> binds

Each operator has **one job**.

## Guard (?= / :)

Guard is a target-defining statement that performs ordered, truth-based value selection.

It evaluates one or more branch expressions from left to right. The first branch whose value evaluates to `true` under Druim's explicit flag-conversion rules becomes the target's value.

If no branch evaluates to `true`, the target is defined as `void`.

Guard is not a general standalone expression and does not itself produce a value for use inside another expression.

---

### Basic Form

```druim  
x ?= y;  
```

Semantics:

1. Evaluate **y**.
2. Convert the resulting value to **flag**.
3. If the result is **true**, define **x** as the evaluated value of **y**.
4. Otherwise, define **x** as **void**.

Equivalent to:

```druim  
x ?= y : void;  
```

### Multiple Branches

```druim
x ?= y : z : v;
```

Semantics:

1. Evaluate **y**. If **flag(y)** is **true**, define **x** as **y** and stop.
2. Otherwise, evaluate **z**. If **flag(z)** is **true**, define **x** as **z** and stop.
3. Otherwise, evaluate **v**. If **flag(v)** is true, define **x** as **v** and stop.
4. If every branch evaluates to **false**, define **x** as **void**.

Every segment is a guarded branch. The final written branch is not an unconditional fallback.

The implicit terminal result of every Guard is **void**.

### Structural Rules

• The target must be exactly one identifier.
• **?=** appears exactly once, immediately after the target.
• At least one branch expression is required.
• **:** separates subsequent branch expressions.
• Each branch must contain exactly one complete expression.
• Empty branches are invalid.
• No unexpected tokens may remain after a branch or before the terminating semicolon.
• Statement operators may not appear inside Guard branches.
• Guard may not be chained with another statement operator.
• The number of branches is not syntactically bounded.

Valid:

```druim
x ?= y;
x ?= y : z;
x ?= first() : second() : void;
loc x ?= a : b : c;
```

Invalid:

```druim
x ?=;
x ?= y :;
x ?= y z;
x ?= y : z v;
x ?= y := z;
loc loc x ?= y;
```

---


### Truth Evaluation

Guard uses Druim's canonical explicit truth-conversion rules.

| Type  | Truth Rule |
|------|------------|
| **flag** | **true** remains true; **false** remains false |
| **num**  | **0** is false; every non-zero value is true |
| **dec**  | **0.0** is false, every non-zero value is true |
| **text** | Every text value is true |
| **void**  | Always false |

There is no undefined value in Druim.

---

### Guarantees

• Guard always defines its target.
• Guard never produces undefined.
• Branches are evaluated from left to right.
• Evaluation stops after the first truthy branch.
• If all branches are false, the target becomes void.
• Guard introduces no block and no additional scope.

---

### Reserved Structural Delimiters

The following delimiter families are recognized at the lexical level but do not yet have defined semantic behavior:

- :[   → ArrayStart
- ]:   → ArrayEnd
- ][   → ArrayChain

These tokens are reserved for future structural constructs.

Until formally defined in a canon revision, they have no guaranteed semantics.

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
