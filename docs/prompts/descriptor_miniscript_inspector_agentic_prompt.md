# Agentic Implementation Prompt: Descriptor / Miniscript Inspector GUI

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
- Receive page planned
- UTXO page
- Transactions page
- Coin control
- Send fixed / send max / sweep / consolidation
- RBF
- CPFP
- Transaction lineage
- Backend health
- PSBT lifecycle

Current frontend architecture pattern:

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

## Lessons learned from prior UTXO feature implementation

These implementation rules are mandatory.

### 1. Always update shared DTOs first

When backend models evolve:

```text
wallet_api DTO
→ shared/types/dtos.ts
→ feature/types.ts
→ component props
→ pages
```

Do not update components before shared DTOs compile.

Typical failure pattern:

```text
TS2739 missing props
TS2339 property does not exist
```

Root cause is usually stale shared DTOs.

---

### 2. Update frontend prop interfaces together with components

Whenever adding new component props:

```text
component implementation
AND
feature/types.ts interfaces
```

must be updated in the same step.

Do not defer prop interface updates.

Common failures:

```text
Property 'emptyVariant' does not exist
Missing hasLockedSelection
Missing lockedCount
```

---

### 3. Never hallucinate existing helpers or functions

Before adding imports:

```text
Read actual helpers file first.
```

Do not invent:

```text
setup_test_wallet
fund_test_wallet
helper modules
```

Always mirror existing project patterns.

---

### 4. Prefer extending existing architecture instead of parallel systems

If wallet already has:

```text
wallet.rs
utxos.rs
send_model.rs
shared DTOs
summary cards
state views
```

extend those instead of creating duplicate flows.

Correct pattern:

```text
small targeted extensions
```

Avoid:

```text
new competing abstractions
```

---

### 5. Keep orchestration only in pages

Pages may:

```text
load data
coordinate actions
handle refresh
compose components
```

Pages must NOT:

```text
format descriptors
parse miniscript
infer business rules
```

Use:

```text
lib.ts for pure logic
format.ts for presentation
api.ts for backend calls
```

strictly.

---

### 6. Keep Tauri commands extremely thin

Correct:

```text
Tauri command
→ wallet_api service
→ wallet_core/storage
```

Wrong:

```text
descriptor parsing in Tauri
policy inference in Tauri
business logic in commands
```

---

### 7. Add lock/state awareness consistently across all UI layers

When introducing a new state:

```text
locked
frozen
watch-only
offline
hardware-required
```

update ALL:

```text
DTOs
summary models
filters
selection logic
header badges
empty states
summary cards
CSS states
button disabling
navigation guards
```

Do not partially implement state handling.

---

### 8. Selection logic must remain workflow-aware

Important UX lesson:

```text
Locked UTXOs still needed to remain selectable
for unlock workflows.
```

Therefore:

```text
select-all may exclude locked items
while manual selection still allows them.
```

Apply same principle to descriptor workflows.

---

### 9. Use defensive backend-driven architecture

Preferred architecture:

```text
backend enforcement
+ frontend UX guidance
```

Never rely only on frontend filtering.

Example:

```text
backend rejects locked UTXO spending
frontend disables spend buttons additionally
```

Descriptor privacy enforcement must follow same rule.

---

### 10. Add compile checkpoints during implementation

After every major phase run:

```bash
cargo check
npm run typecheck
```

Do not implement the entire feature before compiling.

Recommended checkpoints:

```text
DTO phase
API phase
types phase
component phase
page orchestration phase
```

---

### 11. CSS and UI states are part of the feature, not polish

When adding a new feature state:

```text
locked
warning
redacted
unsafe
watch-only
```

implement:

```text
CSS classes
hover states
disabled states
badges
summary indicators
empty states
```

in the same implementation phase.

---

### 12. Security-sensitive features require explicit redaction rules

Never assume frontend data is safe.

Always add:

```text
redaction helpers
safe copy behavior
warning banners
```

before rendering potentially sensitive material.

For descriptors:

```text
xprv
seed
mnemonic
private derivation material
```

must never render.

---

### 13. Prefer incremental patches over massive rewrites

Successful pattern:

```text
small targeted updates
```

Avoid:

```text
rewriting entire pages
rewriting entire feature architecture
```

This project evolves feature-by-feature.

---

### 14. Read existing files before proposing changes

Mandatory rule:

```text
Read actual file
→ inspect existing patterns
→ then update
```

Do not infer architecture from memory.

Especially important for:

```text
Tauri commands
feature/types.ts
summary models
selection helpers
```

---

### 15. Feature completeness checklist

A feature is NOT complete until all are updated:

```text
Backend DTOs
Tauri registration
shared frontend DTOs
feature types
API wrappers
pure helpers
formatters
components
page orchestration
routing/sidebar
CSS
loading/error states
empty states
selection behavior
summary cards
button disabled states
```

Use this checklist explicitly during implementation.

---

## Goal

Implement a professional **Descriptor / Miniscript Inspector** in the GUI.

The user should be able to:

```text
Open Inspector page
→ see selected wallet descriptor metadata
→ view external/internal descriptors
→ inspect script type
→ inspect network and watch-only status
→ inspect fingerprints / derivation paths when available
→ understand spending policy / miniscript structure when available
→ copy descriptors safely
```

This feature is high interview value because it exposes the architecture of a descriptor wallet instead of only showing balances and transactions.

---

## Non-goals for v1

Do NOT implement:

- descriptor editing
- descriptor import wizard
- descriptor mutation
- private key / xprv display
- signing policy editor
- full miniscript compiler UI
- hardware wallet integration
- policy satisfier simulation
- spending path simulator

This is an **inspector**, not an editor.

---

## Security rules

Never display secrets.

Do not show:

```text
xprv
private keys
seed
mnemonic
raw signer secrets
```

If backend returns anything private, redact it before rendering.

Add a safety helper:

```ts
redactDescriptorSecrets(descriptor: string): string
```

Prefer backend to expose public descriptors only.

If unsure whether descriptor contains private material, redact suspicious tokens.

---

## Source of truth checks first

Inspect these files before coding:

```text
crates/wallet_api/src/model.rs
crates/wallet_api/src/service/wallet.rs
crates/wallet_core/src/service/*
crates/wallet_storage/src/*
apps/wallet_desktop/src-tauri/src/commands/wallet.rs
apps/wallet_desktop/src-tauri/src/commands/wallet_model.rs
apps/wallet_desktop/src/features/wallet/api.ts
apps/wallet_desktop/src/shared/types/dtos.ts
apps/wallet_desktop/src/app/router or App.tsx
apps/wallet_desktop/src/app/shell/sidebar files
```

Search for:

```text
descriptor
descriptors
miniscript
policy
xpub
fingerprint
derivation
watch_only
script_type
wallet details
```

Do not guess existing DTO names. Mirror current Rust/API names exactly.

---

## Backend / DTO target

If an existing wallet details DTO already includes descriptors, use it.

If not, add a wallet inspector DTO in `wallet_api/src/model.rs`.

Possible DTO shape:

```rust
pub struct WalletDescriptorInfoDto {
    pub wallet_name: String,
    pub network: String,
    pub watch_only: bool,
    pub external_descriptor: Option<String>,
    pub internal_descriptor: Option<String>,
    pub descriptor_type: Option<String>,
    pub script_type: Option<String>,
    pub fingerprint: Option<String>,
    pub derivation_path: Option<String>,
    pub policy_summary: Option<String>,
}
```

If backend can expose richer information:

```rust
pub struct WalletDescriptorKeyDto {
    pub fingerprint: Option<String>,
    pub derivation_path: Option<String>,
    pub origin: Option<String>,
    pub xpub: Option<String>,
    pub keychain: Option<String>,
}
```

But do not overbuild v1.

---

## Tauri command

If needed, add command:

```rust
#[tauri::command]
pub async fn wallet_descriptor_info(
    state: tauri::State<'_, AppState>,
    request: WalletDescriptorInfoRequest,
) -> Result<WalletDescriptorInfoDto, String>
```

Request:

```rust
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletDescriptorInfoRequest {
    pub wallet_name: String,
}
```

Keep command thin:

```text
Tauri command → wallet_api service → wallet_core/storage
```

Do not parse descriptor deeply in Tauri layer.

---

## Frontend file structure

Create:

```text
apps/wallet_desktop/src/features/descriptor/api.ts
apps/wallet_desktop/src/features/descriptor/types.ts
apps/wallet_desktop/src/features/descriptor/lib.ts
apps/wallet_desktop/src/features/descriptor/format.ts
apps/wallet_desktop/src/features/descriptor/descriptor.css
apps/wallet_desktop/src/features/descriptor/components/DescriptorHeader.tsx
apps/wallet_desktop/src/features/descriptor/components/DescriptorSummaryCards.tsx
apps/wallet_desktop/src/features/descriptor/components/DescriptorViewer.tsx
apps/wallet_desktop/src/features/descriptor/components/DescriptorKeyList.tsx
apps/wallet_desktop/src/features/descriptor/components/DescriptorPolicyPanel.tsx
apps/wallet_desktop/src/pages/DescriptorInspectorPage.tsx
```

Only create components actually used in v1.

---

## Frontend DTO

In `shared/types/dtos.ts`, mirror backend DTO.

Example:

```ts
export type WalletDescriptorInfoDto = {
  wallet_name: string;
  network: string;
  watch_only: boolean;
  external_descriptor?: string | null;
  internal_descriptor?: string | null;
  descriptor_type?: string | null;
  script_type?: string | null;
  fingerprint?: string | null;
  derivation_path?: string | null;
  policy_summary?: string | null;
};
```

Match actual serde field style used by the project.

---

## `features/descriptor/api.ts`

Only backend calls.

Example:

```ts
export async function getWalletDescriptorInfo(
  walletName: string
): Promise<WalletDescriptorInfoDto>
```

Use existing invoke wrapper pattern from wallet/send/transactions features.

Do not put parsing or formatting here.

---

## `features/descriptor/types.ts`

Component props and frontend types.

Examples:

```ts
export type DescriptorKeychain = "external" | "internal";

export type DescriptorViewerProps = {
  title: string;
  descriptor?: string | null;
  keychain: DescriptorKeychain;
  onCopy: (value: string) => void;
};

export type DescriptorSummaryCardsProps = {
  info: WalletDescriptorInfoDto;
};

export type DescriptorPolicyPanelProps = {
  info: WalletDescriptorInfoDto;
};

export type DescriptorHeaderProps = {
  walletName: string | null;
  loading?: boolean;
  onRefresh: () => void;
};
```

No local component prop types.

---

## `features/descriptor/lib.ts`

Pure helpers only.

Add helpers such as:

```ts
export function redactDescriptorSecrets(descriptor: string): string

export function hasDescriptorSecrets(descriptor: string): boolean

export function getDescriptorKeychainLabel(keychain: DescriptorKeychain): string

export function getDescriptorDisplaySections(info: WalletDescriptorInfoDto): ...

export function normalizeDescriptorText(descriptor?: string | null): string | null

export function inferDescriptorScriptType(descriptor?: string | null): string | null
```

Possible simple inference:

```text
wpkh(...)  → Native SegWit single-sig
tr(...)    → Taproot
sh(wpkh)   → Nested SegWit
wsh(...)   → Native SegWit script
sh(wsh)    → Nested script
```

Do not overpromise exact miniscript semantics unless backend provides real parsing.

---

## `features/descriptor/format.ts`

Presentation only.

Examples:

```ts
export function formatDescriptorType(value?: string | null): string
export function formatScriptType(value?: string | null): string
export function formatWatchOnly(value: boolean): string
export function formatFingerprint(value?: string | null): string
export function formatDerivationPath(value?: string | null): string
export function shortDescriptor(value: string): string
```

No business logic here.

---

## Page behavior

`DescriptorInspectorPage.tsx` should:

- read `selectedWalletName` from wallet provider
- load descriptor info when wallet changes
- show loading/error/empty states
- render summary cards
- render external/internal descriptors
- provide copy buttons
- show policy/miniscript summary if available
- never display secrets
- provide refresh button

No formatting helpers inside page.

---

## UI design

Suggested layout:

```text
Descriptor Inspector
Wallet: regtest-local
Network: Regtest
Watch-only: No

[ Script Type ] [ Descriptor Type ] [ Watch-only ] [ Network ]

External Descriptor
[ monospace descriptor block ]
[ Copy ]

Internal / Change Descriptor
[ monospace descriptor block ]
[ Copy ]

Policy / Miniscript
- Type: ...
- Keys: ...
- Derivation: ...
- Notes: ...
```

Use dark UI matching existing app.

Suggested classes:

```css
.descriptor-page
.descriptor-grid
.descriptor-card
.descriptor-summary
.descriptor-viewer
.descriptor-viewer__code
.descriptor-copy-button
.descriptor-warning
.descriptor-policy
.descriptor-key-list
```

