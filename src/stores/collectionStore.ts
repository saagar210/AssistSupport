import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { useToastStore } from "./toastStore";
import type { Collection } from "../types";

interface CollectionState {
  collections: Collection[];
  activeCollectionId: string | null;
  loading: boolean;
  fetchCollections: () => Promise<void>;
  setActiveCollection: (id: string) => void;
  createCollection: (name: string, description: string) => Promise<void>;
  updateCollection: (
    id: string,
    name: string,
    description: string,
  ) => Promise<void>;
  deleteCollection: (id: string) => Promise<void>;
}

export const useCollectionStore = create<CollectionState>((set, get) => ({
  collections: [],
  activeCollectionId: null,
  loading: false,

  fetchCollections: async () => {
    set({ loading: true });
    try {
      const collections = await invoke<Collection[]>("list_collections");
      const currentActive = get().activeCollectionId;
      const activeId =
        currentActive &&
        collections.some((c: Collection) => c.id === currentActive)
          ? currentActive
          : collections.length > 0
            ? collections[0].id
            : null;
      set({ collections, activeCollectionId: activeId, loading: false });
    } catch (error) {
      console.error("Failed to fetch collections:", error);
      useToastStore.getState().addToast("error", "Failed to fetch collections: " + String(error));
      set({ loading: false });
    }
  },

  setActiveCollection: (id) => set({ activeCollectionId: id }),

  createCollection: async (name, description) => {
    try {
      await invoke("create_collection", { name, description });
      await get().fetchCollections();
    } catch (error) {
      console.error("Failed to create collection:", error);
      useToastStore.getState().addToast("error", "Failed to create collection: " + String(error));
    }
  },

  updateCollection: async (id, name, description) => {
    try {
      await invoke("update_collection", { id, name, description });
      await get().fetchCollections();
    } catch (error) {
      console.error("Failed to update collection:", error);
      useToastStore.getState().addToast("error", "Failed to update collection: " + String(error));
    }
  },

  deleteCollection: async (id) => {
    try {
      await invoke("delete_collection", { id });
      await get().fetchCollections();
    } catch (error) {
      console.error("Failed to delete collection:", error);
      useToastStore.getState().addToast("error", "Failed to delete collection: " + String(error));
    }
  },
}));
