const NATIVE_HOST = "com.ruaylabs.hitsu.browser";

async function activeHttpTab() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab?.id || !tab.url) throw new Error("No active browser tab found");

  const url = new URL(tab.url);
  if (url.protocol !== "http:" && url.protocol !== "https:") {
    throw new Error("Hitsu can only fill HTTP and HTTPS pages");
  }
  return { id: tab.id, origin: url.origin };
}

function nativeMessage(message) {
  return new Promise((resolve, reject) => {
    chrome.runtime.sendNativeMessage(NATIVE_HOST, message, (response) => {
      if (chrome.runtime.lastError) {
        reject(new Error(chrome.runtime.lastError.message));
      } else if (!response?.ok) {
        reject(new Error(response?.error ?? "Hitsu did not respond"));
      } else {
        resolve(response);
      }
    });
  });
}

function loginEntries(response) {
  if (
    !Array.isArray(response.entries) ||
    !response.entries.every(
      (entry) =>
        entry &&
        typeof entry.id === "string" &&
        typeof entry.title === "string" &&
        typeof entry.username === "string",
    )
  ) {
    throw new Error("Hitsu returned an invalid login list");
  }
  return response.entries;
}

function credentials(response) {
  if (typeof response.password !== "string" || typeof response.username !== "string") {
    throw new Error("Hitsu returned invalid credentials");
  }
  return response;
}

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (sender.id !== chrome.runtime.id) return false;

  if (message?.type === "list-logins") {
    activeHttpTab()
      .then(({ origin }) => nativeMessage({ type: "listLogins", origin }))
      .then((response) => sendResponse({ ok: true, entries: loginEntries(response) }))
      .catch((error) => sendResponse({ ok: false, error: error.message }));
    return true;
  }

  if (message?.type === "fill-login" && typeof message.id === "string") {
    activeHttpTab()
      .then(async (tab) => {
        const response = credentials(
          await nativeMessage({
            type: "getCredentials",
            id: message.id,
            origin: tab.origin,
          }),
        );
        const currentTab = await chrome.tabs.get(tab.id);
        if (!currentTab.url || new URL(currentTab.url).origin !== tab.origin) {
          throw new Error("The page changed before Hitsu could fill it");
        }
        const fillResponse = await chrome.tabs.sendMessage(tab.id, {
          type: "fill-login",
          username: response.username ?? "",
          password: response.password,
        });
        if (!fillResponse?.ok) {
          throw new Error(fillResponse?.error ?? "Could not fill this page");
        }
        sendResponse({ ok: true });
      })
      .catch((error) => sendResponse({ ok: false, error: error.message }));
    return true;
  }

  return false;
});
