# Receive Page Next Steps — Agentic Implementation Plan

## Context

The Receive Page MVP is now working.

Confirmed working:

- `/receive` route exists.
- Sidebar navigation includes Receive.
- `WalletReceiveAddressDto` exists in frontend DTOs.
- Tauri command `get_receive_address` is registered.
- Frontend calls Tauri with `{ walletName }`.
- Backend returns regtest Taproot receive addresses.
- Bitcoin URI generation works, for example:

```text
bitcoin:bcrt1puvvfhfndf6yhr872pmhjunpycxk70c9kj436we25e5yty9p9yqcq8afxyz
```

Current Receive DTO shape:

```ts
export interface WalletReceiveAddressDto {
  address: string;
  keychain: string;
  index: number | null;
}
```

Important: do **not** add `network` to this DTO. Network should come from wallet summary/status later if needed.

---

## Goal

Add the next Receive Page features in safe vertical slices:

1. Backend-generated Bitcoin URI.
2. Backend-generated QR payloads (SVG + terminal QR).
3. Optional Bitcoin URI request parameters.
4. Copy/share request UX.
5. Backend-persisted receive address history.
6. Backend-persisted receive address labels.
7. Backend-persisted address book.
8. CLI + Tauri parity.
9. Receive-page polish and tests.

Do this incrementally. After each step, run typecheck and app runtime test.

---

## Step 1 — Move Bitcoin URI + QR generation into backend

### Goal

Make Rust/backend the source of truth for:

- Bitcoin URI generation
- QR SVG generation
- terminal/CLI QR generation

This keeps CLI and Tauri behavior identical.

Frontend should render backend-generated QR payloads instead of generating QR content independently.

---

### Backend crates likely involved

```txt
crates/wallet_api
crates/wallet_core
apps/wallet_desktop/src-tauri
apps/wallet_cli
```

---

### Recommended dependencies

Add QR crate:

```toml
qrcode = "0.14"
```

---

### DTO update

Update receive DTOs.

Rust:

```rust
pub struct WalletReceiveAddressDto {
    pub address: String,
    pub keychain: String,
    pub index: Option<u32>,
    pub bitcoin_uri: String,
    pub qr_svg: Option<String>,
}
```

Frontend:

```ts
export interface WalletReceiveAddressDto {
  address: string;
  keychain: string;
  index: number | null;
  bitcoinUri: string;
  qrSvg?: string | null;
}
```

Important: still do NOT add `network`.

---

### Backend helpers

Add helpers such as:

```rust
build_bitcoin_uri(address: &str) -> String
build_qr_svg(payload: &str) -> Result<String>
build_terminal_qr(payload: &str) -> Result<String>
```

Suggested QR rendering APIs:

```rust
use qrcode::{QrCode, render::svg};
use qrcode::render::unicode;
```

---

### Frontend rendering

Files likely affected:

```txt
src/features/receive/components/ReceiveAddressCard.tsx
src/styles/receive.css
```

Render backend-generated SVG:

```tsx
<img
  src={`data:image/svg+xml;utf8,${encodeURIComponent(qrSvg)}`}
  alt="Receive QR"
/>
```

Do not generate QR payloads independently in frontend.

---

### CLI integration

Add optional CLI support:

```bash
wallet_cli receive --name regtest-local --qr
```

Potential formats:

```bash
--qr-format ascii
--qr-format svg
```

---

### Acceptance

- Backend generates canonical Bitcoin URI.
- Backend generates QR SVG.
- CLI can print QR.
- Tauri renders backend QR.
- QR payload equals copied Bitcoin URI.
- No frontend/backend URI drift.
- Typecheck and cargo check pass.

## Step 2 — Add receive request fields

### Goal

Allow user to optionally add:

- amount in sats
- label
- message

and generate a richer Bitcoin URI through backend/shared Rust helpers.

Example:

```text
bitcoin:bcrt1...?amount=0.00100000&label=Invoice&message=Test
```

### Files to update

```txt
src/features/receive/types.ts
src/features/receive/lib.ts
src/features/receive/format.ts
src/features/receive/components/ReceiveAddressCard.tsx
src/pages/ReceivePage.tsx
src/styles/receive.css
```

### Add type

```ts
export type ReceiveRequestFormState = {
  amountSat: string;
  label: string;
  message: string;
};
```

### Add helpers

In `receive/lib.ts`:

```ts
export function satsToBtcString(amountSat: number): string {
  return (amountSat / 100_000_000).toFixed(8);
}

export function buildBitcoinUriWithParams({
  address,
  amountSat,
  label,
  message,
}: {
  address: string;
  amountSat?: number | null;
  label?: string | null;
  message?: string | null;
}): string {
  const normalizedAddress = address.trim();

  if (!normalizedAddress) {
    return "";
  }

  const params = new URLSearchParams();

  if (amountSat !== null && amountSat !== undefined && amountSat > 0) {
    params.set("amount", satsToBtcString(amountSat));
  }

  if (label?.trim()) {
    params.set("label", label.trim());
  }

  if (message?.trim()) {
    params.set("message", message.trim());
  }

  const query = params.toString();

  return query ? `bitcoin:${normalizedAddress}?${query}` : `bitcoin:${normalizedAddress}`;
}
```

### Validation

- Amount must be empty or positive integer.
- Label/message optional.
- If amount invalid, disable Copy URI and QR should use base address URI or show validation warning.

### Acceptance

- Empty fields keep current URI.
- Amount converts sats to BTC in URI.
- Label/message are URL encoded.
- Copy URI copies full request URI.
- QR updates when fields change.

---

## Step 3 — Add copy feedback

### Goal

Show clear UI feedback:

- “Address copied”
- “URI copied”

### Files

```txt
src/features/receive/components/ReceiveAddressCard.tsx
src/pages/ReceivePage.tsx
src/styles/receive.css
```

### Implementation

```ts
type ReceiveCopyTarget = "address" | "uri" | null;
const [copiedTarget, setCopiedTarget] = useState<ReceiveCopyTarget>(null);
```

After copy:

```ts
setCopiedTarget("address");
window.setTimeout(() => setCopiedTarget(null), 1500);
```

Button labels:

```tsx
{copiedTarget === "address" ? "Address copied" : "Copy address"}
{copiedTarget === "uri" ? "URI copied" : "Copy bitcoin URI"}
```

### Acceptance

- Copy feedback appears immediately.
- Feedback clears automatically.
- No stale copied state after generating a new address.

---

## Step 4 — Backend-persisted receive address history

### Goal

Persist generated receive addresses in backend storage.

Do not use frontend-only memory.

History should be available to:

- Tauri GUI
- CLI
- future mobile/web clients

---

### Backend crates likely involved

```txt
crates/wallet_storage
crates/wallet_api
apps/wallet_desktop/src-tauri
apps/wallet_cli
```

---

### Suggested storage fields

```txt
wallet_name
address
keychain
index
bitcoin_uri
label nullable
generated_at
```

---

### Suggested DTO

```rust
pub struct WalletReceiveAddressHistoryDto {
    pub address: String,
    pub keychain: String,
    pub index: Option<u32>,
    pub bitcoin_uri: String,
    pub label: Option<String>,
    pub generated_at: String,
}
```

---

### Suggested commands

```txt
list_receive_addresses
```

Potential future commands:

```txt
clear_receive_history
```

---

### Frontend files likely affected

```txt
src/pages/ReceivePage.tsx
src/features/receive/components/ReceiveAddressHistory.tsx
src/features/receive/types.ts
src/styles/receive.css
```

Frontend should load history from backend.

Do not use localStorage or frontend-only session state.

---

### Acceptance

- Generated addresses persist.
- Restarting app preserves history.
- Duplicate addresses are deduplicated.
- History available to CLI and GUI.
- History list remains wallet-scoped.

---

## Step 5 — Backend-persisted receive address labels

### Goal

Allow users to label generated receive addresses.

Labels must be stored in backend DB, not localStorage.

This keeps CLI and Tauri consistent.

---

### Backend crates likely involved

```txt
crates/wallet_storage
crates/wallet_api
apps/wallet_desktop/src-tauri
apps/wallet_cli
```

---

### Suggested DTO

```rust
pub struct WalletReceiveAddressLabelDto {
    pub address: String,
    pub label: String,
    pub updated_at: String,
}
```

---

### Suggested commands

```txt
label_receive_address
clear_receive_address_label
```

Use camelCase Tauri args.

---

### Frontend files likely affected

```txt
src/features/receive/components/ReceiveAddressHistory.tsx
src/features/receive/components/ReceiveAddressCard.tsx
src/pages/ReceivePage.tsx
```

---

### Acceptance

- Labels persist after restart.
- Labels are wallet-scoped.
- Labels visible in history and receive card.
- CLI can eventually access labels too.

---

## Step 6 — Backend-persisted address book foundation

### Goal

Implement reusable address book management backed by wallet storage.

