import KDBXKit
import QuickLook
import Security
import SwiftUI
import UIKit
import UniformTypeIdentifiers

@MainActor
private final class ClipboardManager {
  private var ownedChangeCount: Int?

  func copy(_ value: String, expirationDate: Date) {
    let pasteboard = UIPasteboard.general
    pasteboard.setItems(
      [[UTType.utf8PlainText.identifier: value]],
      options: [
        .localOnly: true,
        .expirationDate: expirationDate,
      ]
    )
    ownedChangeCount = pasteboard.changeCount
  }

  func clearIfOwned() {
    let pasteboard = UIPasteboard.general
    guard let ownedChangeCount, pasteboard.changeCount == ownedChangeCount else { return }
    pasteboard.items = []
    self.ownedChangeCount = nil
  }
}

/// Keychain-backed storage for the last-vault bookmark. Unlike a UserDefaults
/// plist, Keychain items are unreadable from a filesystem image, and the
/// device-only accessibility keeps the bookmark out of backups.
enum LastVaultBookmark {
  private static let service = "com.ruaylabs.hitsu.lastVaultBookmark"

  static func save(_ data: Data) {
    var query = baseQuery
    SecItemDelete(query as CFDictionary)
    query[kSecValueData as String] = data
    query[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
    SecItemAdd(query as CFDictionary, nil)
  }

  static func load() -> Data? {
    var query = baseQuery
    query[kSecReturnData as String] = true
    var result: AnyObject?
    guard SecItemCopyMatching(query as CFDictionary, &result) == errSecSuccess else { return nil }
    return result as? Data
  }

  static func remove() {
    SecItemDelete(baseQuery as CFDictionary)
  }

  private static var baseQuery: [String: Any] {
    [
      kSecClass as String: kSecClassGenericPassword,
      kSecAttrService as String: service,
    ]
  }
}

/// Remembers the last user touch, seen anywhere in the app's windows
/// (presented sheets included) and never claimed, so normal gestures are
/// unaffected.
@MainActor
private final class InteractionClock: NSObject {
  private(set) var lastInteraction = Date()

  func install() {
    let recognizer = AnyGestureRecognizer(
      target: self,
      action: #selector(touchRecognized)
    )
    recognizer.cancelsTouchesInView = false
    for scene in UIApplication.shared.connectedScenes {
      guard let windowScene = scene as? UIWindowScene else { continue }
      for window in windowScene.windows {
        window.addGestureRecognizer(recognizer)
      }
    }
  }

  @objc private func touchRecognized() {
    lastInteraction = Date()
  }
}

/// A recognizer that fires on every touch and never claims one.
private final class AnyGestureRecognizer: UIGestureRecognizer {
  override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent) {
    state = .ended
  }

  override func canPrevent(_ gestureRecognizer: UIGestureRecognizer) -> Bool {
    false
  }

  override func canBePrevented(by gestureRecognizer: UIGestureRecognizer) -> Bool {
    false
  }
}

/// Returns a URL only for HTTP(S) destinations. Bare hostnames default to HTTPS;
/// every other scheme remains visible as plain text but cannot be opened.
func validatedHTTPURL(_ rawValue: String) -> URL? {
  let value = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
  let candidate = value.contains("://") ? value : "https://\(value)"
  guard let url = URL(string: candidate),
    let scheme = url.scheme?.lowercased(),
    scheme == "http" || scheme == "https",
    url.host != nil
  else {
    return nil
  }
  return url
}

struct ContentView: View {
  @State private var store = VaultStore()
  @Environment(\.scenePhase) private var scenePhase
  @State private var showingImporter = false
  @State private var pendingURL: URL?
  @State private var favoritesSearchText = ""
  @State private var recentSearchText = ""
  @State private var categoriesSearchText = ""
  @State private var favoriteSelectedID: UUID?
  @State private var restoredLastVault = false
  @State private var hasSavedVault = false
  @State private var clipboard = ClipboardManager()
  @State private var interactionClock = InteractionClock()
  /// Seconds without a touch before the idle lock fires.
  @AppStorage("idleLockSeconds") private var idleLockSeconds = 60

  private var favoriteEntries: [VaultEntry] {
    store.entries.filter {
      $0.isFavorite && !$0.isTrashed
        && store.matchesSearch($0, searchText: favoritesSearchText)
    }
  }

  private var recentEntries: [VaultEntry] {
    recentVaultEntries(store.entries).filter {
      store.matchesSearch($0, searchText: recentSearchText)
    }
  }

  private var categorySections: [CategorySection] {
    let entries = store.entries.filter {
      !$0.isTrashed && store.matchesSearch($0, searchText: categoriesSearchText)
    }
    return VaultEntryCategory.allCases.compactMap { category in
      let categoryEntries = entries.filter { $0.category == category }
      guard !categoryEntries.isEmpty else { return nil }
      return CategorySection(category: category, entries: categoryEntries)
    }
  }

  private var trashedEntries: [VaultEntry] {
    store.entries.filter { $0.isTrashed }
  }

  var body: some View {
    ZStack {
      Color(.systemBackground)
        .ignoresSafeArea()

      Group {
        if store.isUnlocked {
          vaultView
        } else {
          welcomeView
        }
      }
      .frame(maxWidth: .infinity, maxHeight: .infinity)

      if scenePhase != .active {
        PrivacyShieldView()
      }
    }
    .onOpenURL { url in
      guard url.pathExtension.lowercased() == "kdbx" else {
        store.showError("Please choose a KeePass .kdbx database.")
        return
      }
      rememberLastVault(url)
      pendingURL = url
      store.clearError()
    }
    .fileImporter(
      isPresented: $showingImporter,
      allowedContentTypes: [UTType(filenameExtension: "kdbx") ?? .data],
      allowsMultipleSelection: false
    ) { result in
      switch result {
      case .success(let urls):
        if let url = urls.first {
          rememberLastVault(url)
        }
        pendingURL = urls.first
        store.clearError()
      case .failure(let error):
        store.showError(error.localizedDescription)
      }
    }
    .sheet(
      isPresented: Binding(
        get: { pendingURL != nil },
        set: { presented in
          if !presented { pendingURL = nil }
        }
      )
    ) {
      if let url = pendingURL {
        PasswordSheet(fileName: url.lastPathComponent) { unlockData in
          pendingURL = nil
          store.open(url: url, unlockData: unlockData)
        } onCancel: {
          pendingURL = nil
        }
      }
    }
    .task {
      interactionClock.install()
      // A killed run can leave decrypted previews behind; drop those first.
      AttachmentPreviewStaging.shared.purgeStale()
      restoreLastVault()
      await autoLockLoop()
    }
    .onChange(of: scenePhase) { _, phase in
      if phase != .active {
        // Dismiss the unlock sheet: the typed master password is released
        // with the torn-down sheet view instead of surviving in @State.
        pendingURL = nil
        lockVault()
        // Previews still on screen are kept; only residue is purged.
        AttachmentPreviewStaging.shared.purgeStale()
      }
    }
  }

