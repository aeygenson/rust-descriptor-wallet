# Agentic Implementation Prompt: Receive Page + Address Management

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
- Current GUI already includes:
  - Overview page
  - Send page
  - UTXO page
  - Transactions page
  - Coin control
  - Send fixed / send max / sweep / consolidate
  - RBF
  - CPFP
  - Backend health
  - Transaction graph/lineage helpers

The recent frontend architecture pattern is:

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

Do not put formatting helpers in components.
Do not put pure domain helpers in pages.
Do not put API calls in components unless already established by existing feature pattern.

---

## Goal

Implement a professional **Receive page + Address Management** workflow.

The user should be able to:

```text
Open Receive page
→ see current wallet/network
→ generate a new receive address
→ copy address
→ optionally label the address locally
→ view recent/generated receive addresses
→ see basic address metadata
→ avoid accidental address reuse
```

This is primarily a GUI/desktop workflow, but backend/Tauri support may be needed depending on existing commands.

---

## Non-goals for first version

Do NOT implement:

- full address book for external recipients
- cloud sync
- QR image export
- contact management
- address gap-limit configuration
- BIP21 payment URI editing beyond basic optional amount/message if not already easy
- database-heavy label persistence unless storage support is already available

Keep v1 clean and useful.

---

## Source of truth checks first

Before coding, inspect these files:

```text
crates/wallet_api/src/model.rs
crates/wallet_api/src/service/wallet.rs
crates/wallet_core/src/service/*
apps/wallet_desktop/src-tauri/src/commands/*
apps/wallet_desktop/src-tauri/src/commands/wallet.rs
apps/wallet_desktop/src-tauri/src/commands/wallet_model.rs
apps/wallet_desktop/src/features/wallet/api.ts
apps/wallet_desktop/src/shared/types/dtos.ts
apps/wallet_desktop/src/app/router or App.tsx
apps/wallet_desktop/src/app/shell/sidebar files
```

Find existing address-generation command/API.

Likely existing CLI/backend command may be named something like:

```text
address
generate_address
get_address
get_receive_address
new_address
```

Do not guess exact names. Inspect current files and mirror existing DTO names exactly.

---

## Important architecture rule

If backend already has address generation, use it.

If backend has CLI but Tauri does not expose it, add a thin Tauri command.

If backend does not have API-level DTO for receive address, add one in `wallet_api/src/model.rs` or use existing DTO if present.

Preferred DTO shape:

```rust
pub struct WalletAddressDto {
    pub address: String,
    pub network: String,
    pub keychain: String,
    pub index: Option<u32>,
    pub is_new: bool,
}
```

But do not force this exact shape if existing backend already has something.

Frontend should mirror backend DTO in:

```text
apps/wallet_desktop/src/shared/types/dtos.ts
```

---

## Desired frontend file structure

Create:

```text
apps/wallet_desktop/src/features/receive/api.ts
apps/wallet_desktop/src/features/receive/types.ts
apps/wallet_desktop/src/features/receive/lib.ts
apps/wallet_desktop/src/features/receive/format.ts
apps/wallet_desktop/src/features/receive/receive.css
apps/wallet_desktop/src/features/receive/components/ReceiveHeader.tsx
apps/wallet_desktop/src/features/receive/components/ReceiveAddressCard.tsx
apps/wallet_desktop/src/features/receive/components/ReceiveAddressActions.tsx
apps/wallet_desktop/src/features/receive/components/ReceiveAddressHistory.tsx
apps/wallet_desktop/src/features/receive/components/ReceiveAddressLabelEditor.tsx
apps/wallet_desktop/src/pages/ReceivePage.tsx
```

Only create components that are actually used in v1.

If project currently keeps CSS globally, add receive styles to the existing global stylesheet instead, but prefer feature-local CSS if the app already supports it.

---

## Receive feature responsibilities

### `features/receive/api.ts`

Only Tauri/backend invocations.

Candidate functions:

```ts
export async function getReceiveAddress(walletName: string): Promise<WalletAddressDto>
export async function generateReceiveAddress(walletName: string): Promise<WalletAddressDto>
```

If backend only has one function that always returns a new address, name frontend API clearly:

```ts
generateReceiveAddress(...)
```

Do not invent fake history API unless backend supports it.

For local-only recent history, manage it in frontend storage or page state.

---

### `features/receive/types.ts`

Put frontend feature types and component props here.

Examples:

