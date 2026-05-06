# Agentic Implementation Prompt: Send Max / Sweep UI for `rust-descriptor-wallet`

## Goal

Implement a complete **Send Max / Sweep UI** in the Tauri + React desktop app for `rust-descriptor-wallet`.

The backend/API/CLI already expose Send Max and Sweep-style functionality. The task is to wire it cleanly into the frontend with a professional wallet UX, using the lessons learned from the completed CPFP implementation.

Do this incrementally, with compile/typecheck after each phase.

---

## Current Project Context

Repository:

```text
rust-descriptor-wallet
```

Important app/crate layout:

```text
crates/
  wallet_core
  wallet_api
  wallet_storage
  wallet_sync

apps/
  wallet_cli
  wallet_desktop
```

Frontend:

```text
apps/wallet_desktop
  src/
    pages/
      SendPage.tsx
      TransactionsPage.tsx
      UtxosPage.tsx
    features/
      transactions/
      send/
    shared/
      types/
        dtos.ts
```

Desktop stack:

```text
Tauri + React + TypeScript + Vite
```

Rust backend/API stack:

```text
wallet_core → wallet_api → Tauri commands → frontend API wrappers → React pages/components
```

---

## Important Lessons Learned From CPFP Work

### 1. DTO source of truth matters

Backend DTO layer:

```text
crates/wallet_api/src/model.rs
```

Frontend DTO mirror:

```text
apps/wallet_desktop/src/shared/types/dtos.ts
```

During CPFP, missing fields in `WalletTxDto` caused UI guesses and manual fallback. The correct fix was:

```rust
WalletTxDto.inputs
WalletTxDto.outputs
```

Then the frontend used real backend-owned data instead of guessing.

For Send Max / Sweep, do the same:

- Do not guess available UTXOs in random UI code.
- Use `listUtxos` / existing DTOs.
- If a required field is missing, add it to Rust DTO first, then mirror in TypeScript.

### 2. Keep API boundaries explicit

Frontend API wrappers should live near feature code, for example:

```text
apps/wallet_desktop/src/features/send/api.ts
```

or current existing API location if Send page already uses one.

Do not call Tauri `invoke` directly from deeply nested components. Use API wrapper functions.

### 3. Keep page state orchestration in page, form display in components

Good pattern:

```text
SendPage.tsx
  owns selectedWalletName, PSBT result, signed result, broadcast result, loading/error state

components/
  SendForm.tsx
  CoinControlSelector.tsx
  PsbtPreviewPanel.tsx
```

Avoid making child components directly invoke backend commands unless that is already the established project pattern.

### 4. Use reusable DTOs and component props

CPFP succeeded once state was clearly named:

```ts
cpfpPsbtDto
createCpfpPsbt
```

Avoid name collisions like:

```ts
cpfpPsbt // both function and state
```

For Send Max / Sweep use names like:

```ts
sendMaxPsbtDto
sweepPsbtDto
createSendMaxPsbt
createSweepPsbt
broadcastSweep
```

### 5. Remove manual fallback once real DTO flow exists

For CPFP, manual outpoint fallback was useful temporarily but removed after `tx.outputs` worked.

For Send Max / Sweep, avoid long-term manual hacks. The UI should rely on:

```text
availableUtxos
coin control include/exclude
confirmedOnly
selectionMode
```

---

## Existing Backend/API/CLI Capabilities To Use

From the API contract already reviewed, these functions likely exist in `WalletApi`:

```rust
create_send_max_psbt(name, to, fee_rate, replaceable)
create_send_max_psbt_with_coin_control(name, to, fee_rate, replaceable, coin_control)
send_max_psbt(...)
send_max_psbt_with_coin_control(...)
sweep_psbt(...)
sweep(...)
create_sweep_psbt(...)
create_sweep_psbt_with_coin_control(...)
```

Exact names must be verified in:

