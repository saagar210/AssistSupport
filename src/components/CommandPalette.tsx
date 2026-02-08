import { useEffect, useState, useCallback } from "react";
import { Command } from "cmdk";
import {
  Network,
  MessageSquare,
  FileText,
  Search,
  Settings,
  Plus,
  Sun,
  Moon,
} from "lucide-react";
import { useAppStore } from "../stores/appStore";
import { useDocumentStore } from "../stores/documentStore";
import { useCollectionStore } from "../stores/collectionStore";
import { useChatStore } from "../stores/chatStore";
import { useTheme } from "../hooks/useTheme";
import type { ViewType, Document } from "../types";

export function CommandPalette() {
  const commandPaletteOpen = useAppStore((s) => s.commandPaletteOpen);
  const toggleCommandPalette = useAppStore((s) => s.toggleCommandPalette);
  const setActiveView = useAppStore((s) => s.setActiveView);
  const setSelectedDocument = useAppStore((s) => s.setSelectedDocument);
  const documents = useDocumentStore((s) => s.documents);
  const collections = useCollectionStore((s) => s.collections);
  const setActiveCollection = useCollectionStore((s) => s.setActiveCollection);
  const activeCollectionId = useCollectionStore((s) => s.activeCollectionId);
  const createConversation = useChatStore((s) => s.createConversation);
  const { theme, setTheme } = useTheme();

  const [search, setSearch] = useState("");

  // Reset search when opening
  useEffect(() => {
    if (commandPaletteOpen) {
      setSearch("");
    }
  }, [commandPaletteOpen]);

  const navigateTo = useCallback(
    (view: ViewType) => {
      setActiveView(view);
      toggleCommandPalette();
    },
    [setActiveView, toggleCommandPalette],
  );

  const handleSelectDocument = useCallback(
    (doc: Document) => {
      setSelectedDocument(doc.id);
      setActiveView("document-detail");
      toggleCommandPalette();
    },
    [setSelectedDocument, setActiveView, toggleCommandPalette],
  );

  const handleSwitchCollection = useCallback(
    (id: string) => {
      setActiveCollection(id);
      toggleCommandPalette();
    },
    [setActiveCollection, toggleCommandPalette],
  );

  const handleNewConversation = useCallback(async () => {
    if (!activeCollectionId) return;
    await createConversation(activeCollectionId, "New Conversation");
    setActiveView("chat");
    toggleCommandPalette();
  }, [activeCollectionId, createConversation, setActiveView, toggleCommandPalette]);

  const handleToggleTheme = useCallback(() => {
    const next = theme === "dark" ? "light" : theme === "light" ? "dark" : "light";
    setTheme(next);
    toggleCommandPalette();
  }, [theme, setTheme, toggleCommandPalette]);

  if (!commandPaletteOpen) return null;

  return (
    <div className="fixed inset-0 z-50">
      <div
        className="absolute inset-0 bg-black/50"
        onClick={toggleCommandPalette}
      />
      <div className="flex items-start justify-center pt-[20vh]">
        <Command
          className="relative w-[520px] overflow-hidden rounded-xl border border-border bg-background shadow-2xl"
          shouldFilter={true}
        >
          <Command.Input
            value={search}
            onValueChange={setSearch}
            placeholder="Type a command or search..."
            className="h-12 w-full border-b border-border bg-transparent px-4 text-sm text-foreground outline-none placeholder:text-muted-foreground"
          />
          <Command.List className="max-h-80 overflow-y-auto p-2">
            <Command.Empty className="py-6 text-center text-sm text-muted-foreground">
              No results found
            </Command.Empty>

            <Command.Group heading="Navigation" className="mb-2">
              <Command.Item
                onSelect={() => navigateTo("graph")}
                className="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm text-foreground data-[selected=true]:bg-accent/10 data-[selected=true]:text-accent"
              >
                <Network size={16} />
                Knowledge Graph
              </Command.Item>
              <Command.Item
                onSelect={() => navigateTo("chat")}
                className="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm text-foreground data-[selected=true]:bg-accent/10 data-[selected=true]:text-accent"
              >
                <MessageSquare size={16} />
                Chat
              </Command.Item>
              <Command.Item
                onSelect={() => navigateTo("documents")}
                className="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm text-foreground data-[selected=true]:bg-accent/10 data-[selected=true]:text-accent"
              >
                <FileText size={16} />
                Documents
              </Command.Item>
              <Command.Item
                onSelect={() => navigateTo("search")}
                className="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm text-foreground data-[selected=true]:bg-accent/10 data-[selected=true]:text-accent"
              >
                <Search size={16} />
                Search
              </Command.Item>
              <Command.Item
                onSelect={() => navigateTo("settings")}
                className="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm text-foreground data-[selected=true]:bg-accent/10 data-[selected=true]:text-accent"
              >
                <Settings size={16} />
                Settings
              </Command.Item>
            </Command.Group>

            <Command.Group heading="Actions" className="mb-2">
              <Command.Item
                onSelect={handleNewConversation}
                className="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm text-foreground data-[selected=true]:bg-accent/10 data-[selected=true]:text-accent"
              >
                <Plus size={16} />
                New Conversation
              </Command.Item>
              <Command.Item
                onSelect={handleToggleTheme}
                className="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm text-foreground data-[selected=true]:bg-accent/10 data-[selected=true]:text-accent"
              >
                {theme === "dark" ? <Sun size={16} /> : <Moon size={16} />}
                Toggle Theme
              </Command.Item>
            </Command.Group>

            {collections.length > 1 && (
              <Command.Group heading="Collections" className="mb-2">
                {collections.map((col) => (
                  <Command.Item
                    key={col.id}
                    onSelect={() => handleSwitchCollection(col.id)}
                    className="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm text-foreground data-[selected=true]:bg-accent/10 data-[selected=true]:text-accent"
                  >
                    <FileText size={16} />
                    {col.name}
                    {col.id === activeCollectionId && (
                      <span className="ml-auto text-[10px] text-accent">Active</span>
                    )}
                  </Command.Item>
                ))}
              </Command.Group>
            )}

            {documents.length > 0 && (
              <Command.Group heading="Documents" className="mb-2">
                {documents.map((doc) => (
                  <Command.Item
                    key={doc.id}
                    onSelect={() => handleSelectDocument(doc)}
                    className="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm text-foreground data-[selected=true]:bg-accent/10 data-[selected=true]:text-accent"
                  >
                    <FileText size={16} />
                    {doc.filename}
                  </Command.Item>
                ))}
              </Command.Group>
            )}
          </Command.List>
        </Command>
      </div>
    </div>
  );
}
