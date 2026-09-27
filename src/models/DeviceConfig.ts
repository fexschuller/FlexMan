export interface DeviceConfig {
  device: string | null;
  deviceRegex: string | null;
  channels: number | null;
  sampleType: string | null;
  suggestedLatencySeconds: number | null;
  wasapiExclusiveMode: boolean | null;
  wasapiAutoConvert: boolean | null;
  wasapiExplicitSampleFormat: boolean | null;
}

// Same as DeviceConfig, minus deviceRegex (raw-editor-only): the form always
// sends a concrete device, never a regex.
export type DeviceConfigUpdate = Omit<DeviceConfig, "deviceRegex">;

// Same fields as DeviceConfigUpdate but stringified (HTML controls only hold
// strings); wasapi* fields are tri-state: "" (default) | "true" | "false".
export type DeviceConfigDraft = { [K in keyof DeviceConfigUpdate]: string };
