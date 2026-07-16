import type { Preferences } from "$lib/bridge/prefs";
import * as prefsBridge from "$lib/bridge/prefs";

let foldersEnabled = $state(false);

export const features = {
  get foldersEnabled() {
    return foldersEnabled;
  },
  hydrate(preferences: Preferences) {
    foldersEnabled = preferences.foldersEnabled ?? false;
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
};
