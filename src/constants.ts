export type Backend = "" | "MME" | "Windows DirectSound" | "Windows WASAPI" | "Windows WDM-KS";

// FlexASIO's own default when `backend` is unset.
export const DEFAULT_BACKEND: Backend = "Windows DirectSound";

export const WASAPI_BACKEND: Backend = "Windows WASAPI";

export const BACKEND_OPTIONS: { value: Backend; label: string }[] = [
  { value: "", label: `(default: ${DEFAULT_BACKEND})` },
  { value: "MME", label: "MME" },
  { value: "Windows DirectSound", label: "Windows DirectSound" },
  { value: WASAPI_BACKEND, label: WASAPI_BACKEND },
  { value: "Windows WDM-KS", label: "Windows WDM-KS" },
];

export const SAMPLE_TYPE_OPTIONS = ["Float32", "Int32", "Int24", "Int16"] as const;
