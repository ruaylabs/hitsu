import type { SidebarFilter } from "$lib/bridge/types";

let selectedId = $state<string | null>(null);
let filter = $state<SidebarFilter>("all");
let search = $state("");

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
};