  private func rememberLastVault(_ url: URL) {
    do {
      let bookmark = try url.bookmarkData(
        options: [.minimalBookmark],
        includingResourceValuesForKeys: nil,
        relativeTo: nil
      )
      LastVaultBookmark.save(bookmark)
      hasSavedVault = true
    } catch {
      // The current open still works; remembering the URL is only a convenience.
    }
  }

  private func restoreLastVault() {
    guard !restoredLastVault else { return }
    restoredLastVault = true
    openLastVault()
  }

  private func openLastVault() {
    guard pendingURL == nil, !store.isUnlocked,
      let bookmark = LastVaultBookmark.load()
    else {
      hasSavedVault = LastVaultBookmark.load() != nil
      return
    }

    do {
      var isStale = false
      let url = try URL(
        resolvingBookmarkData: bookmark,
        options: [],
        relativeTo: nil,
        bookmarkDataIsStale: &isStale
      )
      if isStale {
        rememberLastVault(url)
      }
      hasSavedVault = true
      pendingURL = url
    } catch {
      LastVaultBookmark.remove()
      hasSavedVault = false
      store.showError("The last database is no longer available. Choose it again from Files.")
    }
  }

  private var welcomeView: some View {
    VStack(spacing: 28) {
      Image(systemName: "lock.shield.fill")
        .font(.system(size: 38, weight: .medium))
        .foregroundStyle(.white)
        .frame(width: 88, height: 88)
        .background(
          LinearGradient(colors: [.blue, .teal], startPoint: .top, endPoint: .bottom),
          in: RoundedRectangle(cornerRadius: 22, style: .continuous)
        )
        .shadow(color: .blue.opacity(0.25), radius: 18, y: 8)

      VStack(spacing: 8) {
        Text("Hitsu\nPassword\nManager")
          .font(.largeTitle.bold())
          .multilineTextAlignment(.center)
        Text("Read a KeePass database without changing it.")
          .font(.subheadline)
          .foregroundStyle(.secondary)
          .multilineTextAlignment(.center)
      }

      VStack(spacing: 12) {
        if hasSavedVault {
          Button {
            store.clearError()
            openLastVault()
          } label: {
            Label("Unlock Last Vault", systemImage: "lock.open")
              .frame(maxWidth: 300)
          }
          .buttonStyle(.borderedProminent)
          .controlSize(.large)
          .disabled(store.isLoading)
        }

        Button {
          showingImporter = true
        } label: {
          Label("Open from Files or iCloud Drive", systemImage: "folder")
            .frame(maxWidth: 300)
        }
        .buttonStyle(.bordered)
        .controlSize(.large)
        .disabled(store.isLoading)
      }

      Menu {
        Picker("Lock after inactivity", selection: $idleLockSeconds) {
          Text("After 1 minute").tag(60)
          Text("After 5 minutes").tag(300)
          Text("After 15 minutes").tag(900)
        }
      } label: {
        Label(autoLockLabel, systemImage: "timer")
          .font(.footnote.weight(.medium))
          .foregroundStyle(.secondary)
      }

      if store.isLoading {
        ProgressView("Unlocking…")
      }
      if let errorMessage = store.errorMessage {
        Text(errorMessage)
          .font(.footnote)
          .foregroundStyle(.red)
          .multilineTextAlignment(.center)
          .padding(12)
          .frame(maxWidth: 360)
          .background(Color.red.opacity(0.08), in: RoundedRectangle(cornerRadius: 12))
      }
    }
    .padding(28)
  }

  private var vaultView: some View {
    TabView {
      favoritesView
        .tabItem {
          Label("Favorites", systemImage: "star.fill")
        }

      recentView
        .tabItem {
          Label("Recent", systemImage: "clock.fill")
        }

      categoriesView
        .tabItem {
          Label("Categories", systemImage: "square.grid.2x2.fill")
        }
    }
  }

  private var favoritesView: some View {
    NavigationSplitView {
      List(selection: $favoriteSelectedID) {
        if favoriteEntries.isEmpty {
          ContentUnavailableView(
            favoritesSearchText.isEmpty ? "No favorites" : "No results",
            systemImage: favoritesSearchText.isEmpty ? "star" : "magnifyingglass"
          )
        } else {
          ForEach(favoriteEntries) { entry in
            EntryRow(entry: entry)
              .tag(entry.id)
          }
        }
      }
      .navigationTitle("Favorites")
      .searchable(text: $favoritesSearchText, prompt: "Search favorites")
      .toolbar {
        LockToolbar(action: lockVault)
      }
    } detail: {
      if let favoriteSelectedID,
        let selected = store.entries.first(where: { $0.id == favoriteSelectedID })
      {
        EntryDetailView(entry: selected, store: store, clipboard: clipboard)
          .id(selected.id)
      } else {
        ContentUnavailableView(
          "Select a favorite",
          systemImage: "star",
          description: Text("Choose a favorite to view its details.")
        )
      }
    }
  }

  private var recentView: some View {
    NavigationStack {
      List {
        if recentEntries.isEmpty {
          ContentUnavailableView(
            recentSearchText.isEmpty ? "No recent entries" : "No results",
            systemImage: recentSearchText.isEmpty ? "clock" : "magnifyingglass"
          )
        } else {
          ForEach(recentEntries) { entry in
            NavigationLink {
              EntryDetailView(entry: entry, store: store, clipboard: clipboard)
                .id(entry.id)
            } label: {
              EntryRow(entry: entry)
            }
          }
        }
      }
      .navigationTitle("Recent")
      .searchable(text: $recentSearchText, prompt: "Search recent entries")
      .toolbar {
        LockToolbar(action: lockVault)
      }
    }
  }

