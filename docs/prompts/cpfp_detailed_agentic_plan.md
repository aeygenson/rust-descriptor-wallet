# CPFP Implementation Agentic Plan — Rust Descriptor Wallet
## Goal

Implement **CPFP (Child Pays For Parent)** as a real end-to-end wallet feature.

Current state:

- ✅ Transactions page has a CPFP filter/tab
- ✅ CPFP candidates are visible as pending non-RBF transactions
- ✅ Actions menu has an “Accelerate (CPFP)” action
- ✅ Page-level persistent message is working
- ❌ Actual CPFP PSBT creation is not implemented yet

The goal is to replace:

```text
CPFP transaction creation is not implemented yet.
```

with a working flow:

```text
select parent tx → create child PSBT → sign → broadcast → mine → parent confirms
```

---

## Project Context

Repository:

```text
rust-descriptor-wallet
```

Main architecture:

```text
wallet_core      = Bitcoin wallet domain / BDK logic
wallet_api       = DTO / orchestration layer
wallet_sync      = sync + broadcast backends
wallet_storage   = wallet metadata / persistence
wallet_cli       = CLI boundary
wallet_desktop   = Tauri + React UI
```

Important design rule:

```text
Core/service/API: explicit parameters, no hidden UI defaults.
CLI/Tauri/React: choose UX defaults.
```

Recent important refactor:

```rust
replaceable: bool
```

was pushed explicitly through PSBT creation paths:

- fixed send
- send-max
- sweep
- consolidation
- coin-control variants

Do not reintroduce hidden `*_with_options` wrappers unless already existing in CLI runtime compatibility.

---

## Lessons Learned From Recent Work

### 1. Full-stack signature changes must be synchronized

When changing a DTO or function signature, update all layers together:

```text
wallet_core
wallet_api
wallet_cli
wallet_desktop/src-tauri
wallet_desktop/src React
tests
```

Common errors from partial updates:

```text
E0061: wrong number of arguments
E0308: tuple shape mismatch from into_parts()
E0599: method no longer exists
```

### 2. Keep transaction action messages page-owned

Do not show important messages inside dropdown menus.

Correct pattern:

```tsx
TransactionsPage owns message state
TransactionActionsMenu only calls onCpfp(tx)
TransactionsPage decides what to show
```

### 3. CPFP tab is not CPFP implementation

Current CPFP tab means:

```text
pending && !replaceable
```

These are CPFP *candidates*, not completed CPFP support.

Use clear UI wording:

```text
CPFP transaction creation is not implemented yet.
This tab only shows pending non-RBF CPFP candidates.
```

### 4. CPFP is not RBF

RBF modifies/replaces the parent transaction.

CPFP creates a new child transaction that spends an output from the parent and pays a high fee.

| Feature | RBF | CPFP |
|---|---|---|
| Parent must be replaceable | yes | no |
| Creates a new transaction | no | yes |
| Spends parent output | no | yes |
| Increases package fee rate | no | yes |

### 5. Keep strong logging

Follow the existing pattern:

```text
api psbt: create success ...
wallet_service: create_psbt success ...
selected_utxos=...
fee_rate_sat_per_vb=...
replaceable=...
```

Add CPFP-specific logs:

```text
parent_txid
child_txid
parent_fee
child_fee
package_fee
package_vsize
target_fee_rate
selected_parent_output
```

---

## High-Level CPFP Design

### CPFP candidate

A transaction is a CPFP candidate when:

```rust
!tx.confirmed && !tx.replaceable
```

But this is only UI-level approximation.

A real CPFP candidate must also have:

```text
an unconfirmed wallet-owned output from the parent tx
```

The child transaction must spend that output.

---

## Backend Implementation Plan

## Phase 1 — Define Core CPFP Domain Types

Add or verify types in `wallet_core`.

Suggested location:

```text
crates/wallet_core/src/model.rs
```

or a dedicated module:

```text
crates/wallet_core/src/service/cpfp.rs
```

