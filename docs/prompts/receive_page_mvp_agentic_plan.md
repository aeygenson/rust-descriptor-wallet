# Receive Page MVP — Agentic Implementation Plan

## Goal

Add a first working **Receive** page to the Tauri desktop wallet.

The MVP should let the user:

1. Open a new `Receive` page from the sidebar.
2. Generate or fetch the next wallet receive address.
3. Display the address in a clean card.
4. Copy the address to clipboard.
5. Show basic address metadata if available.
6. Keep frontend/Tauri argument naming aligned with the current canonical desktop convention.

This is the next feature after the UTXO page cleanup.

---

## Current routing context

Current `routes.ts`:

```ts
export const routes = {
  overview: "/",
  utxos: "/utxos",
  send: "/send",
  transactions: "/transactions",
} as const;
```

Need to add:

```ts
receive: "/receive",
```

Navigation should include:

```ts
{ id: "receive", label: "Receive", path: routes.receive },
```

Recommended order:

```ts
Overview
Receive
UTXOs
Send
Transactions
```

---

## Important naming rule

For Tauri command arguments, use **camelCase** frontend keys.

Use:

```ts
{ walletName }
```

Do **not** use:

```ts
{ name: walletName }
{ wallet_name: walletName }
```

Recent bugs came from Tauri errors like:

```txt
invalid args `walletName` ... missing required key walletName
```

So every direct invoke or `invokeCommand` call for wallet-scoped Tauri commands should pass `walletName`.

---

## Step 1 — Add route and navigation

File:

```txt
src/routes.ts
```

Update:

```ts
export const routes = {
  overview: "/",
  receive: "/receive",
  utxos: "/utxos",
  send: "/send",
  transactions: "/transactions",
} as const;
```

Update navigation:

```ts
export const navigationItems: NavigationItem[] = [
  { id: "overview", label: "Overview", path: routes.overview },
  { id: "receive", label: "Receive", path: routes.receive },
  { id: "utxos", label: "UTXOs", path: routes.utxos },
  { id: "send", label: "Send", path: routes.send },
  { id: "transactions", label: "Transactions", path: routes.transactions },
];
```

Acceptance:

- TypeScript accepts new route id.
- Sidebar can render Receive link without type errors.

---

## Step 2 — Wire page route

Find the app/router file. Likely one of:

```txt
src/App.tsx
src/router.tsx
src/main.tsx
```

Add import:

```ts
import { ReceivePage } from "./pages/ReceivePage";
```

Add route mapping:

```tsx
<Route path={routes.receive} element={<ReceivePage />} />
```

or match your existing router pattern.

Acceptance:

- Clicking Receive opens a new page.
- No blank page.
- No route type errors.

---

## Step 3 — Create receive feature folder

Create:

```txt
src/features/receive/
  api.ts
  types.ts
  format.ts
  components/
    ReceiveAddressCard.tsx
    ReceiveEmptyState.tsx
```

Keep the MVP small. Do not add address book yet.

---

## Step 4 — Define receive types

File:

```txt
src/features/receive/types.ts
```

Use canonical shared DTO if already available.

Likely DTO:

```ts
import type { WalletReceiveAddressDto } from "../../shared/types/dtos";

export type ReceiveAddressResult = WalletReceiveAddressDto;
```

Component props:

```ts
import type { WalletReceiveAddressDto } from "../../shared/types/dtos";

export type ReceiveAddressCardProps = {
  address: WalletReceiveAddressDto;
  copied: boolean;
  onCopy: () => void;
};

export type ReceiveEmptyStateProps = {
  loading: boolean;
  error: string | null;
  onGenerate: () => void;
};
```

Important: do not invent fields not present in `WalletReceiveAddressDto`.

Check actual DTO fields in:

```txt
src/shared/types/dtos.ts
```

or Rust source of truth:

```txt
crates/wallet_api/src/model.rs
```

Expected possible fields may include:

```ts
address
network
keychain
index
derivation_path
```

Only render fields that exist.

---

## Step 5 — Create receive API

File:

```txt
src/features/receive/api.ts
```

Pattern should match existing frontend API modules.

Example:

