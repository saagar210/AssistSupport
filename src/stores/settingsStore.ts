import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { useToastStore } from "./toastStore";
import type { OllamaModel } from "../types";

interface SettingsState {
  settings: Record<string, string>;
  models: OllamaModel[];
  loading: boolean;
  fetchSettings: () => Promise<void>;
  fetchModels: () => Promise<void>;
  updateSetting: (key: string, value: string) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: {},
  models: [],
  loading: false,

  fetchSettings: async () => {
    set({ loading: true });
    try {
      const settingsMap = await invoke<Record<string, string>>("get_settings");
      set({ settings: settingsMap, loading: false });
    } catch (error) {
      console.error("Failed to fetch settings:", error);
      useToastStore.getState().addToast("error", "Failed to fetch settings: " + String(error));
      set({ loading: false });
    }
  },

  fetchModels: async () => {
    try {
      const models = await invoke<OllamaModel[]>("list_ollama_models");
      set({ models });
    } catch {
      // Silently fail — models just won't be available in selector
    }
  },

  updateSetting: async (key, value) => {
    try {
      await invoke("update_setting", { key, value });
      const current = get().settings;
      set({ settings: { ...current, [key]: value } });
    } catch (error) {
      console.error("Failed to update setting:", error);
      useToastStore.getState().addToast("error", "Failed to update setting: " + String(error));
    }
  },
}));
