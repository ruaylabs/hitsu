function setInputValue(input, value) {
  const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
  setter?.call(input, value);
  input.dispatchEvent(new Event("input", { bubbles: true }));
  input.dispatchEvent(new Event("change", { bubbles: true }));
}

function findUsernameInput(passwordInput) {
  const form = passwordInput.form ?? document;
  const candidates = [...form.querySelectorAll('input:not([type="hidden"]):not([disabled])')];
  const passwordIndex = candidates.indexOf(passwordInput);
  const beforePassword = candidates.slice(0, passwordIndex).reverse();
  return (
    beforePassword.find(
      (input) =>
        ["email", "text", "tel"].includes(input.type) &&
        /user|email|login|account/i.test(`${input.name} ${input.id} ${input.autocomplete}`),
    ) ?? beforePassword.find((input) => ["email", "text", "tel"].includes(input.type))
  );
}

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (sender.id !== chrome.runtime.id || message?.type !== "fill-login") return false;

  const passwordInput = document.querySelector(
    'input[type="password"]:not([disabled]):not([readonly])',
  );
  if (!passwordInput) {
    sendResponse({ ok: false, error: "No password field found on this page" });
    return false;
  }

  const usernameInput = findUsernameInput(passwordInput);
  if (usernameInput && message.username) setInputValue(usernameInput, message.username);
  setInputValue(passwordInput, message.password);
  passwordInput.focus();
  sendResponse({ ok: true });
  return false;
});
