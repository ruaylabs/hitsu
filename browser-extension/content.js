(() => {
  if (globalThis.hitsuContentScriptLoaded) return;
  globalThis.hitsuContentScriptLoaded = true;

  function setInputValue(input, value) {
    const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
    setter?.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
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

  function findVisiblePasswordInput() {
    return queryAll(document, 'input[type="password"]:not([disabled]):not([readonly])').find(
      isVisible,
    );
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

  function fillPasswordWhenAvailable(password) {
    let timeout;
    const observer = new MutationObserver(() => {
      const passwordInput = findVisiblePasswordInput();
      if (!passwordInput) return;

      observer.disconnect();
      clearTimeout(timeout);
      setInputValue(passwordInput, password);
      passwordInput.focus();
    });

    const observerOptions = {
      attributes: true,
      attributeFilter: ["class", "disabled", "hidden", "readonly", "style", "type"],
      childList: true,
      subtree: true,
    };
    for (const root of openRoots()) observer.observe(root, observerOptions);
    timeout = setTimeout(() => observer.disconnect(), 15_000);
  }

  chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    if (sender.id !== chrome.runtime.id || message?.type !== "fill-login") return false;

    const passwordInput = findVisiblePasswordInput();
    if (passwordInput) {
      const usernameInput = findUsernameInput(passwordInput);
      if (usernameInput && message.username) setInputValue(usernameInput, message.username);
      setInputValue(passwordInput, message.password);
      passwordInput.focus();
      sendResponse({ ok: true });
      return false;
    }

    const usernameInput = findUsernameOnlyInput();
    if (!usernameInput || !message.username) {
      sendResponse({ ok: false, error: "No password field found on this page" });
      return false;
    }

    setInputValue(usernameInput, message.username);
    usernameInput.focus();
    fillPasswordWhenAvailable(message.password);
    sendResponse({ ok: true, usernameOnly: true });
    return false;
  });
})();
