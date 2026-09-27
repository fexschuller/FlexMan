import { createContext, useContext, useState, type ReactNode } from "react";

interface SelfTestLog {
  passed: boolean;
  output: string;
}

interface SelfTestLogContextValue {
  log: SelfTestLog | null;
  recordResult: (passed: boolean, output: string) => void;
}

const SelfTestLogContext = createContext<SelfTestLogContextValue | null>(null);

export function SelfTestLogProvider({ children }: { children: ReactNode }) {
  const [log, setLog] = useState<SelfTestLog | null>(null);

  function recordResult(passed: boolean, output: string) {
    setLog({ passed, output });
  }

  return (
    <SelfTestLogContext.Provider value={{ log, recordResult }}>
      {children}
    </SelfTestLogContext.Provider>
  );
}

export function useSelfTestLog(): SelfTestLogContextValue {
  const ctx = useContext(SelfTestLogContext);
  if (!ctx) {
    throw new Error("useSelfTestLog must be used within a SelfTestLogProvider");
  }
  return ctx;
}
