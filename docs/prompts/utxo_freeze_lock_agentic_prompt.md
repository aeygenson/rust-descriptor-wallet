# Agentic Implementation Prompt: UTXO Freeze / Lock GUI

## Context

You are working in the `rust-descriptor-wallet` project.

This is a descriptor-based Bitcoin wallet with:

- Rust backend crates:
  - `wallet_core`
  - `wallet_api`
  - `wallet_storage`
  - `wallet_sync`
  - `test_support`
- Frontend app:
  - `apps/wallet_desktop`
  - Tauri + React + TypeScript + Vite

Current GUI already supports:

- Overview page
- Send page
- UTXO page
- Transactions page
- Coin control
- Send fixed from selected UTXOs
- Send max
- Sweep
- Consolidation
- RBF
- CPFP
- Transaction graph/lineage helpers
- Backend health
- Shared frontend feature patterns:
  - `api.ts`
  - `types.ts`
  - `lib.ts`
  - `format.ts`
  - `components/`
  - page orchestration only

The recent architecture pattern is:

```text
features/<feature>/
  api.ts       Tauri/backend calls only
  types.ts     DTO-facing frontend types + component prop types
  lib.ts       pure business/state helpers
  format.ts    presentation formatting only
  components/  React components only
pages/
  XxxPage.tsx  orchestration only
```

Follow this pattern strictly.

---

## Goal

Implement **backend-persisted UTXO freeze / lock** functionality across backend, CLI, Tauri, and GUI.

The user should be able to:

```text
CLI or GUI
→ lock/freeze one or more UTXOs
→ persist lock state in backend storage
→ list locked UTXOs from CLI/API/GUI
→ enforce locked-coin protection in spending flows
→ show frozen state in GUI table
→ unlock them later from CLI or GUI
```

The feature should make UTXO management safer and more professional.

---

## Product semantics

A frozen/locked UTXO means:

```text
Do not spend this UTXO unless the user explicitly unlocks it or overrides freeze protection.
```

For v1:

- Frozen UTXOs should not be automatically selected for:
  - Send fixed
  - Send max
  - Sweep
  - Consolidation
- Frozen UTXOs should not be included in `Select All`
- Frozen UTXOs should be visually marked
- Frozen UTXOs should be toggleable from UTXO row/action menu
- Frozen state must be persisted in backend storage, not localStorage
- CLI and GUI must use the same backend source of truth
- Spending flows must reject locked UTXOs unless an explicit future override is implemented

---

## Important implementation decision

Use backend-persisted lock state as the source of truth.

The frontend must be presentation/orchestration only:

```text
wallet_storage
  stores locked/frozen outpoints per wallet

wallet_core
  validates coin-control spending and rejects locked outpoints

wallet_api
  exposes DTOs and service methods

wallet_cli
  exposes lock / unlock / list commands

apps/wallet_desktop/src-tauri
  exposes Tauri commands that call wallet_api

wallet_desktop React frontend
  renders badges/buttons/warnings and calls Tauri commands
```

First inspect existing backend capabilities.

Search for existing support in:

```text
wallet_core
wallet_api
wallet_storage
wallet_sync
wallet_cli
apps/wallet_desktop/src-tauri/src/commands
```

Look for terms:

```text
freeze
lock
locked
coin lock
lock_utxo
unlock_utxo
excluded
spendable
coin control
```

If partial backend support exists, extend it cleanly.

If no support exists, implement backend persistence first. Do **not** use localStorage for the final implementation.

Local frontend freeze state is allowed only as a temporary implementation spike, not as the accepted final result.

---

## Non-goals for v1

Do NOT implement:

- hardware wallet policies
- automatic privacy freezing
- coinjoin-style freeze categories
- complex migration framework redesign
- global cross-device lock sync
- descriptor-level policy changes
- permanent blacklist semantics

Keep v1 simple:

```text
wallet-specific backend-persisted locked outpoint list
```

---

## Required backend architecture

Implement the feature in this direction:

