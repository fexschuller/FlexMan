import { Config, ConfigUpdatePayload, DeviceConfig, DeviceConfigDraft, DeviceConfigUpdate } from "./models";

export interface FormDraft {
  backend: string;
  bufferSizeSamples: string;
  input: DeviceConfigDraft;
  output: DeviceConfigDraft;
}

export const EMPTY_DEVICE_DRAFT: DeviceConfigDraft = {
  device: "",
  channels: "",
  sampleType: "",
  suggestedLatencySeconds: "",
  wasapiExclusiveMode: "",
  wasapiAutoConvert: "",
  wasapiExplicitSampleFormat: "",
};

export const EMPTY_FORM_DRAFT: FormDraft = {
  backend: "",
  bufferSizeSamples: "",
  input: EMPTY_DEVICE_DRAFT,
  output: EMPTY_DEVICE_DRAFT,
};

export function emptyToNull(value: string): string | null {
  return value === "" ? null : value;
}

function boolToTristate(value: boolean | null): string {
  if (value === null) {
    return "";
  }
  return value ? "true" : "false";
}

function tristateToBool(value: string): boolean | null {
  if (value === "") {
    return null;
  }
  return value === "true";
}

function deviceDraftFromConfig(deviceConfig: DeviceConfig): DeviceConfigDraft {
  return {
    device: deviceConfig.device ?? "",
    channels: deviceConfig.channels?.toString() ?? "",
    sampleType: deviceConfig.sampleType ?? "",
    suggestedLatencySeconds: deviceConfig.suggestedLatencySeconds?.toString() ?? "",
    wasapiExclusiveMode: boolToTristate(deviceConfig.wasapiExclusiveMode),
    wasapiAutoConvert: boolToTristate(deviceConfig.wasapiAutoConvert),
    wasapiExplicitSampleFormat: boolToTristate(deviceConfig.wasapiExplicitSampleFormat),
  };
}

export function draftFromConfig(config: Config): FormDraft {
  return {
    backend: config.backend ?? "",
    bufferSizeSamples: config.bufferSizeSamples?.toString() ?? "",
    input: deviceDraftFromConfig(config.input),
    output: deviceDraftFromConfig(config.output),
  };
}

function deviceUpdateFromDraft(draft: DeviceConfigDraft): DeviceConfigUpdate {
  return {
    device: emptyToNull(draft.device),
    channels: draft.channels === "" ? null : Number(draft.channels),
    sampleType: emptyToNull(draft.sampleType),
    suggestedLatencySeconds:
      draft.suggestedLatencySeconds === "" ? null : Number(draft.suggestedLatencySeconds),
    wasapiExclusiveMode: tristateToBool(draft.wasapiExclusiveMode),
    wasapiAutoConvert: tristateToBool(draft.wasapiAutoConvert),
    wasapiExplicitSampleFormat: tristateToBool(draft.wasapiExplicitSampleFormat),
  };
}

export function updateFromDraft(draft: FormDraft): ConfigUpdatePayload {
  return {
    backend: emptyToNull(draft.backend),
    bufferSizeSamples: draft.bufferSizeSamples === "" ? null : Number(draft.bufferSizeSamples),
    input: deviceUpdateFromDraft(draft.input),
    output: deviceUpdateFromDraft(draft.output),
  };
}

// Plain nested string fields with fixed keys, so a JSON round-trip is a
// reliable structural comparison - no field can be silently left out, unlike
// the old hand-listed field-by-field comparison.
export function draftsEqual(a: FormDraft, b: FormDraft): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}
