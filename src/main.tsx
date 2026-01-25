import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ThemeProvider } from "./contexts/ThemeContext";
import { ToastProvider } from "./contexts/ToastContext";
import "./styles/themes.css";

// Global error handler
window.onerror = (message, source, lineno, colno, error) => {
  document.body.innerHTML = `
    <div style="padding: 20px; font-family: monospace; color: red;">
      <h1>JavaScript Error</h1>
      <p><strong>Message:</strong> ${message}</p>
      <p><strong>Source:</strong> ${source}</p>
      <p><strong>Line:</strong> ${lineno}:${colno}</p>
      <pre>${error?.stack || 'No stack trace'}</pre>
    </div>
  `;
  return true;
};

window.onunhandledrejection = (event) => {
  document.body.innerHTML = `
    <div style="padding: 20px; font-family: monospace; color: red;">
      <h1>Unhandled Promise Rejection</h1>
      <pre>${event.reason?.stack || event.reason || 'Unknown error'}</pre>
    </div>
  `;
};

try {
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <ThemeProvider>
        <ToastProvider>
          <App />
        </ToastProvider>
      </ThemeProvider>
    </React.StrictMode>,
  );
} catch (e) {
  document.body.innerHTML = `
    <div style="padding: 20px; font-family: monospace; color: red;">
      <h1>Render Error</h1>
      <pre>${e instanceof Error ? e.stack : String(e)}</pre>
    </div>
  `;
}
