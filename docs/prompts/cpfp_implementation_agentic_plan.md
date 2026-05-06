# CPFP Implementation — Detailed Agentic Plan for rust-descriptor-wallet

This document is a step-by-step implementation plan for adding a complete **CPFP (Child Pays For Parent)** flow to the existing Tauri + Rust + React wallet desktop app.

It is based on the current project state after implementing the Transactions row action menu and the full RBF bump-fee flow:

```text
Transactions row action menu
  -> Bump fee / RBF
  -> create replacement PSBT
  -> sign PSBT
  -> publish PSBT
  -> refresh transactions
```

The goal now is to add a similar end-to-end CPFP workflow:

```text
Transactions row action menu
  -> CPFP
  -> create child PSBT
  -> sign PSBT
  -> publish PSBT
  -> mine / confirm parent + child in regtest
```

---

# Current Known Project Structure

Important frontend files:

```text
apps/wallet_desktop/src/pages/TransactionsPage.tsx

apps/wallet_desktop/src/features/transactions/api.ts
apps/wallet_desktop/src/features/transactions/types.ts
apps/wallet_desktop/src/features/transactions/format.ts

apps/wallet_desktop/src/features/transactions/components/TransactionActionsMenu.tsx
apps/wallet_desktop/src/features/transactions/components/TransactionDetailsModal.tsx
apps/wallet_desktop/src/features/transactions/components/BumpFeePanel.tsx
apps/wallet_desktop/src/features/transactions/components/RbfPsbtWorkflowPanel.tsx

apps/wallet_desktop/src/shared/types/dtos.ts
apps/wallet_desktop/src/styles/transactions.css
apps/wallet_desktop/src/styles/actions.css
```

Important Rust / backend files:

```text
apps/wallet_desktop/src-tauri/src/commands/send.rs
apps/wallet_desktop/src-tauri/src/commands/transactions.rs

crates/wallet_api/src/service/psbt.rs
crates/wallet_api/src/model.rs

crates/wallet_core/src/service/psbt_rbf.rs
crates/wallet_core/src/service/psbt_cpfp.rs
crates/wallet_core/src/model.rs
crates/wallet_core/src/types.rs

tests/regtest_flow.rs
tests/support.rs
crates/test_support/src/wallet.rs
```

---

# Important Lessons From RBF Implementation

## 1. Frontend field naming must match Tauri command DTOs

For fee rate:

```text
UI/component field:    feeRateSatPerVb
Tauri request field:   feeRateSatVb
Rust DTO field:        fee_rate_sat_vb
```

Example from RBF:

```ts
const request = {
  walletName: input.walletName,
  txid: input.txid,
  feeRateSatVb: input.feeRateSatPerVb,
};
```

Do not guess field names. Always inspect Rust DTO request structs and `into_parts()`.

---

## 2. PSBT DTO fee fields can be misleading if generated from minimal PSBT

For RBF, `WalletPsbtInfo::from_psbt_minimal(psbt)` originally returned:

```text
fee_sat=0
fee_rate_sat_per_vb=0
```

because minimal PSBT conversion did not have enough previous-output information to derive fee. We fixed RBF by explicitly populating preview fee fields after building the replacement PSBT.

For CPFP, check whether the CPFP backend already correctly populates:

```text
fee_sat
fee_rate_sat_per_vb
estimated_vsize
input_value_sat
child_output_value_sat
```

If any field shows `0` incorrectly, fix the backend DTO construction, not the UI.

---

## 3. Workflow components should not call backend directly

Keep this separation:

```text
Component
  -> pure UI + callbacks

TransactionsPage
  -> state + orchestration + API calls

transactions/api.ts
  -> Tauri invoke mapping only

Rust API/core
  -> actual wallet logic
```

This worked well for RBF and should be repeated for CPFP.

---

## 4. Integration tests are valuable

RBF now has tests for:

```text
bump_fee_psbt preview only
bump_fee_psbt -> sign_psbt -> publish_psbt -> mine -> confirm
```

CPFP should receive the same level of coverage.

---

# Goal

Add full CPFP support from the transaction action menu.

For an unconfirmed parent transaction, allow the user to create a child transaction that spends one of the wallet-owned unconfirmed outputs from that parent with a higher fee, so miners confirm the parent and child together.

