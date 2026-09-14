// Activation page of the host app (French copy, like the extension). ViewController.swift
// calls show() once loaded; on macOS it passes the real extension state from
// SFSafariExtensionManager, and "Activer dans Safari" asks the app to open Safari's settings.
function show(platform, enabled, useSettingsInsteadOfPreferences) {
    document.body.classList.add(`platform-${platform}`);

    // Before macOS 13, Safari called its settings "Préférences".
    if (useSettingsInsteadOfPreferences === false) {
        document.getElementsByClassName('platform-mac state-on')[0].innerText = "Extension active. Tu peux la désactiver dans les préférences de Safari, section Extensions.";
        document.getElementsByClassName('platform-mac state-off')[0].innerText = "Extension désactivée. Active-la dans les préférences de Safari, section Extensions.";
        document.getElementsByClassName('platform-mac state-unknown')[0].innerText = "Active Cymbra Lingua dans les préférences de Safari, section Extensions.";
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
