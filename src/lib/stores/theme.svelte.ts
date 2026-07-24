import * as prefsBridge from "$lib/bridge/prefs";
import type { ThemePreference } from "$lib/bridge/prefs";

let preference = $state<ThemePreference>("system");

function apply(value: ThemePreference) {
  preference = value;
  if (typeof document === "undefined") return;
  if (value === "system") document.documentElement.removeAttribute("data-theme");
  else document.documentElement.dataset.theme = value;
}

export const theme = {
  get preference() {
    return preference;
  },
  hydrate(value: ThemePreference | undefined) {
    apply(value ?? "system");
  },
  async save(value: ThemePreference) {
    const previous = preference;
    apply(value);
    try {
      await prefsBridge.prefsSetTheme(value);
    } catch (error) {
      apply(previous);
      throw error;
    }
  },
};