```text
crates/wallet_api/src/api.rs
crates/wallet_api/src/service/psbt.rs
apps/wallet_desktop/src-tauri/src/commands/send.rs
apps/wallet_desktop/src-tauri/src/commands/send_model.rs
apps/wallet_cli/src/commands/cli.rs
apps/wallet_cli/src/commands/mod.rs
apps/wallet_cli/src/commands/runtime.rs
```

CLI already has command enum variants in `cli.rs`:

```rust
CreateSendMaxPsbt
CreateSendMaxPsbtWithCoinControl
SendMaxPsbt
SendMaxPsbtWithCoinControl
SweepPsbt
Sweep
```

Important CLI fields:

```rust
name: String
to: String
fee_rate: u64
replaceable: bool
include: Vec<String>
exclude: Vec<String>
confirmed_only: bool
selection_mode: Option<WalletInputSelectionModeDto>
```

Sweep has:

```rust
replaceable default true
```

Send Max has:

```rust
replaceable default false
```

---

## Desired UX

Add Send Max / Sweep support to the existing Send page.

### Modes

The Send page should support at least three send modes:

```text
Fixed amount
Send Max
Sweep selected UTXOs
```

Recommended UI:

```text
Send mode:
  ( ) Fixed amount
  ( ) Send max
  ( ) Sweep selected / included UTXOs
```

### Fixed amount mode

Current behavior should remain unchanged.

Fields:

```text
To address
Amount sats
Fee rate sat/vB
RBF toggle
Confirmed only toggle
Coin control mode
Selected UTXOs
```

### Send Max mode

Meaning:

```text
Spend available wallet funds to destination, subtracting fee from sent amount.
```

Fields:

```text
To address
Fee rate sat/vB
RBF toggle
Confirmed only toggle
Coin control include/exclude if supported
Selection mode if supported
```

Amount field should be hidden or disabled.

Button:

```text
Create Send Max PSBT
```

After preview:

```text
Sign
Broadcast
```

### Sweep mode

Meaning:

```text
Spend explicitly selected UTXOs to destination, subtracting fee from total selected input value.
```

Fields:

```text
To address
Fee rate sat/vB
RBF toggle
Confirmed only toggle
UTXO selector required
Selection mode
```

Sweep should require selected UTXOs or explicit include list.

Button:

```text
Create Sweep PSBT
```

After preview:

```text
Sign
Broadcast
```

### Important UX warnings

Show warnings:

```text
Send Max will spend all eligible coins.
Sweep will spend selected UTXOs and send remaining value after fee to the destination.
```

When no UTXOs selected in Sweep mode:

```text
Select at least one UTXO to sweep.
```

When confirmed-only is enabled and no confirmed UTXOs are available:

```text
No confirmed UTXOs available. Disable confirmed-only or mine/sync first.
```

---

## Implementation Plan

### Phase 1 — Inspect Existing Backend + Tauri Commands

Open and verify exact existing command names and request models:

```text
apps/wallet_desktop/src-tauri/src/commands/send.rs
apps/wallet_desktop/src-tauri/src/commands/send_model.rs
crates/wallet_api/src/api.rs
crates/wallet_api/src/model.rs
```

Confirm which Tauri commands already exist.

Expected possible commands:

```rust
create_send_max_psbt
create_send_max_psbt_with_coin_control
send_max_psbt
send_max_psbt_with_coin_control
sweep_psbt
sweep
```

If commands are missing but API exists, add Tauri command wrappers.

Follow existing command style from CPFP:

```rust
#[tauri::command]
pub async fn cpfp_psbt(...)
```

Do not invent names without checking `send.rs`.

---

### Phase 2 — Ensure TypeScript DTOs Are Complete

Check:

```text
apps/wallet_desktop/src/shared/types/dtos.ts
```

Make sure these exist:

```ts
export interface WalletPsbtDto { ... }
export interface WalletSignedPsbtDto { ... }
export interface TxBroadcastResultDto { ... }
export interface WalletCoinControlDto { ... }
export interface WalletUtxoDto { ... }
```

If Send Max / Sweep has a dedicated DTO in Rust, mirror it in TypeScript.