Suggested core result type:

```rust
pub struct WalletCpfpPsbt {
    pub parent_txid: String,
    pub child_txid: String,
    pub psbt_base64: String,

    pub parent_fee_sat: Option<u64>,
    pub parent_vsize: Option<u64>,

    pub child_fee_sat: u64,
    pub child_vsize: u64,

    pub package_fee_sat: Option<u64>,
    pub package_vsize: Option<u64>,
    pub package_fee_rate_sat_per_vb: Option<u64>,

    pub target_fee_rate_sat_per_vb: u64,

    pub selected_parent_outpoint: String,
    pub selected_parent_output_value_sat: u64,

    pub output_amount_sat: u64,
}
```

If your existing `WalletCpfpPsbtDto` already exists in `wallet_api/src/model.rs`, align with it exactly rather than inventing new fields.

Important:

```text
Do not guess DTO field names.
Search model.rs first.
```

---

## Phase 2 — Add Core Service Function

Suggested function:

```rust
impl WalletService {
    pub fn create_cpfp_psbt(
        &mut self,
        network: Network,
        parent_txid: &str,
        target_fee_rate_sat_per_vb: FeeRateSatPerVb,
    ) -> WalletCoreResult<WalletCpfpPsbt> {
        todo!()
    }
}
```

Optional future parameters:

```rust
coin_control: Option<WalletCoinControl>
replaceable: bool
```

Initial version can keep CPFP child non-RBF:

```rust
replaceable = false
```

---

## Phase 3 — Core CPFP Algorithm

Pseudo-steps:

```text
1. Parse parent txid
2. Find wallet transaction by txid
3. Verify parent is unconfirmed
4. Find wallet-owned outputs created by the parent
5. Select one spendable parent output
6. Build child transaction spending that output
7. Child sends back to wallet internal/change address
8. Fee is set high enough to reach target package fee rate
9. Return unsigned PSBT
```

Important candidate selection logic:

```text
Prefer confirmed? no, CPFP specifically needs unconfirmed parent output.
Prefer wallet-owned output from parent.
Prefer largest parent output first.
Avoid dust output.
```

Suggested selection:

```rust
let candidate_outputs = parent.outputs
    .filter(|output| output.is_mine)
    .filter(|output| output.value > dust + estimated_fee)
```

---

## Phase 4 — Fee Calculation

CPFP target:

```text
(parent_fee + child_fee) / (parent_vsize + child_vsize) >= target_fee_rate
```

Formula:

```text
required_package_fee = target_fee_rate * (parent_vsize + child_vsize)
child_fee = required_package_fee - parent_fee
```

Also enforce:

```text
child_fee >= minimum relay fee for child alone
child_output_value >= dust
child_fee < selected_parent_output_value
```

If parent fee/vsize unavailable:

Option A for initial implementation:

```text
Require parent fee and vsize available, else return error.
```

Option B:

```text
Allow target child fee rate only, but label package rate unknown.
```

Recommended initial version:

```text
Require enough parent tx data to compute package fee.
```

Error examples:

```rust
CpfpParentNotFound
CpfpParentAlreadyConfirmed
CpfpNoWalletOutput
CpfpOutputTooSmall
CpfpParentFeeUnknown
CpfpInsufficientValue
```

---

## Phase 5 — Wallet Core Errors

Add clear errors to `WalletCoreError`:

```rust
#[error("CPFP parent transaction not found: {0}")]
CpfpParentNotFound(String),

#[error("CPFP parent transaction is already confirmed: {0}")]
CpfpParentAlreadyConfirmed(String),

#[error("CPFP parent transaction has no spendable wallet-owned output: {0}")]
CpfpNoSpendableParentOutput(String),

#[error("CPFP parent fee or vsize is unavailable: {0}")]
CpfpParentFeeUnavailable(String),

#[error("CPFP child output would be dust")]
CpfpChildOutputDust,

#[error("CPFP selected parent output is too small for requested fee")]
CpfpInsufficientParentOutputValue,
```

