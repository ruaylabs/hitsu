const NATIVE_HOST = "com.ruaylabs.hitsu.browser";

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

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message?.type === "list-logins") {
    nativeMessage({ type: "listLogins", origin: message.origin })
      .then((response) => sendResponse({ ok: true, entries: response.entries }))
      .catch((error) => sendResponse({ ok: false, error: error.message }));
    return true;
  }

  if (message?.type === "fill-login") {
    nativeMessage({ type: "getCredentials", id: message.id, origin: message.origin })
      .then(async (response) => {
        await chrome.tabs.sendMessage(message.tabId, {
          type: "fill-login",
          username: response.username ?? "",
          password: response.password,
        });
        sendResponse({ ok: true });
      })
      .catch((error) => sendResponse({ ok: false, error: error.message }));
    return true;
  }

  return false;
});
