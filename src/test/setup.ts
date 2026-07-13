import "@testing-library/jest-dom/vitest";

// jsdom has no ResizeObserver; Svelte 5 dimension bindings
// (bind:clientHeight etc.) require one to exist.
class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}
globalThis.ResizeObserver ??= ResizeObserverStub as unknown as typeof ResizeObserver;
