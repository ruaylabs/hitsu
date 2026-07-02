export function timeAgo(iso: string): string {
  const now = Date.now();
  const then = new Date(iso).getTime();
  const diffMs = now - then;
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHr = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHr / 24);

  if (diffSec < 60) return "Just now";
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHr < 24) return `${diffHr}h ago`;
  if (diffDay < 30) return `${diffDay}d ago`;
  if (diffDay < 365) return `${Math.floor(diffDay / 30)}mo ago`;
  return `${Math.floor(diffDay / 365)}y ago`;
}

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * Format a credit card number for display.
 *
 * - American Express (cardType === "amex" or starts with 34/37): 4-6-5 pattern
 *   e.g. 3782 822463 10005
 * - All other cards: grouped by 4 digits from the left
 *   e.g. 4111 1111 1111 1111
 *
 * @param raw     Raw card number (digits only, or with separators).
 * @param cardType Optional card type ("amex", "visa", etc.) to pick format.
 */
export function formatCardNumber(raw: string, cardType?: string): string {
  const digits = raw.replace(/\D/g, "");
  if (!digits) return raw;

  const type = cardType?.toLowerCase() ?? "";
  const isAmex = type === "amex" || type === "american express" || /^3[47]/.test(digits);

  if (isAmex) {
    // Amex: 4-6-5
    const parts: string[] = [];
    if (digits.length > 0) parts.push(digits.slice(0, 4));
    if (digits.length > 4) parts.push(digits.slice(4, 10));
    if (digits.length > 10) parts.push(digits.slice(10, 15));
    return parts.join(" ");
  }

  // Others: groups of 4
  const groups: string[] = [];
  for (let i = 0; i < digits.length; i += 4) {
    groups.push(digits.slice(i, i + 4));
  }
  return groups.join(" ");
}