```ts
import type { WalletReceiveAddressDto } from "../../shared/types/dtos";
import { invokeCommand } from "../../shared/tauri";

export async function getReceiveAddress(
  walletName: string,
): Promise<WalletReceiveAddressDto> {
  return invokeCommand<WalletReceiveAddressDto>("get_receive_address", {
    walletName,
  });
}
```

If the actual command is called differently, inspect Tauri command registration in:

```txt
apps/wallet_desktop/src-tauri/src/commands/
apps/wallet_desktop/src-tauri/src/lib.rs
```

Possible command names:

```txt
get_receive_address
receive_address
wallet_address
get_wallet_address
```

Do not guess permanently. Match the registered command.

Acceptance:

- API function compiles.
- Tauri call sends `{ walletName }`.

---

## Step 6 — Check or add Tauri receive command

Inspect:

```txt
apps/wallet_desktop/src-tauri/src/commands/wallet.rs
```

Look for an existing address command.

If command does not exist, add one using the existing `WalletApi` method.

Expected style:

```rust
#[allow(non_snake_case)]
#[command]
pub async fn get_receive_address(
    api: State<'_, WalletApi>,
    walletName: String,
) -> Result<WalletReceiveAddressDto, String> {
    api.address(&walletName)
        .await
        .map_err(|err| err.to_string())
}
```

If API now uses canonical request DTO instead of string, use the real method signature from `wallet_api`.

Possible canonical method pattern may be:

```rust
api.address(WalletAddressRequestDto { name: walletName })
```

or:

```rust
api.address_from_request(WalletAddressRequestDto { name: walletName })
```

But earlier CLI migration removed old `*_from_request` methods, so prefer the current canonical API method names.

After adding command, register it in the Tauri command handler where other commands are registered.

Acceptance:

```bash
cargo check -p wallet_desktop_tauri
```

passes.

---

## Step 7 — Create ReceiveAddressCard component

File:

```txt
src/features/receive/components/ReceiveAddressCard.tsx
```

Suggested implementation shape:

```tsx
import type { ReceiveAddressCardProps } from "../types";

export function ReceiveAddressCard({
  address,
  copied,
  onCopy,
}: ReceiveAddressCardProps) {
  return (
    <section className="receive-address-card">
      <div className="receive-address-card__header">
        <div>
          <h2>Receive address</h2>
          <p>Share this address to receive funds into the selected wallet.</p>
        </div>

        <button type="button" onClick={onCopy}>
          {copied ? "Copied" : "Copy address"}
        </button>
      </div>

      <div className="receive-address-card__qr">
        <span>QR</span>
        <small>QR code can be added next</small>
      </div>

      <div className="receive-address-card__address">
        <code title={address.address}>{address.address}</code>
      </div>

      <div className="receive-address-card__meta">
        {/* Render only fields that exist on WalletReceiveAddressDto */}
      </div>
    </section>
  );
}
```

Do not add QR dependency in MVP unless already installed.

---

## Step 8 — Create ReceiveEmptyState component

File:

```txt
src/features/receive/components/ReceiveEmptyState.tsx
```

Suggested:

```tsx
import type { ReceiveEmptyStateProps } from "../types";

export function ReceiveEmptyState({
  loading,
  error,
  onGenerate,
}: ReceiveEmptyStateProps) {
  return (
    <section className="receive-empty">
      <div className="receive-empty__icon" aria-hidden="true">
        ₿
      </div>

      <div>
        <h2>Generate a receive address</h2>
        <p>
          Create the next wallet-controlled address and share it with the sender.
        </p>
      </div>

      {error && (
        <div className="receive-empty__error" role="alert">
          {error}
        </div>
      )}

      <button type="button" disabled={loading} onClick={onGenerate}>
        {loading ? "Generating…" : "Generate receive address"}
      </button>
    </section>
  );
}
```

---

## Step 9 — Create ReceivePage

File:

```txt
src/pages/ReceivePage.tsx
```

Use existing wallet selection context/pattern from other pages. Check pages like:

```txt
src/pages/UtxosPage.tsx
src/pages/SendPage.tsx
src/pages/TransactionsPage.tsx
```

Expected state:

```ts
const [address, setAddress] = useState<WalletReceiveAddressDto | null>(null);
const [loading, setLoading] = useState(false);
const [error, setError] = useState<string | null>(null);
const [copied, setCopied] = useState(false);
```

Generate handler:

```ts
const handleGenerate = async () => {
  if (!selectedWalletName) return;

  try {
    setLoading(true);
    setError(null);
    setCopied(false);

    const result = await getReceiveAddress(selectedWalletName);
    setAddress(result);
  } catch (error) {
    setError(error instanceof Error ? error.message : String(error));
  } finally {
    setLoading(false);
  }
};
```

Copy handler:

```ts
const handleCopy = async () => {
  if (!address?.address) return;

  await navigator.clipboard.writeText(address.address);
  setCopied(true);
};
```

Page JSX:

```tsx
<section className="receive-page">
  <header className="receive-page__header">
    <div>
      <h1 className="receive-page__title">Receive</h1>
      <p className="receive-page__subtitle">
        Generate wallet-controlled receive addresses for the active wallet.
      </p>
    </div>

    <div className="receive-wallet-pill">
      {selectedWalletName}
    </div>
  </header>

  {address ? (
    <ReceiveAddressCard
      address={address}
      copied={copied}
      onCopy={handleCopy}
    />
  ) : (
    <ReceiveEmptyState
      loading={loading}
      error={error}
      onGenerate={handleGenerate}
    />
  )}
</section>
```

Add a secondary button on address card later:

```tsx
Generate another address
```

For MVP, card can include it if useful.

---

## Step 10 — Add receive CSS

Create:

```txt
src/styles/receive.css
```

Initial style:

```css
.receive-page {
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-width: 0;
  padding: 24px;
}

.receive-page__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  min-width: 0;
}

.receive-page__title {
  margin: 0;
  color: #f8fafc;
  font-size: 32px;
  font-weight: 900;
  letter-spacing: -0.03em;
}

.receive-page__subtitle {
  max-width: 760px;
  margin: 8px 0 0;
  color: #94a3b8;
  font-size: 15px;
  line-height: 1.55;
}

.receive-wallet-pill {
  width: fit-content;
  max-width: 360px;
  overflow: hidden;
  padding: 8px 12px;
  border: 1px solid rgba(59, 130, 246, 0.28);
  border-radius: 999px;
  background: rgba(59, 130, 246, 0.15);
  color: #bfdbfe;
  font-size: 12px;
  font-weight: 850;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.receive-empty,
.receive-address-card {
  display: grid;
  gap: 16px;
  min-width: 0;
  padding: 22px;
  border: 1px solid #243244;
  border-radius: 18px;
  background: linear-gradient(180deg, rgba(15, 23, 42, 0.92), rgba(2, 6, 23, 0.78));
  box-shadow:
    0 18px 44px rgba(0, 0, 0, 0.18),
    inset 0 1px 0 rgba(255, 255, 255, 0.03);
}

.receive-empty__icon {
  display: grid;
  place-items: center;
  width: 48px;
  height: 48px;
  border: 1px solid rgba(96, 165, 250, 0.32);
  border-radius: 16px;
  background: rgba(59, 130, 246, 0.14);
  color: #bfdbfe;
  font-size: 24px;
  font-weight: 900;
}

.receive-empty h2,
.receive-address-card h2 {
  margin: 0;
  color: #f8fafc;
  font-size: 22px;
}

.receive-empty p,
.receive-address-card p {
  margin: 6px 0 0;
  color: #94a3b8;
  line-height: 1.55;
}

.receive-empty button,
.receive-address-card button {
  width: fit-content;
  min-height: 40px;
  padding: 0 14px;
  border: 1px solid transparent;
  border-radius: 12px;
  background: #2563eb;
  color: #ffffff;
  cursor: pointer;
  font-weight: 850;
}

.receive-empty button:hover:not(:disabled),
.receive-address-card button:hover:not(:disabled) {
  background: #3b82f6;
}

.receive-empty__error {
  padding: 10px 12px;
  border: 1px solid rgba(248, 113, 113, 0.32);
  border-radius: 12px;
  background: rgba(127, 29, 29, 0.25);
  color: #fca5a5;
  font-size: 13px;
  font-weight: 700;
}

.receive-address-card__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.receive-address-card__qr {
  display: grid;
  place-items: center;
  min-height: 220px;
  border: 1px dashed #334155;
  border-radius: 18px;
  background: rgba(2, 6, 23, 0.52);
  color: #64748b;
  text-align: center;
}

.receive-address-card__qr span {
  color: #bfdbfe;
  font-size: 42px;
  font-weight: 900;
}

.receive-address-card__address {
  min-width: 0;
  padding: 14px;
  border: 1px solid #334155;
  border-radius: 14px;
  background: rgba(2, 6, 23, 0.72);
}

.receive-address-card__address code {
  display: block;
  overflow-wrap: anywhere;
  color: #dbeafe;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 14px;
  line-height: 1.55;
}

.receive-address-card__meta {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.receive-address-card__meta-item {
  min-width: 0;
  padding: 12px;
  border: 1px solid #243244;
  border-radius: 14px;
  background: rgba(15, 23, 42, 0.72);
}

.receive-address-card__meta-label {
  color: #64748b;
  font-size: 11px;
  font-weight: 850;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.receive-address-card__meta-value {
  margin-top: 5px;
  overflow: hidden;
  color: #e5e7eb;
  font-size: 13px;
  font-weight: 800;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 760px) {
  .receive-page {
    padding: 16px;
    padding-bottom: 72px;
  }

  .receive-page__header,
  .receive-address-card__header {
    flex-direction: column;
    align-items: stretch;
  }

  .receive-wallet-pill,
  .receive-empty button,
  .receive-address-card button {
    width: 100%;
  }

  .receive-address-card__meta {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 520px) {
  .receive-page {
    padding: 12px;
    padding-bottom: 72px;
  }

  .receive-page__title {
    font-size: 26px;
  }

  .receive-empty,
  .receive-address-card {
    padding: 16px;
    border-radius: 16px;
  }
}
```