Use exact project error style.

---

## Phase 6 — Wallet API DTO Layer

Check:

```text
crates/wallet_api/src/model.rs
```

Existing memory says `model.rs` is the DTO/result layer for CLI/Tauri/API and may already contain:

```rust
WalletCpfpPsbtDto
```

Use it.

If missing, add:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletCpfpPsbtDto {
    pub parent_txid: String,
    pub child_txid: String,
    pub psbt_base64: String,
    pub parent_fee_sat: Option<u64>,
    pub child_fee_sat: u64,
    pub package_fee_sat: Option<u64>,
    pub target_fee_rate_sat_per_vb: u64,
    pub package_fee_rate_sat_per_vb: Option<u64>,
    pub selected_parent_outpoint: String,
    pub selected_parent_output_value_sat: u64,
}
```

Prefer existing naming conventions.

---

## Phase 7 — Wallet API Service

Add service function:

```text
crates/wallet_api/src/service/psbt.rs
```

or dedicated:

```text
crates/wallet_api/src/service/cpfp.rs
```

Suggested API service:

```rust
pub async fn create_cpfp(
    storage: &WalletStorage,
    name: &str,
    parent_txid: &str,
    target_fee_rate_sat_per_vb: u64,
) -> WalletApiResult<WalletCpfpPsbtDto> {
    let config = load_wallet_config(storage, name).await?;
    let fee_rate = FeeRateSatPerVb::new(target_fee_rate_sat_per_vb)?;

    let parent_txid = parent_txid.to_string();
    let name_for_error = name.to_string();

    let result = spawn_wallet_blocking(move || {
        let mut wallet = WalletService::load_or_create(&config)?;
        wallet.create_cpfp_psbt(config.network, &parent_txid, fee_rate)
    })
    .await?;

    Ok(result.into())
}
```

Logging:

```rust
debug!("api cpfp: create start name={} parent_txid={} target_fee_rate={}", ...);

info!(
    "api cpfp: create success name={} parent_txid={} child_txid={} child_fee={} package_fee_rate={:?}",
    ...
);
```

---

## Phase 8 — WalletApi Public Method

In:

```text
crates/wallet_api/src/api.rs
```

Add:

```rust
pub async fn create_cpfp_psbt(
    &self,
    name: &str,
    parent_txid: &str,
    target_fee_rate_sat_per_vb: u64,
) -> WalletApiResult<WalletCpfpPsbtDto> {
    cpfp::create(&self.storage, name, parent_txid, target_fee_rate_sat_per_vb).await
}
```

If placing in `psbt.rs`, use:

```rust
psbt::create_cpfp(...)
```

---

## Phase 9 — CLI Support

Add CLI command:

```text
wallet_cli cpfp-psbt --name regtest-local --parent-txid <txid> --fee-rate 5
```

In CLI command enum:

```rust
CpfpPsbt {
    #[arg(long)]
    name: String,

    #[arg(long = "parent-txid")]
    parent_txid: String,

    #[arg(long = "fee-rate")]
    fee_rate: u64,
}
```

Runtime:

```rust
pub async fn create_cpfp_psbt(
    api: &WalletApi,
    name: &str,
    parent_txid: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    let psbt = api.create_cpfp_psbt(name, parent_txid, fee_rate_sat_per_vb).await?;

    println!("CPFP PSBT created:");
    println!("parent_txid={}", psbt.parent_txid);
    println!("child_txid={}", psbt.child_txid);
    println!("child_fee={} sats", psbt.child_fee_sat);
    println!("psbt_base64:\n{}", psbt.psbt_base64);

    Ok(())
}
```

---

## Phase 10 — Tauri Request DTO

In:

```text
apps/wallet_desktop/src-tauri/src/commands/transactions_model.rs
```

or suitable command model file:

```rust
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCpfpPsbtRequest {
    pub wallet_name: String,
    pub parent_txid: String,
    pub fee_rate_sat_vb: u64,
}

