interface PaletteKeydownOptions<T> {
  items: readonly T[];
  selectedIndex: number;
  onSelectedIndexChange: (index: number) => void;
  onSelect: (item: T) => void;
  onClose: () => void;
  onNavigate?: () => void;
}

export function paletteKeydown<T>(event: KeyboardEvent, options: PaletteKeydownOptions<T>): void {
  const ctrlNext = event.ctrlKey && !event.metaKey && event.key.toLowerCase() === "n";
  const ctrlPrevious = event.ctrlKey && !event.metaKey && event.key.toLowerCase() === "p";

  if (event.key === "Escape") {
    consume(event);
    options.onClose();
  } else if (event.key === "ArrowDown" || ctrlNext) {
    consume(event);
    options.onSelectedIndexChange(Math.min(options.selectedIndex + 1, options.items.length - 1));
    options.onNavigate?.();
  } else if (event.key === "ArrowUp" || ctrlPrevious) {
    consume(event);
    options.onSelectedIndexChange(Math.max(options.selectedIndex - 1, 0));
    options.onNavigate?.();
  } else if (event.key === "Enter") {
    consume(event);
    const selected = options.items[options.selectedIndex];
    if (selected) options.onSelect(selected);
  }
}

function consume(event: KeyboardEvent): void {
  event.preventDefault();
  event.stopPropagation();
}
