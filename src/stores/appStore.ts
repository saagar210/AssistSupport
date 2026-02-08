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
}));
