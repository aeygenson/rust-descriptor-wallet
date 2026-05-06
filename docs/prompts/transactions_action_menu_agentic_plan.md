# Transactions Row Action Menu — Detailed Agentic Implementation Prompts

This document is a step-by-step agentic coding plan for implementing and wiring the **Transactions table row action menu** and the operations behind it in the Tauri + Rust + React wallet desktop app.

The current direction is:

- Keep actions contextual to each transaction row.
- Use a dropdown menu per transaction row.
- Implement actions incrementally.
- Reuse the existing PSBT lifecycle where possible:
  - create/preview PSBT
  - sign PSBT
  - publish/broadcast PSBT

---

# Goal

Add a scalable per-transaction action system to the Transactions page.

Each transaction row should expose an **Actions** dropdown with contextual operations:

1. Details
2. Copy txid
3. Bump fee / RBF
4. CPFP
5. Later: open in explorer, copy raw data, export, labels/notes, etc.

Initial focus:

- Make action menu reliable.
- Wire **RBF bump fee** end-to-end.
- Prepare architecture for **CPFP** next.
- Avoid cluttering the transaction table with many buttons.

---

# UX Principle

The transaction row menu should be a launcher only.

It should **not** contain forms or business logic.

Recommended split:

```text
TransactionActionsMenu
  -> selects action only

TransactionsPage
  -> owns state and orchestrates operations

BumpFeePanel / BumpFeeModal
  -> collects fee rate
  -> calls bumpFeePsbt
  -> returns WalletPsbtDto

Shared PSBT components
  -> preview
  -> sign
  -> publish
```

---

# Expected Action Visibility Rules

For each transaction row:

```ts
const canBumpFee = tx.replaceable === true && tx.confirmed === false;
const canCpfp = tx.confirmed === false;
```

Recommended visible/disabled behavior:

| Transaction State | Menu Items |
|---|---|
| Confirmed | Details, Copy txid |
| Unconfirmed non-RBF | Details, Copy txid, CPFP |
| Unconfirmed RBF | Details, Copy txid, Bump fee, CPFP |

For now, CPFP can be present but disabled or stubbed if backend/UI is not ready.

---

# STEP 1 — Inspect Current Transaction Architecture

## Prompt

```text
Inspect the current Transactions page architecture.

Focus on these areas:

- apps/wallet_desktop/src/pages/TransactionsPage.tsx
- apps/wallet_desktop/src/features/transactions/api.ts
- apps/wallet_desktop/src/features/transactions/types.ts
- apps/wallet_desktop/src/features/transactions/components/TransactionActionsMenu.tsx
- apps/wallet_desktop/src/features/transactions/components/BumpFeePanel.tsx
- apps/wallet_desktop/src/shared/types/dtos.ts
- apps/wallet_desktop/src/styles/transactions.css
- apps/wallet_desktop/src/styles/actions.css
- existing Send page PSBT preview/sign/publish components and API functions

Do not modify code yet.

Return:

1. Existing transaction DTO shape.
2. Current action menu implementation status.
3. Existing bumpFeePsbt frontend API status.
4. Existing sign/publish API function locations.
5. Whether PSBT preview component can be reused outside SendPage.
6. Exact files that need modification.
```

---

# STEP 2 — Confirm Shared DTOs and Inputs

## Prompt

```text
Validate the frontend DTOs used by transaction actions.

Check WalletTxDto fields, especially:

- txid
- confirmed
- replaceable
- fee
- fee_rate_sat_per_vb
- net_value
- confirmation_height
- direction

Check WalletPsbtDto fields needed for preview/sign/publish.

If missing, do not invent fields. Instead, report what is missing and suggest minimal backend DTO additions.

Do not change code unless necessary.
```

---

# STEP 3 — Finalize TransactionActionsMenu Props

## Prompt

```text
Ensure the transaction action menu props are defined in:

apps/wallet_desktop/src/features/transactions/types.ts

Expected type:

export type TransactionActionsMenuProps = {
  tx: WalletTxDto;
  onDetails: (tx: WalletTxDto) => void;
  onCopyTxid: (txid: string) => void;
  onBumpFee: (tx: WalletTxDto) => void;
  onCpfp: (tx: WalletTxDto) => void;
};

Rules:

- Import WalletTxDto from the shared DTO source.
- Do not self-import from ./types.
- Keep action menu type feature-local.
```

---

# STEP 4 — Implement TransactionActionsMenu

## Prompt

