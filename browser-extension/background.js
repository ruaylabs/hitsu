const NATIVE_HOST = "com.ruaylabs.hitsu.browser";

export async function activeHttpTab() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab?.id || !tab.url) throw typedError("no_tab", "No active browser tab found");

  const url = new URL(tab.url);
  if (url.protocol !== "http:" && url.protocol !== "https:") {
    throw typedError("invalid_page", "Hitsu can only fill HTTP and HTTPS pages");
  }
  return { id: tab.id, origin: url.origin };
}

function nativeMessage(message) {
  return new Promise((resolve, reject) => {
    chrome.runtime.sendNativeMessage(NATIVE_HOST, message, (response) => {
      if (chrome.runtime.lastError) {
        reject({ code: "not_installed", message: chrome.runtime.lastError.message });
      } else if (!response?.ok) {
        reject({
          code: response?.code ?? "unknown",
          message: response?.error ?? "Hitsu did not respond",
        });
      } else {
        resolve(response);
      }
    });
  });
}

function typedError(code, message) {
  const error = new Error(message);
  error.code = code;
  return error;
}

export function loginEntries(response) {
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
    throw typedError("invalid_response", "Hitsu returned an invalid login list");
  }
  return response.entries;
}

export function credentials(response) {
  if (typeof response.password !== "string" || typeof response.username !== "string") {
    throw typedError("invalid_response", "Hitsu returned invalid credentials");
  }
  return response;
}

export function pageMatchesOrigin(pageUrl, expectedOrigin) {
  if (!pageUrl) return false;
  try {
    const url = new URL(pageUrl);
    return ["http:", "https:"].includes(url.protocol) && url.origin === expectedOrigin;
  } catch {
    return false;
  }
}

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (sender.id !== chrome.runtime.id) return false;

  if (message?.type === "list-logins") {
    activeHttpTab()
      .then(({ origin }) => nativeMessage({ type: "listLogins", origin }))
      .then((response) => sendResponse({ ok: true, entries: loginEntries(response) }))
      .catch((error) =>
        sendResponse({ ok: false, error: error.message, code: error.code ?? "unknown" }),
      );
    return true;
  }

  if (message?.type === "get-totp" && typeof message.id === "string") {
    activeHttpTab()
      .then((tab) =>
        nativeMessage({
          type: "getTotp",
          id: message.id,
          origin: tab.origin,
        }),
      )
      .then((response) =>
        sendResponse({ ok: true, otp: response.otp, remaining: response.remaining }),
      )
      .catch((error) =>
        sendResponse({ ok: false, error: error.message, code: error.code ?? "unknown" }),
      );
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
        if (!pageMatchesOrigin(currentTab.url, tab.origin)) {
          throw typedError("origin_changed", "The page changed before Hitsu could fill it");
        }

        // Discover all frames and their origins with one allFrames call.
        // executeScript returns { frameId, result } per frame, so we can
        // filter to same-origin frames without needing the webNavigation
        // permission.
        const frameResults = await chrome.scripting.executeScript({
          target: { tabId: tab.id, allFrames: true },
          func: () => window.location.origin,
        });

        const matchingFrameIds = frameResults
          .filter((r) => r.result === tab.origin)
          .map((r) => r.frameId);

        if (matchingFrameIds.length === 0) {
          throw typedError("no_frames", "No matching frames found on this page");
        }

        await chrome.scripting.executeScript({
          target: { tabId: tab.id, frameIds: matchingFrameIds },
          files: ["content.js"],
        });

        const fillMessage = {
          type: "fill-login",
          username: response.username ?? "",
          password: response.password,
          expectedOrigin: tab.origin,
        };

        const fillResults = await Promise.allSettled(
          matchingFrameIds.map((frameId) =>
            chrome.tabs.sendMessage(tab.id, fillMessage, { frameId }),
          ),
        );

        const anyOk = fillResults.some((r) => r.status === "fulfilled" && r.value?.ok);
        if (!anyOk) {
          const firstError = fillResults.find(
            (r) => (r.status === "fulfilled" && !r.value?.ok) || r.status === "rejected",
          );
          const msg =
            firstError?.status === "rejected"
              ? firstError.reason?.message
              : firstError?.value?.error;
          throw typedError("fill_failed", msg ?? "Could not fill any frame on this page");
        }

        sendResponse({ ok: true });
      })
      .catch((error) =>
        sendResponse({ ok: false, error: error.message, code: error.code ?? "unknown" }),
      );
    return true;
  }

  return false;
});
