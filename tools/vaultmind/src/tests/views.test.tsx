import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, act } from "@testing-library/react";
import { useAppStore } from "../stores/appStore";
import { useCollectionStore } from "../stores/collectionStore";
import { useDocumentStore } from "../stores/documentStore";
import { useChatStore } from "../stores/chatStore";
import { invoke } from "@tauri-apps/api/core";

const mockedInvoke = vi.mocked(invoke);

// Mock all Tauri-dependent hooks
vi.mock("../hooks/useOllamaStatus", () => ({
  useOllamaStatus: () => ({ connected: false, version: "", loading: false }),
}));

// Mock react-force-graph-2d since it requires canvas
vi.mock("react-force-graph-2d", () => ({
  __esModule: true,
  default: () => <div data-testid="force-graph">Graph</div>,
}));

const defaultCollection = {
  id: "col1",
  name: "General",
  description: "",
  created_at: "2025-01-01",
  updated_at: "2025-01-01",
};

const emptyPaginatedResponse = { items: [], total: 0, page: 1, page_size: 50, has_more: false };

describe("DocumentsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Make invoke return proper paginated data for any call
    mockedInvoke.mockResolvedValue(emptyPaginatedResponse as never);

    useAppStore.setState({ activeView: "documents", selectedDocumentId: null });
    useCollectionStore.setState({
      collections: [defaultCollection],
      activeCollectionId: "col1",
      loading: false,
    });
    useDocumentStore.setState({
      documents: [],
      loading: false,
      docCount: 0,
      chunkCount: 0,
    });
  });

  it("renders empty state with drop zone", async () => {
    const { DocumentsView } = await import("../views/DocumentsView");
    await act(async () => {
      render(<DocumentsView />);
    });
    // The empty state shows "Drop files here" or "Import" button
    const dropText = screen.queryByText(/drop files/i) ?? screen.queryByText(/import/i) ?? screen.queryByText(/no documents/i);
    expect(dropText).toBeTruthy();
  });
});

describe("SearchView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockedInvoke.mockResolvedValue(emptyPaginatedResponse as never);
    useCollectionStore.setState({
      collections: [defaultCollection],
      activeCollectionId: "col1",
    });
  });

  it("renders search input", async () => {
    const { SearchView } = await import("../views/SearchView");
    await act(async () => {
      render(<SearchView />);
    });
    const input = screen.getByPlaceholderText(/search/i);
    expect(input).toBeInTheDocument();
  });
});

describe("ChatView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockedInvoke.mockResolvedValue(emptyPaginatedResponse as never);
    useCollectionStore.setState({
      collections: [defaultCollection],
      activeCollectionId: "col1",
    });
    useChatStore.setState({
      conversations: [],
      activeConversationId: null,
      messages: [],
      citations: {},
      streaming: false,
      streamingContent: "",
    });
  });

  it("renders chat container", async () => {
    const { ChatView } = await import("../views/ChatView");
    await act(async () => {
      render(<ChatView />);
    });
    // Chat view should render with some message area or input
    const chatContainer = document.querySelector('[class*="flex"]');
    expect(chatContainer).toBeTruthy();
  });
});

describe("GraphView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Return proper data structures per command
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_graph") return Promise.resolve({ nodes: [], links: [] });
      if (cmd === "detect_graph_communities") return Promise.resolve([]);
      return Promise.resolve({});
    });
    useCollectionStore.setState({
      collections: [defaultCollection],
      activeCollectionId: "col1",
    });
  });

  it("renders graph container", async () => {
    const { GraphView } = await import("../views/GraphView");
    await act(async () => {
      render(<GraphView />);
    });
    const container = document.querySelector('[class*="flex"]');
    expect(container).toBeTruthy();
  });
});
