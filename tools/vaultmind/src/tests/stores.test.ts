import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "../stores/appStore";
import { useCollectionStore } from "../stores/collectionStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useToastStore } from "../stores/toastStore";

describe("appStore", () => {
  beforeEach(() => {
    useAppStore.setState({
      activeView: "documents",
      sidebarCollapsed: false,
      selectedDocumentId: null,
      commandPaletteOpen: false,
    });
  });

  it("initializes with correct defaults", () => {
    const state = useAppStore.getState();
    expect(state.activeView).toBe("documents");
    expect(state.sidebarCollapsed).toBe(false);
    expect(state.selectedDocumentId).toBeNull();
    expect(state.commandPaletteOpen).toBe(false);
  });

  it("setActiveView changes view", () => {
    useAppStore.getState().setActiveView("chat");
    expect(useAppStore.getState().activeView).toBe("chat");
  });

  it("setActiveView to settings", () => {
    useAppStore.getState().setActiveView("settings");
    expect(useAppStore.getState().activeView).toBe("settings");
  });

  it("toggleSidebar works", () => {
    expect(useAppStore.getState().sidebarCollapsed).toBe(false);
    useAppStore.getState().toggleSidebar();
    expect(useAppStore.getState().sidebarCollapsed).toBe(true);
    useAppStore.getState().toggleSidebar();
    expect(useAppStore.getState().sidebarCollapsed).toBe(false);
  });

  it("setSelectedDocument updates id", () => {
    useAppStore.getState().setSelectedDocument("doc-123");
    expect(useAppStore.getState().selectedDocumentId).toBe("doc-123");
  });

  it("toggleCommandPalette works", () => {
    expect(useAppStore.getState().commandPaletteOpen).toBe(false);
    useAppStore.getState().toggleCommandPalette();
    expect(useAppStore.getState().commandPaletteOpen).toBe(true);
  });
});

describe("collectionStore", () => {
  beforeEach(() => {
    useCollectionStore.setState({
      collections: [],
      activeCollectionId: null,
      loading: false,
    });
  });

  it("initializes empty", () => {
    const state = useCollectionStore.getState();
    expect(state.collections).toEqual([]);
    expect(state.activeCollectionId).toBeNull();
    expect(state.loading).toBe(false);
  });

  it("setActiveCollection updates id", () => {
    useCollectionStore.getState().setActiveCollection("col-1");
    expect(useCollectionStore.getState().activeCollectionId).toBe("col-1");
  });
});

describe("settingsStore", () => {
  beforeEach(() => {
    useSettingsStore.setState({
      settings: {},
      loading: false,
    });
  });

  it("initializes empty", () => {
    const state = useSettingsStore.getState();
    expect(state.settings).toEqual({});
    expect(state.loading).toBe(false);
  });
});

describe("toastStore", () => {
  beforeEach(() => {
    useToastStore.setState({ toasts: [] });
  });

  it("addToast adds a toast", () => {
    useToastStore.getState().addToast("success", "Test message");
    const toasts = useToastStore.getState().toasts;
    expect(toasts).toHaveLength(1);
    expect(toasts[0].type).toBe("success");
    expect(toasts[0].message).toBe("Test message");
  });

  it("removeToast removes by id", () => {
    useToastStore.getState().addToast("error", "Error msg");
    const id = useToastStore.getState().toasts[0].id;
    useToastStore.getState().removeToast(id);
    expect(useToastStore.getState().toasts).toHaveLength(0);
  });
});
