import { describe, expect, it } from "vitest";
import type { MarkedTranslation } from "@/translate/markup.ts";
import { reconcileMarks } from "@/translate/reconcile.ts";

// Every sentence, tag position and lone translation below is what the real engine answered
// (en→fr, Safari on iOS and macOS, 2026-09-23) — only the expectations are written by hand.

/** The engine's answer, with its marks given as the text they cover, in order. */
function answer(sentence: string, ...marked: string[]): MarkedTranslation {
  let from = 0;
  const marks = marked.map((text) => {
    const start = sentence.indexOf(text, from);
    if (start < 0) throw new Error(`"${text}" is not in the sentence`);
    from = start + text.length;
    return { start, end: from };
  });
  return { sentence, marks };
}

/** What a reader sees marked. */
const shown = ({ sentence, marks }: MarkedTranslation) => marks.map((m) => sentence.slice(m.start, m.end));

const FRIDAY = "Ils expédient rarement le vendredi, même lorsque le client demande bien.";
const ATTEMPT = "Elle a abandonné après la troisième tentative, qui a surpris tout le monde dans la pièce.";
const PROPOSAL =
  "Personne ne s’attendait à ce qu’ils baissent la proposition aussi brusquement après tous les efforts que l’équipe avait déployés.";
const AUTHOR =
  "L'auteur est un économiste bien connu qui a étudié les effets de l'inflation sur l'épargne des ménages sur trois décennies.";

describe("reconcileMarks — a tag on the wrong word", () => {
  it("moves the mark to where the fragment's own translation stands", () => {
    // "They <b>seldom</b> ship…": the tag lands on the verb.
    expect(shown(reconcileMarks(answer(FRIDAY, "expédient"), "rarement"))).toEqual(["rarement"]);
  });

  it("marks the reader's words apart when the translation puts another between them", () => {
    // "<b>They seldom</b> ship…": "expédient" is ship's, and ship was not selected.
    expect(shown(reconcileMarks(answer(FRIDAY, "Ils expédient"), "Ils rarement"))).toEqual(["Ils", "rarement"]);
  });

  it("grows a mark that stopped short, recognising an inflected form", () => {
    // "<b>They seldom ship</b>…": alone it is "Ils sont rarement expédiés".
    expect(shown(reconcileMarks(answer(FRIDAY, "Ils expédient"), "Ils sont rarement expédiés"))).toEqual([
      "Ils expédient rarement",
    ]);
  });

  it("grows to the left as well, but only over neighbours", () => {
    expect(shown(reconcileMarks(answer(FRIDAY, "rarement"), "expédient rarement"))).toEqual(["expédient rarement"]);
    // "Ils" is not beside the mark: the word between is not the fragment's.
    expect(shown(reconcileMarks(answer(FRIDAY, "rarement"), "Ils rarement"))).toEqual(["rarement"]);
  });
});

describe("reconcileMarks — a tag that was right", () => {
  it("keeps a mark the fragment's translation confirms", () => {
    expect(shown(reconcileMarks(answer(ATTEMPT, "a abandonné"), "A abandonné"))).toEqual(["a abandonné"]);
    expect(shown(reconcileMarks(answer(AUTHOR, "les effets de l'inflation"), "Les effets de l’inflation"))).toEqual([
      "les effets de l'inflation",
    ]);
  });

  it("keeps the short words the sentence's grammar put there", () => {
    // Alone, "gave up after the" is "abandonné après le": no "a", and the wrong article.
    expect(shown(reconcileMarks(answer(ATTEMPT, "a abandonné après la"), "abandonné après le"))).toEqual([
      "a abandonné après la",
    ]);
  });

  it("reads through an elided clitic", () => {
    // "s’attendait" is "attendait" with an elided "s’" — the same word as the lone translation's.
    expect(
      shown(reconcileMarks(answer(PROPOSAL, "Personne ne s’attendait", "ce qu’ils"), "Personne ne les attendait")),
    ).toEqual(["Personne ne s’attendait à ce qu’ils"]);
  });

  it("drops a word the reader did not select, and keeps the split the engine made", () => {
    // "author is a well-known" — "économiste" is economist's, which was not selected.
    expect(
      shown(reconcileMarks(answer(AUTHOR, "L'auteur est un", "bien connu"), "L'auteur est un auteur bien connu")),
    ).toEqual(["L'auteur est un", "bien connu"]);
  });

  it("leaves the mark alone when the fragment's translation shares no word with it", () => {
    // Alone, "put up with" is "mis en place avec" — a different sense; it says nothing here.
    const put = "Il a dû supporter le bruit pendant une semaine avant que les constructeurs ne partent enfin.";
    expect(shown(reconcileMarks(answer(put, "supporter"), "mis en place avec"))).toEqual(["supporter"]);
  });

  it("drops nothing when the fragment's translation may have used a synonym", () => {
    const house = "Ils vivent dans une maison énorme au bord du lac.";
    expect(shown(reconcileMarks(answer(house, "maison énorme"), "l'immense maison"))).toEqual(["maison énorme"]);
  });

  it("keeps a whole-sentence mark whole", () => {
    expect(shown(reconcileMarks(answer(FRIDAY, FRIDAY), FRIDAY))).toEqual([FRIDAY]);
  });
});

