export function validatePositiveInteger(value: string): string | null {
  if (value === "") {
    return null;
  }

  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    return "Must be a positive whole number";
  }

  return null;
}

export function validatePositiveNumber(value: string): string | null {
  if (value === "") {
    return null;
  }

  const parsed = Number(value);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return "Must be a positive number";
  }

  return null;
}