The complete desired flow:

```text
Unconfirmed transaction row
  -> Actions
  -> CPFP
  -> choose/select parent outpoint if needed
  -> enter child fee rate
  -> create CPFP PSBT
  -> preview child PSBT
  -> sign PSBT
  -> publish PSBT
  -> refresh transactions
  -> mine block in regtest
  -> parent and child confirmed
```

---

# Existing Action Visibility Rules

Current action logic:

```ts
const canBumpFee = tx.replaceable === true && tx.confirmed === false;
const canCpfp = tx.confirmed === false;
```

Recommended menu behavior:

| Transaction State | Menu Items |
|---|---|
| Confirmed | Details, Copy txid |
| Unconfirmed non-RBF | Details, Copy txid, CPFP |
| Unconfirmed RBF | Details, Copy txid, Bump fee, CPFP |

For CPFP:

```ts
function canCpfp(tx: WalletTxDto): boolean {
  return tx.confirmed === false;
}
```

Also validate again in `TransactionsPage`, not only in disabled menu buttons.

---

# CPFP Conceptual Model

CPFP is different from RBF.

## RBF

```text
replacement transaction spends same inputs as parent
old tx replaced in mempool
```

## CPFP

```text
child transaction spends an output of the unconfirmed parent
parent stays in mempool
child enters mempool
miner confirms both because child pays enough fee
```

Expected after publishing child:

```text
parent in mempool = true
child in mempool = true
```

Expected after mining:

```text
parent confirmed = true
child confirmed = true
parent in mempool = false
child in mempool = false
```

---

# Existing Backend CPFP Clues

Existing integration tests indicate backend has at least partial CPFP support:

```rust
api.cpfp_psbt(wallet_name, &parent.txid, &selected.outpoint, 5).await?;
api.sign_psbt(wallet_name, &cpfp.psbt_base64).await?;
api.publish_psbt(wallet_name, &signed.psbt_base64).await?;
```

Existing CPFP DTO fields used by the current integration tests are:

```text
psbt_base64
txid
parent_txid
selected_outpoint
input_value_sat
child_output_value_sat
fee_sat
fee_rate_sat_per_vb
estimated_vsize
replaceable
```

Use this frontend DTO name:

```ts
WalletCpfpPsbtDto
```

Mirror the Rust API DTO exactly in:

```text
apps/wallet_desktop/src/shared/types/dtos.ts
```

---

# STEP 1 — Inspect Current CPFP Backend/API State

## Prompt

```text
Inspect the current CPFP implementation.

Focus on:

- crates/wallet_core/src/service/psbt_cpfp.rs
- crates/wallet_core/src/model.rs
- crates/wallet_core/src/types.rs
- crates/wallet_api/src/service/psbt.rs
- crates/wallet_api/src/model.rs
- apps/wallet_desktop/src-tauri/src/commands/send.rs
- apps/wallet_desktop/src/shared/types/dtos.ts
- tests/regtest_flow.rs

Return:

1. Existing Rust CPFP core method name and signature.
2. Existing wallet_api CPFP method name and signature.
3. Existing Tauri CPFP command name and request DTO field names.
4. Existing frontend DTO shape for CPFP, if present.
5. Whether CPFP PSBT preview fields are accurate or show zero incorrectly.
6. Existing tests that already cover CPFP.
7. Exact files that need frontend changes.
```

Do not modify code yet.

---

# STEP 2 — Verify Tauri CPFP Request Shape

## Prompt

```text
Inspect the Tauri command and request DTO for CPFP.

Expected command name:

cpfp_psbt

Expected frontend API function name:

cpfpPsbt

Expected frontend input type:

CpfpPsbtInput

Expected logical inputs:

- walletName
- txid
- outpoint
- feeRateSatPerVb

Expected Tauri request mapping:

const request = {
  walletName: input.walletName,
  txid: input.txid,
  outpoint: input.outpoint,
  feeRateSatVb: input.feeRateSatPerVb,
};

The important naming rule is:

UI/component field:    feeRateSatPerVb
Tauri request field:   feeRateSatVb
Rust DTO field:        fee_rate_sat_vb

Do not send feeRateSatPerVb directly to Tauri.
```

---

# STEP 3 — Add/Verify Frontend CPFP DTO Types

## Prompt

