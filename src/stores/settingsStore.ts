import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { useToastStore } from "./toastStore";
import type { Setting, OllamaModel } from "../types";

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
      const settingsList = await invoke<Setting[]>("get_settings");
      const settings: Record<string, string> = {};
      for (const s of settingsList) {
        settings[s.key] = s.value;
      }
      set({ settings, loading: false });
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
