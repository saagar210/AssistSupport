import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { useToastStore } from "./toastStore";
import { useAppStore } from "./appStore";
import type { Conversation, Message, Citation, PaginatedResponse } from "../types";

interface ChatState {
  conversations: Conversation[];
  activeConversationId: string | null;
  messages: Message[];
  citations: Record<string, Citation[]>;
  streaming: boolean;
  streamingContent: string;
  isGenerating: boolean;
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
    modelOverride?: string,
  ) => Promise<void>;
  cancelGeneration: (conversationId: string) => Promise<void>;
  addStreamingToken: (token: string) => void;
  finishStreaming: (message: Message, newCitations: Citation[]) => void;
  updateConversationTitle: (conversationId: string, title: string) => void;
  clearMessages: () => void;
}

export const useChatStore = create<ChatState>((set, get) => ({
  conversations: [],
  activeConversationId: null,
  messages: [],
  citations: {},
  streaming: false,
  streamingContent: "",
  isGenerating: false,

  fetchConversations: async (collectionId: string) => {
    try {
      const response = await invoke<PaginatedResponse<Conversation>>(
        "list_conversations",
        { collectionId },
      );
      set({ conversations: response.items });
    } catch (error) {
      console.error("Failed to fetch conversations:", error);
      useToastStore.getState().addToast("error", "Failed to fetch conversations: " + String(error));
    }
  },

  setActiveConversation: async (id: string) => {
    set({ activeConversationId: id, messages: [], citations: {} });
    try {
      const response = await invoke<PaginatedResponse<Message>>("get_conversation_messages", {
        conversationId: id,
      });
      set({ messages: response.items });
    } catch (error) {
      console.error("Failed to fetch messages:", error);
      useToastStore.getState().addToast("error", "Failed to fetch messages: " + String(error));
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
      useToastStore.getState().addToast("error", "Failed to create conversation: " + String(error));
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
      useToastStore.getState().addToast("error", "Failed to delete conversation: " + String(error));
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
      useToastStore.getState().addToast("error", "Failed to rename conversation: " + String(error));
    }
  },

  sendMessage: async (
    conversationId: string,
    collectionId: string,
    message: string,
    modelOverride?: string,
  ) => {
    set({ streaming: true, isGenerating: true, streamingContent: "" });
    useAppStore.getState().startLoading();
    try {
      await invoke("send_chat_message", {
        conversationId,
        collectionId,
        userMessage: message,
        modelOverride: modelOverride ?? null,
      });
    } catch (error) {
      console.error("Failed to send message:", error);
      useToastStore.getState().addToast("error", "Failed to send message: " + String(error));
      set({ streaming: false, isGenerating: false, streamingContent: "" });
      useAppStore.getState().stopLoading();
    }
  },

  cancelGeneration: async (conversationId: string) => {
    try {
      await invoke("cancel_chat_generation", { conversationId });
    } catch (error) {
      console.error("Failed to cancel generation:", error);
    }
  },

  addStreamingToken: (token: string) => {
    set((state) => ({ streamingContent: state.streamingContent + token }));
  },

  finishStreaming: (message: Message, newCitations: Citation[]) => {
    set((state) => ({
      streaming: false,
      isGenerating: false,
      streamingContent: "",
      messages: [...state.messages, message],
      citations: {
        ...state.citations,
        [message.id]: newCitations,
      },
    }));
    useAppStore.getState().stopLoading();
  },

  updateConversationTitle: (conversationId: string, title: string) => {
    set((state) => ({
      conversations: state.conversations.map((c) =>
        c.id === conversationId ? { ...c, title } : c,
      ),
    }));
  },

  clearMessages: () => {
    set({ messages: [], citations: {}, activeConversationId: null });
  },
}));
