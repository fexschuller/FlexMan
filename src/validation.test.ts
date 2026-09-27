import { describe, expect, it } from "vitest";
import { validatePositiveInteger, validatePositiveNumber } from "./validation";

describe("validatePositiveInteger", () => {
  it("accepts an empty value (means: use the default)", () => {
    const value = "";

    const error = validatePositiveInteger(value);

    expect(error).toBeNull();
  });

  it("accepts a positive whole number", () => {
    const value = "512";

    const error = validatePositiveInteger(value);

    expect(error).toBeNull();
  });

  it("rejects zero", () => {
    const value = "0";

    const error = validatePositiveInteger(value);

    expect(error).not.toBeNull();
  });

  it("rejects a decimal value", () => {
    const value = "512.5";

    const error = validatePositiveInteger(value);

    expect(error).not.toBeNull();
  });

  it("rejects a negative number", () => {
    const value = "-1";

    const error = validatePositiveInteger(value);

    expect(error).not.toBeNull();
  });

  it("rejects a non-numeric value", () => {
    const value = "abc";

    const error = validatePositiveInteger(value);

    expect(error).not.toBeNull();
  });
});

describe("validatePositiveNumber", () => {
  it("accepts an empty value (means: use the default)", () => {
    const value = "";

    const error = validatePositiveNumber(value);

    expect(error).toBeNull();
  });

  it("accepts a positive decimal value", () => {
    const value = "0.02";

    const error = validatePositiveNumber(value);

    expect(error).toBeNull();
  });

  it("rejects zero", () => {
    const value = "0";

    const error = validatePositiveNumber(value);

    expect(error).not.toBeNull();
  });

  it("rejects a negative number", () => {
    const value = "-0.5";

    const error = validatePositiveNumber(value);

    expect(error).not.toBeNull();
  });

  it("rejects a non-numeric value", () => {
    const value = "abc";

    const error = validatePositiveNumber(value);

    expect(error).not.toBeNull();
  });
});
