import React from "react";
import ReactDOM from "react-dom/client";
import "@picocss/pico/css/pico.classless.min.css";
import "./theme.css";
import App from "./App";
import { SelfTestLogProvider } from "./components/SelfTestLogProvider";
import { ToastProvider } from "./components/ToastProvider";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ToastProvider>
      <SelfTestLogProvider>
        <App />
      </SelfTestLogProvider>
    </ToastProvider>
  </React.StrictMode>,
);
