import { NavLink } from "react-router-dom";
import { navigationItems } from "../router/routes";
import { useWallet } from "../providers/useWallet";

const navIcons: Record<string, string> = {
  overview: "⌂",
  receive: "↧",
  addressBook: "⌘",
  send: "➚",
  utxos: "◉",
  transactions: "⇄",
};

export function Sidebar() {
  const { selectedWalletName } = useWallet();

  return (
    <aside className="sidebar">
      <div className="sidebar__brand">
        <div className="sidebar__logo" aria-hidden="true">
          D
        </div>
        <div>
          <div className="sidebar__brand-title">Rust Descriptor</div>
          <div className="sidebar__brand-title">Wallet</div>
        </div>
      </div>

      <nav className="sidebar__nav" aria-label="Main navigation">
        {navigationItems.map((item) => (
          <NavLink
            key={item.id}
            to={item.path}
            end={item.path === "/"}
            className={({ isActive }) =>
              isActive ? "sidebar__link sidebar__link--active" : "sidebar__link"
            }
          >
            <span className="sidebar__link-icon" aria-hidden="true">
              {navIcons[item.id] ?? "•"}
            </span>
            <span>{item.label}</span>
          </NavLink>
        ))}
      </nav>

      <div className="sidebar__footer">
        <div className="sidebar__status-row">
          <span className="sidebar__status-icon" aria-hidden="true">◌</span>
          <span>Wallet</span>
          <span className="sidebar__status-dot" aria-hidden="true" />
        </div>
        <div className="sidebar__status-row sidebar__status-row--wallet">
          <span className="sidebar__status-icon" aria-hidden="true">◇</span>
          <span title={selectedWalletName || "No wallet selected"}>
            {selectedWalletName || "No wallet selected"}
          </span>
        </div>
      </div>
    </aside>
  );
}