```text
Update apps/wallet_desktop/src/shared/types/dtos.ts if needed.

Add or verify this exact DTO:

export interface WalletCpfpPsbtDto {
  psbt_base64: string;
  txid: string;
  parent_txid: string;
  selected_outpoint: string;
  input_value_sat: number;
  child_output_value_sat: number;
  fee_sat: number;
  fee_rate_sat_per_vb: number;
  estimated_vsize: number;
  replaceable: boolean;
}

This DTO is used by the transaction CPFP workflow panel and should mirror the Rust API DTO.

Do not rename fields to camelCase here. DTO fields mirror Rust serde output and use snake_case.
```

---

# STEP 4 — Add CPFP Feature Types

## Prompt

```text
Update apps/wallet_desktop/src/features/transactions/types.ts.

Import:

import type {
  TxBroadcastResultDto,
  WalletCpfpPsbtDto,
  WalletSignedPsbtDto,
  WalletTxDto,
} from "../../shared/types/dtos";

Add:

export type CpfpPsbtInput = {
  walletName: string;
  txid: string;
  outpoint: string;
  feeRateSatPerVb: number;
};

export type CpfpCandidateOutpoint = {
  outpoint: string;
  value: number;
  keychain: string;
};

export type CpfpPanelProps = {
  tx: WalletTxDto;
  walletName: string;
  availableOutpoints: CpfpCandidateOutpoint[];
  loading?: boolean;
  onCancel: () => void;
  onCreatePsbt: (input: CpfpPsbtInput) => Promise<void> | void;
};

export type CpfpPsbtWorkflowPanelProps = {
  cpfp: WalletCpfpPsbtDto;
  signedPsbt: WalletSignedPsbtDto | null;
  broadcastResult: TxBroadcastResultDto | null;
  loading?: boolean;
  onSign: () => Promise<void> | void;
  onBroadcast: () => Promise<void> | void;
  onClose: () => void;
};
```

Use `CpfpCandidateOutpoint[]`, not plain `string[]`, so the UI can display value and keychain and prefer internal change outputs.
---

# STEP 5 — Add CPFP API Function

## Prompt

```text
Update apps/wallet_desktop/src/features/transactions/api.ts.

Imports:

import type { WalletCpfpPsbtDto } from "../../shared/types/dtos";
import type { CpfpPsbtInput } from "./types";

Add:

export async function cpfpPsbt(input: CpfpPsbtInput): Promise<WalletCpfpPsbtDto> {
  const request = {
    walletName: input.walletName,
    txid: input.txid,
    outpoint: input.outpoint,
    feeRateSatVb: input.feeRateSatPerVb,
  };

  console.debug("[transactions/api] cpfp_psbt request", request);

  return invokeCommand<WalletCpfpPsbtDto>("cpfp_psbt", {
    request,
  });
}

Reuse existing:

- signPsbt
- publishPsbt

Do not duplicate sign/publish invoke code.
```

---

# STEP 6 — Explicit CPFP Outpoint Selection

CPFP requires spending a wallet-owned unconfirmed output of the parent transaction.

Use explicit selected outpoint in the first implementation.

Backend and existing tests already use this flow:

```rust
api.cpfp_psbt(wallet_name, &parent.txid, &selected.outpoint, 5).await?;
```

Frontend must therefore pass:

```ts
CpfpPsbtInput {
  walletName,
  txid,
  outpoint,
  feeRateSatPerVb,
}
```

The UI must discover candidate parent outputs from wallet UTXOs.

Do not implement a txid-only CPFP API unless the Rust backend already supports optional outpoint auto-selection.

---

# STEP 7 — Add Parent Output Lookup

## Prompt

```text
Find existing frontend API function that lists wallet UTXOs.

Search for:

- listUtxos
- getUtxos
- walletUtxos
- useUtxos
- features/send/api.ts
- features/wallet/api.ts

Goal:

When user clicks CPFP on tx:
1. Fetch wallet UTXOs.
2. Filter UTXOs where outpoint txid equals tx.txid.
3. Use those as CPFP candidate parent outputs.

Helper:

function outpointTxid(outpoint: string): string {
  return outpoint.split(":")[0] ?? "";
}

Build candidates as:

const candidates: CpfpCandidateOutpoint[] = utxos
  .filter((u) => outpointTxid(u.outpoint) === tx.txid)
  .map((u) => ({
    outpoint: u.outpoint,
    value: u.value,
    keychain: u.keychain,
  }));

Default selection:

const defaultOutpoint =
  candidates.find((u) => u.keychain === "internal")?.outpoint ??
  candidates[0]?.outpoint;

Rules:

- 0 candidates → show action error:
  "No wallet-owned unconfirmed output is available for CPFP"
- 1 candidate → auto-select it
- 2+ candidates → show dropdown and default to internal/change if present
```