```text
wallet_storage
  locked_utxos table or equivalent persisted storage
  keyed by wallet name + outpoint
  stores optional reason and locked_at timestamp

wallet_core
  exposes lock-aware validation helpers
  rejects locked outpoints in coin-control spending paths
  ensures send fixed, send max, sweep, and consolidation do not spend locked coins

wallet_api
  defines DTOs and API methods for lock/list/unlock
  converts backend errors into stable API errors

wallet_cli
  exposes user-facing commands:
    lock-utxo
    unlock-utxo
    list-locked-utxos

apps/wallet_desktop/src-tauri
  exposes commands:
    lock_utxos
    unlock_utxos
    list_locked_utxos

wallet_desktop React
  fetches locked UTXOs from backend
  calls backend lock/unlock commands
  renders frozen state only
```

Do not let the GUI maintain its own independent frozen state.

---

## Backend DTOs and API shape

Add or adapt DTOs in `wallet_api/src/model.rs` or the existing API DTO layer:

```rust
pub struct WalletLockedUtxoDto {
    pub outpoint: String,
    pub reason: Option<String>,
    pub locked_at: Option<String>,
}

pub struct WalletLockUtxosRequestDto {
    pub wallet_name: String,
    pub outpoints: Vec<String>,
    pub reason: Option<String>,
}

pub struct WalletUnlockUtxosRequestDto {
    pub wallet_name: String,
    pub outpoints: Vec<String>,
}

pub struct WalletLockedUtxosDto {
    pub wallet_name: String,
    pub locked_utxos: Vec<WalletLockedUtxoDto>,
}
```

Add API/service methods equivalent to:

```rust
lock_utxos(wallet_name, outpoints, reason)
unlock_utxos(wallet_name, outpoints)
list_locked_utxos(wallet_name)
```

Keep DTOs string-based at API boundaries where that matches existing project style, then parse to typed outpoints inside core/storage layers.

---

## CLI requirements

Add CLI commands equivalent to:

```text
wallet lock-utxo --wallet <name> --outpoint <txid:vout> [--reason <text>]
wallet unlock-utxo --wallet <name> --outpoint <txid:vout>
wallet list-locked-utxos --wallet <name>
```

If the CLI command structure uses subcommands differently, follow the existing CLI style.

CLI behavior:

- locking an already locked UTXO should be idempotent or return a clear already-locked message
- unlocking an unlocked UTXO should be idempotent or return a clear not-locked message
- list command should print outpoint, reason, and locked timestamp if available
- CLI spending commands must respect locked UTXOs the same way GUI spending does

---

## Desired UX

On UTXO page:

### Table

Add a frozen/locked indicator column or badge:

```text
Frozen
Locked
Spendable
```

Suggested row styling:

- frozen row slightly muted
- lock icon/badge
- selected frozen UTXO either disallowed or warned

### Row/action controls

For each UTXO:

```text
Freeze
Unfreeze
```

For selected UTXOs:

```text
Freeze selected
Unfreeze selected
```

Current UTXO actions already include:

```text
Send Fixed
Send Max
Sweep
Consolidate
Clear
```

Extend with:

```text
Freeze
Unfreeze
```

or place freeze/unfreeze in a secondary action group.

GUI state must come from backend:

```text
list_locked_utxos(walletName)
```

GUI lock/unlock actions must call backend:

```text
lock_utxos(walletName, outpoints, reason?)
unlock_utxos(walletName, outpoints)
```

---

## Behavior rules

### Select all

`Select All` should select only spendable/unfrozen UTXOs by default.

Optional future button:

```text
Select all including frozen
```

Do not add this in v1 unless simple.

### Send fixed from selected

If selected UTXOs include frozen UTXOs:

Preferred v1 behavior:
- prevent navigation
- show error/warning:

```text
Selected UTXOs include frozen coins. Unlock them before spending.
```

Alternative:
- automatically filter frozen UTXOs out

Preferred: prevent accidental spend.

### Send max

Should exclude locked UTXOs at backend/core level.

If all available UTXOs are locked:

```text
No spendable UTXOs available.
```

### Sweep

Should exclude locked UTXOs at backend/core level.

If user selected locked UTXOs:

```text
Locked UTXOs cannot be swept until unlocked.
```

### Consolidation

Should exclude locked UTXOs at backend/core level.

Consolidation must require at least two spendable selected UTXOs.

---

## Frontend file structure

You already have:

```text
apps/wallet_desktop/src/features/utxos/
  api.ts
  types.ts
  lib.ts
  format.ts
  components/
  utxos.css
apps/wallet_desktop/src/pages/UtxosPage.tsx
```

Frontend updates:

```text
apps/wallet_desktop/src/features/utxos/types.ts
apps/wallet_desktop/src/features/utxos/lib.ts
apps/wallet_desktop/src/features/utxos/format.ts
apps/wallet_desktop/src/features/utxos/components/UtxosTable.tsx
apps/wallet_desktop/src/features/utxos/components/UtxoActionsBar.tsx
apps/wallet_desktop/src/features/utxos/components/UtxoSelectionSummary.tsx
apps/wallet_desktop/src/features/utxos/utxos.css
apps/wallet_desktop/src/pages/UtxosPage.tsx
```

Backend/CLI/Tauri files to inspect and update as needed:

```text
crates/wallet_api/src/model.rs
crates/wallet_api/src/lib.rs
crates/wallet_core/src/**
crates/wallet_storage/src/**
apps/wallet_cli/src/**
apps/wallet_desktop/src-tauri/src/commands/**
apps/wallet_desktop/src-tauri/src/main.rs or lib.rs
```

Create only if needed:

```text
apps/wallet_desktop/src/features/utxos/components/UtxoFreezeBadge.tsx
apps/wallet_desktop/src/features/utxos/components/UtxoFreezeActions.tsx
```

Do not create unnecessary components if the change is small.

---

## Types to add

In `features/utxos/types.ts` add types similar to:

```ts
export type LockedUtxoDto = {
  outpoint: UtxoOutpoint;
  reason?: string | null;
  lockedAt?: string | null;
};

export type LockedUtxoSet = Set<UtxoOutpoint>;

export type UtxoLockAction = "lock" | "unlock";
```

Update component props:

```ts
export interface UtxosTableProps {
  ...
  lockedOutpoints: UtxoOutpoint[];
  onLockOutpoint: (outpoint: UtxoOutpoint) => void;
  onUnlockOutpoint: (outpoint: UtxoOutpoint) => void;
}
```

Update action bar props:

```ts
export interface UtxoSelectionActionBarProps {
  ...
  lockedSelectedCount?: number;
  spendableSelectedCount?: number;
  onLockSelected?: () => void;
  onUnlockSelected?: () => void;
}
```

Keep prop types in `types.ts`. Do not define prop types inside components.

---

## `features/utxos/lib.ts` helpers

Add pure helpers:

```ts
export function normalizeLockedOutpoints(outpoints: UtxoOutpoint[]): UtxoOutpoint[]

export function isUtxoLocked(
  outpoint: UtxoOutpoint,
  lockedOutpoints: UtxoOutpoint[]
): boolean

export function filterSpendableUtxos(
  utxos: WalletUtxoDto[],
  lockedOutpoints: UtxoOutpoint[]
): WalletUtxoDto[]

export function filterLockedUtxos(
  utxos: WalletUtxoDto[],
  lockedOutpoints: UtxoOutpoint[]
): WalletUtxoDto[]

export function getSpendableSelectedOutpoints(
  selectedOutpoints: UtxoOutpoint[],
  lockedOutpoints: UtxoOutpoint[]
): UtxoOutpoint[]

export function getLockedSelectedOutpoints(
  selectedOutpoints: UtxoOutpoint[],
  lockedOutpoints: UtxoOutpoint[]
): UtxoOutpoint[]

export function selectAllSpendableVisibleOutpoints(
  utxos: WalletUtxoDto[],
  lockedOutpoints: UtxoOutpoint[]
): UtxoOutpoint[]
```

Do not use React inside `lib.ts`.
Do not access localStorage inside `lib.ts` for this feature.
Persistence belongs to backend storage.

---

## `features/utxos/format.ts`

Add presentation helpers:

```ts
export function formatLockedState(isLocked: boolean): string {
  return isLocked ? "Locked" : "Spendable";
}

export function formatLockedSelectionWarning(count: number): string {
  return `${count} selected UTXO${count === 1 ? "" : "s"} locked`;
}
```

Do not place business logic here.

---

## UTXO page orchestration