```ts
export type ReceiveAddressRecord = {
  address: string;
  walletName: string;
  network?: string;
  keychain?: string;
  index?: number | null;
  label?: string;
  createdAtIso: string;
  isNew?: boolean;
};

export type ReceiveAddressCardProps = {
  record: ReceiveAddressRecord | null;
  loading?: boolean;
  error?: string | null;
  onGenerate: () => void;
  onCopy: (address: string) => void;
  onLabelChange?: (address: string, label: string) => void;
};

export type ReceiveAddressHistoryProps = {
  addresses: ReceiveAddressRecord[];
  onCopy: (address: string) => void;
  onSelect?: (record: ReceiveAddressRecord) => void;
};
```

Keep prop types centralized.

---

### `features/receive/lib.ts`

Pure helpers only.

Examples:

```ts
export function buildReceiveAddressRecord(...)
export function upsertReceiveAddressRecord(...)
export function normalizeAddressLabel(...)
export function isAddressAlreadyKnown(...)
export function sortReceiveAddressRecords(...)
export function getReceiveAddressStorageKey(walletName: string)
```

Local storage helper functions may live here only if they are pure enough and not React-specific.
If they touch `localStorage`, keep names explicit:

```ts
loadReceiveAddressHistoryFromStorage(...)
saveReceiveAddressHistoryToStorage(...)
```

That is acceptable for v1 if no backend persistence exists.

---

### `features/receive/format.ts`

Presentation helpers only.

Examples:

```ts
export function shortAddress(address: string): string
export function formatAddressIndex(index?: number | null): string
export function formatReceiveAddressMeta(record: ReceiveAddressRecord): string
export function formatCreatedAt(iso: string): string
```

Do not include validation or storage logic.

---

### `ReceivePage.tsx`

Page orchestration only:

- read `selectedWalletName` from wallet provider
- call receive API
- manage current address state
- manage loading/error
- manage local recent address history if backend has no history
- render components

Do not place formatting helpers here.
Do not place reusable local-storage helpers here.

---

## Backend / Tauri implementation path

### Phase 1 — Inspect existing backend

Check whether any API already exists for address generation.

Possible paths:

```text
wallet_api service
wallet_core service
wallet_cli address command
Tauri wallet commands
```

If CLI already supports:

```bash
wallet_cli address --name regtest-local
```

then backend logic exists. Reuse the same service layer.

---

### Phase 2 — Add or expose Tauri command

If missing, add command:

```rust
#[tauri::command]
pub async fn generate_receive_address(
    state: tauri::State<'_, AppState>,
    request: GenerateReceiveAddressRequest,
) -> Result<WalletAddressDto, String>
```

Possible request:

```rust
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateReceiveAddressRequest {
    pub wallet_name: String,
}
```

Follow current command patterns:

- request structs in `commands/*_model.rs`
- thin command functions in `commands/*.rs`
- use `into_parts()` if consistent with existing code
- map errors same way as existing commands
- do not put business logic in Tauri layer

Register the command in the Tauri command handler.

---

### Phase 3 — Add frontend DTO

In:

```text
apps/wallet_desktop/src/shared/types/dtos.ts
```

Add only what backend returns.

Example:

```ts
export type WalletAddressDto = {
  address: string;
  network?: string;
  keychain?: string;
  index?: number | null;
  is_new?: boolean;
};
```

Prefer matching actual Rust serde output. If Rust uses snake_case, keep snake_case unless project already maps to camelCase.

---

## UI / UX design

### Page header

Show:

```text
Receive
Wallet: <selected wallet>
Network: <network if available>
```

If no wallet selected:

```text
Select a wallet to generate receive addresses.
```

---

### Main address card

Show current generated address prominently:

```text
[ address monospace ]
[ Copy ] [ Generate new address ]
```

Include:

```text
Network
Keychain
Index
Created/generated time
Label
```

Use a warning:

```text
Use a new address for each payment to improve privacy.
```

---

### QR code

If QR dependency already exists, render QR.
If not, skip QR in v1 and leave TODO.

Do not introduce heavy dependencies unless necessary.
If adding dependency, choose lightweight React QR package and document it.

Optional:

```text
QRCodeSVG
```

But first version can be address-only.

---

### Address labels

Support a simple local label:

```text
Label: [ input ]
Save label
```

Label can be stored in frontend local storage keyed by wallet + address.

No backend persistence required for v1.

---

### Recent addresses / history

Show recent generated receive addresses.

Columns/cards:

```text
Address
Label
Created
Keychain/index
Copy
```

If no backend history exists, maintain local history only.

