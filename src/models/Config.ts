import { DeviceConfig, DeviceConfigUpdate } from "./DeviceConfig";

export interface Config {
  backend: string | null;
  bufferSizeSamples: number | null;
  input: DeviceConfig;
  output: DeviceConfig;
}

export interface ConfigUpdatePayload {
  backend: string | null;
  bufferSizeSamples: number | null;
  input: DeviceConfigUpdate;
  output: DeviceConfigUpdate;
}