In `UtxosPage.tsx`:

Add state:

```ts
const [lockedOutpoints, setLockedOutpoints] = useState<UtxoOutpoint[]>([]);
const [selectionWarning, setSelectionWarning] = useState<string | null>(null);
```

On wallet change:

```text
call list_locked_utxos for selected wallet
set locked outpoints from backend response
clear selected outpoints
clear warnings
```

After UTXO load:

```text
do not mutate backend lock state just because a locked UTXO is not currently visible
hidden locked UTXOs may reappear after rescan or reorg
table only displays current matching locked state
```

---

## Navigation to Send page

Currently UTXO actions navigate with:

```ts
mode: "fixed" | "send_max" | "sweep" | "consolidate"
selectedOutpoints
```

Before navigating:

```ts
const lockedSelected = getLockedSelectedOutpoints(validSelectedOutpoints, lockedOutpoints);

if (lockedSelected.length > 0) {
  setSelectionWarning("Selected UTXOs include locked coins. Unlock them before spending.");
  return;
}
```

Then pass only spendable selected outpoints. Backend/core must still validate this independently.

Do this for:

- fixed
- send max
- sweep
- consolidate

Do not allow locked selected UTXOs to silently pass through.

---

## Select all behavior

Current `selectAllVisibleOutpoints(utxos)` should become lock-aware.

Option A:

```ts
selectAllSpendableVisibleOutpoints(utxos, lockedOutpoints)
```

Add helper in lib.

Then in page:

```ts
setSelectedOutpoints(selectAllSpendableVisibleOutpoints(utxos, lockedOutpoints));
```

This is safer than selecting locked coins.

---

## UTXO table changes

Add a column:

```text
State
```

Possible rendering:

```tsx
<span className={isLocked ? "utxo-lock-badge is-locked" : "utxo-lock-badge is-spendable"}>
  Locked / Spendable
</span>
```

Add row class:

```tsx
className={isSelected ? "is-selected" : isLocked ? "is-locked" : undefined}
```

Better:

```tsx
className={[
  isSelected ? "is-selected" : "",
  isLocked ? "is-locked" : "",
].filter(Boolean).join(" ")}
```

Add per-row action button:

```text
Lock / Unlock
```

Do not make table too busy. If crowded, put per-row lock action in a compact button.

---

## UTXO actions bar changes

When selection exists, show:

```text
Lock selected
Unlock selected
```

Suggested behavior:

- if selected contains any spendable UTXOs → show Lock selected
- if selected contains any locked UTXOs → show Unlock selected

Keep spend actions disabled or blocked if selected contains locked UTXOs.

Action bar should show:

```text
2 selected · 1 locked · 1 spendable
```

---

## Send page integration

In v1, most protection should happen in UTXO page before navigation.

But also consider adding protection in Send page if navigation state includes frozen metadata later.

Do not overcomplicate Send page for v1.

Future enhancement:

```ts
SendPageNavigationState {
  mode?: SendMode;
  selectedOutpoints?: string[];
  frozenOutpoints?: string[];
}
```

Not required now if UTXO page blocks frozen spending.

---

## Backend integration requirements

Backend integration is required for this feature.

Implement or extend DTOs similar to:

```rust
pub struct WalletLockedUtxoDto {
    pub outpoint: String,
    pub reason: Option<String>,
    pub locked_at: Option<String>,
}
```

Implement Tauri commands:

```rust
lock_utxos
unlock_utxos
list_locked_utxos
```

Implement frontend API wrappers:

```ts
lockUtxos(walletName, outpoints, reason?)
unlockUtxos(walletName, outpoints)
listLockedUtxos(walletName)
```

Backend/core must prevent accidental spending of locked coins. GUI checks are a safety layer, not the enforcement boundary.

---

## Storage design

Use backend storage, not browser localStorage.

Suggested persisted record shape:

```text
wallet_name
outpoint
reason nullable
locked_at timestamp
```

Suggested uniqueness rule:

```text
unique(wallet_name, outpoint)
```

Rules:

- locking should be idempotent or return a stable already-locked error
- unlocking should be idempotent or return a stable not-locked error
- listing should be wallet-scoped
- no private keys or secrets are stored
- do not delete lock records merely because an outpoint is not visible in the current UTXO set

