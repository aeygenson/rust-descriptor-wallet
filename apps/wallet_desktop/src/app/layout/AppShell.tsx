import { useState } from "react";
import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";
import { Topbar } from "./Topbar";

export function AppShell() {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  return (
    <div className={`app-shell ${sidebarOpen ? "" : "app-shell--collapsed"}`}>
      <Sidebar />
      {sidebarOpen && (
        <div
          className="app-overlay"
          onClick={() => setSidebarOpen(false)}
        />
      )}
      <main className="main-content">
        <Topbar
          sidebarOpen={sidebarOpen}
          onToggleSidebar={() => setSidebarOpen((current) => !current)}
        />
        <div className="page">
          <Outlet />
        </div>
      </main>
    </div>
  );
}