If backend returns regular `WalletPsbtDto`, reuse it.

Avoid creating fake DTOs if Rust does not have them.

---

### Phase 3 — Add Frontend API Wrappers

Find existing Send API wrapper, likely one of:

```text
apps/wallet_desktop/src/features/send/api.ts
apps/wallet_desktop/src/features/transactions/api.ts
apps/wallet_desktop/src/shared/api.ts
```

Add wrappers matching real Tauri commands.

Example shape:

```ts
export type SendMaxPsbtInput = {
  walletName: string;
  address: string;
  feeRateSatPerVb: number;
  replaceable: boolean;
};

export async function createSendMaxPsbt(input: SendMaxPsbtInput): Promise<WalletPsbtDto> {
  return invokeCommand<WalletPsbtDto>("create_send_max_psbt", {
    walletName: input.walletName,
    address: input.address,
    feeRateSatVb: input.feeRateSatPerVb,
    replaceable: input.replaceable,
  });
}
```

For coin control:

```ts
export type SendMaxCoinControlInput = SendMaxPsbtInput & {
  includeOutpoints: string[];
  excludeOutpoints: string[];
  confirmedOnly: boolean;
  selectionMode?: WalletInputSelectionModeDto | null;
};
```

Use exact field casing expected by existing `invokeCommand` conventions.

Lessons from CPFP:

```text
Rust field fee_rate_sat_vb became frontend feeRateSatVb in Tauri wrapper.
```

Use established wrapper style.

---

### Phase 4 — Refactor Send Form State

Find:

```text
SendPage.tsx
```

Add a send mode state:

```ts
type SendMode = "fixed" | "send_max" | "sweep";
const [sendMode, setSendMode] = useState<SendMode>("fixed");
```

Update form state type if needed:

```ts
type SendFormState = {
  to: string;
  amountSat?: string;
  feeRateSatVb: string;
  replaceable: boolean;
  confirmedOnly: boolean;
};
```

Rules:

```text
fixed      → amount required
send_max   → amount ignored
sweep      → amount ignored, selected UTXOs required
```

When switching mode:

```ts
setPsbt(null)
setSignedPsbt(null)
setBroadcastResult(null)
setError(null)
```

This avoids stale state leaks. This was important in CPFP/RBF switching.

---

### Phase 5 — Update Send Page Action Selection

Current Send page probably has handlers:

```ts
handlePreview
handleSign
handlePublish
```

Refactor preview handler:

```ts
const handlePreview = async (form: SendFormState) => {
  switch (sendMode) {
    case "fixed":
      return create fixed PSBT
    case "send_max":
      return create send max PSBT
    case "sweep":
      return create sweep PSBT
  }
}
```

Recommended handler names:

```ts
handleCreateFixedPsbt
handleCreateSendMaxPsbt
handleCreateSweepPsbt
```

But keep one `handlePreview` if easier.

---

### Phase 6 — Coin Control Rules

Reuse existing `CoinControlSelector`.

Rules:

#### Fixed mode

Selection may be:

```text
auto
manual
strict manual
manual with auto completion
```

#### Send Max mode

Allow coin control include/exclude if backend supports it.

If backend only supports simple send max initially, keep coin control disabled for send max and show:

```text
Coin control for Send Max is not available in this build.
```

CLI suggests it exists, so likely wire it.

#### Sweep mode

Sweep should require selected include outpoints.

Validation:

```ts
if (sendMode === "sweep" && selectedUtxos.length === 0) {
  setError("Select at least one UTXO to sweep.");
  return;
}
```

Use selected UTXOs as include list.

For sweep, `exclude` can still be supported but not required.

---

### Phase 7 — Preview Panel Updates

Existing PSBT preview should work if response is `WalletPsbtDto`.

Add display labels for send mode:

```text
Mode: Fixed amount / Send Max / Sweep
```

If DTO has:

```ts
send_amount_mode
selected_utxos
fee
fee_rate
change_amount
```

show them.

If not, show available fields only.

Do not assume DTO fields; check `WalletPsbtDto` in Rust and TypeScript.