Do not call backend CPFP with fake outpoint.

---

# STEP 8 — Create CpfpPanel Component

Create:

```text
apps/wallet_desktop/src/features/transactions/components/CpfpPanel.tsx
```

## Prompt

```text
Implement CpfpPanel.

Props:

CpfpPanelProps

Requirements:

- Shows parent txid using shortTxid().
- Shows available parent outputs in a select/dropdown.
- Uses `CpfpCandidateOutpoint[]`, not string[].
- If one outpoint, select it automatically.
- If multiple outpoints, default to `keychain === "internal"` if present.
- Shows selected outpoint.
- Dropdown label should include:
  - keychain
  - formatted value
  - full or shortened outpoint
- Fee rate input:
  - default 5 sat/vB if no better suggestion.
  - validate fee rate > 0.
- Submit calls onCreatePsbt({
    walletName,
    txid: tx.txid,
    outpoint: selectedOutpoint,
    feeRateSatPerVb
  })
- Cancel button calls onCancel.
- Does not call backend directly.
- Uses existing .primary-button / .secondary-button.
- Uses format helpers from ../format where useful.
```

Recommended UI:

```text
CPFP
Parent: abc123...def456
Parent output: [dropdown]
Child fee rate: [5] sat/vB
[Create CPFP PSBT] [Cancel]
```

---

# STEP 9 — Add CPFP Panel CSS

Use `actions.css` because CPFP panel is action/workflow UI.

## Prompt

```text
Update apps/wallet_desktop/src/styles/actions.css.

Add CPFP classes matching bump-fee style:

.cpfp
.cpfp__header
.cpfp__title
.cpfp__txid
.cpfp__body
.cpfp__row
.cpfp__label
.cpfp__value
.cpfp__field
.cpfp__input
.cpfp__select
.cpfp__hint
.cpfp__error
.cpfp__actions

Reuse style patterns from .bump-fee where possible.

Do not duplicate global button styles.
```

---

# STEP 10 — Create CpfpPsbtWorkflowPanel Component

Create:

```text
apps/wallet_desktop/src/features/transactions/components/CpfpPsbtWorkflowPanel.tsx
```

## Prompt

```text
Implement CpfpPsbtWorkflowPanel.

It should be equivalent in role to RbfPsbtWorkflowPanel but CPFP-specific.

Props:

CpfpPsbtWorkflowPanelProps

Display:

- Child txid
- Parent txid
- Selected parent outpoint
- Input value
- Child output value
- Fee
- Fee rate
- Estimated vsize
- Replaceable
- PSBT base64 textarea
- Signed status
- Broadcast result

Actions:

- Sign PSBT
- Broadcast
- Close

Rules:

- Sign disabled after signed or after broadcast.
- Broadcast disabled until signed.
- No backend calls in component.
- Use shortTxid, formatSats, formatFeeRate.
```

---

# STEP 11 — Wire CPFP State Into TransactionsPage

## Prompt

```text
Update TransactionsPage.tsx.

Add imports:

- cpfpPsbt from transactions/api
- CpfpPanel
- CpfpPsbtWorkflowPanel
- WalletCpfpPsbtDto
- WalletSignedPsbtDto
- TxBroadcastResultDto if not already imported
- CpfpPsbtInput from transactions/types
- UTXO DTO/API helper if needed

Add state:

const [cpfpTx, setCpfpTx] = useState<WalletTxDto | null>(null);
const [cpfpOutpoints, setCpfpOutpoints] = useState<CpfpCandidateOutpoint[]>([]);
const [cpfpLoading, setCpfpLoading] = useState(false);
const [cpfpPsbt, setCpfpPsbt] = useState<WalletCpfpPsbtDto | null>(null);
const [cpfpSignedPsbt, setCpfpSignedPsbt] = useState<WalletSignedPsbtDto | null>(null);
const [cpfpBroadcastResult, setCpfpBroadcastResult] = useState<TxBroadcastResultDto | null>(null);
const [cpfpActionLoading, setCpfpActionLoading] = useState(false);

On wallet switch:
- clear CPFP state.

When starting RBF:
- optionally clear CPFP state.

When starting CPFP:
- clear RBF state to avoid two workflows open.
```

