# Druim Canon (Living Document)

## Purpose

This document defines the **current canonical truths** of the Druim programming language.

It exists to capture what is *intentionally true right now* about Druim’s syntax, structure, and invariants, independent of any single implementation file or development session. Its purpose is continuity, not completeness.

Druim is under active development. As such, this document is **living**: it will evolve as design decisions are validated, revised, or replaced.

---

## Scope

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
