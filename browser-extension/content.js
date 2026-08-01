(() => {
  if (globalThis.hitsuContentScriptLoaded) return;
  globalThis.hitsuContentScriptCleanup?.();
  globalThis.hitsuContentScriptLoaded = true;

  const cleanupController = new AbortController();
  let suggestionHost;
  let suggestionRoot;
  let suggestionPanel;
  let suggestionInput;
  let suggestionEntries = [];
  let suggestionButtons = [];
  let selectedSuggestion = -1;
  let suggestionRequest = 0;
  let suppressSuggestions = false;
  let cachedEntries;
  let cacheTime = 0;

  globalThis.hitsuContentScriptCleanup = () => {
    cleanupController.abort();
    suggestionHost?.remove();
    globalThis.hitsuContentScriptLoaded = false;
  };

  function setInputValue(input, value) {
    const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
    setter?.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
  }

  function focusAfterFill(input) {
    suppressSuggestions = true;
    try {
      input.focus();
    } finally {
      suppressSuggestions = false;
    }
  }

  function isVisible(input) {
    if (typeof input.checkVisibility === "function" && !input.checkVisibility()) return false;
    if (input.offsetParent === null) return false;

    const bounds = input.getBoundingClientRect();
    return bounds.width > 0 && bounds.height > 0;
  }

  function queryAll(root, selector) {
    const matches = [];
    for (const element of root.querySelectorAll("*")) {
      if (element.matches(selector)) matches.push(element);
      if (element.shadowRoot) matches.push(...queryAll(element.shadowRoot, selector));
    }
    return matches;
  }

  function openRoots(root = document) {
    const roots = [root];
    for (const element of root.querySelectorAll("*")) {
      if (element.shadowRoot) roots.push(...openRoots(element.shadowRoot));
    }
    return roots;
  }

  const usernameTypes = ["email", "text", "tel"];
  const usernameHint = /user|email|e-mail|login|account|identifier|member|customer|client/i;

  function usernameMetadata(input) {
    return `${input.name} ${input.id} ${input.autocomplete} ${input.placeholder} ${input.ariaLabel}`;
  }

  function findUsernameInput(passwordInput) {
    const scope = passwordInput.form ?? passwordInput.getRootNode();
    const candidates = queryAll(
      scope,
      'input:not([type="hidden"]):not([disabled]):not([readonly])',
    );
    const passwordIndex = candidates.indexOf(passwordInput);
    const beforePassword = candidates.slice(0, passwordIndex).reverse();
    return (
      beforePassword.find(
        (input) => usernameTypes.includes(input.type) && usernameHint.test(usernameMetadata(input)),
      ) ?? beforePassword.find((input) => usernameTypes.includes(input.type))
    );
  }

  function visiblePasswordInputs() {
    return queryAll(document, 'input[type="password"]:not([disabled]):not([readonly])').filter(
      isVisible,
    );
  }

  function selectPasswordInput(inputs) {
    const current = inputs.find((input) =>
      input.autocomplete.toLowerCase().split(/\s+/).includes("current-password"),
    );
    if (current) return current;

    if (inputs.length !== 1) return null;
    const autocomplete = inputs[0].autocomplete.toLowerCase().split(/\s+/);
    return autocomplete.includes("new-password") ? null : inputs[0];
  }

  function inputHasFocus(input) {
    return input.getRootNode().activeElement === input;
  }

  function isSuggestionField(input) {
    if (
      !(input instanceof HTMLInputElement) ||
      input.disabled ||
      input.readOnly ||
      !isVisible(input)
    ) {
      return false;
    }

    const autocomplete = input.autocomplete.toLowerCase().split(/\s+/);
    const scope = input.form ?? input.getRootNode();
    const passwords = queryAll(
      scope,
      'input[type="password"]:not([disabled]):not([readonly])',
    ).filter(isVisible);

    if (input.type === "password") {
      return !autocomplete.includes("new-password") && selectPasswordInput(passwords) === input;
    }

    if (!usernameTypes.includes(input.type)) return false;
    const looksLikeUsername =
      autocomplete.includes("username") ||
      input.type === "email" ||
      usernameHint.test(usernameMetadata(input));
    return looksLikeUsername && (passwords.length === 0 || selectPasswordInput(passwords) !== null);
  }

  function ensureSuggestionPanel() {
    if (suggestionHost?.isConnected) return;

    suggestionHost = document.createElement("hitsu-login-suggestions");
    for (const [property, value] of Object.entries({
      position: "fixed",
      display: "none",
      "z-index": "2147483647",
      width: "max-content",
      "max-width": "calc(100vw - 16px)",
    })) {
      suggestionHost.style.setProperty(property, value, "important");
    }

    suggestionRoot = suggestionHost.attachShadow({ mode: "closed" });
    const style = document.createElement("style");
    style.textContent = `
      .panel {
        all: initial;
        box-sizing: border-box;
        display: grid;
        width: max-content;
        min-width: 240px;
        max-width: min(360px, calc(100vw - 16px));
        max-height: 240px;
        overflow: auto;
        padding: 6px;
        border: 1px solid #c9ced8;
        border-radius: 8px;
        background: #fff;
        box-shadow: 0 8px 24px rgb(0 0 0 / 18%);
        color: #182033;
        font: 14px/1.35 system-ui, sans-serif;
      }
      button {
        all: unset;
        box-sizing: border-box;
        display: grid;
        gap: 2px;
        width: 100%;
        max-width: 100%;
        padding: 8px 10px;
        border-radius: 5px;
        cursor: pointer;
      }
      button:hover, button.selected { background: #edf1ff; }
      strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
      span { overflow: hidden; color: #566176; text-overflow: ellipsis; white-space: nowrap; }
      .message { padding: 8px 10px; color: #a32323; }
      @media (prefers-color-scheme: dark) {
        .panel { border-color: #46516a; background: #1a2030; color: #e8ebf2; }
        button:hover, button.selected { background: #29334a; }
        span { color: #acb5c7; }
        .message { color: #ffaaaa; }
      }
    `;
    suggestionPanel = document.createElement("div");
    suggestionPanel.className = "panel";
    suggestionPanel.setAttribute("role", "listbox");
    suggestionRoot.append(style, suggestionPanel);
    document.documentElement.append(suggestionHost);
  }

  function positionSuggestions() {
    if (!suggestionInput || !suggestionHost) return;
    const bounds = suggestionInput.getBoundingClientRect();
    const width = suggestionHost.getBoundingClientRect().width || 240;
    const left = Math.max(8, Math.min(bounds.left, window.innerWidth - width - 8));
    suggestionHost.style.setProperty("left", `${left}px`, "important");
    suggestionHost.style.setProperty("top", `${bounds.bottom + 4}px`, "important");
  }

  function hideSuggestions() {
    suggestionRequest += 1;
    suggestionInput = undefined;
    suggestionEntries = [];
    suggestionButtons = [];
    selectedSuggestion = -1;
    suggestionHost?.style.setProperty("display", "none", "important");
  }

  function selectSuggestion(index) {
    if (suggestionEntries.length === 0) return;
    selectedSuggestion = Math.max(0, Math.min(index, suggestionEntries.length - 1));
    for (const [buttonIndex, button] of suggestionButtons.entries()) {
      button.classList.toggle("selected", buttonIndex === selectedSuggestion);
    }
  }

  function fillSuggestion(index) {
    const entry = suggestionEntries[index];
    const input = suggestionInput;
    if (!entry || !input) return;

    hideSuggestions();
    try {
      chrome.runtime.sendMessage({ type: "fill-login", id: entry.id }, (response) => {
        if (chrome.runtime.lastError || response?.ok || !inputHasFocus(input)) return;
        showSuggestionMessage(input, response?.error ?? "Could not fill this page.");
      });
    } catch {
      // The extension was reloaded while the page remained open.
    }
  }

  function showSuggestionMessage(input, message) {
    ensureSuggestionPanel();
    suggestionInput = input;
    suggestionPanel.replaceChildren();
    const status = document.createElement("div");
    status.className = "message";
    status.textContent = message;
    suggestionPanel.append(status);
    suggestionHost.style.setProperty("display", "block", "important");
    positionSuggestions();
  }

  function renderSuggestions(input, entries) {
    if (!inputHasFocus(input) || entries.length === 0) {
      hideSuggestions();
      return;
    }

    ensureSuggestionPanel();
    suggestionInput = input;
    suggestionEntries = entries;
    selectedSuggestion = -1;
    suggestionButtons = entries.map((entry, index) => {
      const button = document.createElement("button");
      button.type = "button";
      button.setAttribute("role", "option");
      const title = document.createElement("strong");
      title.textContent = entry.title || "Untitled login";
      const username = document.createElement("span");
      username.textContent = entry.username || "No username";
      button.append(title, username);
      button.addEventListener("pointerdown", (event) => event.preventDefault());
      button.addEventListener("click", () => fillSuggestion(index));
      return button;
    });
    suggestionPanel.replaceChildren(...suggestionButtons);
    suggestionHost.style.setProperty("display", "block", "important");
    positionSuggestions();
  }

  function requestSuggestions(input) {
    const request = ++suggestionRequest;
    if (cachedEntries && Date.now() - cacheTime < 10_000) {
      renderSuggestions(input, cachedEntries);
      return;
    }

    try {
      chrome.runtime.sendMessage({ type: "list-logins" }, (response) => {
        if (chrome.runtime.lastError || request !== suggestionRequest || !inputHasFocus(input)) {
          return;
        }
        if (!response?.ok) {
          hideSuggestions();
          return;
        }
        cachedEntries = response.entries;
        cacheTime = Date.now();
        renderSuggestions(input, cachedEntries);
      });
    } catch {
      // The extension was reloaded while the page remained open.
    }
  }

  function findUsernameOnlyInput() {
    const candidates = queryAll(
      document,
      'input:not([type="hidden"]):not([disabled]):not([readonly])',
    ).filter((input) => usernameTypes.includes(input.type) && isVisible(input));

    return candidates.find(
      (input) =>
        input.autocomplete.toLowerCase().split(/\s+/).includes("username") ||
        input.type === "email" ||
        usernameHint.test(usernameMetadata(input)),
    );
  }

  const observerOptions = {
    attributes: true,
    attributeFilter: ["class", "disabled", "hidden", "readonly", "style", "type"],
    childList: true,
    subtree: true,
  };

  function observeOpenRoots(observer) {
    for (const root of openRoots()) observer.observe(root, observerOptions);
  }

  function fillPasswordWhenAvailable(password) {
    let timeout;
    const observer = new MutationObserver(() => {
      const passwordInput = selectPasswordInput(visiblePasswordInputs());
      if (!passwordInput) return;

      observer.disconnect();
      clearTimeout(timeout);
      setInputValue(passwordInput, password);
      focusAfterFill(passwordInput);
    });

    observeOpenRoots(observer);
    timeout = setTimeout(() => observer.disconnect(), 15_000);
  }

  function fillAvailableLogin(message) {
    const passwordInputs = visiblePasswordInputs();
    const passwordInput = selectPasswordInput(passwordInputs);
    if (passwordInput) {
      const usernameInput = findUsernameInput(passwordInput);
      if (usernameInput && message.username) setInputValue(usernameInput, message.username);
      setInputValue(passwordInput, message.password);
      focusAfterFill(passwordInput);
      return { ok: true };
    }

    if (passwordInputs.length > 0) {
      const isSignup = passwordInputs.every((input) =>
        input.autocomplete.toLowerCase().split(/\s+/).includes("new-password"),
      );
      return {
        ok: false,
        error: isSignup
          ? "Hitsu does not fill signup forms"
          : "Hitsu could not determine which password field to fill",
      };
    }

    const usernameInput = findUsernameOnlyInput();
    if (!usernameInput || !message.username) return null;

    setInputValue(usernameInput, message.username);
    focusAfterFill(usernameInput);
    fillPasswordWhenAvailable(message.password);
    return { ok: true, usernameOnly: true };
  }

  function fillWhenAvailable(message, sendResponse) {
    let interval;
    let timeout;
    const observer = new MutationObserver(tryFill);

    function stop() {
      observer.disconnect();
      clearInterval(interval);
      clearTimeout(timeout);
    }

    function tryFill() {
      observeOpenRoots(observer);
      const response = fillAvailableLogin(message);
      if (!response) return;

      stop();
      sendResponse(response);
    }

    observeOpenRoots(observer);
    interval = setInterval(tryFill, 250);
    timeout = setTimeout(() => {
      stop();
      sendResponse({ ok: false, error: "No password field found on this page" });
    }, 5_000);
  }

  function focusedInput(event) {
    return event.composedPath().find((element) => element instanceof HTMLInputElement);
  }

  function suggestionsAllowedInFrame() {
    if (window.top === window) return true;
    try {
      return window.top.location.origin === window.location.origin;
    } catch {
      return false;
    }
  }

  if (suggestionsAllowedInFrame()) {
    document.addEventListener(
      "focusin",
      (event) => {
        if (suppressSuggestions) {
          hideSuggestions();
          return;
        }
        const input = focusedInput(event);
        if (!isSuggestionField(input)) {
          hideSuggestions();
          return;
        }
        requestSuggestions(input);
      },
      { capture: true, signal: cleanupController.signal },
    );
    document.addEventListener(
      "pointerdown",
      (event) => {
        const path = event.composedPath();
        if (!path.includes(suggestionHost) && !path.includes(suggestionInput)) hideSuggestions();
      },
      { capture: true, signal: cleanupController.signal },
    );
    document.addEventListener(
      "keydown",
      (event) => {
        if (focusedInput(event) !== suggestionInput || suggestionEntries.length === 0) {
          if (event.key === "Escape") hideSuggestions();
          return;
        }
        if (event.key === "ArrowDown") {
          event.preventDefault();
          selectSuggestion(selectedSuggestion + 1);
        } else if (event.key === "ArrowUp") {
          event.preventDefault();
          selectSuggestion(
            selectedSuggestion <= 0 ? suggestionEntries.length - 1 : selectedSuggestion - 1,
          );
        } else if (event.key === "Enter" && selectedSuggestion >= 0) {
          event.preventDefault();
          fillSuggestion(selectedSuggestion);
        } else if (event.key === "Escape") {
          hideSuggestions();
        }
      },
      { capture: true, signal: cleanupController.signal },
    );
    document.addEventListener("scroll", positionSuggestions, {
      capture: true,
      signal: cleanupController.signal,
    });
    window.addEventListener("resize", positionSuggestions, { signal: cleanupController.signal });
  }

  chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    if (sender.id !== chrome.runtime.id || message?.type !== "fill-login") return false;

    // Only fill frames whose origin matches the expected origin; silently
    // ignore cross-origin child frames (SSO widgets, embedded auth, etc.).
    if (message.expectedOrigin && window.location.origin !== message.expectedOrigin) {
      return false;
    }

    const response = fillAvailableLogin(message);
    if (response) {
      sendResponse(response);
      return false;
    }

    fillWhenAvailable(message, sendResponse);
    return true;
  });
})();
