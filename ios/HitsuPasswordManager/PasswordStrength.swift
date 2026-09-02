import SwiftUI

/// Lightweight, dependency-free password strength estimate for the UI,
/// mirroring the desktop app's `src/lib/utils/passwordStrength.ts`.
///
/// Returns a 0–4 score plus a short label and a fraction (0–1) for a meter.
/// This is a *display* heuristic, not a crypto judgment. It is intentionally
/// simple and runs fully client-side. Bands map roughly to: 0 trivial,
/// 1 weak, 2 fair, 3 good, 4 strong.
struct PasswordStrength: Equatable {
  let level: Int
  /// 0–1, for a progress bar.
  let fraction: Double
  let label: String
}

private let strengthLabels: [Int: String] = [
  0: "Too short",
  1: "Weak",
  2: "Fair",
  3: "Good",
  4: "Strong",
]

func strengthColor(for level: Int) -> Color {
  switch level {
  case 0, 1: .red
  case 2: .orange
  default: .green
  }
}

/// Top common passwords and base words (drawn from public breach top-lists).
/// Matched against the whole password and against its letter core after
/// lowercasing, undoing leetspeak, and stripping digit/symbol padding — so
/// `Password123!` and `qwerty2024` are caught, not just exact `password`.
private let commonPasswords: Set<String> = [
  "123456", "1234567", "12345678", "123456789", "1234567890", "12345",
  "111111", "123123", "000000", "121212", "654321", "666666", "112233",
  "password", "passwort", "qwerty", "qwertyuiop", "qwertz", "azerty",
  "asdfghjkl", "asdfgh", "zxcvbnm", "qazwsx", "abc123", "abcdef",
  "abcd1234", "iloveyou", "admin", "welcome", "login", "letmein",
  "dragon", "monkey", "football", "baseball", "basketball", "soccer",
  "hockey", "superman", "batman", "trustno1", "master", "shadow",
  "sunshine", "princess", "flower", "hello", "freedom", "whatever",
  "ninja", "mustang", "jordan", "harley", "hunter", "ranger", "buster",
  "tigger", "pepper", "ginger", "cookie", "cheese", "banana", "orange",
  "purple", "silver", "golden", "diamond", "monster", "killer", "cowboy",
  "angel", "lovely", "secret", "summer", "winter", "spring", "autumn",
  "starwars", "pokemon", "naruto", "minecraft", "computer", "internet",
  "samsung", "google", "liverpool", "chelsea", "arsenal", "barcelona",
  "thomas", "robert", "michael", "charlie", "daniel", "andrew", "joshua",
  "matthew", "jessica", "jennifer", "michelle", "ashley", "amanda",
  "nicole", "hannah",
]

/// Common leetspeak substitutions, undone before dictionary lookup.
private let leetMap: [Character: Character] = [
  "@": "a", "4": "a", "3": "e", "1": "i", "!": "i", "0": "o", "$": "s",
  "5": "s", "7": "t",
]

private func deleet(_ lower: String) -> String {
  String(lower.map { leetMap[$0] ?? $0 })
}

/// ASCII letters only, matching the desktop's `[a-z]` regex in `commonCore`.
private let asciiLetters = CharacterSet(charactersIn: "abcdefghijklmnopqrstuvwxyz")

/// The common-password base this password is built on, or nil.
/// Checks the lowercased password and its de-leeted form, each both whole and
/// with leading/trailing non-letters (digit/symbol padding) stripped.
private func commonCore(of password: String) -> String? {
  let lower = password.lowercased()
  let deleeted = deleet(lower)
  for variant in [lower, deleeted] {
    if commonPasswords.contains(variant) { return variant }
    let core = variant.trimmingCharacters(in: asciiLetters.inverted)
    if !core.isEmpty && commonPasswords.contains(core) { return core }
  }
  return nil
}

/// True for pure ascending/descending character runs like `abcdefgh`.
private func isSequentialRun(_ password: String) -> Bool {
  let scalars = Array(password.lowercased().unicodeScalars)
  guard scalars.count >= 4 else { return false }
  let step = Int(scalars[1].value) - Int(scalars[0].value)
  guard abs(step) == 1 else { return false }
  for index in 2..<scalars.count {
    if Int(scalars[index].value) - Int(scalars[index - 1].value) != step { return false }
  }
  return true
}

func estimatePasswordStrength(_ password: String) -> PasswordStrength {
  if password.isEmpty {
    return PasswordStrength(level: 0, fraction: 0, label: strengthLabels[0]!)
  }

  let length = password.count
  let lower = password.lowercased()

  // Character-class coverage (ASCII classes, matching the desktop's regexes)
  var classes = 0
  if password.contains(where: { $0.isLowercase && $0.isASCII }) { classes += 1 }
  if password.contains(where: { $0.isUppercase && $0.isASCII }) { classes += 1 }
  if password.contains(where: { $0.isNumber && $0.isASCII }) { classes += 1 }
  if password.contains(where: { !($0.isASCII && ($0.isLetter || $0.isNumber)) }) {
    classes += 1
  }

  // Base score from length
  var score: Int
  if length < 8 {
    score = 0
  } else if length < 12 {
    score = 1
  } else if length < 16 {
    score = 2
  } else if length < 20 {
    score = 3
  } else {
    score = 4
  }

  // Reward diversity — never below 8 chars, so level 0 stays sticky under
  // the backend's minimum and dialogs' strength gates can't pass a password
  // the backend will reject.
  if classes >= 3 && length >= 8 { score = max(score, 2) }
  if classes >= 4 && length >= 12 { score = max(score, 3) }
  if classes >= 4 && length >= 20 { score = 4 }

  // Cap the score for common/low-entropy passwords: an exact dictionary
  // staple, repeated character, or sequential run is trivial regardless of
  // length; a common word dressed up with case, leet swaps, or digit/symbol
  // padding is at best "Weak".
  var capped = false
  if (password.count > 1 && password.allSatisfy({ $0 == password.first! }))
    || isSequentialRun(lower)
  {
    score = 0
    capped = true
  }
  if let core = commonCore(of: password) {
    let cap = core.count == password.count ? 0 : 1
    if score > cap {
      score = cap
      capped = true
    }
  }

  return PasswordStrength(
    level: score,
    fraction: Double(score + 1) / 5,
    // "Too short" would mislabel a long-but-common password forced to 0.
    label: capped && score == 0 ? "Very weak" : strengthLabels[score]!
  )
}

/// Slim progress bar + label shown next to a revealed password, mirroring the
/// desktop's `PasswordStrengthMeter.svelte`.
struct PasswordStrengthMeter: View {
  let password: String

  var body: some View {
    if !password.isEmpty {
      let strength = estimatePasswordStrength(password)
      let color = strengthColor(for: strength.level)
      HStack(spacing: 8) {
        GeometryReader { proxy in
          ZStack(alignment: .leading) {
            Capsule().fill(Color(.tertiarySystemFill))
            Capsule()
              .fill(color)
              .frame(width: proxy.size.width * strength.fraction)
          }
        }
        .frame(height: 6)
        Text("Strength: \(strength.label)")
          .font(.caption.weight(.medium))
          .foregroundStyle(color)
      }
      .accessibilityElement(children: .ignore)
      .accessibilityLabel("Password strength: \(strength.label)")
    }
  }
}
