import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ThemeProvider } from "./contexts/ThemeContext";
import { ToastProvider } from "./contexts/ToastContext";
import "./styles/themes.css";

// Global error handler - logs to console, shows safe message to user
window.onerror = (message, _source, _lineno, _colno, error) => {
  // Log full details to console for debugging
  console.error('JavaScript Error:', { message, error });

  // Show safe message without exposing internals
  const errorDiv = document.createElement('div');
  errorDiv.style.cssText = 'padding: 20px; font-family: system-ui, sans-serif; color: #dc2626;';
  errorDiv.innerHTML = '<h1>Application Error</h1><p>An unexpected error occurred. Please restart the application.</p><p>Check the developer console for details.</p>';
  document.body.innerHTML = '';
  document.body.appendChild(errorDiv);
  return true;
};

window.onunhandledrejection = (event) => {
  // Log full details to console for debugging
  console.error('Unhandled Promise Rejection:', event.reason);

  // Show safe message without exposing internals
  const errorDiv = document.createElement('div');
  errorDiv.style.cssText = 'padding: 20px; font-family: system-ui, sans-serif; color: #dc2626;';
  errorDiv.innerHTML = '<h1>Application Error</h1><p>An unexpected error occurred. Please restart the application.</p><p>Check the developer console for details.</p>';
  document.body.innerHTML = '';
  document.body.appendChild(errorDiv);
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
  // Log full details to console for debugging
  console.error('Render Error:', e);

  // Show safe message without exposing internals
  const errorDiv = document.createElement('div');
  errorDiv.style.cssText = 'padding: 20px; font-family: system-ui, sans-serif; color: #dc2626;';
  errorDiv.innerHTML = '<h1>Application Error</h1><p>Failed to initialize the application. Please restart.</p><p>Check the developer console for details.</p>';
  document.body.innerHTML = '';
  document.body.appendChild(errorDiv);
}
