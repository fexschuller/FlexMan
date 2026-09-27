export type AppError =
  | { kind: "flexAsioNotFound" }
  | { kind: "io"; message: string }
  | { kind: "invalidConfig"; message: string }
  | { kind: "processFailed"; message: string }
  | { kind: "selfTestTimeout"; message: string };

function isAppError(err: unknown): err is AppError {
  return typeof err === "object" && err !== null && "kind" in err;
}

// Backend errors that don't originate from a Tauri command (e.g. a rejected
// dialog promise) may not have this shape; fall back to their string form.
export function toAppError(err: unknown): AppError {
  return isAppError(err) ? err : { kind: "io", message: String(err) };
}

export function errorMessage(err: unknown): string {
  const appError = toAppError(err);
  return appError.kind === "flexAsioNotFound"
    ? "FlexASIO not found in any known install location"
    : appError.message;
}
