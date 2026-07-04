/** Lightweight, dependency-free password strength estimate for the UI.
 *
 *  Returns a 0–4 score plus a short label and a fraction (0–1) for a meter.
 *  This is a *display* heuristic, not a crypto judgment — the master password
 *  is already in plaintext on the frontend (about to be sent over IPC), so
 *  computing this adds no new exposure. It is intentionally simple and
 *  runs fully client-side.
 *
 *  Bands map roughly to: 0 trivial, 1 weak, 2 fair, 3 good, 4 strong.
 */

export type StrengthLevel = 0 | 1 | 2 | 3 | 4;

export interface StrengthResult {
  level: StrengthLevel;
  /** 0–1, for a progress bar. */
  fraction: number;
  label: string;
}

const LABELS: Record<StrengthLevel, string> = {
  0: "Too short",
  1: "Weak",
  2: "Fair",
  3: "Good",
  4: "Strong",
};

const COLORS: Record<StrengthLevel, string> = {
  0: "var(--danger)",
  1: "var(--danger)",
  2: "var(--warning)",
  3: "var(--success)",
  4: "var(--success)",
};

export function estimateStrength(password: string): StrengthResult {
  if (!password) return { level: 0, fraction: 0, label: LABELS[0] };

  const len = password.length;

  // Character-class coverage
  let classes = 0;
  if (/[a-z]/.test(password)) classes++;
  if (/[A-Z]/.test(password)) classes++;
  if (/[0-9]/.test(password)) classes++;
  if (/[^a-zA-Z0-9]/.test(password)) classes++;

  // Crude penalty for very common/low-entropy patterns
  let penalty = 0;
  if (/^(.)\1+$/.test(password)) penalty += 2; // all same char (aaaa…)
  if (/^(0123456789|123456|password|qwerty|abcdef|letmein)$/i.test(password)) penalty += 3;

  // Base score from length
  let score: StrengthLevel;
  if (len < 8) score = 0;
  else if (len < 12) score = 1;
  else if (len < 16) score = 2;
  else if (len < 20) score = 3;
  else score = 4;

  // Reward diversity
  if (classes >= 3) score = Math.max(score, 2) as StrengthLevel;
  if (classes >= 4 && len >= 12) score = Math.max(score, 3) as StrengthLevel;
  if (classes >= 4 && len >= 20) score = 4;

  // Apply penalty
  if (penalty > 0) {
    score = Math.max(0, score - penalty) as StrengthLevel;
  }

  return {
    level: score,
    fraction: (score + 1) / 5,
    label: LABELS[score],
  };
}

export function strengthColor(level: StrengthLevel): string {
  return COLORS[level];
}