  private var categoriesView: some View {
    NavigationStack {
      List {
        if categorySections.isEmpty && trashedEntries.isEmpty {
          ContentUnavailableView(
            categoriesSearchText.isEmpty ? "No categories" : "No results",
            systemImage: categoriesSearchText.isEmpty
              ? "square.grid.2x2"
              : "magnifyingglass"
          )
        } else {
          ForEach(categorySections) { section in
            NavigationLink {
              CategoryEntriesView(
                category: section.category,
                store: store,
                clipboard: clipboard,
                initialSearchText: categoriesSearchText,
                onLock: lockVault
              )
            } label: {
              HStack(spacing: 12) {
                CategoryIconBadge(category: section.category)
                Text(section.category.title)
                  .font(.body.weight(.medium))
                Spacer()
                Text(section.entries.count, format: .number)
                  .font(.subheadline.weight(.semibold))
                  .foregroundStyle(.secondary)
                  .padding(.horizontal, 10)
                  .padding(.vertical, 4)
                  .background(.fill.tertiary, in: Capsule())
              }
              .padding(.vertical, 2)
            }
          }

          if !trashedEntries.isEmpty {
            NavigationLink {
              TrashEntriesView(store: store, clipboard: clipboard, onLock: lockVault)
            } label: {
              HStack(spacing: 12) {
                Image(systemName: "trash")
                  .font(.system(size: 15, weight: .semibold))
                  .foregroundStyle(.gray)
                  .frame(width: 34, height: 34)
                  .background(Color.gray.opacity(0.14))
                  .clipShape(RoundedRectangle(cornerRadius: 9, style: .continuous))
                Text("Trash")
                  .font(.body.weight(.medium))
                Spacer()
                Text(trashedEntries.count, format: .number)
                  .font(.subheadline.weight(.semibold))
                  .foregroundStyle(.secondary)
                  .padding(.horizontal, 10)
                  .padding(.vertical, 4)
                  .background(.fill.tertiary, in: Capsule())
              }
              .padding(.vertical, 2)
            }
          }
        }
      }
      .navigationTitle("Categories")
      .searchable(text: $categoriesSearchText, prompt: "Search categories")
      .toolbar {
        LockToolbar(action: lockVault)
      }
    }
  }

  /// Locks the vault when no touch happened for `idleLockSeconds` while the
  /// app stayed foregrounded.
  private func autoLockLoop() async {
    while !Task.isCancelled {
      try? await Task.sleep(for: .seconds(15))
      // A stale 0 from before the Never option was removed falls back to the
      // 1-minute default.
      let timeout = max(idleLockSeconds, 60)
      guard store.isUnlocked,
        Date().timeIntervalSince(interactionClock.lastInteraction) >= TimeInterval(timeout)
      else { continue }
      lockVault()
    }
  }

  private var autoLockLabel: String {
    switch idleLockSeconds {
    case 60: "Auto-lock: after 1 minute"
    case 300: "Auto-lock: after 5 minutes"
    case 900: "Auto-lock: after 15 minutes"
    default: "Auto-lock"
    }
  }

  private func lockVault() {
    clipboard.clearIfOwned()
    favoriteSelectedID = nil
    favoritesSearchText = ""
    recentSearchText = ""
    categoriesSearchText = ""
    store.lock()
  }
}

private struct CategorySection: Identifiable {
  let category: VaultEntryCategory
  let entries: [VaultEntry]

  var id: VaultEntryCategory { category }
}

/// Read-only listing of the vault's recycle bin, reached from the Trash row at
/// the end of Categories. Mirrors the desktop's Recycle Bin view. There are no
/// restore or empty actions: the app never writes to the database.
private struct TrashEntriesView: View {
  let store: VaultStore
  let clipboard: ClipboardManager
  let onLock: () -> Void

  @State private var searchText = ""

  private var entries: [VaultEntry] {
    store.entries.filter {
      $0.isTrashed && store.matchesSearch($0, searchText: searchText)
    }
  }

  var body: some View {
    List {
      if entries.isEmpty {
        ContentUnavailableView(
          searchText.isEmpty ? "Trash is empty" : "No results",
          systemImage: searchText.isEmpty ? "trash" : "magnifyingglass"
        )
      } else {
        ForEach(entries) { entry in
          NavigationLink {
            EntryDetailView(entry: entry, store: store, clipboard: clipboard)
              .id(entry.id)
          } label: {
            EntryRow(entry: entry)
          }
        }
      }
    }
    .navigationTitle("Trash")
    .searchable(text: $searchText, prompt: "Search trash")
    .toolbar {
      LockToolbar(action: onLock)
    }
  }
}

private struct CategoryEntriesView: View {
  let category: VaultEntryCategory
  let store: VaultStore
  let clipboard: ClipboardManager
  let onLock: () -> Void

  @State private var searchText: String

  init(
    category: VaultEntryCategory,
    store: VaultStore,
    clipboard: ClipboardManager,
    initialSearchText: String,
    onLock: @escaping () -> Void
  ) {
    self.category = category
    self.store = store
    self.clipboard = clipboard
    self.onLock = onLock
    _searchText = State(initialValue: initialSearchText)
  }

  private var entries: [VaultEntry] {
    store.entries.filter { entry in
      entry.category == category && store.matchesSearch(entry, searchText: searchText)
    }
  }

  var body: some View {
    List {
      if entries.isEmpty {
        ContentUnavailableView(
          searchText.isEmpty ? "No entries" : "No results",
          systemImage: searchText.isEmpty ? "list.bullet.rectangle" : "magnifyingglass"
        )
      } else {
        ForEach(entries) { entry in
          NavigationLink {
            EntryDetailView(entry: entry, store: store, clipboard: clipboard)
              .id(entry.id)
          } label: {
            EntryRow(entry: entry)
          }
        }
      }
    }
    .navigationTitle(category.title)
    .searchable(text: $searchText, prompt: "Search \(category.title.lowercased())")
    .toolbar {
      LockToolbar(action: onLock)
    }
  }
}

private struct LockToolbar: ToolbarContent {
  let action: () -> Void