---

## CSS

```css
.utxos-table tbody tr.is-locked {
  opacity: 0.72;
}

.utxos-table tbody tr.is-locked td {
  color: #94a3b8;
}

.utxo-lock-badge {
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  padding: 4px 9px;
  font-size: 12px;
  font-weight: 800;
}

.utxo-lock-badge.is-spendable {
  background: rgba(34, 197, 94, 0.14);
  color: #86efac;
}

.utxo-lock-badge.is-locked {
  background: rgba(148, 163, 184, 0.14);
  color: #cbd5e1;
}

.utxo-warning {
  border: 1px solid rgba(245, 158, 11, 0.32);
  background: rgba(245, 158, 11, 0.08);
  color: #fbbf24;
}
```

Keep styles aligned with existing dark UI.

---

## Manual testing checklist

### Basic lock/unlock

1. Open UTXO page with at least 2 UTXOs.
2. Select one UTXO.
3. Click Lock selected.
4. Confirm row displays Locked.
5. Reload app.
6. Confirm locked state persists.
7. Click Unlock.
8. Confirm row returns to Spendable.

### CLI/backend consistency

1. Lock a UTXO from CLI.
2. Open GUI UTXO page.
3. Confirm the same UTXO appears Locked.
4. Unlock the UTXO from GUI.
5. Run CLI list command.
6. Confirm it no longer appears locked.

### Select all

1. Lock one UTXO.
2. Click Select all.
3. Confirm locked UTXO is not selected.

### Send protection

1. Manually select a locked UTXO.
2. Click Send Fixed.
3. Confirm navigation is blocked and warning shown.
4. Unlock.
5. Click Send Fixed.
6. Confirm Send page opens with selected outpoint.

### Sweep/consolidation protection

1. Lock one UTXO.
2. Select locked + spendable UTXO.
3. Click Sweep.
4. Confirm warning.
5. Click Consolidate.
6. Confirm warning.

### Wallet switching

1. Lock UTXO in `regtest-local`.
2. Switch to another wallet.
3. Confirm no locked state leaks.
4. Switch back.
5. Confirm locked state returns.

---

## TypeScript/code quality checklist

Before finishing:

```text
No local prop types in components
No duplicate formatting helpers in components
No React code in lib.ts
No localStorage usage for lock state
No locked UTXOs passed into Send navigation
Frontend lock state comes only from backend API/Tauri commands
No TypeScript errors
```

---

## Recommended implementation phases

### Phase 1 — Inspect backend

- Check whether backend has lock/freeze support.
- If missing, implement backend storage/API/CLI/Tauri support.
- Report exact backend persistence design.

### Phase 2 — Types + lib helpers

- Add lock DTO-facing types to `utxos/types.ts`
- Add pure lock helpers to `utxos/lib.ts`
- Add formatting helpers to `utxos/format.ts`

### Phase 3 — Page state

- Add `lockedOutpoints`
- Load locked UTXOs from backend by selected wallet
- Add lock/unlock handlers that call Tauri/backend commands
- Make select-all lock-aware
- Block send navigation when locked selected

### Phase 4 — Table UI

- Add lock state column/badge
- Add row class
- Add per-row lock/unlock button if not too crowded

### Phase 5 — Actions bar UI

- Add selected locked/spendable counts
- Add Lock selected / Unlock selected
- Show warning when spending locked selection

### Phase 6 — Polish

- CSS
- empty/warning states
- copy refinement
- manual tests

---

## Expected final result

At the end, GUI supports:

```text
Lock selected UTXOs
Unlock selected UTXOs
Locked badge in UTXO table
Locked rows visually distinct
Locked UTXOs excluded from select-all
Locked UTXOs blocked from send/sweep/consolidation navigation
Backend-persisted wallet-specific lock state
CLI lock/unlock/list commands
GUI and CLI share same source of truth
Clean warning UX
```

This adds a professional Bitcoin wallet capability and improves safety around coin control.

---

## Final response expected from implementation agent

Summarize:

```text
Files created
Files updated
Backend persistence design
CLI commands added
Tauri commands added
Exact behavior rules implemented
Manual tests performed
Known limitations
```
