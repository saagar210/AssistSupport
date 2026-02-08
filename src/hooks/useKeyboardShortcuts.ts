import { useEffect, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../stores/appStore";
import { useCollectionStore } from "../stores/collectionStore";
import { useChatStore } from "../stores/chatStore";
import { useToastStore } from "../stores/toastStore";
import type { ViewType } from "../types";

const VIEW_MAP: Record<string, ViewType> = {
  "1": "graph",
  "2": "chat",
  "3": "documents",
  "4": "search",
};

const SUPPORTED_EXTENSIONS = [
  { name: "All Supported", extensions: ["pdf", "md", "html", "txt", "docx", "csv", "epub"] },
];

export function useKeyboardShortcuts() {
  const setActiveView = useAppStore((state) => state.setActiveView);
  const toggleCommandPalette = useAppStore((state) => state.toggleCommandPalette);
  const activeView = useAppStore((state) => state.activeView);
  const activeCollectionId = useCollectionStore((state) => state.activeCollectionId);
  const createConversation = useChatStore((state) => state.createConversation);
  const addToast = useToastStore((state) => state.addToast);

  const handleFileImport = useCallback(async () => {
    if (!activeCollectionId) {
      addToast("warning", "Select a collection first");
      return;
    }
    try {
      const selected = await open({
        multiple: true,
        filters: SUPPORTED_EXTENSIONS,
      });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        await invoke("ingest_files", {
          collectionId: activeCollectionId,
          filePaths: paths,
        });
        addToast("info", `Ingesting ${paths.length} file(s)...`);
      }
    } catch (error) {
      console.error("File import error:", error);
    }
  }, [activeCollectionId, addToast]);

  const handleNewConversation = useCallback(async () => {
    if (!activeCollectionId) return;
    await createConversation(activeCollectionId, "New Conversation");
    setActiveView("chat");
  }, [activeCollectionId, createConversation, setActiveView]);

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (!e.metaKey && !e.ctrlKey) return;

      const view = VIEW_MAP[e.key];
      if (view) {
        e.preventDefault();
        setActiveView(view);
        return;
      }

      if (e.key === ",") {
        e.preventDefault();
        setActiveView("settings");
        return;
      }

      if (e.key === "k") {
        e.preventDefault();
        toggleCommandPalette();
        return;
      }

      if (e.key === "o") {
        e.preventDefault();
        handleFileImport();
        return;
      }

      if (e.key === "n" && activeView === "chat") {
        e.preventDefault();
        handleNewConversation();
        return;
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [setActiveView, toggleCommandPalette, handleFileImport, handleNewConversation, activeView]);
}
