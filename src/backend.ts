import { invoke } from "@tauri-apps/api/core";
import { AudioDevice, Config, ConfigUpdatePayload, SystemReport, TomlSyntaxError } from "./models";

// Typed wrappers around every Tauri command, so call sites avoid raw
// invoke("name", {...}) strings.

export function listDevices(): Promise<AudioDevice[]> {
  return invoke<AudioDevice[]>("list_devices");
}

export function readConfig(): Promise<Config> {
  return invoke<Config>("read_config");
}

export function writeConfig(update: ConfigUpdatePayload): Promise<void> {
  return invoke("write_config", { update });
}

export function readRawConfig(): Promise<string> {
  return invoke<string>("read_raw_config");
}

export function writeRawConfig(raw: string): Promise<void> {
  return invoke("write_raw_config", { raw });
}

export function previewConfig(update: ConfigUpdatePayload): Promise<string> {
  return invoke<string>("preview_config", { update });
}

export function findTomlSyntaxError(raw: string): Promise<TomlSyntaxError | null> {
  return invoke<TomlSyntaxError | null>("find_toml_syntax_error", { raw });
}

export function testDraft(update: ConfigUpdatePayload): Promise<string> {
  return invoke<string>("run_self_test", { update });
}

export function testRaw(raw: string): Promise<string> {
  return invoke<string>("run_self_test_raw", { raw });
}

export function testSaved(): Promise<string> {
  return invoke<string>("run_self_test_saved");
}

export function isLoggingEnabled(): Promise<boolean> {
  return invoke<boolean>("is_logging_enabled");
}

export function setLoggingEnabled(enabled: boolean): Promise<void> {
  return invoke("set_logging_enabled", { enabled });
}

export function readLogTail(): Promise<string> {
  return invoke<string>("read_log_tail");
}

export function pickFlexAsioDir(): Promise<string | null> {
  return invoke<string | null>("pick_flexasio_dir");
}

export function getSystemReport(): Promise<SystemReport> {
  return invoke<SystemReport>("get_system_report");
}

export type RevealTarget = "flexasioInstallDir" | "config" | "log";

// Resolves and opens/reveals one of a fixed set of paths itself - the
// frontend never handles filesystem paths directly (see reveal_path in the
// Rust backend and capabilities/default.json).
export function revealPath(target: RevealTarget): Promise<void> {
  return invoke("reveal_path", { target });
}
