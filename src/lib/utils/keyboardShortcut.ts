interface ShortcutOptions {
  shift?: boolean;
  platform?: string;
}

function currentPlatform(): string {
  return typeof navigator === "undefined" ? "" : navigator.platform;
}

export function isMacPlatform(platform = currentPlatform()): boolean {
  return platform.toLowerCase().includes("mac");
}

export function keyboardShortcut(key: string, options: ShortcutOptions = {}): string {
  const { shift = false, platform } = options;
  if (isMacPlatform(platform)) {
    const displayKey = key === "Backspace" ? "⌫" : key;
    return `⌘${shift ? "⇧" : ""}${displayKey}`;
  }

  return `Ctrl+${shift ? "Shift+" : ""}${key}`;
}
