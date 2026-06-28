import type { AppView } from "$lib/bridge/types";

let view = $state<AppView>("main");

export const app = {
  get view() {
    return view;
  },
  set view(v: AppView) {
    view = v;
  },
  toggleSettings() {
    view = view === "settings" ? "main" : "settings";
  },
};
