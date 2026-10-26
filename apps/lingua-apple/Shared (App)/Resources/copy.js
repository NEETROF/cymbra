// The activation page's copy, one table per language (localise-lingua-apple-host, D2). The French
// is Main.html's text byte for byte — the page keeps it in place, and
// apps/lingua-extension/test/apple-activation-page.spec.ts holds the two equal. The English,
// Spanish and Italian are drafts after it, the Italian reviewed by the Italian reviewer agent. Each table names every language its natives study in the shipped
// pairs (enable-lingua-french D7), the French page included: English, Spanish and Italian for French
// natives (en-fr, es-fr, it-fr — Italian since enable-lingua-italian D7), Spanish, French and Italian
// for English natives (es-en, fr-en, it-en — Italian since enable-lingua-italian-for-english-speakers
// D6), English, French and Italian for Spanish natives (en-es, fr-es, it-es — Italian since
// enable-lingua-italian-for-spanish-speakers D6), English for Italian natives (en-it, held ahead of
// its pair by localise-lingua-apple-host-italian D3, shown from the Italian programme's change 45;
// changes 50 and 54 name French and Spanish when fr-it and es-it ship). Its last step chooses the
// first language's level, then — for a native with two pairs or more — says where another is
// ticked: the extension's settings, under its Language tab.
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
            lede: "Lis le web en anglais, en espagnol ou en italien : les mots que tu ne connais pas encore sont surlignés, directement dans Safari.",
            step1: "Ouvre <strong>Réglages → Apps → Safari → Extensions</strong>.",
            step2: "Active <strong>Cymbra Lingua</strong>, puis règle « Autres sites web » sur <strong>Autoriser</strong>.",
            step3: "Dans Safari, ouvre l’extension depuis le <strong>menu de la barre d’adresse</strong> et choisis ton niveau d’anglais. Pour l’espagnol ou l’italien, coche-le dans les <strong>Réglages</strong> de l’extension, onglet <strong>Langue</strong>, puis choisis son niveau.",
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
        // A draft after the French, for English natives, who study Spanish, French and Italian (es-en,
        // fr-en, it-en).
        en: {
            iconAlt: "Cymbra Lingua icon",
            title: "Cymbra Lingua",
            lede: "Read the web in Spanish, French or Italian: the words you don't know yet are highlighted, right in Safari.",
            step1: "Open <strong>Settings → Apps → Safari → Extensions</strong>.",
            step2: "Turn on <strong>Cymbra Lingua</strong>, then set “Other Websites” to <strong>Allow</strong>.",
            step3: "In Safari, open the extension from the <strong>address bar menu</strong> and choose your level of Spanish. For French or Italian, check it in the extension's <strong>Settings</strong>, under <strong>Language</strong>, then choose its level.",
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
        // A draft after the French, for Spanish natives, who study English, French and Italian (en-es,
        // fr-es, it-es): tú, no vosotros.
        es: {
            iconAlt: "Icono de Cymbra Lingua",
            title: "Cymbra Lingua",
            lede: "Lee la web en inglés, en francés o en italiano: las palabras que todavía no conoces aparecen resaltadas, directamente en Safari.",
            step1: "Abre <strong>Ajustes → Apps → Safari → Extensiones</strong>.",
            step2: "Activa <strong>Cymbra Lingua</strong> y luego pon «Otros sitios web» en <strong>Permitir</strong>.",
            step3: "En Safari, abre la extensión desde el <strong>menú de la barra de direcciones</strong> y elige tu nivel de inglés. Para el francés o el italiano, márcalo en los <strong>Ajustes</strong> de la extensión, pestaña <strong>Idioma</strong>, y luego elige su nivel.",
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
        // A draft after the French, for Italian natives, who study English (en-it): tu, no voi
        // (localise-lingua-apple-host-italian D3, D6). Unreachable until a build declares `it`.
        it: {
            iconAlt: "Icona di Cymbra Lingua",
            title: "Cymbra Lingua",
            lede: "Leggi il web in inglese: le parole che non conosci ancora sono evidenziate, direttamente in Safari.",
            step1: "Apri <strong>Impostazioni → App → Safari → Estensioni</strong>.",
            step2: "Attiva <strong>Cymbra Lingua</strong>, poi imposta «Altri siti web» su <strong>Consenti</strong>.",
            step3: "In Safari, apri l’estensione dal <strong>menu della barra degli indirizzi</strong> e scegli il tuo livello di inglese.",
            noteIos: "Ricarica le schede già aperte perché vengano evidenziate.",
            stateUnknown: "Attiva Cymbra Lingua nelle impostazioni di Safari, sezione Estensioni.",
            stateOn: "Estensione attiva. Puoi disattivarla nelle impostazioni di Safari, sezione Estensioni.",
            stateOff: "Estensione disattivata. Attivala nelle impostazioni di Safari, sezione Estensioni.",
            openPreferences: "Attiva in Safari",
            noteMac: "Ricarica le schede già aperte perché vengano evidenziate.",
            stateUnknownPreferences: "Attiva Cymbra Lingua nelle preferenze di Safari, sezione Estensioni.",
            stateOnPreferences: "Estensione attiva. Puoi disattivarla nelle preferenze di Safari, sezione Estensioni.",
            stateOffPreferences: "Estensione disattivata. Attivala nelle preferenze di Safari, sezione Estensioni.",
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
