import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Conversation, Message, Citation } from "../types";

interface ChatState {
  conversations: Conversation[];
  activeConversationId: string | null;
  messages: Message[];
  citations: Record<string, Citation[]>;
  streaming: boolean;
  streamingContent: string;
  fetchConversations: (collectionId: string) => Promise<void>;
  setActiveConversation: (id: string) => Promise<void>;
  createConversation: (
    collectionId: string,
    title: string,
  ) => Promise<string | null>;
  deleteConversation: (id: string, collectionId: string) => Promise<void>;
  renameConversation: (
    id: string,
    title: string,
    collectionId: string,
  ) => Promise<void>;
  sendMessage: (
    conversationId: string,
    collectionId: string,
    message: string,
  ) => Promise<void>;
  addStreamingToken: (token: string) => void;
  finishStreaming: (message: Message, newCitations: Citation[]) => void;
  clearMessages: () => void;
}

export const useChatStore = create<ChatState>((set, get) => ({
  conversations: [],
  activeConversationId: null,
  messages: [],
  citations: {},
  streaming: false,
  streamingContent: "",

  fetchConversations: async (collectionId: string) => {
    try {
      const conversations = await invoke<Conversation[]>(
        "list_conversations",
        { collectionId },
      );
      set({ conversations });
    } catch (error) {
      console.error("Failed to fetch conversations:", error);
    }
  },

  setActiveConversation: async (id: string) => {
    set({ activeConversationId: id, messages: [], citations: {} });
    try {
      const messages = await invoke<Message[]>("get_conversation_messages", {
        conversationId: id,
      });
      set({ messages });
    } catch (error) {
      console.error("Failed to fetch messages:", error);
    }
  },

  createConversation: async (collectionId: string, title: string) => {
    try {
      const id = await invoke<string>("create_conversation", {
        collectionId,
        title,
      });
      await get().fetchConversations(collectionId);
      await get().setActiveConversation(id);
      return id;
    } catch (error) {
      console.error("Failed to create conversation:", error);
      return null;
    }
  },

  deleteConversation: async (id: string, collectionId: string) => {
    try {
      await invoke("delete_conversation", { id });
      const state = get();
      if (state.activeConversationId === id) {
        set({ activeConversationId: null, messages: [], citations: {} });
      }
      await get().fetchConversations(collectionId);
    } catch (error) {
      console.error("Failed to delete conversation:", error);
    }
  },

  renameConversation: async (
    id: string,
    title: string,
    collectionId: string,
  ) => {
    try {
      await invoke("rename_conversation", { id, title });
      await get().fetchConversations(collectionId);
    } catch (error) {
      console.error("Failed to rename conversation:", error);
    }
  },

  sendMessage: async (
    conversationId: string,
    collectionId: string,
    message: string,
  ) => {
    set({ streaming: true, streamingContent: "" });
    try {
      await invoke("send_chat_message", {
        conversationId,
        collectionId,
        message,
      });
    } catch (error) {
      console.error("Failed to send message:", error);
      set({ streaming: false, streamingContent: "" });
    }
  },

  addStreamingToken: (token: string) => {
    set((state) => ({ streamingContent: state.streamingContent + token }));
  },

  finishStreaming: (message: Message, newCitations: Citation[]) => {
    set((state) => ({
      streaming: false,
      streamingContent: "",
      messages: [...state.messages, message],
      citations: {
        ...state.citations,
        [message.id]: newCitations,
      },
    }));
  },

  clearMessages: () => {
    set({ messages: [], citations: {}, activeConversationId: null });
  },
}));