  var body: some ToolbarContent {
    ToolbarItem(placement: .topBarTrailing) {
      Button("Lock", systemImage: "lock", action: action)
    }
  }
}

private struct PrivacyShieldView: View {
  var body: some View {
    ZStack {
      Color(.systemBackground)
        .ignoresSafeArea()

      VStack(spacing: 14) {
        Image(systemName: "lock.shield.fill")
          .font(.system(size: 44))
          .foregroundStyle(.tint)
        Text("Hitsu is locked")
          .font(.headline)
      }
    }
    .transition(.opacity)
  }
}

private struct CategoryIconBadge: View {
  let category: VaultEntryCategory

  var body: some View {
    Image(systemName: category.symbolName)
      .font(.system(size: 15, weight: .semibold))
      .foregroundStyle(category.tint)
      .frame(width: 34, height: 34)
      .background(category.tint.opacity(0.14))
      .clipShape(RoundedRectangle(cornerRadius: 9, style: .continuous))
  }
}

private struct EntryIconView: View {
  let icon: VaultEntryIcon
  var tint: Color = .accentColor
  var size: CGFloat = 36

  var body: some View {
    Group {
      if let customData = icon.customData, let image = UIImage(data: customData) {
        Image(uiImage: image)
          .resizable()
          .scaledToFit()
      } else {
        Image(systemName: standardSystemImage)
          .resizable()
          .scaledToFit()
          .padding(size * 0.22)
          .foregroundStyle(tint)
      }
    }
    .frame(width: size, height: size)
    .background(tint.opacity(0.14))
    .clipShape(RoundedRectangle(cornerRadius: size * 0.22, style: .continuous))
  }

  private var standardSystemImage: String {
    let index = Int(icon.standardID)
    guard Self.standardSymbols.indices.contains(index) else { return "key.fill" }
    return Self.standardSymbols[index]
  }

  private static let standardSymbols = [
    "key.fill",
    "globe",
    "exclamationmark.triangle.fill",
    "server.rack",
    "folder.badge.checkmark",
    "bubble.left.and.bubble.right.fill",
    "puzzlepiece.fill",
    "note.text",
    "network",
    "person.crop.circle.fill",
    "doc.fill",
    "camera.fill",
    "antenna.radiowaves.left.and.right",
    "key.horizontal.fill",
    "bolt.fill",
    "scanner",
    "globe.americas.fill",
    "opticaldisc.fill",
    "display",
    "envelope.fill",
    "gearshape.fill",
    "clipboard.fill",
    "doc.badge.plus",
    "rectangle.on.rectangle",
    "bolt.badge.clock",
    "tray.full.fill",
    "externaldrive.fill",
    "internaldrive.fill",
    "doc.text.magnifyingglass",
    "lock.rectangle",
    "terminal.fill",
    "printer.fill",
    "app.dashed",
    "play.fill",
    "slider.horizontal.3",
    "desktopcomputer",
    "archivebox.fill",
    "building.columns.fill",
    "externaldrive.fill",
    "clock.fill",
    "envelope.badge",
    "flag.fill",
    "memorychip.fill",
    "trash.fill",
    "note.text",
    "clock.badge.exclamationmark",
    "info.circle.fill",
    "shippingbox.fill",
    "folder.fill",
    "folder.fill",
    "folder.fill",
    "lock.open.fill",
    "doc.fill",
    "checkmark.circle.fill",
    "pencil",
    "photo.fill",
    "book.closed.fill",
    "list.bullet",
    "person.crop.circle.badge.checkmark",
    "wrench.and.screwdriver.fill",
    "house.fill",
    "star.fill",
    "desktopcomputer",
    "feather",
    "apple.logo",
    "book.closed.fill",
    "banknote.fill",
    "checkmark.seal.fill",
    "smartphone",
  ]
}

private struct ExpirationIndicator: View {
  let date: Date
  let isDue: Bool

  private var label: String {
    if isDue {
      return Calendar.current.isDateInToday(date)
        ? "Expires today"
        : "Expired on \(date.formatted(date: .abbreviated, time: .omitted))"
    }
    return "Expires on \(date.formatted(date: .abbreviated, time: .omitted))"
  }

  var body: some View {
    Label(label, systemImage: isDue ? "exclamationmark.triangle.fill" : "calendar")
      .font(.footnote.weight(.medium))
      .foregroundStyle(isDue ? .red : .secondary)
      .frame(maxWidth: .infinity, alignment: .leading)
      .padding(10)
      .background(
        (isDue ? Color.red : Color.secondary).opacity(0.1),
        in: RoundedRectangle(cornerRadius: 10)
      )
  }
}

private struct EntryRow: View {
  let entry: VaultEntry

  var body: some View {
    HStack(spacing: 12) {
      EntryIconView(icon: entry.icon, tint: entry.category.tint)

      VStack(alignment: .leading, spacing: 3) {
        HStack {
          Text(entry.displayTitle)
            .font(.body.weight(.medium))
          Spacer()
          if entry.isFavorite {
            Image(systemName: "star.fill")
              .font(.caption)
              .foregroundStyle(.yellow)
          }
          if entry.isExpired {
            Image(systemName: "exclamationmark.triangle.fill")
              .font(.caption)
              .foregroundStyle(.red)
          }
          if entry.hasPassword {
            Image(systemName: "key.fill")
              .font(.caption)
              .foregroundStyle(.secondary)
          }
          if entry.hasTOTP {
            Image(systemName: "timer")
              .font(.caption)
              .foregroundStyle(.secondary)
          }
          if !entry.attachments.isEmpty {
            Image(systemName: "paperclip")
              .font(.caption)
              .foregroundStyle(.secondary)
          }
        }
        Text(entry.secondaryText)
          .font(.subheadline)
          .foregroundStyle(.secondary)
          .lineLimit(1)
        if !entry.groupPath.isEmpty {
          Text(entry.groupPath)
            .font(.caption)
            .foregroundStyle(.tertiary)
            .lineLimit(1)
        }
      }
    }
    .padding(.vertical, 3)
  }
}

private struct PasswordSheet: View {
  let fileName: String
  let onUnlock: (UnlockData) -> Void
  let onCancel: () -> Void

  @Environment(\.dismiss) private var dismiss
  @FocusState private var passwordIsFocused: Bool
  @State private var password = ""

