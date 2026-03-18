# 🦀 Rust Descriptor Wallet (BDK + Tauri)

A modern **Bitcoin descriptor-based wallet** built in **Rust**, powered by **BDK (Bitcoin Dev Kit)** and designed with a **Tauri desktop UI**.

This project demonstrates a **production-style wallet architecture** with:

- Descriptor wallets (`wpkh`, `tr`)
- UTXO-based accounting
- PSBT workflow
- Coin selection & transaction building
- Hardware wallet–compatible design
- Clean Rust modular architecture

---

## 🚀 Project Goals

This project is built as:

- 📚 **Learning platform** for Bitcoin wallet internals
- 🧠 **Deep dive into BDK + descriptors + PSBT**
- 💼 **Interview-ready system design project**
- 🧩 **Extensible wallet engine for future use**

---

## 🏗 Architecture
Tauri UI (desktop)
│
├── wallet_api (commands)
│
├── wallet_core (BDK-based engine)
│     ├── descriptor handling
│     ├── address derivation
│     ├── UTXO tracking
│     ├── transaction builder
│     └── PSBT flow
│
├── wallet_sync
│     └── Esplora client (Signet/Testnet/Mainnet)
│
├── wallet_storage
│     └── local persistence (file / SQLite)
│
└── signer
└── software signer (future: hardware wallet)
---

## ⚙️ Features

### ✅ Implemented (MVP)

- Descriptor-based wallet (external + internal)
- Address generation (`wpkh`)
- Blockchain sync via Esplora (Signet)
- UTXO tracking
- Balance calculation
- Transaction building
- PSBT creation & signing (software signer)

---

### 🔜 Planned

- Tauri desktop GUI
- Watch-only wallet mode
- Taproot (`tr()`) support
- PSBT export/import
- Hardware wallet integration
- Fee bump (RBF)
- Transaction history view
- Multi-account support

---

## 🧠 Key Concepts Implemented

This project demonstrates real-world Bitcoin wallet concepts:

- **Descriptors**
  - `wpkh(xpub.../0/*)`
  - `tr(xpub...)`
- **UTXO Model**
- **Coin Selection**
- **PSBT Workflow**
- **Change Addresses (`/1/*`)**
- **Gap Limit Scanning**
- **Watch-only vs Signing Wallets**

---

## 🛠 Tech Stack

- 🦀 Rust
- 🧱 BDK (`bdk_wallet`, `bdk_esplora`)
- ⚡ Tauri (desktop app)
- 🗄 SQLite / File Store
- 🌐 Esplora API (Blockstream / self-hosted)

---

## 📦 Project Structure

bitcoin-wallet-app/
├── Cargo.toml
├── crates/
│   ├── wallet_core/
│   ├── wallet_sync/
│   ├── wallet_storage/
│   └── wallet_api/
├── apps/
│   ├── wallet_cli/
│   └── wallet_desktop/   # Tauri