impl CreateCpfpPsbtRequest {
    pub fn into_parts(self) -> (String, String, u64) {
        (self.wallet_name, self.parent_txid, self.fee_rate_sat_vb)
    }
}
```

---

## Phase 11 — Tauri Command

Add command:

```rust
#[tauri::command]
pub async fn create_cpfp_psbt(
    api: State<'_, WalletApi>,
    request: CreateCpfpPsbtRequest,
) -> Result<WalletCpfpPsbtDto, String> {
    let (wallet_name, parent_txid, fee_rate_sat_vb) = request.into_parts();

    api.create_cpfp_psbt(&wallet_name, &parent_txid, fee_rate_sat_vb)
        .await
        .map_err(api_error_to_string)
}
```

Register command in Tauri command list.

---

## Phase 12 — Frontend API

In:

```text
apps/wallet_desktop/src/features/transactions/api.ts
```

Add:

```ts
export type CreateCpfpPsbtInput = {
  walletName: string;
  parentTxid: string;
  feeRateSatVb: number;
};

export async function createCpfpPsbt(input: CreateCpfpPsbtInput): Promise<WalletCpfpPsbtDto> {
  return invoke("create_cpfp_psbt", {
    request: {
      walletName: input.walletName,
      parentTxid: input.parentTxid,
      feeRateSatVb: input.feeRateSatVb,
    },
  });
}
```

---

## Phase 13 — Frontend DTO Type

In:

```text
apps/wallet_desktop/src/shared/types/dtos.ts
```

Add or align:

```ts
export type WalletCpfpPsbtDto = {
  parent_txid: string;
  child_txid: string;
  psbt_base64: string;
  parent_fee_sat?: number | null;
  child_fee_sat: number;
  package_fee_sat?: number | null;
  target_fee_rate_sat_per_vb: number;
  package_fee_rate_sat_per_vb?: number | null;
  selected_parent_outpoint: string;
  selected_parent_output_value_sat: number;
};
```

If you already convert DTOs to camelCase elsewhere, follow existing convention.

---

## Phase 14 — UI Workflow

In:

```text
TransactionsPage.tsx
```

Add state similar to RBF:

```ts
const [cpfpTx, setCpfpTx] = useState<WalletTxDto | null>(null);
const [cpfpPsbt, setCpfpPsbt] = useState<WalletCpfpPsbtDto | null>(null);
const [cpfpSignedPsbt, setCpfpSignedPsbt] = useState<WalletSignedPsbtDto | null>(null);
const [cpfpBroadcastResult, setCpfpBroadcastResult] = useState<TxBroadcastResultDto | null>(null);
const [cpfpLoading, setCpfpLoading] = useState(false);
```

Change:

```ts
showActionMessage("CPFP transaction creation is not implemented yet...")
```

to:

```ts
setCpfpTx(tx);
setCpfpPsbt(null);
setCpfpSignedPsbt(null);
setCpfpBroadcastResult(null);
```

---

## Phase 15 — CPFP Panel Component

Create:

```text
apps/wallet_desktop/src/features/transactions/components/CpfpPanel.tsx
```

Fields:

```text
Parent txid
Current parent fee rate
Target package fee rate
Create CPFP PSBT button
```

Props:

```ts
type CpfpPanelProps = {
  tx: WalletTxDto;
  walletName: string;
  loading: boolean;
  onCancel: () => void;
  onCreatePsbt: (input: CreateCpfpPsbtInput) => Promise<void>;
};
```

Default fee rate:

```text
current fee rate + 1
or 5 sat/vB
```

---

## Phase 16 — CPFP Workflow Panel

Reuse style from:

```text
RbfPsbtWorkflowPanel
```

Actions:

```text
Create CPFP PSBT
Sign
Broadcast
Close
```

Signing can reuse existing:

```ts
signPsbt
publishPsbt
```

because final object contains `psbt_base64`.

---

## Phase 17 — Tests

Add regtest integration test.

Test flow:

```text
1. Create wallet
2. Fund wallet
3. Send low-fee non-RBF tx
4. Publish parent
5. Do NOT mine
6. Create CPFP PSBT for parent
7. Sign CPFP PSBT
8. Broadcast child
9. Mine block
10. Sync wallet
11. Verify parent confirmed
12. Verify child confirmed
```

Suggested test name:

```rust
wallet_cpfp_psbt_confirms_non_rbf_parent
```

Assertions:

```rust
assert_eq!(parent.replaceable, false);
assert_eq!(parent.confirmed, false);

