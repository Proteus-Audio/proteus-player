# AGENTS.md — Rust + Iced GUI Agent Guide

This repository contains a Rust desktop GUI application built with **Iced** (currently `iced 0.14`) and a small set of platform integrations (`muda`, `rfd`, macOS app APIs).
This document defines expectations and working rules for any coding agent contributing to the codebase.

The agent must prioritize:
- Correctness
- Maintainability
- Clear architecture boundaries
- Idiomatic Rust
- Idiomatic Iced usage
- Small, reviewable changes

---

# Project-Specific Architecture (Current Repo)

The current app is organized around a message-driven Iced architecture:

- `src/main.rs`
  - Process startup
  - macOS app naming setup
  - initial CLI parsing (`--open`)
  - delegates to `app::run(...)`

- `src/app/mod.rs`
  - Iced `daemon(...)` wiring
  - top-level `update`, `view`, `subscription`
  - app title/theme/scale hooks

- `src/app/messages.rs`
  - `Message` enum (UI intents + window events + shortcut events + effect results)

- `src/app/state.rs`
  - root app state (`ProteusApp`)
  - per-window state (`PlayerWindowState`)
  - state transitions and orchestration

- `src/app/view.rs`
  - pure widget tree construction
  - maps state to `Element<Message>`

- `src/app/effects.rs`
  - side effects and platform/UI integration
  - dialogs (`rfd`)
  - window creation settings
  - macOS app icon setup

- `src/playback.rs`
  - playback adapter/wrapper around `proteus-lib`
  - domain-ish API for load/play/seek/status

- `src/native_menu.rs`
  - native menu integration (`muda`)
  - maps menu events to app-level actions

Prefer preserving and strengthening this structure instead of collapsing concerns into `main.rs` or `app/mod.rs`.

---

# Core Principles

## Separation of Concerns (Non-Negotiable)

UI rendering, state transitions, and side effects must remain clearly separated.

- `view.rs` defines layout and widget composition only.
- `messages.rs` defines intent/event types.
- `state.rs` owns application state and synchronous state transitions.
- `effects.rs` owns side effects (dialogs, window creation, platform integration).
- `playback.rs` encapsulates playback API interaction.

**Hard rule:**  
Do not perform filesystem, network, dialog, OS API, or heavy playback setup work directly inside `view` code.

**Rule of thumb:**  
If code touches the filesystem, OS dialogs, native platform APIs, or asynchronous work, it belongs in `effects.rs` (or another service module), triggered via `Task`.

---

## Keep File Lengths Reasonable

Avoid "god files."

- Target ~200–400 lines per file.
- If a file exceeds ~600 lines, refactor.
- Split by:
  - Feature
  - Domain area
  - UI concern
  - Side-effect surface

If `src/app/mod.rs` grows, prefer moving logic into:
- `state/` or additional `app/*` modules
- feature-specific update helpers
- dedicated service/effects modules

---

# Iced Best Practices (Repo-Enforced)

## Keep `view` Pure

`view` functions should be deterministic and side-effect free.

- Build widgets from current state.
- Emit `Message` values for user intent.
- Avoid mutating state in `view`.
- Avoid expensive work in `view` (I/O, decoding, allocations in loops when avoidable).

Do not open dialogs, create windows, or call playback APIs from `view`.

---

## Message-Driven Data Flow

Preferred architecture:

State -> `view(state)` -> Widgets  
User/System Event -> `Message` -> `update(state, message)` -> `Task<Message>`  
Task Result -> `Message` -> `update(...)`

Keep this flow explicit and easy to trace.

Avoid:
- hidden mutations in helpers
- side effects buried in widget closures
- duplicating state transition logic across match arms

---

## `update` Discipline

`update` is the state transition boundary.

- Mutate state synchronously and predictably.
- Return `Task<Message>` for side effects.
- Use `Task::batch` when multiple effects must be scheduled.
- Prefer small helper methods on `ProteusApp` / feature state for repeated transitions.

