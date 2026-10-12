import DeviceFields from "./DeviceFields";
import ErrorText from "./ErrorText";
import InfoBadge from "./InfoBadge";
import { BACKEND_OPTIONS, DEFAULT_BACKEND } from "../constants";
import { FormDraft } from "../draft";
import { useDevices } from "../hooks/useDevices";
import { AudioDevice, Config } from "../models";

function filterByDirection(devices: AudioDevice[], backend: string, direction: "input" | "output") {
  return devices.filter(
    (d) =>
      d.host_api === backend &&
      (direction === "input" ? d.max_input_channels > 0 : d.max_output_channels > 0),
  );
}

interface ConfigFormProps {
  config: Config | null;
  configError: string | null;
  draft: FormDraft;
  onChange: (draft: FormDraft) => void;
}

function ConfigForm({ config, configError, draft, onChange }: ConfigFormProps) {
  const { devices, error: devicesError, reload: reloadDevices, pickFlexAsioDir } = useDevices();

  const effectiveBackend = draft.backend || DEFAULT_BACKEND;

  return (
    <article>
      {configError && <ErrorText>{configError}</ErrorText>}
      {devicesError && (
        <ErrorText>
          {devicesError.kind === "flexAsioNotFound" ? (
            <>
              FlexASIO not found in any known install location{" "}
              <button type="button" onClick={pickFlexAsioDir}>
                Select FlexASIO folder…
              </button>
            </>
          ) : (
            devicesError.message
          )}
        </ErrorText>
      )}
      {config && (
        <>
          <div className="field-row">
            <label>
              Backend{" "}
              <InfoBadge text="Which Windows audio API FlexASIO uses to talk to the device. WASAPI usually gives the lowest latency; MME is the most compatible with older hardware/drivers." />
              <select
                value={draft.backend}
                onChange={(e) => onChange({ ...draft, backend: e.target.value })}
              >
                {BACKEND_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
            <button
              type="button"
              onClick={reloadDevices}
              aria-label="Reload devices"
              title="Reload devices"
            >
              ↻
            </button>
          </div>
          <label>
            Buffer size (samples){" "}
            <InfoBadge text="Audio block size in samples. Smaller = lower latency but more CPU load and risk of dropouts; larger = more stable but higher latency. Leave empty to use the backend's default. Example: 256, 512, 1024." />
            :{" "}
            <input
              type="number"
              min={1}
              placeholder="auto"
              value={draft.bufferSizeSamples}
              onChange={(e) => onChange({ ...draft, bufferSizeSamples: e.target.value })}
            />
          </label>

          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "1rem" }}>
            <DeviceFields
              label="Input"
              draft={draft.input}
              onChange={(input) => onChange({ ...draft, input })}
              deviceOptions={filterByDirection(devices, effectiveBackend, "input")}
              backend={effectiveBackend}
              deviceRegex={config.input.deviceRegex}
            />
            <DeviceFields
              label="Output"
              draft={draft.output}
              onChange={(output) => onChange({ ...draft, output })}
              deviceOptions={filterByDirection(devices, effectiveBackend, "output")}
              backend={effectiveBackend}
              deviceRegex={config.output.deviceRegex}
            />
          </div>
        </>
      )}
    </article>
  );
}

export default ConfigForm;
