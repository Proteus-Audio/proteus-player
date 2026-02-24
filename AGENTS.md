# AGENTS.md — Rust + Slint GUI Agent Guide

This repository contains a Rust GUI application built with **Slint**.
This document defines expectations and working rules for any coding agent contributing to the codebase.

The agent must prioritize:
- Correctness
- Maintainability
- Clear architecture boundaries
- Idiomatic Rust
- Idiomatic Slint usage
- Small, reviewable changes

---

# Core Principles

## Separation of Concerns (Non-Negotiable)

UI and business logic must be clearly separated.

- `.slint` files define layout, styling, and UI structure.
- Rust code defines:
  - domain logic
  - state transitions
  - services (I/O, network, persistence)
  - orchestration

**Hard rule:**  
`.slint` files must not contain business logic beyond simple property bindings and UI-level behavior.

**Rule of thumb:**  
If code touches the filesystem, network, database, timers, or complex state transitions, it belongs in Rust — not in Slint.

---

## Keep File Lengths Reasonable

Avoid "god files."

- Target ~200–400 lines per file.
- If a file exceeds ~600 lines, refactor.
- Split by:
  - Feature
  - Screen
  - Domain area
  - Component

Large `.slint` files should be split into reusable components.

Large Rust modules should be decomposed into cohesive submodules.

---

# Suggested Project Structure

Adapt as needed, but maintain architectural clarity:

```
src/
main.rs
app/
state/
domain/
services/
ui/
ui/
main_window.slint
components/
```


### Responsibilities

- `main.rs`  
  Application startup, wiring UI to Rust logic.

- `ui/*.slint`  
  Pure UI components and layout definitions.

- `state/`  
  App state structs and state transitions.

- `domain/`  
  Business logic and core models (no Slint dependencies).

- `services/`  
  Side effects (filesystem, network, database, time).

**Hard rule:** `domain/` must not depend on Slint.

---

# Slint Best Practices

## Keep Slint Declarative

Slint is declarative UI. Use it as such.

- Use property bindings instead of imperative updates where possible.
- Avoid embedding complex logic in callbacks.
- Keep logic inside `.slint` limited to UI concerns.

---

## Data Flow Direction

Preferred architecture:

Rust State → Slint Properties → UI  
UI Events → Rust Callbacks → State Updates → Property Updates

Avoid:
- Mutating Rust state directly from arbitrary closures.
- Embedding logic in `.slint` that duplicates Rust logic.

---

## Property Binding Discipline

- Prefer binding properties instead of manually synchronizing values.
- Avoid circular bindings.
- Keep derived properties computed in Rust if they are complex.

---

## Callbacks

- Define callbacks in `.slint` for user interactions.
- Connect them in Rust using `on_<callback_name>()`.
- Callbacks should:
  - Emit intent
  - Not implement full logic inline

Example pattern:

- Slint defines: `callback save_clicked();`
- Rust connects: `ui.on_save_clicked(move || { ... })`

Complex logic must live in Rust modules.

---

## Avoid Blocking the UI Thread

- Never perform file/network I/O in a Slint callback directly.
- Spawn background tasks (threads or async runtime).
- Send results back to UI safely (e.g., via `invoke_from_event_loop`).

---

## Thread Safety

Slint UI components are generally not `Send`.

Rules:
- UI must only be updated on the main thread.
- Use `slint::invoke_from_event_loop` to update UI from background threads.
- Never share UI handles across threads without proper coordination.

---

## Component Design

- Break UI into reusable components.
- Avoid massive top-level `.slint` files.
- Use clear, stable property interfaces between components.
- Keep components cohesive.

---

# Rust Best Practices

## Ownership and Borrowing

- Avoid unnecessary cloning.
- Prefer borrowing where possible.
- Use `Arc` only when needed.
- Do not introduce `Rc<RefCell<...>>` without clear justification.

---

## Error Handling

- Use `thiserror` for structured errors.
- Use `anyhow` only at application boundaries (if chosen).
- Never `unwrap()` in non-test code unless guaranteed safe and documented.
- Surface user-facing errors clearly in the UI.

---

## Avoid Global State

- No mutable statics.
- No hidden singletons.
- Pass state explicitly.

---

## State Management

Keep a single root state struct (e.g., `AppState`).

Split logically into feature-specific structs if needed:

```.slint
AppState {
  settings: SettingsState,
  editor: EditorState,
  session: SessionState,
}
```


State mutations should:
- Be centralized
- Be predictable
- Be easy to test

---

## Testing Strategy

GUI code is hard to test; logic is not.

- Put testable logic in `domain/` and `state/`.
- Unit test:
  - Parsers
  - Validators
  - Reducers
  - State transitions
- Keep `.slint` files thin so logic is testable outside the UI.

---

## Formatting and Linting

Before considering work complete:

- `cargo build`
- `cargo fmt`
- `cargo clippy --all-targets --all-features`
- `cargo test`

No new warnings should be introduced.

---

# Performance Guidelines

- Avoid repeated allocations in tight UI update loops.
- Cache derived data if expensive.
- Avoid unnecessary cloning of large data structures.
- Keep UI updates minimal and targeted.

---

# Documentation-First Development (Required)

Before implementing or modifying functionality that depends on Slint or any third-party crate:

1. **Search the crate documentation first.**
   - Read official API docs on `docs.rs`.
   - Review macro usage and generated Rust bindings.
   - Study property, callback, and threading documentation carefully.

2. **Read Slint language documentation.**
   - Understand:
     - Property bindings
     - Animations
     - Callbacks
     - Models
     - Threading constraints
   - Confirm correct syntax and idioms before writing `.slint` code.

3. **Review official examples.**
   - Check the Slint GitHub repository examples.
   - Follow established patterns rather than inventing architecture.

4. **Search the crate source when needed.**
   - Use the “source” view in docs.rs.
   - Understand trait bounds and generated code behavior.

5. **Do not guess APIs.**
   - Never assume a property type or callback signature.
   - Verify macro expansion expectations.
   - Confirm thread-safety requirements before introducing concurrency.

---

### Rule of Thumb

If you are about to:
- invent a workaround for a binding issue,
- bypass the Slint threading model,
- duplicate functionality provided by Slint,
- guess how a macro behaves,

→ **Stop and search the documentation first.**

Deep understanding prevents fragile UI code and architectural drift.

---

# Change Management Rules (Agent Workflow)

When making changes:

1. State intent clearly (what and why).
2. Keep diffs small.
3. Improve structure when possible:
   - Reduce file size
   - Improve naming
   - Clarify boundaries
   - Remove duplication

---

# Definition of Done

- Builds successfully.
- No formatting or clippy warnings.
- Tests pass.
- No UI thread blocking.
- Clear separation between:
  - Slint UI
  - Rust state
  - Business logic
  - Side effects

---

# Quick Do / Don’t Checklist

## Do

- Keep Slint declarative.
- Keep business logic in Rust.
- Use bindings instead of manual synchronization.
- Use background threads for heavy work.
- Update UI from main thread only.
- Keep files small and cohesive.

## Don’t

- Block the UI thread.
- Put complex logic in `.slint`.
- Create giant modules.
- Use global mutable state.
- Guess how Slint works instead of checking documentation.

---

If any guideline conflicts with established project conventions, follow the existing convention and leave a note explaining the deviation.