Keep `update` readable:
- group related `Message` arms
- avoid giant inline blocks
- delegate to `state.rs` methods when logic grows

---

## `Task` / Async Effects

Use `Task` for side effects and asynchronous work.

- `Task::perform(...)` for async/background operations.
- `Task::done(...)` for immediate results.
- `Task::batch(...)` for multiple follow-up effects.

Rules:
- Do not block the UI thread inside `update`.
- Do not run long-running I/O or CPU work synchronously in response to UI events.
- Route effect results back into `Message`.

Platform note:
- This repo already uses sync dialog opening on macOS in one path and async task-based dialog opening elsewhere. Preserve that platform-specific reasoning unless you verify a better cross-platform behavior.

---

## `Subscription` Discipline

Subscriptions should describe event sources, not perform logic.

- Keep subscription construction declarative.
- Map events to `Message`.
- Put interpretation logic in helpers (as done for keyboard shortcuts).
- Avoid mutating state from subscription callbacks.

Be careful with high-frequency subscriptions (like the 16ms tick):
- keep per-tick work minimal
- avoid repeated expensive allocations
- move expensive sampling/processing behind throttling or state guards

---

## Multi-Window State Management (Important in This Repo)

This app supports multiple windows keyed by `iced::window::Id`.

Rules:
- Keep per-window data in a dedicated window state struct.
- Use `window::Id` to route UI events and shortcuts.
- Ensure cleanup happens when windows close (e.g., playback shutdown).
- Keep focused-window tracking explicit and updated from window events.

When adding features:
- decide whether state is global (`ProteusApp`) or per-window (`PlayerWindowState`)
- do not accidentally store per-window UI state globally

---

## Widget Styling and UI Modules

The current project uses `src/app/styles.rs` for theme constants and widget style functions.

- Keep reusable style functions in `styles.rs`.
- Keep `view.rs` focused on composition/layout.
- Avoid scattering color constants and magic dimensions across multiple files.

If UI complexity grows:
- split reusable widget builders/components into new modules (for example `app/widgets/*`)
- keep message interfaces explicit

---

# Rust Best Practices

## Ownership and Borrowing

- Avoid unnecessary cloning (especially icons, strings, and large state).
- Prefer borrowing where possible.
- Use `Arc` only when cross-thread/shared ownership is necessary.
- Do not introduce `Rc<RefCell<...>>` without clear justification.

---

## Error Handling

- Use structured errors where practical (`thiserror` is preferred for reusable error types).
- `anyhow` is acceptable at app/service boundaries and integration layers (already used in this repo).
- Never `unwrap()` in non-test code unless guaranteed safe and documented.
- Surface user-facing errors in state so `view` can render them clearly.

---

## Avoid Global State

- No mutable statics.
- No hidden singletons.
- Pass state explicitly through `ProteusApp`, modules, and helpers.

---

## State Management

Keep a single root state struct (`ProteusApp`) and split by responsibility.

Example pattern for this repo:

```rust
struct ProteusApp {
    windows: HashMap<window::Id, PlayerWindowState>,
    focused_window: Option<window::Id>,
    native_menu: Option<NativeMenu>,
    global_error: Option<String>,
    // ...
}
```

State mutations should be:
- centralized
- predictable
- easy to test
- explicit about global vs per-window scope

---

# Concurrency and Threading

## Do Not Block the UI Thread

- No long filesystem/network/dialog/playback setup calls in `view`.
- Keep `update` fast.
- Move heavy work into `Task::perform` or dedicated worker threads/services when needed.

---

## Thread Safety (Iced + Native Integrations)

- Assume UI-facing state is main-thread owned.
- Do not mutate UI state from background threads directly.
- Return results through `Task<Message>` / message passing.
- Respect platform threading constraints (especially macOS AppKit calls).

If introducing threads:
- keep non-`Send` UI values out of thread closures
- communicate using channels/messages
- re-enter the app through `Message`

---

# Platform Integration Guidelines