---

### Phase 8 — Result UX

After broadcast, show:

```text
Broadcasted txid
Replaceable: yes/no
Mode: Send Max/Sweep
```

For sweep, show:

```text
Swept outpoints:
- txid:vout
```

Optional:

After broadcast:

```ts
await listUtxos(walletName)
```

Refresh UTXOs and clear selected inputs.

This was useful during CPFP.

---

### Phase 9 — CSS

Use existing Send page card styles where possible.

If adding mode selector, use classes like:

```css
.send-mode-selector
.send-mode-card
.send-mode-card--active
.send-warning
```

Keep visual language aligned with transactions CSS:

```text
dark cards
rounded inputs
subtle warning panels
monospace outpoints
```

---

### Phase 10 — Tests

Add unit tests for pure helpers first.

Possible file:

```text
apps/wallet_desktop/src/features/send/sendMode.test.ts
```

Test:

```text
fixed mode requires amount
send max ignores amount
sweep requires selected UTXOs
confirmed-only warning when no confirmed UTXOs
```

If Vitest already added:

```json
"test": "vitest run"
```

Run:

```bash
npm run test
```

Also run:

```bash
npm run build
```

---

## Implementation Order

Do not do everything at once.

Recommended exact order:

1. Verify Tauri commands and request DTO names.
2. Add TypeScript API wrappers only.
3. Add `sendMode` UI selector, no backend calls yet.
4. Wire Send Max PSBT creation.
5. Wire Sweep PSBT creation.
6. Update sign/broadcast flow to reuse existing handlers.
7. Add warnings and polish.
8. Add tests.

Compile/typecheck after each step.

---

## Manual Runtime Test Plan

Use regtest wallet:

```bash
cargo run -p wallet_cli -- sync --name regtest-local
cargo run -p wallet_cli -- utxos --name regtest-local
```

### Send Max UI test

1. Open Send page.
2. Select Send Max.
3. Enter destination address.
4. Set fee rate.
5. Create PSBT.
6. Verify preview.
7. Sign.
8. Broadcast.
9. Sync.
10. UTXOs should change.

### Sweep UI test

1. Open Send page.
2. Select Sweep.
3. Select one or more UTXOs.
4. Enter destination address.
5. Create Sweep PSBT.
6. Verify selected inputs in preview.
7. Sign.
8. Broadcast.
9. Sync.
10. Selected UTXOs should disappear.

### Confirmed-only edge case

1. Create unconfirmed UTXO.
2. Enable confirmed-only.
3. Try Sweep selected unconfirmed UTXO.
4. Expect UI/backend error.

---

## Important Pitfalls To Avoid

### Do not reuse CPFP naming

Avoid:

```ts
cpfpPsbt
```

Use:

```ts
sendMaxPsbtDto
sweepPsbtDto
```

### Do not silently send all funds

Send Max and Sweep are dangerous. Require clear button labels and warning text.

### Do not allow Sweep with no selected UTXOs

This creates confusing backend errors.

### Do not rely on stale UTXO state

After broadcast, refresh UTXOs.

### Do not invent DTO fields

Check Rust `model.rs` first.

### Do not add manual hacks

If data is missing, add DTO field properly.

---

## Done Criteria

Feature is done when:

```text
Send page supports Fixed / Send Max / Sweep modes
Send Max creates PSBT
Sweep creates PSBT
Sign + broadcast works for both
UTXOs refresh after broadcast
Errors are clear
No stale PSBT state when switching modes
TypeScript build passes
Frontend tests pass
Manual regtest flow passes
```

---

## Suggested Commit Message

```text
feat(desktop): add send max and sweep UI flows
```

Detailed:

```text
- add Send mode selector for fixed/send-max/sweep
- wire send-max PSBT creation through Tauri API
- wire sweep PSBT creation with coin control
- reuse sign/broadcast PSBT workflow
- refresh UTXOs after broadcast
- add validation and warnings for sweep/send-max
- add frontend tests for send mode validation
```