---

# STEP 12 — Implement handleCpfp

Current placeholder:

```ts
const handleCpfp = (tx: WalletTxDto) => {
  showActionMessage("CPFP is not implemented yet");
};
```

Replace with real flow.

## Prompt

```text
Implement handleCpfp(tx).

Rules:

1. Validate tx.confirmed === false.
2. Validate selectedWalletName exists.
3. Fetch wallet UTXOs using existing UTXO API.
4. Convert matching UTXOs into `CpfpCandidateOutpoint[]`:
   - outpoint
   - value
   - keychain
5. Prefer `keychain === "internal"` as default selection inside `CpfpPanel`.
6. If zero candidates:
   - setActionError("No wallet-owned unconfirmed output is available for CPFP")
   - return.
7. Set cpfpTx(tx).
8. Set cpfpOutpoints(candidates).
9. Clear cpfpPsbt/signed/broadcast result.
10. Clear RBF state.
```

Helper:

```ts
function outpointTxid(outpoint: string): string {
  return outpoint.split(":")[0] ?? "";
}
```

---

# STEP 13 — Implement Create CPFP PSBT Handler

## Prompt

```text
Add handler:

const handleCreateCpfpPsbt = async (input: CpfpPsbtInput) => {
  try {
    setCpfpLoading(true);
    setActionError(null);
    setError(null);

    const result = await cpfpPsbt(input);

    setCpfpPsbt(result);
    setCpfpSignedPsbt(null);
    setCpfpBroadcastResult(null);

    showActionMessage("CPFP PSBT created. Review, sign, and broadcast it next.");
  } catch (e) {
    setActionError(e instanceof Error ? e.message : String(e));
  } finally {
    setCpfpLoading(false);
  }
};
```

Do not refresh transactions after PSBT creation because nothing has been broadcast yet.

---

# STEP 14 — Implement CPFP Sign Handler

## Prompt

```text
Add handler:

const handleSignCpfpPsbt = async () => {
  if (!selectedWalletName || !cpfpPsbt) {
    setActionError("No CPFP PSBT is available to sign");
    return;
  }

  try {
    setCpfpActionLoading(true);
    setActionError(null);

    const signed = await signPsbt({
      walletName: selectedWalletName,
      psbtBase64: cpfpPsbt.psbt_base64,
    });

    setCpfpSignedPsbt(signed);
    setCpfpBroadcastResult(null);
    showActionMessage("CPFP PSBT signed. Broadcast it next.");
  } catch (e) {
    setActionError(e instanceof Error ? e.message : String(e));
  } finally {
    setCpfpActionLoading(false);
  }
};
```

---

# STEP 15 — Implement CPFP Broadcast Handler

## Prompt

```text
Add handler:

const handleBroadcastCpfpPsbt = async () => {
  if (!selectedWalletName || !cpfpSignedPsbt) {
    setActionError("No signed CPFP PSBT is available to broadcast");
    return;
  }

  try {
    setCpfpActionLoading(true);
    setActionError(null);

    const result = await publishPsbt({
      walletName: selectedWalletName,
      psbtBase64: cpfpSignedPsbt.psbt_base64,
    });

    setCpfpBroadcastResult(result);
    showActionMessage("CPFP child transaction broadcast");

    await refreshTransactions();
  } catch (e) {
    setActionError(e instanceof Error ? e.message : String(e));
  } finally {
    setCpfpActionLoading(false);
  }
};
```

After broadcast:

```text
parent should remain in mempool until mined
child should appear in mempool
```

---

# STEP 16 — Implement Cancel CPFP Handler

## Prompt

```text
Add:

const handleCancelCpfp = () => {
  setCpfpTx(null);
  setCpfpOutpoints([]);
  setCpfpPsbt(null);
  setCpfpSignedPsbt(null);
  setCpfpBroadcastResult(null);
  setCpfpActionLoading(false);
  setActionError(null);
};
```

