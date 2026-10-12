import { useEffect, useState } from "react";
import * as backend from "../backend";
import { useToast } from "../components/ToastProvider";
import {
  draftFromConfig,
  draftsEqual,
  EMPTY_FORM_DRAFT,
  FormDraft,
  updateFromDraft,
} from "../draft";
import { Config, errorMessage } from "../models";
import { validatePositiveInteger, validatePositiveNumber } from "../validation";

export type Mode = "form" | "raw" | "diagnostics";

function validateFormDraft(draft: FormDraft): string | null {
  const inputChannelsError = validatePositiveInteger(draft.input.channels);
  if (inputChannelsError) {
    return `Input channels: ${inputChannelsError}`;
  }

  const outputChannelsError = validatePositiveInteger(draft.output.channels);
  if (outputChannelsError) {
    return `Output channels: ${outputChannelsError}`;
  }

  const bufferSizeError = validatePositiveInteger(draft.bufferSizeSamples);
  if (bufferSizeError) {
    return `Buffer size: ${bufferSizeError}`;
  }

  const inputLatencyError = validatePositiveNumber(draft.input.suggestedLatencySeconds);
  if (inputLatencyError) {
    return `Input suggested latency: ${inputLatencyError}`;
  }

  const outputLatencyError = validatePositiveNumber(draft.output.suggestedLatencySeconds);
  if (outputLatencyError) {
    return `Output suggested latency: ${outputLatencyError}`;
  }

  return null;
}

// Owns the config draft (form + raw text) and save/test actions; reloads the
// raw draft fresh whenever mode switches to "raw" (unless there's already an
// unsaved raw edit in progress - see the isRawDirty guard below).
export function useConfigDraft(mode: Mode) {
  const [config, setConfig] = useState<Config | null>(null);
  const [configError, setConfigError] = useState<string | null>(null);
  const [formDraft, setFormDraft] = useState<FormDraft>(EMPTY_FORM_DRAFT);

  const [raw, setRaw] = useState("");
  const [onDisk, setOnDisk] = useState<string | null>(null);
  const [rawLoadError, setRawLoadError] = useState<string | null>(null);

  const showToast = useToast();

  async function reloadConfig() {
    setConfigError(null);
    try {
      const loaded = await backend.readConfig();
      setConfig(loaded);
      setFormDraft(draftFromConfig(loaded));
    } catch (err) {
      setConfigError(errorMessage(err));
    }
  }

  useEffect(() => {
    let cancelled = false;

    backend
      .readConfig()
      .then((loaded) => {
        if (cancelled) return;
        setConfig(loaded);
        setFormDraft(draftFromConfig(loaded));
      })
      .catch((err) => {
        if (!cancelled) setConfigError(errorMessage(err));
      });

    return () => {
      cancelled = true;
    };
  }, []);

  const isFormDirty = config !== null && !draftsEqual(formDraft, draftFromConfig(config));
  const isRawDirty = onDisk !== null && raw !== onDisk;

  function validateDraft(): string | null {
    return validateFormDraft(formDraft);
  }

  async function saveConfig() {
    const error = validateFormDraft(formDraft);
    if (error) {
      throw new Error(error);
    }

    await backend.writeConfig(updateFromDraft(formDraft));
    await reloadConfig();
  }

  async function handleSaveClick() {
    try {
      await saveConfig();
    } catch (err) {
      showToast(errorMessage(err));
    }
  }

  // Reload only when switching into Raw mode, capturing the current form
  // draft (merged onto disk) as the raw editor's starting text - but not if
  // there's already an unsaved raw edit, so it never gets silently clobbered.
  useEffect(() => {
    if (mode !== "raw" || isRawDirty) {
      return;
    }

    let cancelled = false;
    Promise.all([backend.readRawConfig(), backend.previewConfig(updateFromDraft(formDraft))])
      .then(([diskText, preview]) => {
        if (cancelled) return;
        setRawLoadError(null);
        setOnDisk(diskText);
        setRaw(preview);
      })
      .catch((err) => {
        if (!cancelled) setRawLoadError(errorMessage(err));
      });

    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- isRawDirty is a guard, not a trigger
  }, [mode, formDraft]);

  async function saveRaw() {
    await backend.writeRawConfig(raw);
    setOnDisk(raw);
    await reloadConfig();
  }

  async function handleSaveRawClick() {
    try {
      await saveRaw();
    } catch (err) {
      showToast(errorMessage(err));
    }
  }

  return {
    config,
    configError,
    formDraft,
    setFormDraft,
    isFormDirty,
    validateDraft,
    handleSaveClick,
    raw,
    setRaw,
    rawLoadError,
    isRawDirty,
    handleSaveRawClick,
  };
}
