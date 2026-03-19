## 🏗 Architecture

### High-Level Diagram

> ⚠️ Note: Mermaid diagrams render correctly on GitHub, but may not display in some IDEs (e.g., RustRover preview).  
> If the diagram does not render, view this README directly on GitHub.

```mermaid
flowchart TD
    UI[Tauri UI]
    API[wallet_api]
    CORE[wallet_core (BDK)]
    SYNC[wallet_sync]
    STORAGE[wallet_storage]
    SIGNER[signer]

    UI --> API --> CORE
    CORE --> SYNC
    CORE --> STORAGE
    CORE --> SIGNER
```

#### Fallback (ASCII)

```
Tauri UI
   |
   v
wallet_api
   |
   v
wallet_core (BDK)
   |     |        |
   v     v        v
 sync  storage   signer
```