Important: local history should not pretend to be complete wallet history.

Add copy:

```text
Recently generated in this app
```

---

## Address reuse warning

If generated address already exists in local history, show:

```text
This address was already generated before. Consider generating a fresh address.
```

Do not block user.

---

## Copy behavior

Use `navigator.clipboard.writeText(address)`.

Show short success state:

```text
Copied
```

Avoid alert dialogs.

---

## Routing and navigation

Add route:

```text
/receive
```

Add sidebar item:

```text
Receive
```

Place near:

```text
Overview
Send
UTXOs
Transactions
```

Suggested order:

```text
Overview
Receive
Send
UTXOs
Transactions
```

---

## Styling

Use same visual language as current app:

- dark cards
- rounded panels
- small uppercase labels
- blue accent highlights
- monospace address pills
- status/warning badges

Suggested classes:

```css
.receive-page
.receive-grid
.receive-card
.receive-address
.receive-address__value
.receive-actions
.receive-history
.receive-warning
.receive-label-editor
```

Keep responsive layout:

- desktop: main address card + side info/history
- mobile: single column

---

## Validation and edge cases

Handle:

```text
no selected wallet
backend unavailable
watch-only wallet
address generation error
clipboard failure
duplicate local address
network mismatch display
empty address response
```

Watch-only wallet should still be able to generate receive addresses if backend supports it.

---

## Testing checklist

### Manual UI tests

1. Open Receive with no wallet selected.
2. Select wallet.
3. Generate address.
4. Copy address.
5. Generate second address.
6. Confirm both appear in recent list.
7. Add/edit label.
8. Reload app and confirm local recent list/labels persist if local storage implemented.
9. Switch wallet and confirm history is wallet-specific.
10. Switch back and confirm previous wallet history returns.
11. Try backend offline and confirm clean error state.

### Backend tests

If adding backend/Tauri command:

```bash
cargo check
cargo run -p wallet_cli -- address --name regtest-local
```

Frontend:

```bash
npm run typecheck
npm run build
```

or project equivalent.

---

## Implementation phases

### Phase 1 — Backend/Tauri discovery

- Inspect existing address-generation support.
- Decide whether frontend can call existing command.
- If missing, add Tauri command only as thin wrapper.

Stop after this phase if uncertain and report exact existing backend names.

---

### Phase 2 — Frontend receive feature skeleton

Create:

```text
features/receive/api.ts
features/receive/types.ts
features/receive/lib.ts
features/receive/format.ts
components/*
pages/ReceivePage.tsx
```

Add route/sidebar.

Keep first UI simple.

---

### Phase 3 — Generate/copy flow

Implement:

```text
Generate address
Copy address
Loading/error states
```

No labels yet if time is short.

---

### Phase 4 — Local address history

Implement local storage by wallet name.

Helpers in `receive/lib.ts`.

Do not store secrets.
Only store public addresses and labels.

Storage key:

```ts
rust-descriptor-wallet:receive-addresses:<walletName>
```

---

### Phase 5 — Labels

Add label editor and persistence.

Use simple string labels.

Normalize:

```text
trim
max length 80
empty label removes label
```

---

### Phase 6 — Polish

Add:
- duplicate warning
- privacy warning
- better empty states
- responsive CSS
- optional QR if dependency already exists

---

## Code quality rules

- No `any`.
- No local duplicate prop types inside components.
- Component props go into `features/receive/types.ts`.
- Formatting goes into `features/receive/format.ts`.
- Pure helpers/storage helpers go into `features/receive/lib.ts`.
- API calls go into `features/receive/api.ts`.
- Page should orchestrate only.
- Match exact DTO names from `shared/types/dtos.ts`.
- Match exact Tauri command names from Rust.
- Keep backend/Tauri thin.
- Use `thiserror`/existing error conventions in Rust layers.
- Do not change existing send/utxo/transaction flows.

---

## Expected final result

At the end, the GUI should have:

```text
Receive page
Generate receive address
Copy receive address
Recently generated address list
Optional labels
Wallet-specific local address history
Clean loading/error/empty states
Route + sidebar navigation
```

And architecture should match the existing matured frontend feature style.

---

## Final validation checklist

Before finishing, verify:

```text
cargo check
npm typecheck/build
Receive route works
Generate button works
Copy button works
No TypeScript errors
No duplicate local prop types
No helpers left inside page that belong in lib/format
```

Then summarize:

```text
Files created
Files updated
Backend command used/added
Manual tests passed
Known limitations
```
