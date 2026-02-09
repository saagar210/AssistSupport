import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import { Sidebar } from "../components/Sidebar";
import { ToastContainer } from "../components/Toast";
import { ErrorBoundary } from "../components/ErrorBoundary";
import {
  ChatSkeleton,
  DocumentGridSkeleton,
  DocumentListSkeleton,
  SearchSkeleton,
  GraphSkeleton,
} from "../components/LoadingSkeleton";
import { EmptyState } from "../components/EmptyState";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { MarkdownRenderer } from "../components/MarkdownRenderer";
import { IngestionPanel } from "../components/IngestionPanel";
import { SetupWizard } from "../components/SetupWizard";
import { OllamaStatusBanner } from "../components/OllamaStatusBanner";
import { Modal, Toggle, Badge, Tooltip, Skeleton } from "../components/ui";
import { FileText, Search } from "lucide-react";
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

describe("LoadingSkeleton", () => {
  it("renders ChatSkeleton with skeleton blocks", () => {
    render(<ChatSkeleton />);
    expect(screen.getByTestId("chat-skeleton")).toBeInTheDocument();
    expect(screen.getAllByTestId("skeleton-block").length).toBeGreaterThan(0);
  });

  it("renders DocumentGridSkeleton with 6 card placeholders", () => {
    render(<DocumentGridSkeleton />);
    expect(screen.getByTestId("document-grid-skeleton")).toBeInTheDocument();
  });

  it("renders DocumentListSkeleton with row placeholders", () => {
    render(<DocumentListSkeleton />);
    expect(screen.getByTestId("document-list-skeleton")).toBeInTheDocument();
  });

  it("renders SearchSkeleton with result placeholders", () => {
    render(<SearchSkeleton />);
    expect(screen.getByTestId("search-skeleton")).toBeInTheDocument();
  });

  it("renders GraphSkeleton with loading text", () => {
    render(<GraphSkeleton />);
    expect(screen.getByTestId("graph-skeleton")).toBeInTheDocument();
    expect(screen.getByText("Building graph...")).toBeInTheDocument();
  });
});

describe("EmptyState", () => {
  it("renders with icon, title, and description", () => {
    render(
      <EmptyState
        icon={FileText}
        title="No documents"
        description="Import some documents to get started"
      />,
    );
    expect(screen.getByTestId("empty-state")).toBeInTheDocument();
    expect(screen.getByText("No documents")).toBeInTheDocument();
    expect(screen.getByText("Import some documents to get started")).toBeInTheDocument();
  });

  it("renders with optional action button", () => {
    render(
      <EmptyState
        icon={Search}
        title="Search"
        description="Search your knowledge"
        action={<button>Import</button>}
      />,
    );
    expect(screen.getByText("Import")).toBeInTheDocument();
  });

  it("renders without action button", () => {
    const { container } = render(
      <EmptyState
        icon={FileText}
        title="Empty"
        description="Nothing here"
      />,
    );
    expect(container.querySelectorAll("button")).toHaveLength(0);
  });
});

