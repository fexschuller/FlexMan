import CodeMirror from "@uiw/react-codemirror";
import { StreamLanguage } from "@codemirror/language";
import { linter, Diagnostic } from "@codemirror/lint";
import { toml } from "@codemirror/legacy-modes/mode/toml";
import * as backend from "../backend";
import { usePrefersDark } from "../hooks/usePrefersDark";

const tomlLanguage = StreamLanguage.define(toml);

const tomlLinter = linter(async (view): Promise<Diagnostic[]> => {
  const text = view.state.doc.toString();
  const syntaxError = await backend.findTomlSyntaxError(text);

  if (!syntaxError) {
    return [];
  }

  return [
    {
      from: syntaxError.start,
      to: Math.max(syntaxError.end, syntaxError.start + 1),
      severity: "error",
      message: syntaxError.message,
    },
  ];
});

const extensions = [tomlLanguage, tomlLinter];

interface ConfigEditorProps {
  value: string;
  onChange: (value: string) => void;
  loadError: string | null;
}

function ConfigEditor({ value, onChange, loadError }: ConfigEditorProps) {
  const prefersDark = usePrefersDark();

  return (
    <article>
      {loadError && <p style={{ color: "red" }}>{loadError}</p>}
      <CodeMirror
        value={value}
        height="400px"
        theme={prefersDark ? "dark" : "light"}
        extensions={extensions}
        onChange={onChange}
      />
    </article>
  );
}

export default ConfigEditor;
