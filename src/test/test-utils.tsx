import { render, RenderOptions } from '@testing-library/react';
import { ReactElement, ReactNode } from 'react';
import { vi } from 'vitest';
import { ToastProvider } from '../contexts/ToastContext';
import { ThemeProvider } from '../contexts/ThemeContext';

// Wrap components with providers for testing
function AllProviders({ children }: { children: ReactNode }) {
  return (
    <ThemeProvider>
      <ToastProvider>
        {children}
      </ToastProvider>
    </ThemeProvider>
  );
}

// Custom render that includes all providers
function customRender(
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) {
  return render(ui, { wrapper: AllProviders, ...options });
}

// Re-export everything from testing-library
export * from '@testing-library/react';
export { customRender as render };

// Tauri mock helpers
import { invoke } from '@tauri-apps/api/core';

export function mockInvoke(responses: Record<string, unknown>) {
  const mockFn = vi.mocked(invoke);

  mockFn.mockImplementation(async (cmd: string, args?: unknown) => {
    if (cmd in responses) {
      const response = responses[cmd];
      if (typeof response === 'function') {
        return response(args);
      }
      return response;
    }
    throw new Error(`Unexpected invoke call: ${cmd}`);
  });

  return mockFn;
}

// Helper to create a mock invoke that tracks calls
export function createMockInvoke() {
  const calls: Array<{ cmd: string; args: unknown }> = [];
  const responses: Record<string, unknown> = {};

  const mockFn = vi.fn(async (cmd: string, args?: unknown) => {
    calls.push({ cmd, args });
    if (cmd in responses) {
      const response = responses[cmd];
      if (typeof response === 'function') {
        return response(args);
      }
      return response;
    }
    return undefined;
  });

  return {
    invoke: mockFn,
    calls,
    setResponse: (cmd: string, response: unknown) => {
      responses[cmd] = response;
    },
  };
}
