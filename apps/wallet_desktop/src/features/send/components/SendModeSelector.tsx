import type { SendMode, SendModeSelectorProps } from "../types";
import { SEND_MODE_LABELS } from "../types";
import { getSendModeDescription } from "../format";

const SEND_MODES: SendMode[] = [
  "fixed",
  "send_max",
  "sweep",
  "consolidate",
];

const SEND_MODE_ICONS: Record<SendMode, string> = {
  fixed: "→",
  send_max: "⇉",
  sweep: "🧹",
  consolidate: "◫",
};

function getSendModeTitle(mode: SendMode): string {
  return `${SEND_MODE_LABELS[mode]}: ${getSendModeDescription(mode)}`;
}

export function SendModeSelector({
  mode,
  disabled = false,
  onModeChange,
}: SendModeSelectorProps) {
  return (
    <section className="send-card send-mode-selector" aria-label="Send mode">
      <div className="send-section-header">
        <div>
          <p className="send-eyebrow">Mode</p>
          <h3>Choose send type</h3>
        </div>
      </div>

      <div
        className="send-mode-tabs"
        role="tablist"
        aria-label="Send mode options"
      >
        {SEND_MODES.map((candidate) => {
          const isActive = candidate === mode;
          const label = SEND_MODE_LABELS[candidate];
          const title = getSendModeTitle(candidate);

          return (
            <button
              key={candidate}
              type="button"
              className={`send-mode-tab${isActive ? " is-active" : ""}`}
              aria-pressed={isActive}
              title={title}
              aria-label={label}
              disabled={disabled}
              onClick={() => onModeChange(candidate)}
            >
              <span className="send-mode-tab__icon" aria-hidden="true">
                {SEND_MODE_ICONS[candidate]}
              </span>
              <span className="send-mode-tab__label">
                {label}
              </span>
            </button>
          );
        })}
      </div>

      <div className="send-helper-text">
        <span>
          Different send modes optimize for regular payments, spending all
          funds, sweeping external UTXOs, or reducing future transaction costs
          through consolidation.
        </span>
      </div>

      <p className="send-helper-text">{getSendModeDescription(mode)}</p>
    </section>
  );
}