  var body: some View {
    NavigationStack {
      Form {
        Section {
          SecureField("Master password", text: $password)
            .focused($passwordIsFocused)
            .submitLabel(.continue)
            .onSubmit(unlock)
        } header: {
          Text(fileName)
            .textCase(nil)
        } footer: {
          Text("The file is opened read-only. It is never written back.")
        }
      }
      .navigationTitle("Unlock database")
      .navigationBarTitleDisplayMode(.inline)
      .toolbar {
        ToolbarItem(placement: .cancellationAction) {
          Button("Cancel") {
            onCancel()
            dismiss()
          }
        }
        ToolbarItem(placement: .confirmationAction) {
          Button("Unlock", action: unlock)
            .disabled(password.isEmpty)
        }
      }
    }
    .presentationDetents([.medium])
    .onAppear { passwordIsFocused = true }
    .onDisappear { password = "" }
  }

  private func unlock() {
    guard !password.isEmpty else { return }
    // Consume the cleartext at the earliest point: UnlockData discards it at
    // init and keeps only the mlock'd pre-hash, so the password never leaves
    // this view (or its @State) as a plain String.
    let unlockData = UnlockData(masterPassword: password)
    password = ""
    onUnlock(unlockData)
    dismiss()
  }
}

private struct EntryDetailView: View {
  let entry: VaultEntry
  let store: VaultStore
  let clipboard: ClipboardManager

  @State private var revealedFields: [String: String] = [:]
  @State private var copiedPassword = false
  @State private var previewRequest: AttachmentPreviewRequest?
  @State private var shareRequest: AttachmentShareRequest?
  @State private var previewTempDirectory: URL?
  /// Guards against double taps while a preview is being staged.
  @State private var isStagingAttachment = false
  @State private var maskedCardNumber: String?

  private static let clipboardLifetime: TimeInterval = 30

  private var hasAccountInfo: Bool {
    !entry.username.isEmpty || entry.isUsernameProtected || !entry.url.isEmpty
      || entry.isURLProtected || entry.hasNotes
  }

  var body: some View {
    ScrollView {
      VStack(alignment: .leading, spacing: 16) {
        HStack(alignment: .top, spacing: 14) {
          EntryIconView(icon: entry.icon, tint: entry.category.tint, size: 56)
          VStack(alignment: .leading, spacing: 6) {
            if entry.isTitleProtected {
              protectedRow(label: "Title", field: "Title")
            } else {
              Text(entry.displayTitle)
                .font(.title2.bold())
            }
            if !entry.groupPath.isEmpty {
              Label(entry.groupPath, systemImage: "folder")
                .font(.subheadline)
                .foregroundStyle(.secondary)
            }
          }
        }
        .padding(.top, 4)

        if entry.isTrashed {
          Label("This entry is in the Trash.", systemImage: "trash")
            .font(.footnote)
            .foregroundStyle(.secondary)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(10)
            .background(Color(.tertiarySystemFill), in: RoundedRectangle(cornerRadius: 10))
        }

        if let expirationDate = entry.expirationDate {
          ExpirationIndicator(date: expirationDate, isDue: entry.isExpired)
        }

        if !entry.typedFields.isEmpty {
          DetailSection(
            title: typedSectionTitle,
            systemImage: entry.category.symbolName,
            tint: entry.category.tint
          ) {
            VStack(alignment: .leading, spacing: 12) {
              ForEach(entry.typedFields) { typed in
                typedRow(typed)
              }
            }
          }
        }

        if hasAccountInfo {
          DetailSection(
            title: "Account",
            systemImage: "person.crop.circle",
            tint: entry.category.tint
          ) {
            VStack(alignment: .leading, spacing: 12) {
              if entry.isUsernameProtected {
                protectedRow(label: "Username", field: "UserName")
              } else {
                DetailRow(label: "Username", value: entry.username)
              }
              if entry.isURLProtected {
                protectedRow(label: "URL", field: "URL")
              } else {
                DetailRow(label: "URL", value: entry.url, isLink: true)
              }
              if entry.hasNotes {
                if entry.isNotesProtected {
                  protectedRow(label: "Notes", field: "Notes")
                } else {
                  DetailRow(
                    label: "Notes",
                    value: store.value(for: entry.id, field: "Notes") ?? ""
                  )
                }
              }
            }
          }
        }

        if entry.hasPassword {
          DetailSection(title: "Password", systemImage: "key.fill", tint: .orange) {
            VStack(alignment: .leading, spacing: 12) {
              protectedRow(label: "Password", field: "Password")
              if let revealedPassword = revealedFields["Password"] {
                PasswordStrengthMeter(password: revealedPassword)
              }
              Button(action: copyPassword) {
                Label(
                  copiedPassword ? "Copied — clears in 30 seconds" : "Copy password",
                  systemImage: copiedPassword ? "checkmark" : "doc.on.doc"
                )
              }
              .buttonStyle(.bordered)
            }
          }
        }

        if entry.hasTOTP {
          DetailSection(title: "One-Time Password", systemImage: "timer", tint: .green) {
            TOTPView(entryID: entry.id, store: store, clipboard: clipboard)
          }
        }

        if !entry.attachments.isEmpty {
          DetailSection(title: "Attachments", systemImage: "paperclip", tint: .gray) {
            VStack(alignment: .leading, spacing: 12) {
              ForEach(entry.attachments) { attachment in
                Button {
                  openAttachment(attachment)
                } label: {
                  HStack {
                    Text(attachment.name)
                      .lineLimit(1)
                    Spacer()
                    Text(formatAttachmentSize(attachment.byteCount))
                      .font(.footnote)
                      .foregroundStyle(.secondary)
                    Image(
                      systemName: isPreviewableAttachment(named: attachment.name)
                        ? "eye" : "square.and.arrow.up"
                    )
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                  }
                }
                .buttonStyle(.plain)
              }
            }
          }
        }

        if !entry.history.isEmpty {
          DetailSection(title: "History", systemImage: "clock.arrow.circlepath", tint: .blue) {
            VStack(alignment: .leading, spacing: 8) {
              ForEach(entry.history.reversed()) { item in
                NavigationLink {
                  HistoryVersionView(entry: entry, item: item, store: store)
                } label: {
                  HStack {
                    Text("Version \(item.index + 1)")
                      .font(.body.weight(.medium))
                    Spacer()
                    if let date = item.lastModified {
                      Text(date, format: Date.FormatStyle(date: .abbreviated, time: .shortened))
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                    }
                  }
                }
                .buttonStyle(.plain)
              }
            }
          }
        }

        if !entry.fields.isEmpty {
          DetailSection(title: "Fields", systemImage: "square.grid.2x2", tint: .indigo) {
            VStack(alignment: .leading, spacing: 12) {
              ForEach(entry.fields) { field in
                if let value = revealedFields[field.name] {
                  DetailRow(label: field.displayName, value: value, allowsSelection: false)
                } else if field.isProtected {
                  Button {
                    reveal(field.name)
                  } label: {
                    Label("Reveal \(field.displayName)", systemImage: "eye")
                  }
                } else {
                  DetailRow(
                    label: field.displayName,
                    value: store.value(for: entry.id, field: field.name) ?? ""
                  )
                }
              }
            }
          }
        }

        if !entry.tags.isEmpty {
          DetailSection(title: "Tags", systemImage: "tag.fill", tint: .gray) {
            FlowLayout(spacing: 8) {
              ForEach(entry.tags, id: \.self) { tag in
                HStack(spacing: 5) {
                  Circle()
                    .fill(tagColor(for: tag))
                    .frame(width: 6, height: 6)
                  Text(tag)
                    .font(.footnote.weight(.medium))
                }
                .foregroundStyle(tagColor(for: tag))
                .padding(.horizontal, 11)
                .padding(.vertical, 5)
                .background(tagColor(for: tag).opacity(0.12), in: Capsule())
                .overlay(
                  Capsule()
                    .stroke(tagColor(for: tag).opacity(0.28), lineWidth: 0.5)
                )
              }
            }
          }
        }
      }
      .frame(maxWidth: 680, alignment: .leading)
      .padding()
    }
    .background(Color(.systemGroupedBackground))
    .navigationTitle("Details")
    .navigationBarTitleDisplayMode(.inline)
    .textSelection(.enabled)
    .onDisappear {
      revealedFields.removeAll()
      copiedPassword = false
      cleanupPreview()
    }
    .sheet(item: $previewRequest, onDismiss: { cleanupPreview() }) { request in
      AttachmentPreview(url: request.url)
    }
    .sheet(item: $shareRequest, onDismiss: { cleanupPreview() }) { request in
      ActivityShareSheet(items: [request.url])
    }
    .task(id: entry.id) {
      guard entry.typedFields.contains(where: \.isCardNumber) else { return }
      // The full number is materialized only to derive the mask; just the
      // masked form is retained.
      maskedCardNumber = maskCardNumber(store.value(for: entry.id, field: "card.number") ?? "")
    }
  }

