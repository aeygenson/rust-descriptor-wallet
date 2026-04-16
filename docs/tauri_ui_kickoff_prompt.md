# Tauri UI Kickoff Prompt for `rust-descriptor-wallet`

Use this prompt to start a **new discussion** focused on building the Tauri desktop UI for the existing `rust-descriptor-wallet` project.

---

## Prompt

I want to start the **Tauri desktop UI** for my existing Rust Bitcoin wallet project, `rust-descriptor-wallet`.

Please help me design and implement the UI **on top of the current backend**, not by rewriting the wallet logic.

You should assume the backend already exists and is meaningful. The goal is to create a real desktop wallet shell that uses the existing crates, APIs, CLI/runtime patterns, and regtest-tested transaction flows.

Please first **analyze the current architecture**, then propose a clean Tauri UI plan, recommended folder structure, command surface, state management, and a step-by-step implementation order.

I want the response to be practical and tailored to this project, not generic Tauri advice.

---

## Project context

This is a multi-crate Rust wallet project centered around descriptor-based Bitcoin wallet functionality.

### Main project goals
- Bitcoin descriptor wallet
- Rust-first architecture
- clear crate boundaries
- no unnecessary ORM / framework complexity
- strong backend correctness before GUI
- regtest-driven integration testing
- realistic wallet behaviors: funding, send, RBF, CPFP, coin control, send-max, sweep, consolidation

### High-level crate intent

#### `wallet_core`
Core wallet/business logic.
This is where transaction-building behavior lives.

Important domain/model concepts already exist or were recently added:
- `WalletPsbtInfo`
- `WalletCoinControlInfo`
- `WalletCpfpPsbtInfo`
- `WalletSendAmountMode`
  - `Fixed(AmountSat)`
  - `Max`

Important behavior already exists:
- normal PSBT creation
- coin control
- strict input selection
- send-max
- sweep semantics
- wallet-internal consolidation
- RBF bump PSBT
- CPFP PSBT

Sweep is intentionally modeled as:
- `WalletSendAmountMode::Max`
- plus explicit include set in `WalletCoinControlInfo`

#### `wallet_api`
Async API facade over core logic.
This is the main layer the future UI should call.

Important API capabilities already exist:
- wallet sync
- wallet address
- wallet balance
- wallet transactions
- wallet UTXOs
- PSBT create
- PSBT create with coin control
- send PSBT
- send with coin control
- bump fee PSBT / bump fee
- CPFP PSBT / CPFP
- send-max create/send
- sweep create/send
- consolidation create/consolidate

Important DTOs already exist:
- `WalletPsbtDto`
- `WalletCoinControlDto`
- transaction broadcast result DTOs
- wallet/tx/utxo DTOs

`WalletPsbtDto` includes useful UI-facing fields such as:
- `psbt_base64`
- `txid`
- `original_txid`
- `to_address`
- `amount_sat`
- `fee_sat`
- `fee_rate_sat_per_vb`
- `replaceable`
- `change_amount_sat`
- `selected_utxo_count`
- `selected_inputs`
- `input_count`
- `output_count`
- `recipient_count`
- `estimated_vsize`

#### `wallet_cli`
This is important.
The CLI already acts as a thin user-facing layer over the API/runtime, and I want the Tauri UI to stay consistent with it.

The CLI/runtime currently exposes flows such as:
- create PSBT
- create PSBT with coin control
- sign PSBT
- publish PSBT
- send PSBT
- send PSBT with coin control
- create send-max PSBT
- create send-max PSBT with coin control
- send-max
- send-max with coin control
- create sweep PSBT
- sweep
- create consolidation PSBT
- consolidate
- bump fee PSBT
- bump fee
- CPFP PSBT
- CPFP

I want the Tauri UI to treat `wallet_cli` as a strong reference for:
- user-facing workflow shape
- naming consistency
- output/operation expectations
- practical command grouping

I do **not** want to replace the backend with frontend-specific logic.

#### `test_support`
Regtest support utilities.

### Current quality level
The backend is not toy-level anymore.
It already has regtest coverage for:
- receiving funds after sync
- self-send with change
- RBF replacement
- CPFP build / broadcast / confirm
- explicit outpoint selection
- coin control include/exclude/conflicts
- strict insufficient-input behavior
- send-max build / sweep
- sweep as first-class API path
- consolidation candidate selection and wallet-internal output checks
- multi-input strict flows
- confirmed-only behavior
- no-internal-change invariants in sweep-like flows

This means the backend should be treated as relatively stable, and the UI should build on it rather than reshaping it.

---

## Important product/UX expectations

I want the Tauri UI to be useful for real wallet work, not just a demo shell.

At minimum it should include, but not be limited to:

### 1. Wallet summary/home screen
- wallet name
- network
- main balance
- quick actions
- sync action / sync status
- maybe latest transactions summary

### 2. UTXO table
This is especially important because it will shine with coin control.

