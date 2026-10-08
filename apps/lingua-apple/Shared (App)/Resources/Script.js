// Activation page of the host app, in the language copy.js filled it in (localise-lingua-apple-host).
// ViewController.swift calls show() once loaded; on macOS it passes the real extension state from
// SFSafariExtensionManager, and the button asks the app to open Safari's settings.
function show(platform, enabled, useSettingsInsteadOfPreferences) {
    document.body.classList.add(`platform-${platform}`);

    // Before macOS 13, Safari called its settings "Préférences": the page's language has those variants too.
    if (useSettingsInsteadOfPreferences === false) {
        document.getElementsByClassName('platform-mac state-on')[0].textContent = linguaCopy.text("stateOnPreferences");
        document.getElementsByClassName('platform-mac state-off')[0].textContent = linguaCopy.text("stateOffPreferences");
        document.getElementsByClassName('platform-mac state-unknown')[0].textContent = linguaCopy.text("stateUnknownPreferences");
    }

    if (typeof enabled === "boolean") {
        document.body.classList.toggle(`state-on`, enabled);
        document.body.classList.toggle(`state-off`, !enabled);
    } else {
        document.body.classList.remove(`state-on`);
        document.body.classList.remove(`state-off`);
    }
}

function openPreferences() {
    webkit.messageHandlers.controller.postMessage("open-preferences");
}

document.querySelector("button.open-preferences").addEventListener("click", openPreferences);