  private var cardType: String? {
    store.value(for: entry.id, field: "card.type")
  }

  private func openAttachment(_ attachment: VaultAttachment) {
    guard previewRequest == nil, shareRequest == nil, !isStagingAttachment else { return }
    let previewable = isPreviewableAttachment(named: attachment.name)
    isStagingAttachment = true
    // Previews re-stream the payload from the vault file, so staging is async.
    Task {
      defer { isStagingAttachment = false }
      guard let data = await store.attachmentData(for: entry.id, index: attachment.index)
      else { return }

      do {
        let url = try AttachmentPreviewStaging.shared.stagePreview(
          data: data,
          fileName: attachment.name
        )
        previewTempDirectory = url.deletingLastPathComponent()
        if previewable {
          previewRequest = AttachmentPreviewRequest(url: url)
        } else {
          shareRequest = AttachmentShareRequest(url: url)
        }
      } catch {
        // Staging already removed its directory; nothing is left to clean up.
      }
    }
  }

  private func cleanupPreview() {
    guard let directory = previewTempDirectory else { return }
    previewTempDirectory = nil
    AttachmentPreviewStaging.shared.removePreview(in: directory)
  }

  private var typedSectionTitle: String {
    switch entry.category {
    case .card: "Card"
    case .identity: "Identity"
    case .softwareLicense: "License"
    case .passport: "Passport"
    case .pgpKey: "PGP Key"
    case .login, .password, .note: "Details"
    }
  }

  @ViewBuilder
  private func typedRow(_ typed: VaultTypedField) -> some View {
    if let displayValue = typed.displayValue {
      DetailRow(label: typed.label, value: displayValue)
    } else if let field = typed.field {
      if typed.isCardNumber {
        cardNumberRow(field: field)
      } else if typed.isProtected {
        protectedRow(label: typed.label, field: field)
      } else {
        DetailRow(label: typed.label, value: plainTypedValue(for: field))
      }
    }
  }

  /// Unprotected typed values resolve on demand; card type is shown as the
  /// human-readable brand name.
  private func plainTypedValue(for field: String) -> String {
    let raw = store.value(for: entry.id, field: field) ?? ""
    return field == "card.type" ? cardBrandName(for: raw) : raw
  }

  /// Card number: masked preview until revealed, brand-aware grouping after.
  @ViewBuilder
  private func cardNumberRow(field: String) -> some View {
    if let revealed = revealedFields[field] {
      DetailRow(
        label: "Number",
        value: formatCardNumber(revealed, cardType: cardType),
        allowsSelection: false
      )
    } else if let mask = maskedCardNumber {
      DetailRow(label: "Number", value: mask)
      Button {
        reveal(field)
      } label: {
        Label("Reveal Number", systemImage: "eye")
      }
    } else {
      protectedRow(label: "Number", field: field)
    }
  }

  @ViewBuilder
  private func protectedRow(label: String, field: String) -> some View {
    if let value = revealedFields[field] {
      DetailRow(label: label, value: value, allowsSelection: false)
    } else {
      Button {
        reveal(field)
      } label: {
        Label("Reveal \(label)", systemImage: "eye")
      }
    }
  }

  private func reveal(_ field: String) {
    guard let value = store.value(for: entry.id, field: field) else { return }
    revealedFields[field] = value
  }

  private func copyPassword() {
    guard let password = store.value(for: entry.id, field: "Password") else { return }
    clipboard.copy(
      password,
      expirationDate: Date().addingTimeInterval(Self.clipboardLifetime)
    )
    copiedPassword = true

    Task { @MainActor in
      try? await Task.sleep(for: .seconds(2))
      copiedPassword = false
    }
  }
}

