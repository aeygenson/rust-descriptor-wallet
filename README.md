# 🦀 Rust Descriptor Wallet (BDK + Tauri)

![Rust](https://img.shields.io/badge/Rust-1.75+-orange)
![BDK](https://img.shields.io/badge/BDK-2.x-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Status](https://img.shields.io/badge/status-in--progress-yellow)

A modern **Bitcoin descriptor-based wallet** built in **Rust**, powered by **BDK (Bitcoin Dev Kit)** with a **Tauri desktop UI**.

---

## 🚀 Overview

This project demonstrates a **production-style Bitcoin wallet architecture**:

- Descriptor wallets (`wpkh`, `tr`)
- UTXO model
- PSBT workflow
- Coin selection
- Modular Rust design

Built as both:

- 📚 Learning project  
- 💼 Interview showcase  
- 🧠 Deep dive into Bitcoin internals  

---

## 🏗 Architecture

Tauri UI (desktop)
│
├── wallet_api        → command layer (CLI / UI)
├── wallet_core       → BDK wallet logic
├── wallet_sync       → blockchain sync (Esplora)
├── wallet_storage    → persistence (SQLite/file)
└── signer            → signing (future HW wallet)

---

## ⚙️ Features

### ✅ MVP

- Descriptor wallet (`wpkh`)
- Address generation
- External/internal chains (`/0/*`, `/1/*`)
- Sync via Esplora (Signet)
- UTXO tracking
- Balance calculation
- Transaction builder
- PSBT creation & signing

---

### 🔜 Planned

- 🖥 Tauri UI
- 👁 Watch-only wallets
- 🌳 Taproot (`tr`)
- 🔐 Hardware wallet support
- 🔄 RBF (fee bump)
- 🧾 PSBT import/export
- 🧠 Miniscript policies

---

## 🧠 Concepts Demonstrated

### Bitcoin
- UTXO model
- Transaction lifecycle
- Fee calculation

### Wallet Design
- Descriptor wallets
- HD wallets (BIP32/84)
- Change addresses
- Gap limit scanning

### Advanced
- PSBT workflow
- Coin selection
- Watch-only vs signing

---

## 🛠 Tech Stack

- 🦀 Rust
- 🧱 BDK (`bdk_wallet`, `bdk_esplora`)
- ⚡ Tauri
- 🗄 SQLite / file storage
- 🌐 Esplora API

---

## 📦 Project Structure

rust-descriptor-wallet/
├── crates/
│   ├── wallet_core/
│   ├── wallet_sync/
│   ├── wallet_storage/
│   └── wallet_api/
├── apps/
│   ├── wallet_cli/
│   └── wallet_desktop/

---

## ▶️ Getting Started

### Clone

git clone https://github.com/aeygenson/rust-descriptor-wallet.git
cd rust-descriptor-wallet

### Build

cargo build

### Run CLI

cargo run -p wallet_cli

---

## 🧪 Example Usage (planned)

wallet address
wallet sync
wallet balance
wallet utxos
wallet send --to bc1... --amount 0.01

---

## 🌐 Network

Default: Signet

https://blockstream.info/signet/api/

---

## 🔐 Security Model

- Descriptor-based architecture
- PSBT signing workflow
- Separation of:
  - wallet logic
  - UI
  - signing
- Hardware wallet–ready design

---

## 🧪 Example Descriptor

External:

wpkh([fingerprint/84'/1'/0']tpub.../0/*)

Internal:

wpkh([fingerprint/84'/1'/0']tpub.../1/*)

---

## 🧭 Roadmap

Phase 1: Wallet core, CLI, Sync  
Phase 2: Transactions + PSBT  
Phase 3: Persistence  
Phase 4: Tauri UI  
Phase 5: Taproot + hardware wallet  

---

## 🎯 Why This Project Matters

This project shows:

- Real-world Bitcoin wallet architecture
- Strong Rust engineering practices
- Clean modular system design
- Deep understanding of:
  - descriptors
  - PSBT
  - UTXO model

---

## 📸 Demo

Coming soon (Tauri UI screenshots)

---

## 👤 Author

Alex Eygenson  
Rust | Finance | Trading Systems  

---

## ⭐ Support

If you find this useful:

Star the repo  
Follow development