---

## Copy behavior

Use:

```ts
navigator.clipboard.writeText(value)
```

Show short copied state.

Never copy unredacted private descriptors.

---

## Descriptor secret redaction

In `descriptor/lib.ts`, implement defensive redaction.

Detect suspicious tokens:

```text
xprv
tprv
yprv
zprv
uprv
vprv
L...
K...
c...
```

Basic v1 redaction:

```ts
descriptor.replace(/xprv[^\])+/g, "[redacted-xprv]")
```

Better: redact by known private key prefixes.

If any private marker is detected, show warning:

```text
Descriptor contains private material and has been redacted.
```

---

## Miniscript / policy panel

If backend exposes policy/miniscript summary, render it.

If backend only exposes descriptor string, provide inferred info carefully:

```text
Inferred from descriptor prefix
```

Do not claim full miniscript analysis unless backend actually parses it.

Possible output:

```text
Descriptor prefix: wpkh
Likely script type: Native SegWit single-sig
Spending policy: single key path
```

For `tr(...)`:

```text
Likely script type: Taproot
```

For `wsh(...)`:

```text
Likely script type: Native SegWit script
```

---

## Route and sidebar

Add route:

```text
/descriptor
```

Sidebar label:

```text
Descriptor
```

Suggested sidebar order:

```text
Overview
Receive
Send
UTXOs
Transactions
Descriptor
```

or place Descriptor under advanced section if sidebar supports grouping.

---

## Backend implementation option

If backend can access descriptor from wallet config/storage, expose that through `wallet_api`.

Preferred path:

```text
wallet_storage/config → wallet_api DTO → Tauri command → frontend
```

Do not reconstruct descriptors in frontend.

Do not expose secrets.

If only public descriptor is available, that is enough.

---

## Testing checklist

### Manual UI tests

1. Open page with no wallet selected.
2. Select wallet.
3. Confirm descriptor info loads.
4. Copy external descriptor.
5. Copy internal descriptor.
6. Switch wallet.
7. Confirm page reloads for new wallet.
8. Test watch-only wallet if available.
9. Confirm no private material is displayed.
10. Confirm loading and error states work.

### Backend tests

If adding Rust command:

```bash
cargo check
```

Frontend:

```bash
npm run typecheck
npm run build
```

or project equivalent.

---

## Code quality rules

- No `any`.
- No local prop types inside components.
- No descriptor parsing in page.
- No API calls inside pure components.
- No secrets displayed.
- No formatting helpers inside components.
- No backend business logic in Tauri command.
- Match exact DTO names.
- Keep feature independent from send/utxo/transactions features.
- Use existing CSS visual language.

---

## Implementation phases

### Phase 1 — Discovery

- Search backend/frontend for existing descriptor data.
- Identify DTO or missing gap.
- Decide whether backend command is needed.

### Phase 2 — Backend/Tauri support

Only if needed:

- add DTO
- add wallet_api service method
- add Tauri request/command
- register command
- add frontend DTO

### Phase 3 — Frontend feature skeleton

Create:

```text
features/descriptor/api.ts
features/descriptor/types.ts
features/descriptor/lib.ts
features/descriptor/format.ts
components/*
pages/DescriptorInspectorPage.tsx
```

Add route/sidebar.

### Phase 4 — Summary + descriptor viewer

Implement:

- summary cards
- external descriptor
- internal descriptor
- copy buttons
- redaction

### Phase 5 — Policy/miniscript panel

Implement careful inferred display or backend-provided policy display.

### Phase 6 — Polish

- CSS
- empty/error states
- copy feedback
- privacy warning
- responsive layout

---

## Expected final result

The GUI should include:

```text
Descriptor Inspector page
Wallet/network/watch-only summary
External descriptor viewer
Internal/change descriptor viewer
Copy buttons
Secret redaction
Descriptor/script type summary
Optional miniscript/policy panel
Route + sidebar navigation
Clean loading/error states
```

This feature should make the wallet look like a serious descriptor-native Bitcoin wallet and provide strong interview/demo value.

---

## Final response expected from implementation agent

Summarize:

```text
Files created
Files updated
Backend command used/added
Descriptor fields available
Security/redaction behavior
Manual tests performed
Known limitations
```