```text
Implement or verify:

apps/wallet_desktop/src/features/transactions/components/TransactionActionsMenu.tsx

Requirements:

- Use local `open` state.
- Close on outside click.
- Use accessible menu attributes:
  - aria-haspopup="menu"
  - aria-expanded
  - role="menu"
  - role="menuitem"
- Menu items:
  - Details
  - Copy txid
  - Bump fee
  - CPFP
- Bump fee disabled unless tx.replaceable && !tx.confirmed.
- CPFP disabled unless !tx.confirmed, or stubbed if not implemented yet.
- Always close menu after a valid action.
- Do not call API directly here.
- Do not contain modal/panel state here.
```

---

# STEP 5 — Add / Verify Dropdown CSS

## Prompt

```text
Verify dropdown CSS exists in:

apps/wallet_desktop/src/styles/actions.css

The file should contain only:

- .tx-actions*
- .bump-fee*
- responsive bump fee styles

Do not duplicate:

- .secondary-button
- transactions table styles
- global button styles

Expected classes:

.tx-actions
.tx-actions__button
.tx-actions__menu
.tx-actions__menu button
.tx-actions__menu button:disabled

Ensure z-index is high enough to appear above table rows.
```

---

# STEP 6 — Add Actions Column to Transactions Table

## Prompt

```text
Wire TransactionActionsMenu into TransactionsPage.tsx.

Add an Actions column to the transactions table.

For every row render:

<TransactionActionsMenu
  tx={tx}
  onDetails={handleDetails}
  onCopyTxid={handleCopyTxid}
  onBumpFee={handleBumpFee}
  onCpfp={handleCpfp}
/>

Add handlers in TransactionsPage:

const handleDetails = (tx: WalletTxDto) => { ... };
const handleCopyTxid = async (txid: string) => { ... };
const handleBumpFee = (tx: WalletTxDto) => { ... };
const handleCpfp = (tx: WalletTxDto) => { ... };

For now:

- Details can set selected transaction state.
- Copy txid should use navigator.clipboard.writeText.
- Bump fee should open BumpFeePanel.
- CPFP can set a placeholder error/message if not implemented.

Do not wire backend CPFP yet unless the API already exists.
```

---

# STEP 7 — Add Transaction Details State

## Prompt

```text
Add basic transaction details support in TransactionsPage.

State:

const [detailsTx, setDetailsTx] = useState<WalletTxDto | null>(null);

Behavior:

- handleDetails(tx) sets detailsTx.
- Render a lightweight details panel/card above or below the table.
- Include:
  - txid
  - confirmed
  - confirmation height
  - direction
  - replaceable
  - net value
  - fee
  - fee rate

Keep it simple for now. This can become a modal later.
```

---

# STEP 8 — Add Copy Txid UX

## Prompt

```text
Implement copy txid UX in TransactionsPage.

Requirements:

- Use navigator.clipboard.writeText(txid).
- Store copied txid or status message.
- Show a small success message like “Copied txid”.
- Clear it after 2 seconds.
- Handle clipboard failure gracefully.

Suggested state:

const [actionMessage, setActionMessage] = useState<string | null>(null);

Suggested helper:

const showActionMessage = (message: string) => {
  setActionMessage(message);
  window.setTimeout(() => setActionMessage(null), 2000);
};

Avoid blocking other actions.
```

---

# STEP 9 — Verify RBF Frontend API

## Prompt

```text
Verify or add RBF API function in:

apps/wallet_desktop/src/features/transactions/api.ts

Expected:

export type BumpFeePsbtInput = {
  walletName: string;
  txid: string;
  feeRateSatPerVb: number;
};

export async function bumpFeePsbt(input: BumpFeePsbtInput): Promise<WalletPsbtDto> {
  const request = {
    walletName: input.walletName,
    txid: input.txid,
    feeRateSatVb: input.feeRateSatPerVb,
  };

  console.debug("[transactions/api] bump_fee_psbt request", request);

  return invokeCommand<WalletPsbtDto>("bump_fee_psbt", { request });
}

Important:

Check the Rust request field name.

The frontend may use `feeRateSatPerVb`, but the Tauri/Rust DTO might expect one of:

- feeRateSatPerVb
- feeRateSatVb
- fee_rate_sat_per_vb

Do not guess. Inspect existing working send API mapping and Rust DTO.

Use the exact field expected by the Tauri command.
```

---

# STEP 10 — Implement BumpFeePanel Props

## Prompt

```text
Verify BumpFeePanelProps in transactions/types.ts:

export type BumpFeePanelProps = {
  tx: WalletTxDto;
  walletName: string;
  loading?: boolean;
  onCancel: () => void;
  onCreatePsbt: (input: {
    walletName: string;
    txid: string;
    feeRateSatPerVb: number;
  }) => Promise<void> | void;
};

Use Promise<void> | void if TransactionsPage will await it.
```

---

# STEP 11 — Implement BumpFeePanel

## Prompt

