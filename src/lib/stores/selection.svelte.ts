import type { SidebarFilter } from "$lib/bridge/types";

export type SelectionMode = "immediate" | "keyboard";

let selectedId = $state<string | null>(null);
let detailFetchMode = $state<SelectionMode>("immediate");
let filter = $state<SidebarFilter>({ kind: "all" });
let search = $state("");

type NavigationGuard = (navigate: () => void) => boolean;
let navigationGuard: NavigationGuard | null = null;

export const selection = {
  get selectedId() {
    return selectedId;
  },
  set selectedId(v: string | null) {
    detailFetchMode = "immediate";
    selectedId = v;
  },
  get detailFetchMode() {
    return detailFetchMode;
  },
  select(id: string | null, mode: SelectionMode = "immediate") {
    detailFetchMode = mode;
    selectedId = id;
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
