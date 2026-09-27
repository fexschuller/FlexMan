import { useState } from "react";
import { errorMessage } from "../models";
import { useSelfTestLog } from "./SelfTestLogProvider";
import { useToast } from "./ToastProvider";

const TOAST_LOG_CHARS = 100;

function lastChars(text: string, n: number): string {
  return text.length > n ? `…${text.slice(-n)}` : text;
}

interface TestConfigButtonProps {
  runTest: () => Promise<string>;
  // Pass a `key` prop from the parent (e.g. the current draft's JSON) to
  // reset the pass/fail color whenever the draft changes.
  isDirty?: boolean;
}

function buttonLabel(running: boolean, passed: boolean | null, isDirty: boolean) {
  const labelString = `Test ${isDirty ? "Draft" : "Config"}`;
  if (running) {
    return "Testing...";
  }
  if (passed !== null) {
    return passed ? `✓ ${labelString}` : `✗ ${labelString}`;
  }
  return labelString;
}

function TestConfigButton({ runTest, isDirty = false }: TestConfigButtonProps) {
  const [running, setRunning] = useState(false);
  const [passed, setPassed] = useState<boolean | null>(null);

  const { recordResult } = useSelfTestLog();
  const showToast = useToast();

  async function handleClick() {
    setRunning(true);
    setPassed(null);

    try {
      const output = await runTest();
      setPassed(true);
      recordResult(true, output);
    } catch (err) {
      setPassed(false);
      const message = errorMessage(err);
      recordResult(false, message);
      showToast(`Config test failed: ${lastChars(message, TOAST_LOG_CHARS)}`);
    } finally {
      setRunning(false);
    }
  }

  const status = passed === true ? "passed" : passed === false ? "failed" : undefined;

  return (
    <button
      type="button"
      data-status={status}
      onClick={handleClick}
      disabled={running}
      aria-busy={running}
    >
      {buttonLabel(running, passed, isDirty)}
    </button>
  );
}

export default TestConfigButton;
