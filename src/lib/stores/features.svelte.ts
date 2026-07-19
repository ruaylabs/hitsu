import type { Preferences } from "$lib/bridge/prefs";
import * as prefsBridge from "$lib/bridge/prefs";

let foldersEnabled = $state(false);
let browserIntegrationEnabled = $state(false);

export const features = {
  get foldersEnabled() {
    return foldersEnabled;
  },
  get browserIntegrationEnabled() {
    return browserIntegrationEnabled;
  },
  hydrate(preferences: Preferences) {
    foldersEnabled = preferences.foldersEnabled ?? false;
    browserIntegrationEnabled = preferences.browserIntegrationEnabled ?? false;
  },
  async setFoldersEnabled(enabled: boolean) {
    const previous = foldersEnabled;
    foldersEnabled = enabled;
    try {
      await prefsBridge.prefsSetFoldersEnabled(enabled);
    } catch (error) {
      foldersEnabled = previous;
      throw error;
    }
  },
  async setBrowserIntegrationEnabled(enabled: boolean) {
    const previous = browserIntegrationEnabled;
    browserIntegrationEnabled = enabled;
    try {
      await prefsBridge.prefsSetBrowserIntegrationEnabled(enabled);
    } catch (error) {
      browserIntegrationEnabled = previous;
      throw error;
    }
  },
};
