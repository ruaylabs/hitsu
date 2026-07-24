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

type StrengthLevel = 0 | 1 | 2 | 3 | 4;

interface StrengthResult {
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

/** Top common passwords and base words (drawn from public breach top-lists).
 *  Matched against the whole password and against its letter core after
 *  lowercasing, undoing leetspeak, and stripping digit/symbol padding — so
 *  `Password123!` and `qwerty2024` are caught, not just exact `password`. */
const COMMON_PASSWORDS = new Set([
  "123456",
  "1234567",
  "12345678",
  "123456789",
  "1234567890",
  "12345",
  "111111",
  "123123",
  "000000",
  "121212",
  "654321",
  "666666",
  "112233",
  "password",
  "passwort",
  "qwerty",
  "qwertyuiop",
  "qwertz",
  "azerty",
  "asdfghjkl",
  "asdfgh",
  "zxcvbnm",
  "qazwsx",
  "abc123",
  "abcdef",
  "abcd1234",
  "iloveyou",
  "admin",
  "welcome",
  "login",
  "letmein",
  "dragon",
  "monkey",
  "football",
  "baseball",
  "basketball",
  "soccer",
  "hockey",
  "superman",
  "batman",
  "trustno1",
  "master",
  "shadow",
  "sunshine",
  "princess",
  "flower",
  "hello",
  "freedom",
  "whatever",
  "ninja",
  "mustang",
  "jordan",
  "harley",
  "hunter",
  "ranger",
  "buster",
  "tigger",
  "pepper",
  "ginger",
  "cookie",
  "cheese",
  "banana",
  "orange",
  "purple",
  "silver",
  "golden",
  "diamond",
  "monster",
  "killer",
  "cowboy",
  "angel",
  "lovely",
  "secret",
  "summer",
  "winter",
  "spring",
  "autumn",
  "starwars",
  "pokemon",
  "naruto",
  "minecraft",
  "computer",
  "internet",
  "samsung",
  "google",
  "liverpool",
  "chelsea",
  "arsenal",
  "barcelona",
  "thomas",
  "robert",
  "michael",
  "charlie",
  "daniel",
  "andrew",
  "joshua",
  "matthew",
  "jessica",
  "jennifer",
  "michelle",
  "ashley",
  "amanda",
  "nicole",
  "hannah",
]);

/** Common leetspeak substitutions, undone before dictionary lookup. */
const LEET_MAP: Record<string, string> = {
  "@": "a",
  "4": "a",
  "3": "e",
  "1": "i",
  "!": "i",
  "0": "o",
  $: "s",
  "5": "s",
  "7": "t",
};

/** The common-password base this password is built on, or null.
 *  Checks the lowercased password and its de-leeted form, each both whole and
 *  with leading/trailing non-letters (digit/symbol padding) stripped. */
function commonCore(password: string): string | null {
  const lower = password.toLowerCase();
  const deleeted = lower.replace(/[@43!10$57]/g, (c) => LEET_MAP[c]);
  for (const variant of [lower, deleeted]) {
    if (COMMON_PASSWORDS.has(variant)) return variant;
    const core = variant.replace(/^[^a-z]+|[^a-z]+$/g, "");
    if (core && COMMON_PASSWORDS.has(core)) return core;
  }
  return null;
}

/** True for pure ascending/descending character runs like `abcdefgh`. */
function isSequentialRun(lower: string): boolean {
  if (lower.length < 4) return false;
  const step = lower.charCodeAt(1) - lower.charCodeAt(0);
  if (Math.abs(step) !== 1) return false;
  for (let i = 2; i < lower.length; i++) {
    if (lower.charCodeAt(i) - lower.charCodeAt(i - 1) !== step) return false;
  }
  return true;
}

export function estimateStrength(password: string): StrengthResult {
  if (!password) return { level: 0, fraction: 0, label: LABELS[0] };

  const len = password.length;
  const lower = password.toLowerCase();

  // Character-class coverage
  let classes = 0;
  if (/[a-z]/.test(password)) classes++;
  if (/[A-Z]/.test(password)) classes++;
  if (/[0-9]/.test(password)) classes++;
  if (/[^a-zA-Z0-9]/.test(password)) classes++;

  // Base score from length
  let score: StrengthLevel;
  if (len < 8) score = 0;
  else if (len < 12) score = 1;
  else if (len < 16) score = 2;
  else if (len < 20) score = 3;
  else score = 4;

  // Reward diversity — never below 8 chars, so level 0 stays sticky under
  // the backend's minimum (MIN_MASTER_PASSWORD_LEN in vault.rs) and the
  // dialogs' strength gate can't pass a password the backend will reject.
  if (classes >= 3 && len >= 8) score = Math.max(score, 2) as StrengthLevel;
  if (classes >= 4 && len >= 12) score = Math.max(score, 3) as StrengthLevel;
  if (classes >= 4 && len >= 20) score = 4;

  // Cap the score for common/low-entropy passwords: an exact dictionary
  // staple, repeated character, or sequential run is trivial regardless of
  // length; a common word dressed up with case, leet swaps, or digit/symbol
  // padding is at best "Weak".
  let capped = false;
  if (/^(.)\1+$/.test(password) || isSequentialRun(lower)) {
    score = 0;
    capped = true;
  }
  const core = commonCore(password);
  if (core !== null) {
    const cap = core.length === len ? 0 : 1;
    if (score > cap) {
      score = cap as StrengthLevel;
      capped = true;
    }
  }

  return {
    level: score,
    fraction: (score + 1) / 5,
    // "Too short" would mislabel a long-but-common password forced to 0.
    label: capped && score === 0 ? "Very weak" : LABELS[score],
  };
}

export function strengthColor(level: StrengthLevel): string {
  return COLORS[level];
}
