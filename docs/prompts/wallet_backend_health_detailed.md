# Wallet Backend Health — Detailed Agentic Implementation Prompts

This document is a **production-grade, step-by-step agent prompt guide** to implement backend health checks in a Tauri + Rust + React wallet.

---

# 🎯 Objective

Implement **real backend health checks** (NOT just UI ping):

1. Wallet sync backend reachable
2. Bitcoin tip reachable
3. Broadcast backend reachable

---

# 🧠 Important Architecture Principle

Separate clearly:

| Status | Meaning |
|------|--------|
| Desktop Backend Connected | React ↔ Tauri ↔ Rust works |
| Backend Health | Bitcoin infrastructure works |

---

# 🧩 STEP 1 — Architecture Discovery

### Prompt

```text
Analyze the project structure and identify:

1. wallet_api DTO layer (model.rs)
2. wallet_sync backend abstraction
3. existing sync logic
4. broadcast logic
5. Tauri command layer
6. frontend wallet API functions
7. OverviewPage implementation

Return:
- exact file paths
- existing DTO patterns
- where to plug health check
```

---

# 🧩 STEP 2 — Add Rust DTO

### Prompt

```text
Add WalletBackendHealthDto using existing DTO conventions.

Fields:
- sync_backend_reachable: bool
- bitcoin_tip_reachable: bool
- broadcast_backend_reachable: bool
- tip_height: Option<u32>
- message: Option<String>

Match derives used in project.
```

---

# 🧩 STEP 3 — Sync Layer Health Checks

### Prompt

```text
Implement lightweight health checks in wallet_sync.

Rules:
- NO full sync
- NO state mutation
- NO broadcasting

Functions:

check_sync_backend()
get_tip_height()
check_broadcast_backend()

Return structured results.
```

---

# 🧩 STEP 4 — wallet_api Service

### Prompt

```text
Create:

check_wallet_backend_health(wallet_name: String)

Behavior:
- load wallet
- detect backend
- run health checks
- aggregate results
- return partial success if possible
```

---

# 🧩 STEP 5 — Tauri Command

### Prompt

```text
Add command:

check_wallet_backend_health

Ensure:
- async
- uses AppState
- consistent error handling
```

---

# 🧩 STEP 6 — Frontend DTO

### Prompt

```text
Add shared TS type:

WalletBackendHealthDto
```

---

# 🧩 STEP 7 — Frontend API

### Prompt

```text
Add function:

checkWalletBackendHealth(walletName)

Use invokeCommand pattern.
```

---

# 🧩 STEP 8 — Overview State

### Prompt

```text
Add React state:

backendHealth
backendHealthLoading
backendHealthError

Load on wallet change.
```

---

# 🧩 STEP 9 — UI

### Prompt

```text
Add new UI block:

- Sync backend
- Bitcoin tip
- Broadcast backend

Use:
- green = OK
- yellow = unknown
- red = error
```

---

# 🧩 STEP 10 — CSS

### Prompt

```text
Add styles:

.overview-health
.overview-health-row
.status-dot--ok
.status-dot--warning
.status-dot--error
```

---

# 🧩 STEP 11 — Refresh Button

### Prompt

```text
Add button:

Check backend health

Behavior:
- triggers only health call
- no sync
```

---

# 🧩 STEP 12 — Validation

### Prompt

```text
Test:

1. backend running → all green
2. backend down → red
3. wallet switch → reload
4. refresh → updates only health
```

---

# 🧩 STEP 13 — Optional Cache

### Prompt

```text
Add 30–60s cache for health.

Manual refresh bypasses cache.
```

---

# ✅ Final Acceptance

- Backend health independent from Desktop backend
- No full sync triggered
- UI resilient to failures
- Works across wallets
- Clean architecture maintained

---

# 🚀 If You Want Next Level

You can extend later with:

- latency measurement
- backend type display (Electrum/Esplora/Core)
- auto-retry logic
- background health polling

---

End of document.