private struct TOTPView: View {
  let entryID: UUID
  let store: VaultStore
  let clipboard: ClipboardManager

  @State private var currentCode: TOTPCode?
  @State private var copiedCode = false

  var body: some View {
    Group {
      if let currentCode {
        HStack(spacing: 14) {
          Text(currentCode.code)
            .font(.title2.monospacedDigit().weight(.bold))
            .textSelection(.disabled)

          Spacer()

          ZStack {
            Circle()
              .stroke(.quaternary, lineWidth: 4)
            Circle()
              .trim(from: 0, to: progress(of: currentCode))
              .stroke(.green, style: StrokeStyle(lineWidth: 4, lineCap: .round))
              .rotationEffect(.degrees(-90))
            Text("\(currentCode.remaining)")
              .font(.caption2.monospacedDigit().weight(.semibold))
              .foregroundStyle(.secondary)
          }
          .frame(width: 44, height: 44)
          .accessibilityElement(children: .ignore)
          .accessibilityLabel("\(currentCode.remaining) seconds remaining")

          Button {
            copy(currentCode)
          } label: {
            Image(systemName: copiedCode ? "checkmark" : "doc.on.doc")
              .font(.body.weight(.semibold))
              .frame(width: 38, height: 38)
              .background(
                copiedCode ? Color.green.opacity(0.14) : Color.accentColor.opacity(0.12)
              )
              .foregroundStyle(copiedCode ? .green : .accentColor)
              .clipShape(Circle())
          }
          .buttonStyle(.plain)
        }
      } else {
        Text("This entry contains an invalid one-time password configuration.")
          .font(.subheadline)
          .foregroundStyle(.secondary)
      }
    }
    .task(id: entryID) {
      while !Task.isCancelled {
        currentCode = store.totpCode(for: entryID)
        try? await Task.sleep(for: .seconds(1))
      }
    }
  }

  private func progress(of code: TOTPCode) -> Double {
    guard code.period > 0 else { return 0 }
    return Double(code.remaining) / Double(code.period)
  }

  private func copy(_ code: TOTPCode) {
    clipboard.copy(
      code.code,
      expirationDate: Date().addingTimeInterval(TimeInterval(max(code.remaining, 1)))
    )
    copiedCode = true

    Task { @MainActor in
      try? await Task.sleep(for: .seconds(2))
      copiedCode = false
    }
  }
}

private struct AttachmentPreviewRequest: Identifiable {
  let id = UUID()
  let url: URL
}

private struct AttachmentShareRequest: Identifiable {
  let id = UUID()
  let url: URL
}

/// Attachment types QuickLook may render, matched by the resolved UTType.
/// Deliberately concrete rather than conformance-based: .html and .xml
/// conform to .text and .svg conforms to .image, so supertype checks would
/// let script-bearing formats reach a system previewer.
private let previewableAttachmentTypes: [UTType] = [
  .png, .jpeg, .gif, .webP, .heic, .heif, .tiff, .bmp, .pdf, .plainText,
]

/// Whether QuickLook may render the attachment; everything else — including
/// anything the filename can't resolve — goes to the share sheet instead.
private func isPreviewableAttachment(named name: String) -> Bool {
  guard let type = UTType(filenameExtension: (name as NSString).pathExtension) else {
    return false
  }
  return previewableAttachmentTypes.contains(type)
}

/// Share sheet for attachments QuickLook is not allowed to render.
private struct ActivityShareSheet: UIViewControllerRepresentable {
  let items: [Any]

  func makeUIViewController(context: Context) -> UIActivityViewController {
    UIActivityViewController(activityItems: items, applicationActivities: nil)
  }

  func updateUIViewController(_ controller: UIActivityViewController, context: Context) {}
}

/// Staging area for decrypted attachment previews. Each preview gets a fresh
/// UUID directory under one dedicated temporary root, and the payload is
/// written with complete file protection so the bytes are unreadable at rest
/// while the device is locked. The root is purged on launch (dropping
/// whatever a killed run left behind) and whenever the app leaves the
/// foreground, keeping only the previews that are still on screen.
@MainActor
final class AttachmentPreviewStaging {
  static let shared = AttachmentPreviewStaging()

  /// Directories of previews currently presented; `purgeStale` keeps these.
  private var activeDirectories: Set<URL> = []

  private static var stagingRoot: URL {
    FileManager.default.temporaryDirectory
      .appendingPathComponent("AttachmentPreviews", isDirectory: true)
  }

  /// Writes one attachment payload to a fresh protected directory and
  /// returns the file URL to preview. A staging failure removes the fresh
  /// directory, so nothing partial is left behind.
  func stagePreview(data: Data, fileName: String) throws -> URL {
    let safeName =
      fileName.isEmpty ? "attachment" : fileName.replacingOccurrences(of: "/", with: "_")
    let directory = Self.stagingRoot
      .appendingPathComponent(UUID().uuidString, isDirectory: true)
    let url = directory.appendingPathComponent(safeName)
    do {
      try FileManager.default.createDirectory(
        at: directory,
        withIntermediateDirectories: true,
        attributes: [FileAttributeKey.protectionKey: FileProtectionType.complete]
      )
      try data.write(to: url, options: [.atomic, .completeFileProtection])
    } catch {
      try? FileManager.default.removeItem(at: directory)
      throw error
    }
    activeDirectories.insert(directory)
    return url
  }

  /// Removes one preview's directory once its sheet is gone.
  func removePreview(in directory: URL) {
    activeDirectories.remove(directory)
    try? FileManager.default.removeItem(at: directory)
  }

  /// Removes every staged preview directory except those still presented.
  /// On launch the active set is empty, so this drops all residue from a
  /// previous run.
  func purgeStale() {
    let contents =
      (try? FileManager.default.contentsOfDirectory(
        at: Self.stagingRoot,
        includingPropertiesForKeys: nil
      )) ?? []
    for directory in contents where !activeDirectories.contains(directory) {
      try? FileManager.default.removeItem(at: directory)
    }
  }
}

/// Single-item QuickLook preview over a temp-file copy of the attachment.
private struct AttachmentPreview: UIViewControllerRepresentable {
  let url: URL

