import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { Sidebar } from "../components/Sidebar";
import { ToastContainer } from "../components/Toast";
import { ErrorBoundary } from "../components/ErrorBoundary";
import { useToastStore } from "../stores/toastStore";
import { useAppStore } from "../stores/appStore";

// Mock the hooks used by StatusBar to avoid Tauri invoke calls
vi.mock("../hooks/useOllamaStatus", () => ({
  useOllamaStatus: () => ({ connected: true, version: "0.1.0", loading: false }),
}));

describe("Sidebar", () => {
  beforeEach(() => {
    useAppStore.setState({ activeView: "documents", sidebarCollapsed: false });
  });

  it("renders 4 nav icons + settings button", () => {
    render(<Sidebar />);
    const buttons = screen.getAllByRole("button");
    // 4 nav items + 1 settings button = 5
    expect(buttons).toHaveLength(5);
  });

  it("renders correct nav labels", () => {
    render(<Sidebar />);
    expect(screen.getByTitle("Knowledge Graph")).toBeInTheDocument();
    expect(screen.getByTitle("Chat")).toBeInTheDocument();
    expect(screen.getByTitle("Documents")).toBeInTheDocument();
    expect(screen.getByTitle("Search")).toBeInTheDocument();
    expect(screen.getByTitle("Settings")).toBeInTheDocument();
  });
});

describe("ToastContainer", () => {
  beforeEach(() => {
    useToastStore.setState({ toasts: [] });
  });

  it("renders nothing when no toasts", () => {
    const { container } = render(<ToastContainer />);
    expect(container.innerHTML).toBe("");
  });

  it("renders toast with message", () => {
    useToastStore.setState({
      toasts: [{ id: "t1", type: "success", message: "File uploaded" }],
    });
    render(<ToastContainer />);
    expect(screen.getByText("File uploaded")).toBeInTheDocument();
  });

  it("renders multiple toasts", () => {
    useToastStore.setState({
      toasts: [
        { id: "t1", type: "success", message: "Success msg" },
        { id: "t2", type: "error", message: "Error msg" },
      ],
    });
    render(<ToastContainer />);
    expect(screen.getByText("Success msg")).toBeInTheDocument();
    expect(screen.getByText("Error msg")).toBeInTheDocument();
  });
});

describe("ErrorBoundary", () => {
  // Suppress console.error for error boundary tests
  const originalError = console.error;
  beforeEach(() => {
    console.error = vi.fn();
  });

  it("renders children when no error", () => {
    render(
      <ErrorBoundary>
        <div>Child content</div>
      </ErrorBoundary>,
    );
    expect(screen.getByText("Child content")).toBeInTheDocument();
  });

  it("renders fallback when child throws", () => {
    function ThrowingComponent(): never {
      throw new Error("Test error");
    }

    render(
      <ErrorBoundary>
        <ThrowingComponent />
      </ErrorBoundary>,
    );
    expect(screen.getByText("Something went wrong")).toBeInTheDocument();
    expect(screen.getByText("Test error")).toBeInTheDocument();
    expect(screen.getByText("Reload")).toBeInTheDocument();
  });

  // Restore console.error
  afterEach(() => {
    console.error = originalError;
  });
});