This repo includes platform-specific behavior (macOS menu/app icon/name).

- Keep `#[cfg(...)]` branches localized and readable.
- Prefer wrapper functions in `effects.rs` / platform modules over sprinkling OS-specific code throughout the app.
- Preserve cross-platform behavior parity when adding features, or document intentional differences.

Native menu handling (`muda`) should remain an integration layer:
- map menu IDs to semantic actions
- convert actions into app messages/state transitions
- avoid embedding app logic inside `native_menu.rs`

---

# Testing Strategy

GUI rendering is harder to test directly; logic is not.

- Put testable logic in `state.rs`, `playback.rs`, and pure helpers.
- Unit test:
  - message-to-state transitions (where practical)
  - formatting helpers
  - validation/parsing
  - playback wrapper behavior (especially feature-flag behavior)
- Keep `view` thin so most behavior is testable outside widget construction.

When adding non-trivial logic to `update`, consider extracting a pure helper that can be unit tested.

---

# Formatting and Linting

Before considering work complete:

- `cargo check`
- `cargo fmt`
- `cargo clippy --all-targets --all-features`
- `cargo test`

No new warnings should be introduced.

---

# Performance Guidelines

- Avoid repeated allocations in high-frequency UI paths (especially per-tick refresh logic).
- Keep `Message` payloads lean and intentional.
- Cache derived data only when measurement shows it matters.
- Keep UI updates minimal and targeted per window.
- Avoid cloning large strings/state just to satisfy a match arm; restructure borrows instead.

---

# Documentation-First Development (Required)

Before implementing or modifying functionality that depends on Iced or any third-party crate:

1. **Search crate documentation first.**
   - Read official API docs on `docs.rs`.
   - Confirm exact types and signatures (`Task`, `Subscription`, `Element`, widget builders).
   - Verify thread-safety and runtime behavior before introducing async or threads.

2. **Read Iced documentation/examples for the specific feature.**
   - Focus on:
     - `daemon` / application wiring
     - `Task`
     - `Subscription`
     - multi-window APIs (`iced::window`)
     - keyboard/event handling
     - styling APIs for current Iced version

3. **Review official examples / source.**
   - Follow established Iced patterns instead of inventing ad hoc architecture.
   - Check version-specific examples that match the crate version used in this repo.

4. **Search crate source when needed.**
   - Use docs.rs source view / upstream source to confirm trait bounds and behavior.
   - Verify whether a widget/style API is current or deprecated before coding.

5. **Do not guess APIs.**
   - Never assume a widget method, callback signature, or style function shape.
   - Verify platform constraints for `muda`, `rfd`, and macOS APIs before changing threading behavior.

---

### Rule of Thumb

If you are about to:
- invent a workaround for an Iced API mismatch,
- bypass message-driven state flow,
- block inside `update`,
- duplicate functionality already provided by Iced,
- guess how a widget or subscription API behaves,

-> **Stop and check the documentation first.**

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
4. Preserve existing architecture patterns unless there is a clear reason to refactor.

---

# Definition of Done

- Builds successfully.
- No formatting or clippy warnings.
- Tests pass.
- No UI thread blocking introduced.
- Clear separation between:
  - Iced view rendering
  - App state transitions
  - Business/domain logic
  - Side effects/platform integration

---

# Quick Do / Don’t Checklist

## Do

- Keep `view` pure and declarative.
- Keep side effects in `Task`-driven effect modules.
- Use `Message` enums to represent intent/results explicitly.
- Keep global vs per-window state boundaries clear.
- Keep platform-specific code isolated.
- Keep files small and cohesive.

## Don’t

- Block inside `update` or `view`.
- Put OS/dialog logic directly in widget closures.
- Collapse `messages`, `state`, `view`, and `effects` into one file.
- Introduce hidden global mutable state.
- Guess how Iced APIs work instead of checking docs/examples.

---

If any guideline conflicts with established project conventions, follow the existing convention and leave a note explaining the deviation.
