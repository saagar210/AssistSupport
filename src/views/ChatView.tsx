import { useState, useEffect, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
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
} from "lucide-react";
import { useCollectionStore } from "../stores/collectionStore";
import { useChatStore } from "../stores/chatStore";
import { useToastStore } from "../stores/toastStore";
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
  const addStreamingToken = useChatStore((s) => s.addStreamingToken);
  const finishStreaming = useChatStore((s) => s.finishStreaming);
  const addToast = useToastStore((s) => s.addToast);

  const [sidebarOpen, setSidebarOpen] = useState(true);
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

  // Listen for streaming tokens
  useEffect(() => {
    const unlistenToken = listen<ChatTokenPayload>("chat-token", (event) => {
      addStreamingToken(event.payload.token);
    });

    const unlistenComplete = listen<ChatCompletePayload>("chat-complete", (event) => {
      finishStreaming(event.payload.message, event.payload.citations);
    });

    return () => {
      unlistenToken.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
    };
  }, [addStreamingToken, finishStreaming]);

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

    await sendMessage(convId, activeCollectionId, msg);
  }, [
    input,
    activeCollectionId,
    activeConversationId,
    streaming,
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

  const selectedCitations =
    selectedMessageId && citations[selectedMessageId]
      ? citations[selectedMessageId]
      : [];

  const sortedConversations = [...conversations].sort(
    (a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime(),
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
            >
              <PanelLeftOpen size={14} />
            </button>
          )}
          <div className="flex-1" />
          <button
            onClick={() => setSourcesOpen(!sourcesOpen)}
            className="rounded p-1 text-muted-foreground hover:bg-muted"
            title={sourcesOpen ? "Hide sources" : "Show sources"}
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
                <div
                  key={msg.id}
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
                    <p className="whitespace-pre-wrap leading-relaxed">{msg.content}</p>
                    {msg.role === "assistant" && citations[msg.id] && (
                      <div className="mt-2 flex flex-wrap gap-1">
                        {citations[msg.id].map((cit) => (
                          <span
                            key={cit.id}
                            className="inline-flex items-center gap-1 rounded bg-accent/10 px-1.5 py-0.5 text-[10px] text-accent"
                          >
                            <FileText size={8} />
                            {cit.document_title}
                          </span>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              ))}

              {/* Streaming Message */}
              {streaming && (
                <div className="flex justify-start">
                  <div className="max-w-[80%] rounded-lg border border-border bg-card px-4 py-2.5 text-sm text-card-foreground">
                    {streamingContent ? (
                      <p className="whitespace-pre-wrap leading-relaxed">
                        {streamingContent}
                        <span className="ml-0.5 inline-block h-4 w-0.5 animate-pulse bg-accent" />
                      </p>
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
            <button
              onClick={handleSend}
              disabled={streaming || !input.trim()}
              className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-accent text-accent-foreground transition-colors hover:bg-accent/90 disabled:opacity-50"
            >
              {streaming ? (
                <Loader2 size={16} className="animate-spin" />
              ) : (
                <Send size={16} />
              )}
            </button>
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
