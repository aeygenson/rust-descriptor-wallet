import type { SendMode, SendModeSelectorProps } from "../types";
import { SEND_MODE_LABELS } from "../types";
import { getSendModeDescription } from "../format";
const SEND_MODES: SendMode[] = ["fixed", "send_max", "sweep", "consolidate"];

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

      <div className="send-mode-tabs" role="tablist" aria-label="Send mode options">
        {SEND_MODES.map((candidate) => {
          const isActive = candidate === mode;

          return (
            <button
              key={candidate}
              type="button"
              className={`send-mode-tab${isActive ? " is-active" : ""}`}
              aria-pressed={isActive}
              disabled={disabled}
              onClick={() => onModeChange(candidate)}
            >
              {SEND_MODE_LABELS[candidate]}
            </button>
          );
        })}
      </div>

      <p className="send-helper-text">{getSendModeDescription(mode)}</p>
    </section>
  );
}