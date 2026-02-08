import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Setting } from "../types";

interface SettingsState {
  settings: Record<string, string>;
  loading: boolean;
  fetchSettings: () => Promise<void>;
  updateSetting: (key: string, value: string) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: {},
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
      set({ loading: false });
    }
  },

  updateSetting: async (key, value) => {
    try {
      await invoke("update_setting", { key, value });
      const current = get().settings;
      set({ settings: { ...current, [key]: value } });
    } catch (error) {
      console.error("Failed to update setting:", error);
    }
  },
}));
