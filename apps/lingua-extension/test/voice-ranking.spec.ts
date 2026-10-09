import { readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { pickVoice, rankVoices } from "@/reading/speech.ts";
import { voiceFixture } from "./helpers.ts";

// add-lingua-french-read-aloud D9: English's and Spanish's voices, pinned before French preferred
// a voice of France. For every list captured on a real browser (`test/fixtures/voices/`), both
// languages, Android's voices allowed or not and the online voices allowed or not — 56 rankings —
// the automatic voice (`pickVoice` with no preference) and the ranked `voiceURI`s (`rankVoices`).
// Recorded on `main` before any edit of `speech.ts`, and passing as committed from then on: a rule
// that moves an English or Spanish voice moves a line here. Re-blessed only with
// `yarn vitest run test/voice-ranking.spec.ts -u`, in a pull request that says why.

const VOICES = join(dirname(fileURLToPath(import.meta.url)), "fixtures", "voices");
const CAPTURES = readdirSync(VOICES)
  .filter((file) => file.endsWith(".json"))
  .map((file) => file.slice(0, -".json".length))
  .sort();
const LANGUAGES = ["en", "es"];
const SWITCHES = [false, true];

/** One block per ranking: its settings, the automatic voice, then the ranked voices, one per line. */
function ranking(capture: string, lang: string, androidVoices: boolean, remoteVoices: boolean): string {
  const voices = voiceFixture(capture);
  const automatic = pickVoice(voices, lang, null, androidVoices, remoteVoices);
  const ranked = rankVoices(voices, lang, androidVoices, remoteVoices);
  const settings = `android voices ${androidVoices ? "on" : "off"}, online voices ${remoteVoices ? "on" : "off"}`;
  return [
    `## ${capture} ${lang} — ${settings}`,
    `automatic: ${automatic?.voiceURI ?? "(none)"}`,
    `ranked (${ranked.length}):`,
    ...ranked.map((voice) => `  ${voice.voiceURI}`),
  ].join("\n");
}

describe("English and Spanish voices over every captured list", () => {
  it("rank and choose as they did before French preferred France", async () => {
    const blocks = CAPTURES.flatMap((capture) =>
      LANGUAGES.flatMap((lang) =>
        SWITCHES.flatMap((android) => SWITCHES.map((remote) => ranking(capture, lang, android, remote))),
      ),
    );
    expect(blocks).toHaveLength(56);
    await expect(`${blocks.join("\n\n")}\n`).toMatchFileSnapshot("./baseline/voice-ranking.txt");
  });
});
