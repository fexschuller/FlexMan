import { useCallback, useEffect, useState } from "react";
import * as backend from "../backend";
import { useToast } from "../components/ToastProvider";
import { AppError, AudioDevice, errorMessage, toAppError } from "../models";

export function useDevices() {
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [error, setError] = useState<AppError | null>(null);
  const showToast = useToast();

  const reload = useCallback(async () => {
    setError(null);
    try {
      setDevices(await backend.listDevices());
    } catch (err) {
      setError(toAppError(err));
    }
  }, []);

  useEffect(() => {
    let cancelled = false;

    backend
      .listDevices()
      .then((list) => {
        if (!cancelled) setDevices(list);
      })
      .catch((err) => {
        if (!cancelled) setError(toAppError(err));
      });

    return () => {
      cancelled = true;
    };
  }, []);

  async function pickFlexAsioDir() {
    try {
      const dir = await backend.pickFlexAsioDir();
      if (dir !== null) {
        await reload();
      }
    } catch (err) {
      showToast(errorMessage(err));
    }
  }

  return { devices, error, reload, pickFlexAsioDir };
}
