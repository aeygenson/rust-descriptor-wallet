

import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { AppShell } from "../layout/AppShell";
import { OverviewPage } from "../../pages/OverviewPage";
import { UtxosPage } from "../../pages/UtxosPage";
import { SendPage } from "../../pages/SendPage";
import { routes } from "./routes";
import { TransactionsPage } from "../../pages/TransactionsPage";


export function AppRouter() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppShell />}>
          <Route index element={<OverviewPage />} />
          <Route path={routes.utxos.slice(1)} element={<UtxosPage />} />
          <Route path={routes.send.slice(1)} element={<SendPage />} />
          <Route path={routes.transactions.slice(1)} element={<TransactionsPage />} />
          <Route path="*" element={<Navigate to={routes.overview} replace />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}