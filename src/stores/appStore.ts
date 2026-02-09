import { create } from "zustand";
import type { ViewType } from "../types";

interface AppState {
  activeView: ViewType;
  setActiveView: (view: ViewType) => void;
  sidebarCollapsed: boolean;
  toggleSidebar: () => void;
  selectedDocumentId: string | null;
  setSelectedDocument: (id: string | null) => void;
  commandPaletteOpen: boolean;
  toggleCommandPalette: () => void;
  shortcutCheatsheetOpen: boolean;
  toggleShortcutCheatsheet: () => void;
  globalLoadingCount: number;
  startLoading: () => void;
  stopLoading: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  activeView: "documents",
  setActiveView: (view) => set({ activeView: view }),
  sidebarCollapsed: false,
  toggleSidebar: () =>
    set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
  selectedDocumentId: null,
  setSelectedDocument: (id) => set({ selectedDocumentId: id }),
  commandPaletteOpen: false,
  toggleCommandPalette: () =>
    set((state) => ({ commandPaletteOpen: !state.commandPaletteOpen })),
  shortcutCheatsheetOpen: false,
  toggleShortcutCheatsheet: () =>
    set((state) => ({ shortcutCheatsheetOpen: !state.shortcutCheatsheetOpen })),
  globalLoadingCount: 0,
  startLoading: () =>
    set((state) => ({ globalLoadingCount: state.globalLoadingCount + 1 })),
  stopLoading: () =>
    set((state) => ({ globalLoadingCount: Math.max(0, state.globalLoadingCount - 1) })),
}));
