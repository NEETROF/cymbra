import type { onboarding as fr } from "../fr/onboarding.ts";

// The onboarding page's copy in Italian — a draft after the French (src/i18n/README.md;
// add-lingua-italian-interface D1). « Bienvenue », genderless in French, is « Ti diamo il
// benvenuto », never the masculine « Benvenuto » (D4); « Principiante » is both genders. Unverified:
// `stepPin`'s « Fissa » (Chrome's and Firefox's Italian pin action, « Fissa nella barra degli strumenti »
// in Firefox as far as known; no vendor page cited) — the owner's dogfood on an Italian Chrome and
// Firefox settles it (review-lingua-italian-wording).

export const onboarding: typeof fr = {
  pageTitle: "Ti diamo il benvenuto — Cymbra Lingua",
  heading: "Ti diamo il benvenuto in Cymbra Lingua",
  lead: "Evidenzia le parole che non conosci ancora mentre leggi, poi ripassale al momento giusto, senza lasciare la tua pagina.",
  whichLanguages: "Quali lingue stai imparando?",
  levelNote:
    "Le parole al di sotto del tuo livello non verranno evidenziate. Nessuna parola viene data per conosciuta finché non hai scelto. Potrai cambiare livello in qualsiasi momento nelle impostazioni.",
  toStart: "Per iniziare",
  stepPin: "Fissa l'icona di Cymbra Lingua nella barra degli strumenti del browser.",
  stepOpen: "Apri una pagina nella lingua che stai imparando e fai clic sull'icona per analizzarla.",
  stepClick: "Fai clic su una parola evidenziata per tradurla o aggiungerla al tuo mazzo.",
  accountTitle: "Ritrova le tue parole ovunque (facoltativo)",
  accountLead:
    "Con un account Cymbra, lo stesso di Cymbra Music, le tue parole, il tuo mazzo e le tue statistiche si sincronizzano tra i tuoi dispositivi. Senza account, le tue parole, il loro stato e il tuo mazzo restano su questo dispositivo.",
  createAccount: "Crea un account",
  later: "Più tardi",
  beginnerChip: "Principiante: parto da zero",
  levelSaved: (level) => `Livello salvato: ${level}. Puoi chiudere questa scheda e iniziare a leggere.`,
  beginnerSaved: "Fatto: si parte da zero. Puoi chiudere questa scheda e iniziare a leggere.",
  // — The anonymous counts' first-run sentence (add-lingua-usage-counts D7) —
  usageInform:
    "Lingua conta in forma anonima alcune tappe (installazione, primi passi, ritorni), senza alcun identificativo; nulla di ciò che leggi lascia il tuo dispositivo. Puoi disattivare questi conteggi in Impostazioni → Dati.",
  usageAsk:
    "Permetti a Lingua di contare in forma anonima alcune tappe (installazione, primi passi, ritorni), senza alcun identificativo? Nulla di ciò che leggi lascia il tuo dispositivo, e potrai cambiare idea in Impostazioni → Dati.",
  usageYes: "Sì, consenti",
  usageNo: "No, grazie",
  usageYesDone: "Grazie! Puoi disattivare questi conteggi in qualsiasi momento in Impostazioni → Dati.",
  usageNoDone: "Fatto: non verrà contato nulla.",
  usagePrivacy: "Informativa sulla privacy",
};