---

# STEP 17 — Render CPFP UI In TransactionsPage

## Prompt

```text
Render CPFP workflow near the RBF workflow.

Recommended:

{cpfpTx && selectedWalletName && (
  <div className="transactions-workflow-panel">
    <h2>CPFP</h2>
    <CpfpPanel
      tx={cpfpTx}
      walletName={selectedWalletName}
      availableOutpoints={cpfpOutpoints}
      loading={cpfpLoading}
      onCancel={handleCancelCpfp}
      onCreatePsbt={handleCreateCpfpPsbt}
    />

    {cpfpPsbt && (
      <CpfpPsbtWorkflowPanel
        cpfp={cpfpPsbt}
        signedPsbt={cpfpSignedPsbt}
        broadcastResult={cpfpBroadcastResult}
        loading={cpfpActionLoading}
        onSign={handleSignCpfpPsbt}
        onBroadcast={handleBroadcastCpfpPsbt}
        onClose={handleCancelCpfp}
      />
    )}
  </div>
)}
```

Ensure RBF and CPFP do not both remain open at the same time.

---

# STEP 18 — Improve TransactionActionsMenu CPFP Behavior

## Prompt

```text
Verify TransactionActionsMenu.

Rules:

- CPFP disabled unless tx.confirmed === false.
- CPFP click calls onCpfp(tx).
- Component does not call API directly.
- Component closes menu after valid action.
- Use aria-disabled for disabled item.
```

Do not add CPFP state inside menu.

---

# STEP 19 — Verify Backend CPFP Preview Fields

Run the CPFP flow and check logs/UI.

Expected CPFP PSBT preview should show:

```text
fee_sat > 0
fee_rate_sat_per_vb == requested
estimated_vsize > 0
input_value_sat > child_output_value_sat
input_value_sat - child_output_value_sat == fee_sat
parent_txid == selected parent tx
selected_outpoint == chosen outpoint
```

If fee fields are zero, inspect:

```text
crates/wallet_core/src/service/psbt_cpfp.rs
```

and fix DTO construction similarly to the RBF fix, but CPFP probably can compute exact fee:

```text
fee = input_value_sat - child_output_value_sat
```

Do not fake the value in frontend.

---

# STEP 20 — Remove Temporary Debug Logs If Desired

RBF currently has useful debug logs. For CPFP, add logs during implementation, then decide whether to keep or remove.

Useful temporary logs:

```rust
tracing::info!(
  parent_txid = %parent_txid,
  selected_outpoint = %selected_outpoint,
  requested_fee_rate_sat_per_vb = fee_rate.as_u64(),
  "api psbt: cpfp_psbt request received"
);
```

```rust
tracing::info!(
  parent_txid = %parent_txid,
  child_txid = %info.txid,
  fee_sat = info.fee_sat.as_u64(),
  fee_rate_sat_per_vb = info.fee_rate_sat_per_vb.as_u64(),
  "wallet_core: cpfp_psbt built"
);
```

Keep logs if they are useful and not too noisy.

---

# STEP 21 — TypeScript Build Validation

## Prompt

```text
Run frontend build/typecheck.

Likely commands:

npm run build
npm run typecheck

Fix:

- missing imports
- wrong DTO field names
- wrong API request field names
- unused types
- duplicate local props
- stale placeholder CPFP code
```

Pay special attention to:

```text
WalletCpfpPsbtDto exact snake_case field names
CpfpPsbtInput field names
cpfpPsbt request mapping: walletName, txid, outpoint, feeRateSatVb
```

---

# STEP 22 — Rust Build Validation

Run:

```bash
cargo check
```

Then targeted tests:

```bash
cargo test -p wallet_core cpfp
cargo test -p wallet_api cpfp
cargo test --test regtest_flow wallet_cpfp_psbt_builds_for_unconfirmed_parent
cargo test --test regtest_flow wallet_cpfp_child_broadcasts_and_confirms
```

---

# STEP 23 — Add/Update Integration Tests

Existing tests already include:

```rust
wallet_cpfp_psbt_builds_for_unconfirmed_parent
wallet_cpfp_psbt_uses_requested_parent_outpoint
wallet_cpfp_child_broadcasts_and_confirms
wallet_cpfp_psbt_fails_for_confirmed_parent
wallet_cpfp_psbt_fails_when_parent_not_found
```