Import it in your central stylesheet entry, likely:

```txt
src/styles/index.css
```

or similar:

```css
@import "./receive.css";
```

---

## Step 11 — Add optional metadata rendering helper

In `ReceiveAddressCard.tsx`, add small helper component:

```tsx
function MetaItem({ label, value }: { label: string; value?: string | number | null }) {
  if (value === null || value === undefined || value === "") {
    return null;
  }

  return (
    <div className="receive-address-card__meta-item">
      <div className="receive-address-card__meta-label">{label}</div>
      <div className="receive-address-card__meta-value" title={String(value)}>
        {value}
      </div>
    </div>
  );
}
```

Use only fields that exist on the DTO.

Example:

```tsx
<div className="receive-address-card__meta">
  <MetaItem label="Network" value={address.network} />
  <MetaItem label="Keychain" value={address.keychain} />
  <MetaItem label="Index" value={address.index} />
</div>
```

Adjust names based on actual DTO.

---

## Step 12 — Typecheck

Run:

```bash
npm run typecheck
```

If no typecheck script exists:

```bash
npx tsc --noEmit
```

Then:

```bash
cargo check -p wallet_desktop_tauri
```

---

## Step 13 — Runtime test

Run:

```bash
npx tauri dev
```

Manual test:

1. Open app.
2. Select `regtest-local`.
3. Click Receive in sidebar.
4. Click Generate receive address.
5. Confirm card appears.
6. Copy address.
7. Switch wallet.
8. Generate again.
9. Confirm no Tauri arg error.

---

## Step 14 — Acceptance criteria

MVP is done when:

- Receive appears in navigation.
- `/receive` route works.
- Page uses active wallet.
- Generate button returns a wallet receive address.
- Address displays in a polished card.
- Copy button works.
- Loading and error states render.
- No Tauri argument mismatch.
- `npm run typecheck` passes.
- `cargo check -p wallet_desktop_tauri` passes.

---

## Step 15 — Next after MVP

After MVP works, add:

1. Real QR code rendering.
2. Address history table.
3. Address labels.
4. Address book / recipient book.
5. Persist address labels in storage.
6. Send page integration: choose saved recipient from address book.
7. Duplicate detection and validation.
