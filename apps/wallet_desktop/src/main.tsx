import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./styles/base.css";
import "./styles/layout.css";
import "./styles/overview.css";
import "./styles/send.css";
import "./styles/utxos.css";
import "./styles/transactions.css";
import "./styles/actions.css";
import "./styles/receive.css";
import App from "./App.tsx";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
