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
      const message =
        response?.code === "not_installed"
          ? "Hitsu browser integration is not installed."
          : response?.code === "not_running" || response?.code === "locked"
            ? "Open and unlock Hitsu first."
            : response?.code === "insecure_page"
              ? "Hitsu will not fill passwords on HTTP pages."
              : (response?.error ?? "Open and unlock Hitsu first.");
      showMessage(message, "error");
      return;
    }
    if (response.entries.length === 0) {
      showMessage("No matching logins for this site. Open Hitsu to add or unlock entries.");
      return;
    }

    content.replaceChildren();

    const search = document.createElement("input");
    search.type = "text";
    search.className = "search";
    search.placeholder = "Filter logins…";

    const buttons = response.entries.map((entry) => {
      const button = document.createElement("button");
      button.className = "login";
      button.dataset.title = (entry.title || "").toLowerCase();
      button.dataset.username = (entry.username || "").toLowerCase();
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
      return button;
    });

    function visibleButtons() {
      return buttons.filter((b) => !b.classList.contains("hidden"));
    }

    function moveFocus(direction) {
      const visible = visibleButtons();
      if (visible.length === 0) return;
      const current = document.activeElement;
      const idx = visible.indexOf(current);
      const next = idx === -1 ? 0 : Math.min(Math.max(idx + direction, 0), visible.length - 1);
      visible[next].focus();
    }

    search.addEventListener("input", () => {
      const query = search.value.toLowerCase();
      for (const button of buttons) {
        const match =
          !query || button.dataset.title.includes(query) || button.dataset.username.includes(query);
        button.classList.toggle("hidden", !match);
      }
    });

    search.addEventListener("keydown", (event) => {
      if (event.key === "ArrowDown") {
        event.preventDefault();
        moveFocus(1);
      } else if (event.key === "Escape") {
        search.value = "";
        search.dispatchEvent(new Event("input"));
      }
    });

    for (const button of buttons) {
      button.addEventListener("keydown", (event) => {
        if (event.key === "ArrowDown") {
          event.preventDefault();
          moveFocus(1);
        } else if (event.key === "ArrowUp") {
          event.preventDefault();
          moveFocus(-1);
        } else if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          button.click();
        }
      });
    }

    content.append(search, ...buttons);
  });
}
