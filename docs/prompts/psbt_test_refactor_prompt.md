# 🧠 Agentic Prompt: Refactor Large Rust Integration Test File (PSBT / Wallet API)

## 🎯 Goal
Refactor a large Rust integration test file (`regtest_flow.rs`) into a **modular, maintainable test suite** without changing behavior.

The project is a **Bitcoin descriptor wallet (BDK-based)** with PSBT workflows, coin control, sweep/send-max, consolidation, CPFP, and RBF.

---

## 📦 Current Problem
- One file contains **hundreds of tests across multiple domains**
- Hard to navigate, debug, and extend
- Slows down development and increases cognitive load

---

## 🧩 Target Structure

```
crates/wallet_api/tests/

├── common/
│   ├── mod.rs
│   ├── regtest_env.rs
│   ├── helpers.rs
│
├── psbt_coin_control.rs
├── send_max.rs
├── sweep.rs
├── consolidation.rs
├── cpfp.rs
├── rbf.rs
```

---

## 🔪 Step-by-Step Refactor Plan

### 1. Extract Shared Utilities

Move all reusable helpers from `regtest_flow.rs` into:

#### `common/regtest_env.rs`
- `RegtestEnv`
- `.start()`
- `.mine()`
- `.fund_sats()`

#### `common/helpers.rs`
- `ensure_confirmed_wallet_utxos`
- `fund_exact_confirmed_wallet_utxos`
- `decode_psbt_inputs`
- `parse_txid`
- `outpoint_txid`
- `mempool_contains`
- `parse_regtest_address`

#### `common/mod.rs`
```rust
pub mod regtest_env;
pub mod helpers;

pub use regtest_env::*;
pub use helpers::*;
```

---

### 2. Split Tests by Domain

#### 📄 `psbt_coin_control.rs`
Move:
- `wallet_create_psbt_with_coin_control_*`
- `wallet_send_psbt_with_coin_control_*`
- input/output consistency tests

---

#### 📄 `send_max.rs`
Move:
- `wallet_create_send_max_psbt_*`
- `wallet_send_max_psbt_*`

---

#### 📄 `sweep.rs`
Move:
- `wallet_create_sweep_psbt_*`
- `wallet_sweep_psbt_*`
- strict/no-change behavior

---

#### 📄 `consolidation.rs`
Move:
- ALL `wallet_create_consolidation_psbt_*`
- `consolidate_and_broadcast`
- strategy tests (largest/smallest/oldest)
- filters (min/max value)
- fuzz test

⚠️ This file will be the largest — structure it internally:

```rust
mod happy_path {}
mod filters {}
mod strategies {}
mod edge_cases {}
mod fuzz {}
```

---

#### 📄 `cpfp.rs`
Move:
- `wallet_cpfp_psbt_*`

---

#### 📄 `rbf.rs`
Move:
- `wallet_create_psbt_respects_replaceable_flag`
- `wallet_create_send_max_psbt_respects_replaceable_flag`

---

### 3. Fix Imports

Each test file should include:

```rust
use crate::common::*;
```

And:

```rust
use serial_test::serial;
```

---

### 4. Preserve Test Behavior

DO NOT:
- Change test logic
- Change assertions
- Change test names

ONLY:
- Move code
- Fix imports
- Keep compilation working

---

### 5. Validate

Run:

```
cargo test -p wallet_api --tests
```

Then verify individual modules:

```
cargo test -p wallet_api --test consolidation
cargo test -p wallet_api --test send_max
```

---

## 🧪 Optional Improvements (Do After Split)

### Group tests inside files:

```rust
mod happy_path { ... }
mod error_cases { ... }
mod invariants { ... }
```

---

### Add naming consistency

Prefer:

```
wallet_<feature>_<expected_behavior>
```

Example:

```
wallet_consolidation_rejects_insufficient_inputs
```

---

## 🚨 Constraints

- Do NOT introduce new abstractions
- Do NOT rewrite logic
- Do NOT change async/test runtime
- Keep `#[tokio::test(flavor = "current_thread")]`
- Keep `#[serial]`

---

## ✅ Expected Result

- Each feature has its own test file
- Shared utilities centralized
- Faster navigation and debugging
- Easier to extend (RBF, policies, fee strategies)

---

## 🧠 Mental Model

Think of this as:

> “Turn one monolithic integration test into a domain-driven test suite”

---

## 🔚 Deliverables

- New folder structure
- Updated test files
- Clean imports
- All tests passing

---

## 💬 Notes

This is a **pure refactor task** — no behavior change allowed.