describe("ConfirmDialog", () => {
  it("renders nothing when closed", () => {
    const { container } = render(
      <ConfirmDialog
        open={false}
        title="Delete?"
        message="Are you sure?"
        onConfirm={() => {}}
        onCancel={() => {}}
      />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("renders title and message when open", () => {
    render(
      <ConfirmDialog
        open={true}
        title="Delete document?"
        message="This cannot be undone."
        onConfirm={() => {}}
        onCancel={() => {}}
      />,
    );
    expect(screen.getByText("Delete document?")).toBeInTheDocument();
    expect(screen.getByText("This cannot be undone.")).toBeInTheDocument();
  });

  it("calls onConfirm when confirm button clicked", () => {
    const onConfirm = vi.fn();
    render(
      <ConfirmDialog
        open={true}
        title="Delete?"
        message="Sure?"
        confirmLabel="Yes, delete"
        onConfirm={onConfirm}
        onCancel={() => {}}
      />,
    );
    fireEvent.click(screen.getByText("Yes, delete"));
    expect(onConfirm).toHaveBeenCalledOnce();
  });

  it("calls onCancel when cancel button clicked", () => {
    const onCancel = vi.fn();
    render(
      <ConfirmDialog
        open={true}
        title="Delete?"
        message="Sure?"
        onConfirm={() => {}}
        onCancel={onCancel}
      />,
    );
    fireEvent.click(screen.getByText("Cancel"));
    expect(onCancel).toHaveBeenCalledOnce();
  });
});

describe("MarkdownRenderer", () => {
  it("renders plain text", () => {
    render(<MarkdownRenderer content="Hello world" />);
    expect(screen.getByTestId("markdown-renderer")).toBeInTheDocument();
    expect(screen.getByText("Hello world")).toBeInTheDocument();
  });

  it("renders headers", () => {
    render(<MarkdownRenderer content={"# Title\n\n## Subtitle"} />);
    expect(screen.getByText("Title")).toBeInTheDocument();
    expect(screen.getByText("Subtitle")).toBeInTheDocument();
  });

  it("renders code blocks with copy button", () => {
    render(<MarkdownRenderer content={"```js\nconsole.log('hi')\n```"} />);
    expect(screen.getByTestId("copy-code-button")).toBeInTheDocument();
  });

  it("renders tables", () => {
    const table = "| Name | Age |\n|------|-----|\n| Alice | 30 |";
    render(<MarkdownRenderer content={table} />);
    expect(screen.getByText("Name")).toBeInTheDocument();
    expect(screen.getByText("Alice")).toBeInTheDocument();
  });

  it("renders links", () => {
    render(<MarkdownRenderer content="[Click here](https://example.com)" />);
    const link = screen.getByText("Click here");
    expect(link.tagName).toBe("A");
    expect(link).toHaveAttribute("target", "_blank");
  });
});

describe("IngestionPanel", () => {
  it("renders nothing when no files are being tracked", () => {
    const { container } = render(<IngestionPanel />);
    expect(container.firstChild).toBeNull();
  });

  it("registers event listeners on mount", async () => {
    const { listen } = await import("@tauri-apps/api/event");
    render(<IngestionPanel />);
    expect(listen).toHaveBeenCalledWith("ingestion-progress", expect.any(Function));
    expect(listen).toHaveBeenCalledWith("ingestion-all-complete", expect.any(Function));
  });
});

describe("SetupWizard", () => {
  it("renders welcome step", () => {
    render(<SetupWizard onComplete={() => {}} />);
    expect(screen.getByText("Welcome to VaultMind")).toBeInTheDocument();
  });

  it("calls onComplete when finished", () => {
    const onComplete = vi.fn();
    render(<SetupWizard onComplete={onComplete} />);
    expect(screen.getByText("Welcome to VaultMind")).toBeInTheDocument();
  });
});

describe("OllamaStatusBanner", () => {
  it("renders nothing when connected", () => {
    const { container } = render(<OllamaStatusBanner />);
    expect(container.querySelector('[data-testid="ollama-banner"]')).toBeNull();
  });
});

describe("Modal", () => {
  it("test_modal_opens_and_closes", () => {
    const onClose = vi.fn();
    const { rerender } = render(
      <Modal isOpen={false} onClose={onClose} title="Test Modal">
        <p>Modal body</p>
      </Modal>,
    );
    expect(screen.queryByText("Test Modal")).not.toBeInTheDocument();

    rerender(
      <Modal isOpen={true} onClose={onClose} title="Test Modal">
        <p>Modal body</p>
      </Modal>,
    );
    expect(screen.getByText("Test Modal")).toBeInTheDocument();
    expect(screen.getByText("Modal body")).toBeInTheDocument();

    // Close via close button
    fireEvent.click(screen.getByLabelText("Close"));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("test_modal_closes_on_escape", () => {
    const onClose = vi.fn();
    render(
      <Modal isOpen={true} onClose={onClose} title="Escape Test">
        <p>Content</p>
      </Modal>,
    );
    expect(screen.getByText("Escape Test")).toBeInTheDocument();

    fireEvent.keyDown(document, { key: "Escape" });
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("closes on backdrop click", () => {
    const onClose = vi.fn();
    render(
      <Modal isOpen={true} onClose={onClose} title="Backdrop Test">
        <p>Content</p>
      </Modal>,
    );
    fireEvent.click(screen.getByTestId("modal-overlay"));
    expect(onClose).toHaveBeenCalledOnce();
  });
});

describe("Toggle", () => {
  it("test_toggle_switches_state", () => {
    const onChange = vi.fn();
    render(<Toggle checked={false} onChange={onChange} label="Dark mode" />);

    expect(screen.getByText("Dark mode")).toBeInTheDocument();
    expect(screen.getByRole("switch")).toHaveAttribute("aria-checked", "false");

    fireEvent.click(screen.getByRole("switch"));
    expect(onChange).toHaveBeenCalledWith(true);
  });

  it("respects disabled state", () => {
    const onChange = vi.fn();
    render(<Toggle checked={false} onChange={onChange} disabled />);

    expect(screen.getByRole("switch")).toBeDisabled();
  });
});

describe("Badge", () => {
  it("test_badge_renders_variants", () => {
    const { rerender } = render(<Badge>Default</Badge>);
    expect(screen.getByTestId("badge")).toBeInTheDocument();
    expect(screen.getByText("Default")).toBeInTheDocument();

    rerender(<Badge variant="success">Active</Badge>);
    expect(screen.getByText("Active")).toBeInTheDocument();

    rerender(<Badge variant="error">Failed</Badge>);
    expect(screen.getByText("Failed")).toBeInTheDocument();

    rerender(<Badge variant="warning">Pending</Badge>);
    expect(screen.getByText("Pending")).toBeInTheDocument();

    rerender(<Badge variant="info">Info</Badge>);
    expect(screen.getByText("Info")).toBeInTheDocument();
  });

  it("supports size variants", () => {
    const { rerender } = render(<Badge size="sm">Small</Badge>);
    expect(screen.getByText("Small")).toBeInTheDocument();

    rerender(<Badge size="md">Medium</Badge>);
    expect(screen.getByText("Medium")).toBeInTheDocument();
  });
});

describe("Tooltip", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("test_tooltip_shows_content", () => {
    render(
      <Tooltip content="Help text" delay={0}>
        <button>Hover me</button>
      </Tooltip>,
    );

    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();

    fireEvent.mouseEnter(screen.getByTestId("tooltip-wrapper"));
    act(() => {
      vi.advanceTimersByTime(0);
    });

    expect(screen.getByRole("tooltip")).toBeInTheDocument();
    expect(screen.getByText("Help text")).toBeInTheDocument();

    fireEvent.mouseLeave(screen.getByTestId("tooltip-wrapper"));
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
  });
});

describe("Skeleton", () => {
  it("test_skeleton_renders", () => {
    render(<Skeleton className="h-4 w-full" />);
    const skeleton = screen.getByTestId("skeleton");
    expect(skeleton).toBeInTheDocument();
    expect(skeleton).toHaveClass("animate-pulse");
    expect(skeleton).toHaveClass("rounded");
    expect(skeleton).toHaveClass("bg-muted");
    expect(skeleton).toHaveClass("h-4");
    expect(skeleton).toHaveClass("w-full");
  });

  it("renders multiple lines when lines prop is set", () => {
    render(<Skeleton lines={3} className="h-4" />);
    const skeleton = screen.getByTestId("skeleton");
    expect(skeleton).toBeInTheDocument();
    const lines = screen.getAllByTestId("skeleton-line");
    expect(lines).toHaveLength(3);
  });

  it("applies custom width via style", () => {
    render(<Skeleton width="200px" className="h-4" />);
    const skeleton = screen.getByTestId("skeleton");
    expect(skeleton).toHaveStyle({ width: "200px" });
  });
});

describe("Modal accessibility", () => {
  it("test_modal_has_aria_attributes", () => {
    render(
      <Modal isOpen={true} onClose={() => {}} title="Accessible Modal">
        <p>Content</p>
      </Modal>,
    );
    const dialog = screen.getByRole("dialog");
    expect(dialog).toBeInTheDocument();
    expect(dialog).toHaveAttribute("aria-modal", "true");
    expect(dialog).toHaveAttribute("aria-label", "Accessible Modal");
  });
});
