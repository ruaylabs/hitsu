const content = document.querySelector("#content");
const site = document.querySelector("#site");

function showMessage(message, kind = "muted") {
  content.replaceChildren();
  const paragraph = document.createElement("p");
  paragraph.className = kind;
  paragraph.textContent = message;
  content.append(paragraph);
}

const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
let origin;
try {
  origin = new URL(tab.url).origin;
  site.textContent = new URL(tab.url).hostname;
} catch {
  showMessage("Hitsu can only fill HTTP and HTTPS pages.", "error");
}

if (origin) {
  chrome.runtime.sendMessage({ type: "list-logins" }, (response) => {
    if (!response?.ok) {
      const unavailable = response?.error?.includes("host")
        ? "Hitsu browser integration is not installed."
        : (response?.error ?? "Open and unlock Hitsu first.");
      showMessage(unavailable, "error");
      return;
    }
    if (response.entries.length === 0) {
      showMessage("No matching logins for this site.");
      return;
    }

    content.replaceChildren();
    for (const entry of response.entries) {
      const button = document.createElement("button");
      button.className = "login";
      const title = document.createElement("strong");
      title.textContent = entry.title || "Untitled login";
      const username = document.createElement("span");
      username.textContent = entry.username || "No username";
      button.append(title, username);
      button.addEventListener("click", () => {
        button.disabled = true;
        chrome.runtime.sendMessage({ type: "fill-login", id: entry.id }, (fillResponse) => {
          if (fillResponse?.ok) window.close();
          else {
            button.disabled = false;
            showMessage(fillResponse?.error ?? "Could not fill this page.", "error");
          }
        });
      });
      content.append(button);
    }
  });
}
