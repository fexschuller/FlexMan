import { useState } from "react";
import * as backend from "./backend";
import ConfigEditor from "./components/ConfigEditor";
import ConfigForm from "./components/ConfigForm";
import Diagnostics from "./components/Diagnostics";
import TestConfigButton from "./components/TestConfigButton";
import { updateFromDraft } from "./draft";
import { Mode, useConfigDraft } from "./hooks/useConfigDraft";

function App() {
  const [mode, setMode] = useState<Mode>("form");

  const {
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
  } = useConfigDraft(mode);

  function handleModeChange(next: Mode) {
    if (mode === "raw" && isRawDirty && next !== "raw") {
      const discard = window.confirm(
        "You have unsaved TOML changes that will be lost. Switch anyway?",
      );
      if (!discard) {
        return;
      }
    }
    setMode(next);
  }

  return (
    <main style={{ paddingBottom: "4rem" }}>
      <div role="group">
        <button onClick={() => handleModeChange("form")} disabled={mode === "form"}>
          Config
        </button>
        <button onClick={() => handleModeChange("raw")} disabled={mode === "raw"}>
          TOML
        </button>
        <button onClick={() => handleModeChange("diagnostics")} disabled={mode === "diagnostics"}>
          Diagnostics
        </button>
      </div>

      {mode === "form" && (
        <ConfigForm
          config={config}
          configError={configError}
          draft={formDraft}
          onChange={setFormDraft}
        />
      )}

      {mode === "raw" && <ConfigEditor value={raw} onChange={setRaw} loadError={rawLoadError} />}

      {mode === "diagnostics" && <Diagnostics />}

      {mode === "form" && config && (
        <footer>
          <span>{isFormDirty && "Unsaved changes."}</span>
          <div>
            <TestConfigButton
              key={JSON.stringify(formDraft)}
              runTest={() => {
                const error = validateDraft();
                if (error) {
                  return Promise.reject(new Error(error));
                }
                return backend.testDraft(updateFromDraft(formDraft));
              }}
              isDirty={isFormDirty}
            />
            <button onClick={handleSaveClick} disabled={!isFormDirty}>
              Save
            </button>
          </div>
        </footer>
      )}

      {mode === "raw" && (
        <footer>
          <span>{isRawDirty && "Unsaved changes."}</span>
          <div>
            <TestConfigButton key={raw} runTest={() => backend.testRaw(raw)} isDirty={isRawDirty} />
            <button onClick={handleSaveRawClick} disabled={!isRawDirty}>
              Save
            </button>
          </div>
        </footer>
      )}

      {mode === "diagnostics" && (
        <footer>
          <span />
          <div>
            <TestConfigButton runTest={() => backend.testSaved()} />
          </div>
        </footer>
      )}
    </main>
  );
}

export default App;
