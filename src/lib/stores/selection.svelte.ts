import type { SidebarFilter } from "$lib/bridge/types";

let selectedId = $state<string | null>(null);
let filter = $state<SidebarFilter>({ kind: "all" });
let search = $state("");

type NavigationGuard = (navigate: () => void) => boolean;
let navigationGuard: NavigationGuard | null = null;

export const selection = {
  get selectedId() {
    return selectedId;
  },
  set selectedId(v: string | null) {
    selectedId = v;
  },
  get filter() {
    return filter;
  },
  set filter(v: SidebarFilter) {
    filter = v;
  },
  get search() {
    return search;
  },
  set search(v: string) {
    search = v;
  },
  requestNavigation(navigate: () => void) {
    if (navigationGuard && !navigationGuard(navigate)) return false;
    navigate();
    return true;
  },
  setNavigationGuard(guard: NavigationGuard) {
    navigationGuard = guard;
    return () => {
      if (navigationGuard === guard) navigationGuard = null;
    };
  },
};
