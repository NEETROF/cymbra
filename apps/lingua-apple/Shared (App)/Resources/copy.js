// The activation page's copy, one table per language (localise-lingua-apple-host, D2). The French
// is Main.html's text byte for byte — the page keeps it in place, and
// apps/lingua-extension/test/apple-activation-page.spec.ts holds the two equal. The English and
// Spanish are drafts after it. Each table names every language its natives study in the shipped
// pairs (enable-lingua-french D7), the French page included: English and Spanish for French natives
// (en-fr, es-fr), Spanish and French for English natives (es-en, fr-en), English and French for
// Spanish natives (en-es, fr-es). Its last step chooses the first language's level, then says where
// another is ticked: the extension's settings, under its Language tab.
//
// ViewController.swift sets window.linguaLanguage before the page loads — the extension's
// interface language once the extension has run, the device's language before, among the
// languages the app declares. On DOMContentLoaded the page is filled from that language's table
// and says it in `lang`; a page asked for its own language is left untouched. Entries that hold
// markup (<strong>) are HTML fragments of this bundled file, set with innerHTML: nothing in them
// ever comes from Swift or from a link.
(function () {
    "use strict";

    const COPY = {
        fr: {
            iconAlt: "Icône Cymbra Lingua",
            title: "Cymbra Lingua",
            lede: "Lis le web en anglais ou en espagnol : les mots que tu ne connais pas encore sont surlignés, directement dans Safari.",
            step1: "Ouvre <strong>Réglages → Apps → Safari → Extensions</strong>.",
            step2: "Active <strong>Cymbra Lingua</strong>, puis règle « Autres sites web » sur <strong>Autoriser</strong>.",
            step3: "Dans Safari, ouvre l’extension depuis le <strong>menu de la barre d’adresse</strong> et choisis ton niveau d’anglais. Pour l’espagnol, coche-le dans les <strong>Réglages</strong> de l’extension, onglet <strong>Langue</strong>, puis choisis ton niveau d’espagnol.",
            noteIos: "Recharge les onglets déjà ouverts pour qu’ils soient surlignés.",
            stateUnknown: "Active Cymbra Lingua dans les réglages de Safari, section Extensions.",
            stateOn: "Extension active. Tu peux la désactiver dans les réglages de Safari, section Extensions.",
            stateOff: "Extension désactivée. Active-la dans les réglages de Safari, section Extensions.",
            openPreferences: "Activer dans Safari",
            noteMac: "Recharge les onglets déjà ouverts pour qu’ils soient surlignés.",
            // Before macOS 13, Safari called its settings "Préférences".
            stateUnknownPreferences: "Active Cymbra Lingua dans les préférences de Safari, section Extensions.",
            stateOnPreferences: "Extension active. Tu peux la désactiver dans les préférences de Safari, section Extensions.",
            stateOffPreferences: "Extension désactivée. Active-la dans les préférences de Safari, section Extensions.",
        },
        // A draft after the French, for English natives, who study Spanish and French (es-en, fr-en).
        en: {
            iconAlt: "Cymbra Lingua icon",
            title: "Cymbra Lingua",
            lede: "Read the web in Spanish or French: the words you don't know yet are highlighted, right in Safari.",
            step1: "Open <strong>Settings → Apps → Safari → Extensions</strong>.",
            step2: "Turn on <strong>Cymbra Lingua</strong>, then set “Other Websites” to <strong>Allow</strong>.",
            step3: "In Safari, open the extension from the <strong>address bar menu</strong> and choose your level of Spanish. For French, check it in the extension's <strong>Settings</strong>, under <strong>Language</strong>, then choose your level of French.",
            noteIos: "Reload the tabs you already had open so they get highlighted.",
            stateUnknown: "Turn on Cymbra Lingua in Safari's settings, under Extensions.",
            stateOn: "Extension on. You can turn it off in Safari's settings, under Extensions.",
            stateOff: "Extension off. Turn it on in Safari's settings, under Extensions.",
            openPreferences: "Turn on in Safari",
            noteMac: "Reload the tabs you already had open so they get highlighted.",
            stateUnknownPreferences: "Turn on Cymbra Lingua in Safari's preferences, under Extensions.",
            stateOnPreferences: "Extension on. You can turn it off in Safari's preferences, under Extensions.",
            stateOffPreferences: "Extension off. Turn it on in Safari's preferences, under Extensions.",
        },
        // A draft after the French, for Spanish natives, who study English and French (en-es, fr-es): tú,
        // no vosotros.
        es: {
            iconAlt: "Icono de Cymbra Lingua",
            title: "Cymbra Lingua",
            lede: "Lee la web en inglés o en francés: las palabras que todavía no conoces aparecen resaltadas, directamente en Safari.",
            step1: "Abre <strong>Ajustes → Apps → Safari → Extensiones</strong>.",
            step2: "Activa <strong>Cymbra Lingua</strong> y luego pon «Otros sitios web» en <strong>Permitir</strong>.",
            step3: "En Safari, abre la extensión desde el <strong>menú de la barra de direcciones</strong> y elige tu nivel de inglés. Para el francés, márcalo en los <strong>Ajustes</strong> de la extensión, pestaña <strong>Idioma</strong>, y luego elige tu nivel de francés.",
            noteIos: "Recarga las pestañas que ya tenías abiertas para que se resalten.",
            stateUnknown: "Activa Cymbra Lingua en los ajustes de Safari, sección Extensiones.",
            stateOn: "Extensión activa. Puedes desactivarla en los ajustes de Safari, sección Extensiones.",
            stateOff: "Extensión desactivada. Actívala en los ajustes de Safari, sección Extensiones.",
            openPreferences: "Activar en Safari",
            noteMac: "Recarga las pestañas que ya tenías abiertas para que se resalten.",
            stateUnknownPreferences: "Activa Cymbra Lingua en las preferencias de Safari, sección Extensiones.",
            stateOnPreferences: "Extensión activa. Puedes desactivarla en las preferencias de Safari, sección Extensiones.",
            stateOffPreferences: "Extensión desactivada. Actívala en las preferencias de Safari, sección Extensiones.",
        },
    };

    // The entries that hold markup, set with innerHTML; every other one is text.
    const FRAGMENTS = ["step1", "step2", "step3"];

    // The language the page is written in: what it shows when none is asked, or one it has not.
    const PAGE_LANGUAGE = document.documentElement.lang;

    function language() {
        const asked = window.linguaLanguage;
        return Object.prototype.hasOwnProperty.call(COPY, asked) ? asked : PAGE_LANGUAGE;
    }

    function text(key) {
        return COPY[language()][key];
    }

    function fill(doc, lang) {
        const table = COPY[lang];
        for (const node of doc.querySelectorAll("[data-copy]")) {
            const key = node.dataset.copy;
            if (FRAGMENTS.includes(key)) {
                node.innerHTML = table[key];
            } else {
                node.textContent = table[key];
            }
        }
        for (const node of doc.querySelectorAll("[data-copy-alt]")) {
            node.setAttribute("alt", table[node.dataset.copyAlt]);
        }
        doc.documentElement.lang = lang;
    }

    // Script.js reads the page's language and its texts here; the test reads the tables.
    window.linguaCopy = { COPY, FRAGMENTS, language, text, fill };

    document.addEventListener("DOMContentLoaded", () => {
        const lang = language();
        if (lang !== PAGE_LANGUAGE) {
            fill(document, lang);
        }
    });
})();
