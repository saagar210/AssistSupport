import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { useToastStore } from "./toastStore";
import { useAppStore } from "./appStore";
import type { Document, PaginatedResponse } from "../types";

interface DocumentState {
  documents: Document[];
  loading: boolean;
  error: string | null;
  docCount: number;
  chunkCount: number;
  fetchDocuments: (collectionId: string) => Promise<void>;
  deleteDocument: (id: string, collectionId: string) => Promise<void>;
  fetchStats: (collectionId: string) => Promise<void>;
}

export const useDocumentStore = create<DocumentState>((set, get) => ({
  documents: [],
  loading: false,
  error: null,
  docCount: 0,
  chunkCount: 0,

  fetchDocuments: async (collectionId: string) => {
    set({ loading: true, error: null });
    useAppStore.getState().startLoading();
    try {
      const response = await invoke<PaginatedResponse<Document>>("list_documents", {
        collectionId,
      });
      set({ documents: response.items, loading: false });
    } catch (error) {
      console.error("Failed to fetch documents:", error);
      const errorMsg = "Failed to fetch documents: " + String(error);
      useToastStore.getState().addToast("error", errorMsg);
      set({ loading: false, error: errorMsg });
    } finally {
      useAppStore.getState().stopLoading();
    }
  },

  deleteDocument: async (id: string, collectionId: string) => {
    try {
      await invoke("delete_document", { id });
      await get().fetchDocuments(collectionId);
      await get().fetchStats(collectionId);
    } catch (error) {
      console.error("Failed to delete document:", error);
      useToastStore.getState().addToast("error", "Failed to delete document: " + String(error));
    }
  },

  fetchStats: async (collectionId: string) => {
    try {
      const stats = await invoke<[number, number]>("get_stats", {
        collectionId,
      });
      set({ docCount: stats[0], chunkCount: stats[1] });
    } catch (error) {
      console.error("Failed to fetch stats:", error);
      useToastStore.getState().addToast("error", "Failed to fetch stats: " + String(error));
    }
  },
}));
