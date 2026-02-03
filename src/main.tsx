import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ThemeProvider } from "./contexts/ThemeContext";
import { ToastProvider } from "./contexts/ToastContext";
import "./styles/design-tokens.css";
import "./styles/themes.css";
import "./styles/components.css";

// Global error handler - logs to console without destroying the React app.
// Destroying document.body kills the entire React tree and prevents recovery.
// Instead, let React's ErrorBoundary components handle UI errors gracefully.
window.onerror = (message, _source, _lineno, _colno, error) => {
  console.error('JavaScript Error:', { message, error });
  // Return false to let the error propagate to React's error boundaries
  return false;
};

window.onunhandledrejection = (event) => {
  console.error('Unhandled Promise Rejection:', event.reason);
  // Don't destroy the page — let React error boundaries and try-catch handle recovery
};

async function bootstrap() {
  try {
    if (import.meta.env.VITE_E2E_MOCK_TAURI === '1') {
      const { setupE2eTauriMock } = await import('./test/e2eTauriMock');
      setupE2eTauriMock();
    }

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
}

void bootstrap();