```text
Implement or verify BumpFeePanel.

Requirements:

- Shows short txid.
- Shows current fee rate from tx.fee_rate_sat_per_vb.
- Suggests next fee rate:
  - if current missing/invalid, default 2 sat/vB
  - else max(ceil(current * 1.5), current + 2)
- Allows user to edit fee rate.
- Validates fee rate > current fee rate if current exists.
- Calls onCreatePsbt with:
  - walletName
  - txid
  - feeRateSatPerVb
- Has cancel button.
- Uses existing .primary-button / .secondary-button classes.
- Does not sign or broadcast directly.
```

Validation rule:

```ts
const isHigherThanCurrent =
  currentFeeRate === null || parsedFeeRate > currentFeeRate;
```

Disable submit unless valid.

---

# STEP 12 — Wire BumpFeePanel in TransactionsPage

## Prompt

```text
Wire BumpFeePanel into TransactionsPage.

Add state:

const [bumpFeeTx, setBumpFeeTx] = useState<WalletTxDto | null>(null);
const [bumpFeeLoading, setBumpFeeLoading] = useState(false);
const [rbfPsbt, setRbfPsbt] = useState<WalletPsbtDto | null>(null);

Handlers:

const handleBumpFee = (tx: WalletTxDto) => {
  setBumpFeeTx(tx);
  setRbfPsbt(null);
};

const handleCreateBumpFeePsbt = async (input: BumpFeePsbtInput) => {
  try {
    setBumpFeeLoading(true);
    setError(null);
    const result = await bumpFeePsbt(input);
    setRbfPsbt(result);
  } catch (e) {
    setError(e instanceof Error ? e.message : String(e));
  } finally {
    setBumpFeeLoading(false);
  }
};

Render BumpFeePanel when bumpFeeTx is set.
```

---

# STEP 13 — Reuse PSBT Preview for RBF

## Prompt

```text
Inspect existing PSBT preview/sign/publish components used by SendPage.

Goal:

Reuse PSBT preview lifecycle for RBF instead of duplicating UI.

Options:

1. Extract shared component:
   features/psbt/components/PsbtWorkflowPanel.tsx

2. Or create transaction-local RbfPsbtPanel first, then refactor later.

Recommended long-term:

features/psbt/
  api.ts
  components/PsbtPreviewPanel.tsx
  components/PsbtWorkflowPanel.tsx
  types.ts

For now, choose the smallest safe refactor.

Requirements:
- Show WalletPsbtDto preview.
- Sign button calls existing signPsbt.
- Broadcast button calls existing publishPsbt.
- Disable sign after signed.
- Disable broadcast until finalized.
- Show txid after publish.
```

---

# STEP 14 — Add PSBT API Reuse

## Prompt

```text
Find existing frontend functions for:

- signPsbt
- publishPsbt / broadcastPsbt

They may live under send/api.ts or wallet/api.ts.

Refactor only if needed.

Goal:

TransactionsPage should be able to call:

const signed = await signPsbt({ walletName, psbt: rbfPsbt.psbt });
const published = await publishPsbt({ walletName, psbt: signed.psbt });

Use actual existing DTO fields and function names.

Do not duplicate Tauri invoke code if a working helper already exists.
```

---

# STEP 15 — Implement RBF PSBT Workflow State

## Prompt

```text
Add RBF PSBT workflow state to TransactionsPage.

Suggested state:

const [rbfPsbt, setRbfPsbt] = useState<WalletPsbtDto | null>(null);
const [rbfSignedPsbt, setRbfSignedPsbt] = useState<WalletSignedPsbtDto | null>(null);
const [rbfBroadcastResult, setRbfBroadcastResult] = useState<TxBroadcastResultDto | null>(null);
const [rbfActionLoading, setRbfActionLoading] = useState(false);

When bumpFeeTx changes:
- clear previous RBF PSBT state.

Handlers:
- handleSignRbfPsbt
- handleBroadcastRbfPsbt
- handleCancelRbfFlow

Use existing SendPage patterns.
```

---

# STEP 16 — Render RBF PSBT Workflow

## Prompt

```text
Render RBF workflow in TransactionsPage below the BumpFeePanel.

UI sections:

1. BumpFeePanel
2. PSBT Preview
3. Actions:
   - Sign PSBT
   - Broadcast
4. Result:
   - signed txid
   - broadcast txid

Rules:
- BumpFeePanel creates replacement PSBT.
- Sign button appears only after PSBT exists.
- Broadcast disabled until signed/finalized.
- Cancel clears bumpFeeTx + PSBT state.
```

---

# STEP 17 — Refresh Transactions After Broadcast

## Prompt

```text
After successful RBF broadcast:

- Show success message.
- Refresh transactions list.
- Optionally refresh wallet status if available.
- Clear or keep the workflow panel based on UX.

Recommended:
- Keep result visible.
- Add button “Close”.
- Refresh list immediately.
```