describe("reconcileMarks — measured on the evaluation corpus", () => {
  // Cases from the 100-sentence evaluation recorded in the change's design.

  it("drops a stray article the engine tagged away from the rest", () => {
    const always = "Elle arrive toujours avant tout le monde.";
    expect(shown(reconcileMarks(answer(always, "Elle", "toujours", "le"), "Elle toujours"))).toEqual([
      "Elle",
      "toujours",
    ]);
  });

  it("does not grow over a short word, which is evidence of nothing", () => {
    // "broke down" alone is "en panne": its "en" is not the "en" of "en larmes".
    const tears = "Il a fondu en larmes quand il a appris la nouvelle.";
    expect(shown(reconcileMarks(answer(tears, "a fondu"), "en panne"))).toEqual(["a fondu"]);
  });

  it("never pulls in a full word the engine left out between two marks", () => {
    const often = "Nous rendons souvent visite à nos grands-parents en été.";
    expect(shown(reconcileMarks(answer(often, "Nous", "souvent"), "Nous avons souvent"))).toEqual(["Nous", "souvent"]);
  });

  it("glues two marks over the short words between them, but not past their edge", () => {
    const keys = "Elle a perdu ses clés de voiture quelque part dans le parc.";
    expect(shown(reconcileMarks(answer(keys, "clés", "voiture"), "clés de voiture"))).toEqual(["clés de voiture"]);
    const bag = "Il a laissé le chat sortir du sac à propos de la fête surprise.";
    expect(
      shown(reconcileMarks(answer(bag, "a laissé le chat sortir du sac", "de"), "Laisser le chat sortir du sac")),
    ).toEqual(["a laissé le chat sortir du sac"]);
  });

  it("trims a word that belongs to an unselected neighbour", () => {
    // "had been working" — "depuis" is "for" (ten years), which was not selected.
    const years = "Ils y travaillaient depuis dix ans.";
    expect(shown(reconcileMarks(answer(years, "travaillaient depuis"), "Il travaillait"))).toEqual(["travaillaient"]);
  });

  it("never takes every mark away", () => {
    const friday = "Ils expédient le vendredi.";
    const both = answer(friday, "expédient", "vendredi");
    expect(reconcileMarks(both, "le le")).toBe(both);
  });
});

describe("reconcileMarks — what it will not do", () => {
  it("invents no mark where the engine placed none", () => {
    // "The" before "author": the engine dropped the tag. "Le" is not in the sentence anyway,
    // but even a match would not be a reason to mark.
    const none = answer(AUTHOR);
    expect(reconcileMarks(none, "Le")).toBe(none);
    expect(reconcileMarks(answer(FRIDAY), "rarement").marks).toEqual([]);
  });

  it("leaves a mark that holds no word as the engine gave it", () => {
    const comma = answer(FRIDAY, ",");
    expect(reconcileMarks(comma, "jamais")).toBe(comma);
  });

  it("does not move the mark to a translation that stands in the sentence twice", () => {
    const twice = "Le client paie toujours, et le client attend.";
    // "client" twice: no telling which is meant, and neither is beside the mark — it stays.
    expect(shown(reconcileMarks(answer(twice, "toujours"), "client"))).toEqual(["toujours"]);
  });

  it("grows by no more words than the fragment's translation has", () => {
    // One word alone: the mark may take one neighbour, not the whole run of matching ones.
    const run = "Il regardait rarement rarement rarement.";
    expect(shown(reconcileMarks(answer(run, "regardait"), "rarement"))).toEqual(["rarement"]);
  });

  it("keeps a mark the fragment's lone translation matches only in part", () => {
    // "hey seldom ship" (a cut selection) — alone "Hey navire rarement": "rarement" is found
    // beside the mark, "navire" nowhere, so nothing is dropped and the mark grows.
    const cut = "Ils expédient rarement vendredi, même quand le client demande bien.";
    expect(shown(reconcileMarks(answer(cut, "Ils expédient"), "Hey navire rarement"))).toEqual([
      "Ils expédient rarement",
    ]);
  });
});
