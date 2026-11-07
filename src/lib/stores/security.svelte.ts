import * as prefsBridge from "$lib/bridge/prefs";
import { clipboard } from "./clipboard.svelte";
import { theme } from "./theme.svelte";

let idleLockMinutes = $state(5);
let clipboardClearSeconds = $state(15);
let hideOnBlur = $state(true);

export const security = {
  get hideOnBlur() {
    return hideOnBlur;
  },
  get idleLockMinutes() {
    return idleLockMinutes;
  },
  get idleLockMs() {
    return idleLockMinutes * 60 * 1000;
  },
  get clipboardClearSeconds() {
    return clipboardClearSeconds;
  },
  async load() {
    const prefs = await prefsBridge.prefsGet();
    idleLockMinutes = prefs.idleLockMinutes ?? 5;
    clipboardClearSeconds = prefs.clipboardClearSeconds ?? 15;
    hideOnBlur = prefs.hideOnBlur ?? true;
    clipboard.defaultTimeoutSecs = clipboardClearSeconds;
    theme.hydrate(prefs.theme);
    return prefs;
  },
  async save(idleMins: number, clipboardSecs: number, hideOnBlurEnabled = hideOnBlur) {
    idleLockMinutes = idleMins;
    clipboardClearSeconds = clipboardSecs;
    clipboard.defaultTimeoutSecs = clipboardSecs;
    await prefsBridge.prefsSetSecurity(idleMins, clipboardSecs, hideOnBlurEnabled);
    hideOnBlur = hideOnBlurEnabled;
  },
};