let cpfp = api.create_cpfp_psbt(wallet_name, &parent_txid, 5).await?;
assert_eq!(cpfp.parent_txid, parent_txid);
assert!(cpfp.child_fee_sat > 0);
assert!(!cpfp.psbt_base64.is_empty());
```

After mining:

```rust
let txs = api.transactions(wallet_name).await?;
assert!(txs.iter().any(|tx| tx.txid == parent_txid && tx.confirmed));
assert!(txs.iter().any(|tx| tx.txid == cpfp.child_txid && tx.confirmed));
```

---

## Phase 18 — Error Tests

Add tests for:

```text
parent tx not found
parent already confirmed
parent has no wallet-owned output
fee too high / output too small
```

Suggested test names:

```rust
wallet_cpfp_rejects_unknown_parent
wallet_cpfp_rejects_confirmed_parent
wallet_cpfp_rejects_parent_without_wallet_output
wallet_cpfp_rejects_fee_too_high_for_parent_output
```

---

## Important Implementation Notes

### CPFP requires wallet-owned parent output

Do not assume every outgoing transaction has a spendable output.

Example:

```text
If parent spends all funds to external recipient and no change output exists,
wallet cannot CPFP it.
```

UI should display:

```text
No wallet-owned output is available for CPFP.
```

### CPFP may use change output

Most likely candidate:

```text
the parent change output
```

### Non-RBF does not mean CPFP possible

UI filter currently shows all pending non-RBF txs. Later refine candidate detection if backend exposes:

```rust
can_cpfp: bool
```

Future DTO improvement:

```rust
WalletTxDto {
    can_cpfp: bool,
    cpfp_reason: Option<String>,
}
```

---

## Final Target User Experience

In Transactions page:

```text
CPFP tab → shows pending non-RBF txs
Actions → Accelerate (CPFP)
Panel opens → choose target fee
Create PSBT
Sign
Broadcast
Sync/mine
Confirmed
```

---

## Do Not Do

- Do not implement CPFP as RBF.
- Do not require parent to be replaceable.
- Do not silently no-op if no parent output exists.
- Do not hide backend errors with generic messages.
- Do not add another temporary “not implemented” message once backend is wired.

---

## Suggested First Concrete Step

Start with `wallet_core` only:

```rust
WalletService::create_cpfp_psbt(...)
```

Get a unit/regtest backend test passing before touching React UI.

Then wire upward.

---

## Build / Validation Commands

Use:

```bash
cargo check -p wallet_core
cargo check -p wallet_api --tests
cargo check -p wallet_cli
cargo check -p wallet_desktop_tauri
cargo test -p wallet_api --test regtest_flow --no-run
```

Run focused test:

```bash
cargo test -p wallet_api --test regtest_flow wallet_cpfp_psbt_confirms_non_rbf_parent
```

---

## Completion Checklist

- [ ] Core CPFP service added
- [ ] Core errors added
- [ ] API DTO added/aligned
- [ ] API service added
- [ ] WalletApi method added
- [ ] CLI command added
- [ ] Tauri command added
- [ ] Frontend API added
- [ ] CPFP panel added
- [ ] Sign/broadcast flow reused
- [ ] Regtest success test added
- [ ] Error tests added
- [ ] “not implemented” message removed
- [ ] cargo check passes
- [ ] focused CPFP test passes
