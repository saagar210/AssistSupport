import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Document } from "../types";

interface DocumentState {
  documents: Document[];
  loading: boolean;
  docCount: number;
  chunkCount: number;
  fetchDocuments: (collectionId: string) => Promise<void>;
  deleteDocument: (id: string, collectionId: string) => Promise<void>;
  fetchStats: (collectionId: string) => Promise<void>;
}

export const useDocumentStore = create<DocumentState>((set, get) => ({
  documents: [],
  loading: false,
  docCount: 0,
  chunkCount: 0,

  fetchDocuments: async (collectionId: string) => {
    set({ loading: true });
    try {
      const documents = await invoke<Document[]>("list_documents", {
        collectionId,
      });
      set({ documents, loading: false });
    } catch (error) {
      console.error("Failed to fetch documents:", error);
      set({ loading: false });
    }
  },

  deleteDocument: async (id: string, collectionId: string) => {
    try {
      await invoke("delete_document", { id });
      await get().fetchDocuments(collectionId);
      await get().fetchStats(collectionId);
    } catch (error) {
      console.error("Failed to delete document:", error);
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
    }
  },
}));