---

# STEP 18 — Add CPFP Placeholder

## Prompt

```text
Add CPFP placeholder behavior.

In handleCpfp(tx):

- If CPFP backend/API is not implemented:
  - show action message: “CPFP is not implemented yet”
  - do not throw
  - do not open a broken panel

Later, CPFP will follow a similar flow:
cpfp_psbt -> preview -> sign -> publish
```

---

# STEP 19 — Add CPFP API Skeleton Later

## Prompt

```text
When ready, add CPFP API function.

Expected shape may be:

export type CpfpPsbtInput = {
  walletName: string;
  txid: string;
  feeRateSatPerVb: number;
};

export async function cpfpPsbt(input: CpfpPsbtInput): Promise<WalletCpfpPsbtDto> {
  return invokeCommand<WalletCpfpPsbtDto>("cpfp_psbt", {
    request: {
      walletName: input.walletName,
      txid: input.txid,
      feeRateSatVb: input.feeRateSatPerVb,
    },
  });
}

Before implementing, inspect Rust DTO and command names.
```

---

# STEP 20 — Improve Transaction Row Conditions

## Prompt

```text
Improve transaction action availability helpers.

Add small helpers:

function canBumpFee(tx: WalletTxDto): boolean {
  return tx.replaceable === true && tx.confirmed === false;
}

function canCpfp(tx: WalletTxDto): boolean {
  return tx.confirmed === false;
}

Use these helpers in:
- TransactionActionsMenu
- TransactionsPage validation before opening panels

Do not rely only on disabled buttons. Validate in handlers too.
```

---

# STEP 21 — Error Handling

## Prompt

```text
Improve error handling for row actions.

Requirements:
- Bump fee backend error should appear near the BumpFeePanel.
- Copy txid errors should show action message.
- CPFP placeholder should not set global fatal error.
- Existing transaction loading errors remain separate.

Use separate state if needed:

const [actionError, setActionError] = useState<string | null>(null);
```

---

# STEP 22 — CSS for Action Feedback

## Prompt

```text
Add CSS for action message/error in transactions.css.

Suggested classes:

.transactions-action-message
.transactions-action-error
.transactions-workflow-panel

Keep action dropdown styles in actions.css only.
Keep table/page styles in transactions.css.
```

---

# STEP 23 — TypeScript Build Validation

## Prompt

```text
Run TypeScript validation.

Use the project’s package scripts, likely one of:

- npm run build
- npm run typecheck
- npm run lint

Fix:
- missing imports
- incorrect DTO field names
- wrong async return types
- stale self-imports
- unused variables

Do not ignore TypeScript errors.
```

---

# STEP 24 — Rust/Tauri Command Validation

## Prompt

```text
Validate that Tauri command names and request shapes match Rust.

Specifically verify:

- bump_fee_psbt command name
- request field names
- return DTO shape
- sign_psbt command name
- publish/broadcast command name

If runtime error says missing field:
- inspect Rust DTO
- update frontend mapping
- do not guess
```

---

# STEP 25 — Manual Test Plan

## Prompt

```text
Manual test the transaction action menu end-to-end.

Test cases:

1. Confirmed transaction
   - menu shows Details + Copy
   - Bump disabled/hidden
   - CPFP disabled/hidden

2. Unconfirmed non-RBF transaction
   - CPFP available or placeholder
   - Bump disabled/hidden

3. Unconfirmed RBF transaction
   - Bump Fee available
   - click Bump Fee
   - panel opens
   - suggested fee > current fee
   - create replacement PSBT works
   - sign works
   - broadcast works

4. Copy txid
   - txid copied
   - message appears
   - message disappears

5. Switch wallet
   - open workflow clears or reloads safely
   - no stale tx action remains

6. Table scroll
   - dropdown appears above rows
   - no clipping
```

---

# Acceptance Criteria

The feature is complete when:

```text
- Transactions table has a clean Actions dropdown per row.
- Menu actions are contextual.
- Details action works.
- Copy txid works with feedback.
- RBF bump fee flow works:
  bump_fee_psbt -> preview -> sign -> publish.
- Transactions refresh after broadcast.
- CPFP is safely stubbed or implemented.
- No action logic is inside TransactionActionsMenu.
- No duplicate CSS is added to globals.css.
- TypeScript build passes.
- Tauri command request mappings are verified.
```

---

# Future Enhancements

After RBF is stable, add:

1. CPFP full flow
2. Open in explorer
3. Copy raw transaction
4. Transaction labels/notes
5. Export transaction JSON
6. Mempool status
7. Fee comparison chart
8. Replace-by-fee history

---

End of document.
