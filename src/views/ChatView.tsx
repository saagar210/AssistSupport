import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import {
  MessageSquare,
  Plus,
  Send,
  Loader2,
  PanelLeftClose,
  PanelLeftOpen,
  PanelRightClose,
  PanelRightOpen,
  Trash2,
  Edit3,
  FileText,
  BookOpen,
  Square,
  RefreshCw,
  ChevronDown,
  ChevronRight,
  Download,
} from "lucide-react";
import { useCollectionStore } from "../stores/collectionStore";
import { useChatStore } from "../stores/chatStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useToastStore } from "../stores/toastStore";
import { MarkdownRenderer } from "../components/MarkdownRenderer";
import type { Message, Citation } from "../types";

interface ChatTokenPayload {
  token: string;
}

interface ChatCompletePayload {
  message: Message;
  citations: Citation[];
}

export function ChatView() {
  const activeCollectionId = useCollectionStore((s) => s.activeCollectionId);
  const conversations = useChatStore((s) => s.conversations);
  const activeConversationId = useChatStore((s) => s.activeConversationId);
  const messages = useChatStore((s) => s.messages);
  const citations = useChatStore((s) => s.citations);
  const streaming = useChatStore((s) => s.streaming);
  const streamingContent = useChatStore((s) => s.streamingContent);
  const fetchConversations = useChatStore((s) => s.fetchConversations);
  const setActiveConversation = useChatStore((s) => s.setActiveConversation);
  const createConversation = useChatStore((s) => s.createConversation);
  const deleteConversation = useChatStore((s) => s.deleteConversation);
  const renameConversation = useChatStore((s) => s.renameConversation);
  const sendMessage = useChatStore((s) => s.sendMessage);
  const cancelGeneration = useChatStore((s) => s.cancelGeneration);
  const addStreamingToken = useChatStore((s) => s.addStreamingToken);
  const finishStreaming = useChatStore((s) => s.finishStreaming);
  const updateConversationTitle = useChatStore((s) => s.updateConversationTitle);
  const isGenerating = useChatStore((s) => s.isGenerating);
  const models = useSettingsStore((s) => s.models);
  const addToast = useToastStore((s) => s.addToast);

  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [selectedModel, setSelectedModel] = useState<string | undefined>(undefined);
  const [expandedCitations, setExpandedCitations] = useState<Set<string>>(new Set());
  const [sourcesOpen, setSourcesOpen] = useState(true);
  const [input, setInput] = useState("");
  const [selectedMessageId, setSelectedMessageId] = useState<string | null>(null);
  const [contextMenuId, setContextMenuId] = useState<string | null>(null);
  const [renameId, setRenameId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (activeCollectionId) {
      fetchConversations(activeCollectionId);
    }
  }, [activeCollectionId, fetchConversations]);

  // Listen for streaming tokens, completion, cancellation, and title updates
  useEffect(() => {
    let mounted = true;
    let cleanupToken: (() => void) | null = null;
    let cleanupComplete: (() => void) | null = null;
    let cleanupCancelled: (() => void) | null = null;
    let cleanupTitle: (() => void) | null = null;

    listen<ChatTokenPayload>("chat-token", (event) => {
      if (mounted) addStreamingToken(event.payload.token);
    }).then((fn) => { cleanupToken = fn; });

    listen<ChatCompletePayload>("chat-complete", (event) => {
      if (mounted) finishStreaming(event.payload.message, event.payload.citations);
    }).then((fn) => { cleanupComplete = fn; });

    listen("chat-cancelled", () => {
      if (mounted) {
        useChatStore.setState({ streaming: false, isGenerating: false });
      }
    }).then((fn) => { cleanupCancelled = fn; });

    listen<{ conversationId: string; title: string }>("conversation-title-updated", (event) => {
      if (mounted) {
        updateConversationTitle(event.payload.conversationId, event.payload.title);
      }
    }).then((fn) => { cleanupTitle = fn; });

    return () => {
      mounted = false;
      cleanupToken?.();
      cleanupComplete?.();
      cleanupCancelled?.();
      cleanupTitle?.();
    };
  }, [addStreamingToken, finishStreaming, updateConversationTitle]);

  // Scroll to bottom on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingContent]);

  // Auto-resize textarea
  const handleInputChange = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      setInput(e.target.value);
      const el = e.target;
      el.style.height = "auto";
      el.style.height = `${Math.min(el.scrollHeight, 160)}px`;
    },
    [],
  );

  const handleSend = useCallback(async () => {
    if (!input.trim() || !activeCollectionId || streaming) return;

    let convId = activeConversationId;
    if (!convId) {
      const firstLine = input.trim().slice(0, 50);
      convId = await createConversation(activeCollectionId, firstLine);
      if (!convId) {
        addToast("error", "Failed to create conversation");
        return;
      }
    }

    const userMessage: Message = {
      id: `temp-${Date.now()}`,
      conversation_id: convId,
      role: "user",
      content: input.trim(),
      created_at: new Date().toISOString(),
    };

    useChatStore.setState((state) => ({
      messages: [...state.messages, userMessage],
    }));

    const msg = input.trim();
    setInput("");
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }

    await sendMessage(convId, activeCollectionId, msg, selectedModel);
  }, [
    input,
    activeCollectionId,
    activeConversationId,
    streaming,
    selectedModel,
    createConversation,
    sendMessage,
    addToast,
  ]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend],
  );

  const handleNewConversation = useCallback(async () => {
    if (!activeCollectionId) return;
    await createConversation(activeCollectionId, "New Conversation");
  }, [activeCollectionId, createConversation]);

  const handleDeleteConversation = useCallback(
    async (id: string) => {
      if (!activeCollectionId) return;
      await deleteConversation(id, activeCollectionId);
      setContextMenuId(null);
    },
    [activeCollectionId, deleteConversation],
  );

  const handleRenameStart = useCallback(
    (id: string, currentTitle: string) => {
      setRenameId(id);
      setRenameValue(currentTitle);
      setContextMenuId(null);
    },
    [],
  );

  const handleRenameSubmit = useCallback(async () => {
    if (!renameId || !activeCollectionId || !renameValue.trim()) return;
    await renameConversation(renameId, renameValue.trim(), activeCollectionId);
    setRenameId(null);
    setRenameValue("");
  }, [renameId, renameValue, activeCollectionId, renameConversation]);

  const handleStopGeneration = useCallback(async () => {
    if (!activeConversationId) return;
    await cancelGeneration(activeConversationId);
  }, [activeConversationId, cancelGeneration]);

  const handleRegenerate = useCallback(async () => {
    if (!activeConversationId || !activeCollectionId || streaming) return;
    try {
      const lastUserContent = await invoke<string>("delete_last_assistant_message", {
        conversationId: activeConversationId,
      });
      // Remove last assistant message from local state
      useChatStore.setState((state) => {
        const msgs = [...state.messages];
        for (let i = msgs.length - 1; i >= 0; i--) {
          if (msgs[i].role === "assistant") {
            msgs.splice(i, 1);
            break;
          }
        }
        return { messages: msgs };
      });
      await sendMessage(activeConversationId, activeCollectionId, lastUserContent, selectedModel);
    } catch (error) {
      addToast("error", "Failed to regenerate: " + String(error));
    }
  }, [activeConversationId, activeCollectionId, streaming, selectedModel, sendMessage, addToast]);

  const handleExport = useCallback(async () => {
    if (!activeConversationId) return;
    try {
      const markdown = await invoke<string>("export_conversation_markdown", {
        conversationId: activeConversationId,
      });
      const { save } = await import("@tauri-apps/plugin-dialog");
      const path = await save({
        defaultPath: "conversation.md",
        filters: [{ name: "Markdown", extensions: ["md"] }],
      });
      if (path) {
        const { writeTextFile } = await import("@tauri-apps/plugin-fs");
        await writeTextFile(path, markdown);
        addToast("success", "Conversation exported");
      }
    } catch (error) {
      addToast("error", "Export failed: " + String(error));
    }
  }, [activeConversationId, addToast]);

  const toggleCitationExpand = useCallback((citId: string) => {
    setExpandedCitations((prev) => {
      const next = new Set(prev);
      if (next.has(citId)) {
        next.delete(citId);
      } else {
        next.add(citId);
      }
      return next;
    });
  }, []);

  const selectedCitations =
    selectedMessageId && citations[selectedMessageId]
      ? citations[selectedMessageId]
      : [];

  const sortedConversations = useMemo(
    () => [...conversations].sort(
      (a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime(),
    ),
    [conversations],
  );

  if (!activeCollectionId) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-4 text-muted-foreground">
        <MessageSquare size={48} strokeWidth={1.5} />
        <p className="text-sm">Select a collection to start chatting</p>
      </div>
    );
  }

  return (
    <div className="flex flex-1 overflow-hidden">
      {/* Conversation Sidebar */}
      {sidebarOpen && (
        <div className="flex w-[250px] shrink-0 flex-col border-r border-border bg-muted/30">
          <div className="flex items-center justify-between border-b border-border px-3 py-2">
            <button
              onClick={handleNewConversation}
              className="flex items-center gap-1.5 rounded px-2 py-1 text-xs font-medium text-accent transition-colors hover:bg-accent/10"
            >
              <Plus size={14} />
              New Conversation
            </button>
            <button
              onClick={() => setSidebarOpen(false)}
              aria-label="Hide conversations sidebar"
              className="rounded p-1 text-muted-foreground hover:bg-muted"
            >
              <PanelLeftClose size={14} />
            </button>
          </div>
          <div className="flex-1 overflow-y-auto">
            {sortedConversations.length === 0 ? (
              <p className="p-3 text-xs text-muted-foreground">No conversations yet</p>
            ) : (
              sortedConversations.map((conv) => (
                <div
                  key={conv.id}
                  className="relative"
                >
                  {renameId === conv.id ? (
                    <div className="px-2 py-1">
                      <input
                        type="text"
                        value={renameValue}
                        onChange={(e) => setRenameValue(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") handleRenameSubmit();
                          if (e.key === "Escape") setRenameId(null);
                        }}
                        onBlur={handleRenameSubmit}
                        className="w-full rounded border border-accent bg-background px-2 py-1 text-xs text-foreground outline-none"
                        autoFocus
                      />
                    </div>
                  ) : (
                    <button
                      onClick={() => setActiveConversation(conv.id)}
                      onContextMenu={(e) => {
                        e.preventDefault();
                        setContextMenuId(contextMenuId === conv.id ? null : conv.id);
                      }}
                      className={`w-full truncate px-3 py-2 text-left text-xs transition-colors ${
                        activeConversationId === conv.id
                          ? "bg-accent/10 text-accent"
                          : "text-muted-foreground hover:bg-muted hover:text-foreground"
                      }`}
                    >
                      {conv.title}
                    </button>
                  )}

                  {contextMenuId === conv.id && (
                    <div className="absolute left-2 top-8 z-10 w-32 rounded-md border border-border bg-background py-1 shadow-lg">
                      <button
                        onClick={() => handleRenameStart(conv.id, conv.title)}
                        className="flex w-full items-center gap-2 px-3 py-1.5 text-xs text-foreground hover:bg-muted"
                      >
                        <Edit3 size={12} /> Rename
                      </button>
                      <button
                        onClick={() => handleDeleteConversation(conv.id)}
                        className="flex w-full items-center gap-2 px-3 py-1.5 text-xs text-destructive hover:bg-muted"
                      >
                        <Trash2 size={12} /> Delete
                      </button>
                    </div>
                  )}
                </div>
              ))
            )}
          </div>
        </div>
      )}

      {/* Chat Area */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {/* Chat Toolbar */}
        <div className="flex items-center gap-1 border-b border-border px-2 py-1">
          {!sidebarOpen && (
            <button
              onClick={() => setSidebarOpen(true)}
              className="rounded p-1 text-muted-foreground hover:bg-muted"
              title="Show conversations"
              aria-label="Show conversations sidebar"
            >
              <PanelLeftOpen size={14} />
            </button>
          )}
          {/* Model selector */}
          {models.length > 0 && (
            <select
              value={selectedModel ?? ""}
              onChange={(e) => setSelectedModel(e.target.value || undefined)}
              className="rounded border border-border bg-background px-2 py-0.5 text-xs text-foreground outline-none"
            >
              <option value="">Default model</option>
              {models.map((m) => (
                <option key={m.name} value={m.name}>
                  {m.name}
                </option>
              ))}
            </select>
          )}
          <div className="flex-1" />
          <button
            onClick={handleExport}
            disabled={!activeConversationId || messages.length === 0}
            className="rounded p-1 text-muted-foreground hover:bg-muted disabled:opacity-50"
            title="Export conversation"
            aria-label="Export conversation"
          >
            <Download size={14} />
          </button>
          <button
            onClick={() => setSourcesOpen(!sourcesOpen)}
            className="rounded p-1 text-muted-foreground hover:bg-muted"
            title={sourcesOpen ? "Hide sources" : "Show sources"}
            aria-label={sourcesOpen ? "Hide sources panel" : "Show sources panel"}
          >
            {sourcesOpen ? <PanelRightClose size={14} /> : <PanelRightOpen size={14} />}
          </button>
        </div>

        {/* Messages */}
        <div className="flex-1 overflow-y-auto px-4 py-4">
          {messages.length === 0 && !streaming ? (
            <div className="flex h-full flex-col items-center justify-center gap-4 text-muted-foreground">
              <BookOpen size={40} strokeWidth={1.5} />
              <h3 className="text-sm font-medium text-foreground">
                Start a conversation
              </h3>
              <p className="text-xs">Ask questions about your documents</p>
              <div className="mt-2 space-y-2">
                {[
                  "Summarize the key findings",
                  "What are the main topics covered?",
                  "Compare the approaches discussed",
                ].map((suggestion) => (
                  <button
                    key={suggestion}
                    onClick={() => setInput(suggestion)}
                    className="block w-full rounded-lg border border-border px-4 py-2 text-left text-xs text-muted-foreground transition-colors hover:border-accent/50 hover:text-foreground"
                  >
                    {suggestion}
                  </button>
                ))}
              </div>
            </div>
          ) : (
            <div className="mx-auto max-w-3xl space-y-4">
              {messages.map((msg) => (
                <motion.div
                  key={msg.id}
                  initial={{ opacity: 0, y: 12 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.2 }}
                  className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}
                >
                  <div
                    className={`max-w-[80%] rounded-lg px-4 py-2.5 text-sm ${
                      msg.role === "user"
                        ? "bg-accent text-accent-foreground"
                        : "border border-border bg-card text-card-foreground"
                    }`}
                    onClick={() => {
                      if (msg.role === "assistant") {
                        setSelectedMessageId(
                          selectedMessageId === msg.id ? null : msg.id,
                        );
                      }
                    }}
                  >
                    {msg.role === "assistant" ? (
                      <MarkdownRenderer content={msg.content} />
                    ) : (
                      <p className="whitespace-pre-wrap leading-relaxed">{msg.content}</p>
                    )}
                    {msg.role === "assistant" && citations[msg.id] && (
                      <div className="mt-2 space-y-1">
                        <div className="flex flex-wrap gap-1">
                          {citations[msg.id].map((cit) => (
                            <button
                              key={cit.id}
                              onClick={(e) => {
                                e.stopPropagation();
                                toggleCitationExpand(cit.id);
                              }}
                              className="inline-flex items-center gap-1 rounded bg-accent/10 px-1.5 py-0.5 text-[10px] text-accent hover:bg-accent/20 transition-colors"
                            >
                              <FileText size={8} />
                              {cit.document_title}
                              {cit.section_title && ` · ${cit.section_title}`}
                              {expandedCitations.has(cit.id) ? <ChevronDown size={8} /> : <ChevronRight size={8} />}
                            </button>
                          ))}
                        </div>
                        {citations[msg.id]
                          .filter((cit) => expandedCitations.has(cit.id))
                          .map((cit) => (
                            <div
                              key={`expanded-${cit.id}`}
                              className="rounded border border-border bg-muted/50 p-2 text-xs text-muted-foreground"
                            >
                              <p className="leading-relaxed">{cit.snippet}</p>
                              <p className="mt-1 text-[10px] text-accent">
                                {Math.round(cit.relevance_score * 100)}% relevant
                                {cit.page_number ? ` · Page ${cit.page_number}` : ""}
                              </p>
                            </div>
                          ))}
                      </div>
                    )}
                  </div>
                </motion.div>
              ))}

              {/* Regenerate button */}
              {!streaming && messages.length > 0 && messages[messages.length - 1].role === "assistant" && (
                <div className="flex justify-start">
                  <button
                    onClick={handleRegenerate}
                    className="flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                  >
                    <RefreshCw size={12} />
                    Regenerate
                  </button>
                </div>
              )}

              {/* Streaming Message */}
              {streaming && (
                <div className="flex justify-start">
                  <div className="max-w-[80%] rounded-lg border border-border bg-card px-4 py-2.5 text-sm text-card-foreground">
                    {streamingContent ? (
                      <div>
                        <MarkdownRenderer content={streamingContent} />
                        <span className="ml-0.5 inline-block h-4 w-0.5 animate-pulse bg-accent" />
                      </div>
                    ) : (
                      <div className="flex items-center gap-2 text-muted-foreground">
                        <Loader2 size={14} className="animate-spin" />
                        <span className="text-xs">Thinking...</span>
                      </div>
                    )}
                  </div>
                </div>
              )}

              <div ref={messagesEndRef} />
            </div>
          )}
          <div aria-live="polite" className="sr-only">
            {streaming && !streamingContent && "AI is thinking..."}
            {streaming && streamingContent && "AI is responding..."}
          </div>
        </div>

        {/* Input Area */}
        <div className="border-t border-border px-4 py-3">
          <div className="mx-auto flex max-w-3xl items-end gap-2">
            <textarea
              ref={textareaRef}
              value={input}
              onChange={handleInputChange}
              onKeyDown={handleKeyDown}
              placeholder="Ask a question... (Cmd+Enter to send)"
              disabled={streaming}
              rows={1}
              className="flex-1 resize-none rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none transition-colors focus:border-accent focus:ring-1 focus:ring-accent disabled:opacity-50"
            />
            {isGenerating ? (
              <button
                onClick={handleStopGeneration}
                className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-destructive text-white transition-colors hover:bg-destructive/90"
                title="Stop generation"
                aria-label="Stop generation"
              >
                <Square size={16} />
              </button>
            ) : (
              <button
                onClick={handleSend}
                disabled={!input.trim()}
                className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-accent text-accent-foreground transition-colors hover:bg-accent/90 disabled:opacity-50"
                aria-label="Send message"
              >
                <Send size={16} />
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Sources Panel */}
      {sourcesOpen && (
        <div className="flex w-[300px] shrink-0 flex-col border-l border-border bg-muted/30">
          <div className="border-b border-border px-3 py-2">
            <h3 className="text-xs font-semibold text-foreground">Sources</h3>
          </div>
          <div className="flex-1 overflow-y-auto p-3">
            {selectedCitations.length === 0 ? (
              <p className="text-xs text-muted-foreground">
                {selectedMessageId
                  ? "No citations for this message"
                  : "Click an assistant message to view its sources"}
              </p>
            ) : (
              <div className="space-y-2">
                {selectedCitations.map((cit) => (
                  <div
                    key={cit.id}
                    className="rounded-lg border border-border bg-card p-3"
                  >
                    <div className="mb-1 flex items-center gap-2">
                      <FileText size={12} className="text-accent" />
                      <span className="truncate text-xs font-medium text-card-foreground">
                        {cit.document_title}
                      </span>
                    </div>
                    {cit.section_title && (
                      <p className="mb-1 text-[10px] text-accent">
                        {cit.section_title}
                      </p>
                    )}
                    {cit.page_number && (
                      <p className="mb-1 text-[10px] text-muted-foreground">
                        Page {cit.page_number}
                      </p>
                    )}
                    <p className="text-xs leading-relaxed text-muted-foreground">
                      {cit.snippet}
                    </p>
                    <div className="mt-1 text-right">
                      <span className="text-[10px] text-accent">
                        {Math.round(cit.relevance_score * 100)}% relevant
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Click-away handler for context menus */}
      {contextMenuId && (
        <div
          className="fixed inset-0 z-0"
          onClick={() => setContextMenuId(null)}
        />
      )}
    </div>
  );
}
