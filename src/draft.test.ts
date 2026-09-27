import { describe, expect, it } from "vitest";
import { draftFromConfig, draftsEqual, updateFromDraft, EMPTY_FORM_DRAFT } from "./draft";
import { Config } from "./models";

const FULL_CONFIG: Config = {
  backend: "Windows WASAPI",
  bufferSizeSamples: 256,
  input: {
    device: "Mic",
    deviceRegex: null,
    channels: 2,
    sampleType: "Float32",
    suggestedLatencySeconds: 0.02,
    wasapiExclusiveMode: true,
    wasapiAutoConvert: false,
    wasapiExplicitSampleFormat: null,
  },
  output: {
    device: null,
    deviceRegex: null,
    channels: null,
    sampleType: null,
    suggestedLatencySeconds: null,
    wasapiExclusiveMode: null,
    wasapiAutoConvert: null,
    wasapiExplicitSampleFormat: null,
  },
};

describe("draftFromConfig / updateFromDraft round-trip", () => {
  it("round-trips a fully populated config", () => {
    const draft = draftFromConfig(FULL_CONFIG);

    const update = updateFromDraft(draft);

    expect(update).toEqual({
      backend: "Windows WASAPI",
      bufferSizeSamples: 256,
      input: {
        device: "Mic",
        channels: 2,
        sampleType: "Float32",
        suggestedLatencySeconds: 0.02,
        wasapiExclusiveMode: true,
        wasapiAutoConvert: false,
        wasapiExplicitSampleFormat: null,
      },
      output: {
        device: null,
        channels: null,
        sampleType: null,
        suggestedLatencySeconds: null,
        wasapiExclusiveMode: null,
        wasapiAutoConvert: null,
        wasapiExplicitSampleFormat: null,
      },
    });
  });

  it("round-trips an all-default (empty) config", () => {
    const draft = draftFromConfig({
      backend: null,
      bufferSizeSamples: null,
      input: FULL_CONFIG.output,
      output: FULL_CONFIG.output,
    });

    const update = updateFromDraft(draft);

    expect(update.backend).toBeNull();
    expect(update.bufferSizeSamples).toBeNull();
    expect(update.input.device).toBeNull();
  });

  it("represents an unset wasapi toggle as neither true nor false", () => {
    const draft = draftFromConfig(FULL_CONFIG);

    expect(draft.input.wasapiExplicitSampleFormat).toBe("");

    const update = updateFromDraft(draft);

    expect(update.input.wasapiExplicitSampleFormat).toBeNull();
  });
});

describe("draftsEqual", () => {
  it("treats two drafts with the same values as equal", () => {
    const a = draftFromConfig(FULL_CONFIG);
    const b = draftFromConfig(FULL_CONFIG);

    const equal = draftsEqual(a, b);

    expect(equal).toBe(true);
  });

  it("treats a draft that differs in a nested device field as unequal", () => {
    const a = draftFromConfig(FULL_CONFIG);
    const b = { ...a, input: { ...a.input, channels: "4" } };

    const equal = draftsEqual(a, b);

    expect(equal).toBe(false);
  });

  it("treats the empty draft as equal to itself", () => {
    const equal = draftsEqual(EMPTY_FORM_DRAFT, EMPTY_FORM_DRAFT);

    expect(equal).toBe(true);
  });
});
