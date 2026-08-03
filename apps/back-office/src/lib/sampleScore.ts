// The shared audition sample: the same bundled score the Flutter app plays when
// you tap a sound (Ode to Joy), so the back-office "Listen" preview auditions a
// font with the exact same piece. Bundled as a raw MusicXML string and encoded
// once to the bytes the wasm scheduler/renderer expect.
import sampleXml from "@/assets/ode_to_joy.musicxml?raw";

export const sampleScoreBytes: Uint8Array = new TextEncoder().encode(sampleXml);
