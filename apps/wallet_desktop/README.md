# Wallet Desktop

`apps/wallet_desktop` is the first shipped desktop UI for this repository.

It is a two-part app:

- a React + TypeScript + Vite frontend under [src](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src)
- a Tauri/Rust host under [src-tauri](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src-tauri)

The desktop app sits on top of `wallet_api` and exposes the same wallet flows already covered by the CLI and regtest suite.

## Current Scope

Implemented screens:

- Overview
- UTXOs
- Send
- Transactions

Implemented flows:

- wallet selection and wallet status loading
- backend health inspection
- UTXO inspection and multi-select handoff into send flows
- fixed send PSBT preview
- fixed send with coin control
- send-max preview
- send-max with coin control
- sweep preview
- consolidation preview
- PSBT sign and publish
- transaction history inspection
- RBF PSBT workflow
- CPFP PSBT workflow

Not implemented yet:

- dedicated Receive screen
- settings/configuration screen
- production packaging guidance beyond local Tauri build

## Commands

Frontend scripts:

```bash
npm run dev
npm run build
npm run lint
npm run test
npm run tauri:dev
npm run tauri:build
```

Rust-side smoke test:

```bash
cargo test -p wallet_desktop_tauri
```

## Documentation

Desktop docs live under [docs](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/docs):

- [overview](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/docs/overview.md)
- [UI architecture](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/docs/ui-architecture.md)
- [screen map](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/docs/screen-map.md)
- [command surface](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/docs/command-surface.md)