  func makeUIViewController(context: Context) -> QLPreviewController {
    let controller = QLPreviewController()
    controller.dataSource = context.coordinator
    return controller
  }

  func updateUIViewController(_ controller: QLPreviewController, context: Context) {
    context.coordinator.url = url
    controller.reloadData()
  }

  func makeCoordinator() -> Coordinator {
    Coordinator(url: url)
  }

  final class Coordinator: NSObject, QLPreviewControllerDataSource {
    var url: URL

    init(url: URL) {
      self.url = url
    }

    func numberOfPreviewItems(in controller: QLPreviewController) -> Int {
      1
    }

    func previewController(
      _ controller: QLPreviewController,
      previewItemAt index: Int
    ) -> QLPreviewItem {
      url as NSURL
    }
  }
}

/// Read-only snapshot of one prior entry version. Secrets follow the same
/// reveal-on-demand flow as the live detail view.
private struct HistoryVersionView: View {
  let entry: VaultEntry
  let item: VaultHistoryItem
  let store: VaultStore

  @State private var revealedFields: [String: String] = [:]

  private struct StandardRow {
    let label: String
    let field: String
    let alwaysProtected: Bool
  }

  private static let standardRows: [StandardRow] = [
    StandardRow(label: "Title", field: "Title", alwaysProtected: false),
    StandardRow(label: "Username", field: "UserName", alwaysProtected: false),
    StandardRow(label: "URL", field: "URL", alwaysProtected: false),
    StandardRow(label: "Password", field: "Password", alwaysProtected: true),
    StandardRow(label: "Notes", field: "Notes", alwaysProtected: false),
  ]

  var body: some View {
    ScrollView {
      VStack(alignment: .leading, spacing: 16) {
        if let date = item.lastModified {
          let formatted = date.formatted(date: .abbreviated, time: .shortened)
          Text("Last modified \(formatted)")
            .font(.subheadline)
            .foregroundStyle(.secondary)
        }

        DetailSection(title: "Fields", systemImage: "clock.arrow.circlepath", tint: .gray) {
          VStack(alignment: .leading, spacing: 12) {
            ForEach(Self.standardRows, id: \.field) { row in
              versionRow(label: row.label, field: row.field, alwaysProtected: row.alwaysProtected)
            }
            ForEach(store.historyFieldNames(for: entry.id, index: item.index), id: \.self) {
              name in
              versionRow(label: name, field: name, alwaysProtected: false)
            }
          }
        }
      }
      .frame(maxWidth: 680, alignment: .leading)
      .padding()
    }
    .background(Color(.systemGroupedBackground))
    .navigationTitle("Version \(item.index + 1)")
    .navigationBarTitleDisplayMode(.inline)
    .onDisappear {
      revealedFields.removeAll()
    }
  }

  @ViewBuilder
  private func versionRow(label: String, field: String, alwaysProtected: Bool) -> some View {
    if let isProtected = store.historyFieldIsProtected(
      for: entry.id, index: item.index, field: field
    ) {
      if let value = revealedFields[field] {
        DetailRow(label: label, value: value, allowsSelection: false)
        if field == "Password" {
          PasswordStrengthMeter(password: value)
        }
      } else if alwaysProtected || isProtected {
        Button {
          reveal(field)
        } label: {
          Label("Reveal \(label)", systemImage: "eye")
        }
      } else {
        DetailRow(
          label: label,
          value: store.historyValue(for: entry.id, index: item.index, field: field) ?? ""
        )
      }
    }
  }

  private func reveal(_ field: String) {
    revealedFields[field] =
      store.historyValue(for: entry.id, index: item.index, field: field) ?? ""
  }
}

private struct DetailSection<Content: View>: View {
  let title: String
  let systemImage: String
  let tint: Color
  let content: Content

  init(
    title: String,
    systemImage: String,
    tint: Color,
    @ViewBuilder content: () -> Content
  ) {
    self.title = title
    self.systemImage = systemImage
    self.tint = tint
    self.content = content()
  }

  var body: some View {
    VStack(alignment: .leading, spacing: 12) {
      Label(title, systemImage: systemImage)
        .font(.subheadline.weight(.semibold))
        .foregroundStyle(tint)
      content
        .frame(maxWidth: .infinity, alignment: .leading)
    }
    .padding(16)
    .background(Color(.secondarySystemGroupedBackground))
    .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
  }
}

private struct FlowLayout: Layout {
  var spacing: CGFloat = 8

  func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
    let width = proposal.width ?? 0
    var x: CGFloat = 0
    var y: CGFloat = 0
    var rowHeight: CGFloat = 0
    for subview in subviews {
      let size = subview.sizeThatFits(.unspecified)
      if x > 0, x + size.width > width {
        x = 0
        y += rowHeight + spacing
        rowHeight = 0
      }
      x += size.width + spacing
      rowHeight = max(rowHeight, size.height)
    }
    return CGSize(width: width, height: y + rowHeight)
  }

  func placeSubviews(
    in bounds: CGRect,
    proposal: ProposedViewSize,
    subviews: Subviews,
    cache: inout ()
  ) {
    var x = bounds.minX
    var y = bounds.minY
    var rowHeight: CGFloat = 0
    for subview in subviews {
      let size = subview.sizeThatFits(.unspecified)
      if x > bounds.minX, x + size.width > bounds.maxX {
        x = bounds.minX
        y += rowHeight + spacing
        rowHeight = 0
      }
      subview.place(at: CGPoint(x: x, y: y), anchor: .topLeading, proposal: .unspecified)
      x += size.width + spacing
      rowHeight = max(rowHeight, size.height)
    }
  }
}

private struct DetailRow: View {
  let label: String
  let value: String
  var isLink = false
  var allowsSelection = true

  var body: some View {
    if !value.isEmpty {
      VStack(alignment: .leading, spacing: 5) {
        Text(label)
          .font(.caption.weight(.semibold))
          .foregroundStyle(.secondary)
        if isLink, let url = validatedHTTPURL(value) {
          Link(value, destination: url)
            .lineLimit(1)
            .truncationMode(.middle)
            .frame(maxWidth: .infinity, alignment: .leading)
        } else if allowsSelection {
          Text(value)
            .textSelection(.enabled)
        } else {
          Text(value)
            .textSelection(.disabled)
        }
      }
    }
  }
}

#Preview {
  ContentView()
}