Add a UI-matching full workflow test if missing:

```rust
wallet_cpfp_psbt_signs_publishes_and_confirms_parent_and_child
```

Test flow:

```text
1. Start regtest.
2. Ensure wallet has funds.
3. Create low-fee unconfirmed parent.
4. Sync.
5. Pick wallet-owned parent output.
6. Create CPFP PSBT.
7. Assert PSBT preview fields.
8. Assert parent in mempool, child not in mempool before publish.
9. Sign PSBT.
10. Publish PSBT.
11. Assert parent in mempool and child in mempool.
12. Mine 1 block.
13. Sync.
14. Assert parent and child confirmed.
15. Assert both removed from mempool.
```

This test may already be partially covered by:

```rust
wallet_cpfp_child_broadcasts_and_confirms
```

If so, update that test instead of duplicating.

---

# STEP 24 — Manual Test Plan

Use regtest wallet.

## Test 1 — Confirmed tx

```text
- Open confirmed tx menu.
- CPFP disabled or hidden.
- Details and Copy work.
```

## Test 2 — Unconfirmed self-send

```text
- Create low-fee unconfirmed tx.
- Open tx menu.
- CPFP enabled.
- Click CPFP.
- CPFP panel opens.
- Parent outpoint shown/preselected.
- Create CPFP PSBT works.
```

## Test 3 — Sign and broadcast CPFP

```text
- Click Sign PSBT.
- Signed message appears.
- Click Broadcast.
- Broadcast result txid appears.
- Transactions refresh.
```

## Test 4 — Confirm with mine script

```bash
./infra/regtest/scripts/mine.sh 1
cargo run -p wallet_cli -- sync --name regtest-local
cargo run -p wallet_cli -- txs --name regtest-local
```

Expected:

```text
parent confirmed=true
child confirmed=true
```

## Test 5 — No parent output

```text
- Try CPFP for an unconfirmed tx that has no wallet-owned spendable output.
- UI shows clear action error.
- No backend call made with fake outpoint.
```

## Test 6 — Multiple parent outputs

```text
- Parent has external output and internal change output.
- CPFP panel offers dropdown.
- Selected outpoint is passed to backend.
- Result selected_outpoint matches selection.
```

---

# STEP 25 — Acceptance Criteria

Feature is complete when:

```text
- CPFP action is available only for unconfirmed transactions.
- Clicking CPFP opens a real CPFP panel.
- UI finds wallet-owned parent outputs.
- User can select a parent output.
- User can enter child fee rate.
- cpfp_psbt creates a child PSBT.
- CPFP PSBT preview shows:
  - parent txid
  - child txid
  - selected outpoint
  - input value
  - child output value
  - fee
  - fee rate
  - estimated vsize
- Sign PSBT works.
- Broadcast works.
- Transactions refresh after broadcast.
- Mining confirms parent and child in regtest.
- Existing RBF flow still works.
- TypeScript build passes.
- cargo check passes.
- CPFP integration tests pass.
```

---

# Future Improvements After CPFP

1. Decode PSBT human-readable preview instead of raw base64 only.
2. Add “Open in explorer” for Signet/testnet/mainnet.
3. Add fee comparison:
   ```text
   parent fee rate
   child fee rate
   package effective fee rate
   ```
4. Add “Mine 1 block” dev-only button for regtest.
5. Add mempool package status.
6. Add transaction graph:
   ```text
   parent -> child
   original -> replacement
   ```
7. Add labels/notes for CPFP child transactions.
8. Add warning if CPFP child fee may still be too low.

---

# Recommended Implementation Order

Do this in order:

```text
1. Inspect backend CPFP request/DTO shape.
2. Add/update frontend DTO and types.
3. Add cpfpPsbt API function.
4. Implement parent outpoint lookup from UTXOs.
5. Create CpfpPanel.
6. Create CpfpPsbtWorkflowPanel.
7. Wire state and handlers in TransactionsPage.
8. Test CPFP PSBT creation.
9. Wire sign/publish.
10. Test full UI flow.
11. Run Rust integration tests.
12. Add/update full CPFP workflow integration test if missing.
13. Clean debug logs and CSS.
```

---

End of document.
