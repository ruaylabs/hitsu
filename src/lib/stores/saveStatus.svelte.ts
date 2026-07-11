export type SaveState = "saved" | "dirty" | "saving" | "error";

let state = $state<SaveState>("saved");
let errorMessage = $state("");

export const saveStatus = {
  get state() {
    return state;
  },
  get errorMessage() {
    return errorMessage;
  },
  markSaved() {
    state = "saved";
    errorMessage = "";
  },
  markDirty() {
    state = "dirty";
    errorMessage = "";
  },
  markSaving() {
    state = "saving";
    errorMessage = "";
  },
  markError(message: string) {
    state = "error";
    errorMessage = message;
  },
};
