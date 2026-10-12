import type { ReactNode } from "react";
import InfoBadge from "./InfoBadge";
import { SAMPLE_TYPE_OPTIONS, WASAPI_BACKEND } from "../constants";
import { AudioDevice, DeviceConfigDraft } from "../models";

const TRISTATE_OPTIONS: { value: string; label: string }[] = [
  { value: "", label: "(default)" },
  { value: "true", label: "Yes" },
  { value: "false", label: "No" },
];

interface DeviceFieldsProps {
  label: string;
  draft: DeviceConfigDraft;
  onChange: (draft: DeviceConfigDraft) => void;
  deviceOptions: AudioDevice[];
  backend: string;
  deviceRegex: string | null;
}

function Field({
  label,
  tooltip,
  children,
}: {
  label: string;
  tooltip: string;
  children: ReactNode;
}) {
  return (
    <label>
      {label} <InfoBadge text={tooltip} />: {children}
    </label>
  );
}

function TristateField({
  label,
  tooltip,
  value,
  onChange,
}: {
  label: string;
  tooltip: string;
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <Field label={label} tooltip={tooltip}>
      <select value={value} onChange={(e) => onChange(e.target.value)}>
        {TRISTATE_OPTIONS.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </Field>
  );
}

function DeviceFields({
  label,
  draft,
  onChange,
  deviceOptions,
  backend,
  deviceRegex,
}: DeviceFieldsProps) {
  const isWasapi = backend === WASAPI_BACKEND;
  const isDeviceAvailable = deviceOptions.some((d) => d.name === draft.device);

  return (
    <fieldset>
      <legend>{label}</legend>
      {deviceRegex ? (
        <p>
          Device selected via <code>deviceRegex</code>: <code>{deviceRegex}</code> - edit in the
          TOML tab.
        </p>
      ) : (
        <Field
          label="Device"
          tooltip="Which audio device FlexASIO uses for this direction. Leave as (default) to let FlexASIO pick automatically."
        >
          <select
            value={draft.device}
            onChange={(e) => onChange({ ...draft, device: e.target.value })}
          >
            <option value="">(default)</option>
            {draft.device !== "" && !isDeviceAvailable && (
              <option value={draft.device}>{draft.device} (not available)</option>
            )}
            {deviceOptions.map((d) => (
              <option key={d.name} value={d.name}>
                {d.name}
              </option>
            ))}
          </select>
        </Field>
      )}{" "}
      <Field
        label="Channels"
        tooltip="Number of channels to open on the device. Leave empty to use the device's default channel count. Example: 2 for stereo, 8 for surround."
      >
        <input
          type="number"
          min={1}
          placeholder="auto"
          value={draft.channels}
          onChange={(e) => onChange({ ...draft, channels: e.target.value })}
        />
      </Field>{" "}
      <Field
        label="Sample type"
        tooltip="Sample format/bit depth used for audio data. Float32 is the most common and usually the safest choice; leave as (default) to let FlexASIO pick."
      >
        <select
          value={draft.sampleType}
          onChange={(e) => onChange({ ...draft, sampleType: e.target.value })}
        >
          <option value="">(default)</option>
          {SAMPLE_TYPE_OPTIONS.map((option) => (
            <option key={option} value={option}>
              {option}
            </option>
          ))}
        </select>
      </Field>{" "}
      <Field
        label="Suggested latency (s)"
        tooltip="Target latency in seconds requested from the backend; actual latency may differ depending on hardware/driver support. Example: 0.01 (10 ms), 0.02 (20 ms)."
      >
        <input
          type="number"
          min={0}
          step="any"
          placeholder="auto"
          value={draft.suggestedLatencySeconds}
          onChange={(e) => onChange({ ...draft, suggestedLatencySeconds: e.target.value })}
        />
      </Field>
      {isWasapi && (
        <>
          {" "}
          <TristateField
            label="Exclusive mode"
            tooltip="If enabled, FlexASIO takes exclusive control of the device, bypassing the Windows audio mixer for lower latency - but no other app can use the device at the same time."
            value={draft.wasapiExclusiveMode}
            onChange={(v) => onChange({ ...draft, wasapiExclusiveMode: v })}
          />{" "}
          <TristateField
            label="Auto convert"
            tooltip="If enabled, Windows automatically converts the audio format when the device doesn't natively support the one FlexASIO requested."
            value={draft.wasapiAutoConvert}
            onChange={(v) => onChange({ ...draft, wasapiAutoConvert: v })}
          />{" "}
          <TristateField
            label="Explicit sample format"
            tooltip="If enabled, forces the exact sample format FlexASIO requested instead of letting WASAPI pick a compatible one automatically."
            value={draft.wasapiExplicitSampleFormat}
            onChange={(v) => onChange({ ...draft, wasapiExplicitSampleFormat: v })}
          />
        </>
      )}
    </fieldset>
  );
}

export default DeviceFields;
