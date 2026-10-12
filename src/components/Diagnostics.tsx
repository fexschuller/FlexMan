import { useEffect, useState } from "react";
import * as backend from "../backend";
import { errorMessage, SystemReport } from "../models";
import ErrorText from "./ErrorText";
import PathLink from "./PathLink";
import { useSelfTestLog } from "./SelfTestLogProvider";
import { useToast } from "./ToastProvider";

function Diagnostics() {
  const [loggingEnabled, setLoggingEnabled] = useState(false);
  const [loggingError, setLoggingError] = useState<string | null>(null);

  const [flexasioLog, setFlexasioLog] = useState<string | null>(null);
  const [flexasioLogError, setFlexasioLogError] = useState<string | null>(null);

  const [report, setReport] = useState<SystemReport | null>(null);
  const [reportError, setReportError] = useState<string | null>(null);

  const showToast = useToast();
  const { log } = useSelfTestLog();

  // Auto-expand once a new self-test result comes in, regardless of which view ran it.
  // Adjusted during render rather than in an effect to avoid an extra render pass.
  const [selfTestOpen, setSelfTestOpen] = useState(log !== null);
  const [seenLog, setSeenLog] = useState(log);
  if (log !== seenLog) {
    setSeenLog(log);
    if (log !== null) {
      setSelfTestOpen(true);
    }
  }

  useEffect(() => {
    backend
      .isLoggingEnabled()
      .then(setLoggingEnabled)
      .catch((err) => setLoggingError(errorMessage(err)));
  }, []);

  useEffect(() => {
    backend
      .getSystemReport()
      .then(setReport)
      .catch((err) => setReportError(errorMessage(err)));
  }, []);

  async function toggleLogging() {
    const next = !loggingEnabled;

    try {
      await backend.setLoggingEnabled(next);
      setLoggingEnabled(next);
    } catch (err) {
      showToast(errorMessage(err));
    }
  }

  async function loadFlexasioLog() {
    setFlexasioLogError(null);
    try {
      setFlexasioLog(await backend.readLogTail());
    } catch (err) {
      setFlexasioLogError(errorMessage(err));
    }
  }

  return (
    <>
      <h2>Diagnostics</h2>

      <article>
        <fieldset>
          <legend>System Report</legend>
          {reportError && <ErrorText>{reportError}</ErrorText>}
          {report && (
            <table>
              <tbody>
                <tr>
                  <td>FlexASIO</td>
                  <td>{report.flexasioVersion ?? "Not found"}</td>
                </tr>
                <tr>
                  <td>Install location</td>
                  <td>
                    {report.flexasioInstallDir ? (
                      <PathLink path={report.flexasioInstallDir} target="flexasioInstallDir" />
                    ) : (
                      "Not found"
                    )}
                  </td>
                </tr>
                <tr>
                  <td>Config file</td>
                  <td>
                    {report.configExists ? (
                      <PathLink path={report.configPath} target="config" />
                    ) : (
                      <>{report.configPath} (does not exist yet)</>
                    )}
                  </td>
                </tr>
                <tr>
                  <td>FlexASIO.log</td>
                  <td>
                    {report.logExists ? (
                      <PathLink path={report.logPath} target="log" />
                    ) : (
                      <>{report.logPath} (does not exist yet)</>
                    )}
                  </td>
                </tr>
                <tr>
                  <td>Windows</td>
                  <td>{report.osVersion}</td>
                </tr>
              </tbody>
            </table>
          )}
        </fieldset>

        <fieldset>
          <legend>Logging</legend>
          <label style={{ margin: 0 }}>
            <input type="checkbox" checked={loggingEnabled} onChange={toggleLogging} />
            Enable FlexASIO logging
          </label>
          {loggingError && <ErrorText>{loggingError}</ErrorText>}
        </fieldset>

        <fieldset>
          <legend>Logs</legend>

          <details open={selfTestOpen} onToggle={(e) => setSelfTestOpen(e.currentTarget.open)}>
            <summary>Self-Test</summary>
            {log ? (
              <pre className="log-output">
                <code>{log.output}</code>
              </pre>
            ) : (
              <p>No self-test has been run yet.</p>
            )}
          </details>

          <details onToggle={(e) => e.currentTarget.open && loadFlexasioLog()}>
            <summary>FlexASIO.log</summary>
            {flexasioLogError && <ErrorText>{flexasioLogError}</ErrorText>}
            {flexasioLog !== null &&
              (flexasioLog === "" ? (
                <p>Log is empty (enable logging above, then run FlexASIO).</p>
              ) : (
                <pre className="log-output">
                  <code>{flexasioLog}</code>
                </pre>
              ))}
          </details>
        </fieldset>
      </article>
    </>
  );
}

export default Diagnostics;
