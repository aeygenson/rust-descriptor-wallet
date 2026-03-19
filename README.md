# 🦀 Rust Descriptor Wallet  
### BDK • Tauri • Production-Grade Architecture

![Rust](https://img.shields.io/badge/Rust-1.75+-orange)
![BDK](https://img.shields.io/badge/BDK-2.x-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Status](https://img.shields.io/badge/status-in--progress-yellow)

A modern **Bitcoin descriptor-based wallet** written in **Rust**, powered by **BDK (Bitcoin Dev Kit)** with a **Tauri desktop UI**.

---

## 🚀 Overview

This project is a **production-style Bitcoin wallet** designed to demonstrate real-world architecture:

- Descriptor wallets (`wpkh`, `tr`)
- UTXO model
- PSBT workflow (create → sign → finalize)
- Coin selection & fee handling
- Clean modular Rust design

Built as:
- 📚 Learning project  
- 💼 Interview showcase  
- 🧠 Deep dive into Bitcoin internals  

---

## 🎯 Goals

- Understand Bitcoin at system level
- Build interview-ready architecture
- Demonstrate production-grade Rust design

---

## 🏗 Architecture

### High-Level Diagram

![Architecture](docs/architecture.svg)

---

### 🔧 Components

#### 🖥 Tauri UI
- Desktop frontend
- Calls Rust commands via `wallet_api`

#### 🔌 wallet_api
- Command layer (CLI + UI bridge)
- Input validation
- Orchestration

#### 🧠 wallet_core (BDK)
- Descriptor handling
- Address derivation
- UTXO tracking
- Transaction builder
- PSBT workflow

#### 🌐 wallet_sync
- Esplora client
- Supports Signet / Testnet / Mainnet

#### 🗄 wallet_storage
- Local persistence (file / SQLite)

#### 🔐 signer
- Software signer (current)
- Hardware wallet support (planned)

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

- 🖥 Full Tauri UI
- 👁 Watch-only wallets
- 🌳 Taproot (`tr`)
- 🔐 Hardware wallet support
- 🔄 RBF (fee bumping)
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
- HD wallets (BIP32 / BIP84)
- Change address management
- Gap limit scanning

### Advanced
- PSBT workflow
- Coin selection
- Watch-only vs signing separation

---

## 📦 Project Structure

![Project Structure](docs/project-structure.svg)

---

## ▶️ Getting Started

```bash
git clone https://github.com/aeygenson/rust-descriptor-wallet.git
cd rust-descriptor-wallet
cargo build
cargo run -p wallet_cli
```

---

## 🧪 Example Usage

```bash
wallet address
wallet sync
wallet balance
wallet utxos
wallet send --to bc1... --amount 0.01
```

---

## 🌐 Network

Default: **Signet**

https://blockstream.info/signet/api/

---

## 🔐 Security Model

- Descriptor-based architecture
- PSBT signing workflow (no raw key exposure)
- Separation of:
  - wallet logic
  - UI layer
  - signing layer
- Hardware wallet–ready design

---

## 🧪 Example Descriptor

External:
```
wpkh([fingerprint/84'/1'/0']tpub.../0/*)
```

Internal:
```
wpkh([fingerprint/84'/1'/0']tpub.../1/*)
```

---

## 🧭 Roadmap

- Phase 1 → Wallet core, CLI, Sync  
- Phase 2 → Transactions + PSBT  
- Phase 3 → Persistence  
- Phase 4 → Tauri UI  
- Phase 5 → Taproot + Hardware wallet  

---

## 🎯 Why This Project Matters

This project demonstrates:

- Real-world Bitcoin wallet architecture
- Production-quality Rust engineering
- Clean modular system design
- Deep understanding of:
  - descriptors
  - PSBT
  - UTXO model

---

## 👤 Author

**Alex Eygenson**  
Rust • Finance • Trading Systems  

---

## ⭐ Support

If you find this useful:

- ⭐ Star the repo  
- 👀 Follow development  