The UTXO table should support:
- outpoint
- value
- confirmed/unconfirmed
- keychain (external/internal)
- maybe spendable state if available
- sorting
- filtering
- multi-select checkboxes
- ability to prefill send / send-max / sweep / consolidation forms from selected UTXOs

### 3. Send form
A real send form, not just a single address/amount screen.

It should support:
- destination address
- amount
- fee rate
- RBF enabled by default if appropriate
- manual UTXO selection
- exclude selected UTXOs
- confirmed-only toggle
- PSBT preview before broadcast
- normal fixed send
- send-max
- sweep
- consolidation

### 4. PSBT preview / confirmation panel
Before signing or broadcasting, show:
- txid if already derivable
- destination
- amount
- fee
- fee rate
- selected inputs
- selected input count
- estimated vsize
- change amount
- replaceable yes/no
- whether this is fixed send / send-max / sweep / consolidation / CPFP / RBF

### 5. Transactions screen
Should show:
- txid
- direction
- amount / net value
- fee if present
- confirmed/unconfirmed
- maybe action buttons for eligible cases:
  - bump fee
  - CPFP

### 6. CPFP and RBF actions
I want the UI architecture to anticipate:
- selecting an unconfirmed parent output for CPFP
- bumping an RBF transaction
- surfacing replaceability clearly

### 7. Sweep-specific UX
Sweep should feel explicit, not hidden.
For example:
- select one or more UTXOs from UTXO table
- choose “Sweep selected”
- enter destination + fee rate
- preview no-change behavior
- sign/broadcast

### 8. Consolidation-specific UX
Consolidation should be presented as wallet maintenance, not as a recipient payment.
For example:
- select two or more UTXOs from the UTXO table or use automatic selection
- choose “Consolidate selected”
- choose fee rate, confirmation policy, input-count/value filters, fee ceiling, and strategy
- preview that the output is wallet-internal
- sign/broadcast

---

## Technical constraints and preferences

### Core preference
Use the **existing Rust backend** through Tauri commands.
Do not duplicate wallet business logic in TypeScript.

### UI stack
I am open, but prefer a modern practical frontend stack inside Tauri, for example:
- Tauri v2
- React + TypeScript
- simple component system
- maintainable state management
- good table support

### Architecture preferences
Please propose:
- Tauri command layout
- frontend folder structure
- domain-oriented state model
- DTO mapping strategy
- async loading/error handling strategy
- how to share types safely where useful
- how to keep the UI thin and backend-driven

### Important design principle
The Tauri app should initially be a **thin shell** over the API:
- read wallet state
- render it
- call API methods
- preview results
- sign/broadcast intentionally

Do not start with heavy abstraction layers unless truly justified.

---

## What I want from you

Please help me with all of the following:

### A. Analyze the current backend and explain how the Tauri UI should map onto it
Especially:
- `wallet_core`
- `wallet_api`
- `wallet_cli`
- regtest-backed flows

### B. Propose a Tauri project structure
Include:
- Rust side structure
- frontend structure
- command modules
- state/store modules
- reusable UI components

### C. Recommend the first implementation milestone
I want the best first vertical slice.
For example, maybe:
- wallet summary
- UTXO table
- send form with manual selection
- PSBT preview

Please justify the order.

### D. Suggest concrete Tauri commands to implement first
Especially commands around:
- wallet summary
- balance
- tx list
- utxo list
- create PSBT
- create send-max
- create sweep
- create consolidation
- send / broadcast

### E. Design the UTXO table and send flow UX in detail
This is a priority area.

### F. Explain how the UI should integrate with the existing CLI/runtime mental model
Because I want naming and behavior consistency with `wallet_cli`.

### G. Provide a phased implementation plan
For example:
1. shell/app layout
2. summary data
3. UTXO table
4. send fixed
5. send-max
6. sweep
7. consolidation
8. RBF / CPFP
9. polish

### H. Highlight likely pitfalls
Such as:
- overcoupling frontend state
- duplicating backend logic
- making coin control confusing
- not representing strict sweep semantics clearly
- presenting consolidation like an external payment instead of wallet maintenance
- poor PSBT preview UX

---

## Extra guidance

Please be opinionated.
I do not want a vague answer.

I want:
- practical architecture
- suggested file/module layout
- recommended first commands
- best UI flows for coin control / sweep / send-max / consolidation
- concrete next coding steps

Also, because this project already has a meaningful CLI and backend, please keep the plan tightly anchored to what is already built.

If useful, you can also suggest:
- how to reuse or mirror DTO types
- whether the Tauri UI should have tabs/pages/panels
- what the first mock screens should be
- how to represent unconfirmed and replaceable transactions clearly

---

## Notes for the new discussion

If you need to make assumptions, prefer assumptions that preserve:
- backend-first design
- correctness
- regtest-testability
- explicit user control
- consistency with the current `wallet_cli`

When discussing transaction flows, use the existing project vocabulary:
- PSBT
- coin control
- send-max
- sweep
- consolidation
- RBF
- CPFP
- selected inputs
- explicit include set
- strict mode