Do not use localStorage.

Address book entries are for known recipients, not wallet receive addresses.

---

### Backend crates likely involved

```txt
crates/wallet_storage
crates/wallet_api
apps/wallet_desktop/src-tauri
apps/wallet_cli
```

---

### Suggested storage fields

```txt
id
label
address
notes nullable
created_at
updated_at
```

---

### Suggested Rust DTO

```rust
pub struct AddressBookEntryDto {
    pub id: String,
    pub label: String,
    pub address: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
```

---

### Suggested commands

```txt
list_address_book_entries
create_address_book_entry
update_address_book_entry
delete_address_book_entry
```

---

### CLI integration

Potential CLI support:

```bash
wallet_cli address-book list
wallet_cli address-book add --label "Kraken" --address "bc1..."
wallet_cli address-book delete --id ...
```

---

### Frontend structure

```txt
src/features/address-book/
  types.ts
  lib.ts
  components/
    AddressBookPanel.tsx
    AddressBookEntryForm.tsx
    AddressBookPicker.tsx
```

---

### Acceptance

- Can add/edit/delete entries.
- Entries persist via backend DB.
- Entries available to CLI and GUI.
- No localStorage dependency.

---

## Step 7 — Integrate address book into Send page

### Goal

Allow selecting a saved recipient in Send/Sweep/Send Max forms.

### Files

```txt
src/features/send/components/FixedSendForm.tsx
src/features/send/components/SendMaxForm.tsx
src/features/send/components/SweepForm.tsx
src/features/address-book/*
```

### Approach

Add optional prop to forms:

```ts
addressBookEntries?: AddressBookEntry[];
```

Add dropdown:

```tsx
<select onChange={(event) => updateField("toAddress", event.target.value)}>
  <option value="">Select saved recipient…</option>
</select>
```

### Acceptance

- Selecting entry fills destination address.
- Manual entry still works.
- Existing send payload shape unchanged.

---

## Step 8 — CLI + Tauri parity review

### Goal

Ensure Receive functionality behaves consistently across:

- wallet_cli
- Tauri desktop
- future API/mobile clients

---

### Review checklist

Ensure these features exist consistently:

```txt
receive address generation
bitcoin URI generation
QR generation
receive history
receive labels
address book
```

---

### Important architecture rule

Business logic should live in Rust/shared backend layers.

Frontend React code should primarily:

- render state
- trigger commands
- manage temporary UI state

Avoid frontend-only business logic duplication.

---

### Acceptance

- CLI and Tauri generate identical URIs.
- CLI and Tauri generate identical QR payloads.
- Address labels/history visible in both.
- Address book behaves consistently.

---

## Step 9 — Testing checklist

Run after every step:

```bash
npm run typecheck
cargo check
npx tauri dev
```

Manual checks:

1. Open Receive page.
2. Generate address.
3. Copy address.
4. Copy Bitcoin URI.
5. QR code scans to same URI.
6. Generate new address.
7. Confirm history updates.
8. Confirm no Tauri arg errors.
9. Confirm no network field assumptions.
10. Confirm Send page still works.

---

## Known pitfalls from previous session

### Do not use localStorage for long-term wallet data

Avoid storing these in frontend localStorage:

```txt
receive history
receive labels
address book
```

These belong in backend wallet storage.

Frontend local state is acceptable only for temporary UI state:

```txt
copy feedback
form inputs
loading indicators
selected tabs
```

### Do not add network to Receive DTO

Correct:

```ts
address
keychain
index
```

Incorrect:

```ts
network
```

### Tauri invoke arg must be camelCase

Correct:

```ts
{ walletName }
```

Incorrect:

```ts
{ name }
{ wallet_name }
```

### Tauri command is now registered

Already added:

```rust
commands::wallet::get_receive_address
```

### Backend API method exists

Use:

```rust
api.address(&walletName).await
```

### Keep Receive feature separate from Send

Receive should not import from Send feature unless intentionally sharing generic helpers.

---

## Recommended next first task

Start with backend Bitcoin URI + QR generation.

Do not implement address book yet.

A good first mini-goal:

> Extend `WalletReceiveAddressDto` with `bitcoin_uri` and `qr_svg`, generate both in Rust backend helpers, return them through Tauri, and render backend-generated SVG in `ReceiveAddressCard`.

After that:

1. Add CLI QR output.
2. Add backend receive history persistence.
3. Add backend receive labels.
4. Add backend address book.
5. Integrate address book into Send/Sweep/Send Max.
