// A read-only SVG notation painter, mirroring the Flutter app's PartitionPainter so
// the moderation console draws the same engraving. It consumes the geometry the wasm
// module returns (the app's own `layout_systems`) and emits an SVG string: staff
// lines, the grand-staff bracket, clefs, key/time signatures, barlines, note heads,
// stems, flags, beams, accidentals, augmentation dots, rests and ledger lines.
//
// Percussion (change: add-drum-notation-render): a percussion document routes to the
// percussion engraving rules — single five-line staff under the percussion clef, no
// armature ever, written-position placement, the crate's head classes (x heads for
// cymbals), two-voice stems/rests — per `music-percussion-engraving`.
//
// Intentionally NOT drawn in v1 (best-effort later, not required by the preview
// contract): expression/dynamics directions, lyrics, ties, slurs, tuplet
// numbers/brackets, and the playback cursor (playback is out of scope here).

import type { Clef, NoteEvent, RenderedScore, RepeatMarks, ScoreDocument, System, Unpitched } from "./geometry";
import * as S from "./smufl";

// Staff space (px): every dimension derives from it, matching partition_painter.dart.
const s = 12;
const staffHeight = 4 * s;
const interStaff = 8 * s; // treble bottom → bass top
const topPad = 5 * s;
const bottomPad = 4.5 * s;
const systemGap = 3 * s;
const stemLen = 3.5; // staff spaces

const stepOrder: Record<string, number> = { C: 0, D: 1, E: 2, F: 3, G: 4, A: 5, B: 6 };

// A tiny SVG string builder — keeps the paint routines declarative and testable.
class Svg {
  private readonly parts: string[] = [];

  line(x1: number, y1: number, x2: number, y2: number, cls: string, extra = ""): void {
    this.parts.push(`<line x1="${r(x1)}" y1="${r(y1)}" x2="${r(x2)}" y2="${r(y2)}" class="${cls}"${extra}/>`);
  }

  circle(cx: number, cy: number, radius: number): void {
    this.parts.push(`<circle cx="${r(cx)}" cy="${r(cy)}" r="${r(radius)}" fill="currentColor"/>`);
  }

  /** The conventional open mark above an open hi-hat head: a small UNFILLED circle
   *  (stroked in ink), kept off the font so it cannot render as tofu. */
  openMark(cx: number, cy: number, radius: number): void {
    this.parts.push(`<circle cx="${r(cx)}" cy="${r(cy)}" r="${r(radius)}" class="open-mark"/>`);
  }

  label(text: string, x: number, y: number, size: number): void {
    this.parts.push(
      `<text x="${r(x)}" y="${r(y)}" class="ink" font-size="${r(size)}" font-weight="600">${escapeText(text)}</text>`,
    );
  }

  glyph(g: string, x: number, y: number, size: number, cls = "ink", center = false, extra = ""): void {
    if (!g) return;
    const anchor = center ? ` text-anchor="middle"` : "";
    this.parts.push(
      `<text x="${r(x)}" y="${r(y)}" class="glyph ${cls}" font-size="${r(size * 4)}"${anchor}${extra}>${escapeText(g)}</text>`,
    );
  }

  toString(width: number, height: number): string {
    // color: var(--text) drives currentColor; staff/ledger lines are muted. Scales
    // down responsively (max-width:100%) but never blurs (crisp SVG strokes).
    return (
      `<svg xmlns="http://www.w3.org/2000/svg" width="${r(width)}" height="${r(height)}" ` +
      `viewBox="0 0 ${r(width)} ${r(height)}" role="img" class="notation-svg" ` +
      `style="max-width:100%;height:auto;color:var(--text)">` +
      `<style>` +
      `.notation-svg .glyph{font-family:'Bravura';fill:currentColor}` +
      `.notation-svg .ink{fill:currentColor}` +
      `.notation-svg .stem{stroke:currentColor;stroke-width:${r(S.stemThickness * s)};stroke-linecap:round}` +
      `.notation-svg .beam{stroke:currentColor;stroke-width:${r(S.beamThickness * s)}}` +
      `.notation-svg .staff{stroke:var(--muted);stroke-width:${r(S.staffLineThickness * s)};opacity:.7}` +
      `.notation-svg .ledger{stroke:var(--muted);stroke-width:${r(S.legerLineThickness * s)};opacity:.8}` +
      `.notation-svg .bar{stroke:currentColor;stroke-width:${r(S.thinBarlineThickness * s)};opacity:.7}` +
      `.notation-svg .bracket{stroke:currentColor;stroke-width:${r(s * 0.35)};stroke-linecap:round;opacity:.8}` +
      // The open hi-hat's "o": stroked, never filled (an unfilled circle above the head).
      `.notation-svg .open-mark{fill:none;stroke:currentColor;stroke-width:${r(s * 0.14)}}` +
      // Playback overlay: the moving cursor and the currently-sounding note heads.
      `.notation-svg .cursor{stroke:var(--teal, #44e2cd);stroke-width:${r(s * 0.18)};stroke-linecap:round}` +
      `.notation-svg .glyph.playing{fill:var(--teal, #44e2cd)}` +
      `</style>` +
      this.parts.join("") +
      `</svg>`
    );
  }
}

/** A measure's box in SVG coordinates — lets the playhead map an elapsed time to an
 *  on-screen cursor position and scroll it into view. */
export interface MeasureRect {
  index: number;
  x: number;
  width: number;
  top: number;
  bottom: number;
}

/** The painted SVG plus a layout map for the playback overlay. */
export interface NotationRender {
  kind: "notation";
  svg: string;
  layout: { width: number; height: number; measures: MeasureRect[] };
  /** True when the document routed through the percussion engraving rules. Kept on
   * the result so the views' Play guard (installed by `add-drums-access`, and
   * `add-drum-audio-channel`'s to lift) can still refuse a drum audition for a
   * score whose recorded instrument is `unknown`. */
  percussion: boolean;
}

/** What a render produces. The `percussion_unsupported` arm is retired (change:
 * add-drum-notation-render) — a percussion score now draws like any other. */
export type RenderResult = NotationRender;

/** True when the parsed document is percussion notation: a percussion clef on any
 *  staff (document attributes or a mid-piece change), or any note carried on the
 *  unpitched channel. Either signal alone ROUTES the score to the percussion paint
 *  path (change: add-drum-notation-render) — a drum part exported without its
 *  `<clef sign="percussion">` still carries `<unpitched>` notes and still needs
 *  the percussion rules, not treble-staff assumptions. */
export function isPercussionScore(doc: ScoreDocument): boolean {
  const percClef = (c: Clef): boolean => c.sign === "percussion";
  return (
    doc.attributes.clefs.some(percClef) ||
    doc.measures.some((m) => m.clefs.some(percClef) || m.notes.some((n) => n.unpitched != null))
  );
}

/** Invariant render state threaded through the paint routines (keeps their parameter
 *  lists small). `measures` is accumulated as systems are painted. */
interface Ctx {
  svg: Svg;
  doc: ScoreDocument;
  width: number;
  divPerMeasure: number;
  twoStaff: boolean;
  /** Percussion route (change: add-drum-notation-render): single staff, percussion
   *  clef, no armature, written-position heads, voice-aware stems and rests. */
  percussion: boolean;
  clefAt: Map<number, Clef>[];
  measures: MeasureRect[];
}

/** Render laid-out geometry to an SVG string sized to `width` (the same width handed
 *  to the wasm layout, so the systems' line breaks match), plus a layout map. Each
 *  pitched note head carries `data-note="<measureIndex>:<noteIndex>"` so the playhead
 *  can highlight sounding notes. */
export function renderNotation(rendered: RenderedScore, width: number): RenderResult {
  const svg = new Svg();
  const { document: doc, systems } = rendered;
  // Percussion routing (change: add-drum-notation-render): a percussion document —
  // detected by clef OR by unpitched notes — takes the percussion engraving rules
  // on a single five-line staff, whatever `staves` the file declares.
  const percussion = isPercussionScore(doc);
  const twoStaff = !percussion && doc.staves >= 2;
  const systemHeight = topPad + staffHeight + (twoStaff ? interStaff + staffHeight : 0) + bottomPad;
  const height = systems.length * (systemHeight + systemGap) + systemGap;
  const measures: MeasureRect[] = [];
  if (systems.length === 0) {
    return {
      kind: "notation",
      svg: svg.toString(width, systemHeight),
      layout: { width, height: systemHeight, measures },
      percussion,
    };
  }

  const ctx: Ctx = {
    svg,
    doc,
    width,
    divPerMeasure: divisionsPerMeasure(doc),
    twoStaff,
    percussion,
    clefAt: computeClefAt(doc),
    measures,
  };

  let y = systemGap;
  for (let i = 0; i < systems.length; i++) {
    paintSystem(ctx, systems[i], y, i === 0);
    y += systemHeight + systemGap;
  }
  return { kind: "notation", svg: svg.toString(width, height), layout: { width, height, measures }, percussion };
}

/** Vertical position of a rest on a two-voice percussion staff: voice 1 sits
 *  above the middle line, voice 2 below, so a rest never lands on the midline
 *  where the other voice's material runs. A single-voice measure keeps the
 *  midline. */
function percussionRestY(twoVoice: boolean, voice: number, midY: number, s: number): number {
  if (!twoVoice) return midY;
  return voice <= 1 ? midY - s : midY + s;
}

function divisionsPerMeasure(doc: ScoreDocument): number {
  const a = doc.attributes;
  const beatType = a.time.beat_type === 0 ? 4 : a.time.beat_type;
  const perMeasure = Math.floor((a.divisions * a.time.beats * 4) / beatType);
  return perMeasure > 0 ? perMeasure : a.divisions * 4;
}

// Clef in effect per staff for each measure, honouring mid-piece clef changes.
function computeClefAt(doc: ScoreDocument): Map<number, Clef>[] {
  const running = new Map<number, Clef>();
  for (const c of doc.attributes.clefs) running.set(c.staff, c);
  const out: Map<number, Clef>[] = [];
  for (const m of doc.measures) {
    for (const c of m.clefs) running.set(c.staff, c);
    out.push(new Map(running));
  }
  return out;
}

function clefFor(clefs: Map<number, Clef>, staff: number): Clef {
  return clefs.get(staff) ?? (staff >= 2 ? { staff: 2, sign: "F", line: 4 } : { staff: 1, sign: "G", line: 2 });
}

function paintSystem(ctx: Ctx, system: System, yTop: number, isFirst: boolean): void {
  const { svg, doc, width, twoStaff } = ctx;
  const trebleBottom = yTop + topPad + staffHeight;
  const bassBottom = trebleBottom + interStaff + staffHeight;
  const systemBottom = twoStaff ? bassBottom : trebleBottom;
  const systemTop = trebleBottom - staffHeight;
  const headerClefs = ctx.clefAt[system.measures[0]];

  drawStaffLines(svg, trebleBottom, width);
  if (twoStaff) drawStaffLines(svg, bassBottom, width);
  // Left system bracket connecting the grand staff.
  svg.line(1.2, systemTop, 1.2, systemBottom, "bracket");

  // Header: clef, key signature, and (first system only) time signature. A
  // percussion system always opens with the percussion clef — even for a drum
  // export that never declared one (the route, not the file, decides).
  const headerClef1: Clef = ctx.percussion ? { staff: 1, sign: "percussion", line: 3 } : clefFor(headerClefs, 1);
  drawClef(svg, headerClef1, s * 0.4, trebleBottom, s);
  if (twoStaff) drawClef(svg, clefFor(headerClefs, 2), s * 0.4, bassBottom, s);
  let hx = s * 3.0;

  // Armure of THIS system: the key in force at its first measure, not one
  // document-wide value — so a piece that modulates shows the right signature on
  // every line, and a key change at the system boundary shows the change
  // (cancelling naturals + new signature). A percussion staff NEVER draws an
  // armature or its cancelling naturals, whatever `fifths` the file declares — an
  // unpitched part has no tonality (spec: music-percussion-engraving).
  if (!ctx.percussion) {
    const firstIdx = system.measures[0];
    const systemKey = doc.measures[firstIdx].key_fifths;
    const prevKey = firstIdx > 0 ? doc.measures[firstIdx - 1].key_fifths : null;
    const drawHeaderKey = (sb: number, bass: boolean): number =>
      prevKey !== null && prevKey !== systemKey
        ? drawKeyChange(svg, hx, sb, prevKey, systemKey, bass)
        : drawKeySignature(svg, hx, sb, systemKey, bass);
    const keyWidth = drawHeaderKey(trebleBottom, false);
    if (twoStaff) drawHeaderKey(bassBottom, true);
    hx += keyWidth;
  }

  if (isFirst) {
    const t = doc.attributes.time;
    const timeWidth = drawTimeSignature(svg, hx, trebleBottom, t.beats, t.beat_type);
    if (twoStaff) drawTimeSignature(svg, hx, bassBottom, t.beats, t.beat_type);
    hx += timeWidth;
  }
  const headerX = hx + s * 0.6;

  // Justify measures to fill the line after the header.
  let totalMin = 0;
  for (const idx of system.measures) totalMin += doc.measures[idx].min_width;
  const usable = width - headerX;
  const scale = totalMin > 0 ? usable / totalMin : 1.0;

  let x = headerX;
  for (let k = 0; k < system.measures.length; k++) {
    const idx = system.measures[k];
    const mWidth = doc.measures[idx].min_width * scale;
    svg.line(x + mWidth, systemTop, x + mWidth, systemBottom, "bar");
    drawRepeatMarks(
      ctx.svg,
      doc.measures[idx].repeats,
      x,
      mWidth,
      systemTop,
      systemBottom,
      trebleBottom,
      bassBottom,
      doc.staves >= 2,
    );
    ctx.measures.push({ index: idx, x, width: mWidth, top: systemTop, bottom: systemBottom });
    const previousKeyFifths = k > 0 ? doc.measures[system.measures[k - 1]].key_fifths : null;
    paintMeasure(ctx, idx, x, mWidth, trebleBottom, bassBottom, previousKeyFifths);
    x += mWidth;
  }
}

// Repeat notation on one measure (change: add-repeat-unrolling): repeat
// barlines (thick line + dots on the repeated side), the volta bracket +
// number, the measure-repeat % sign and segno/coda signs — so a moderator sees
// the same notation the app's Partition engraves.
function drawRepeatMarks(
  svg: Svg,
  marks: RepeatMarks | undefined,
  x: number,
  mWidth: number,
  systemTop: number,
  systemBottom: number,
  trebleBottom: number,
  bassBottom: number,
  twoStaff: boolean,
): void {
  if (!marks) return;
  const dots = (dx: number): void => {
    for (const base of twoStaff ? [trebleBottom, bassBottom] : [trebleBottom]) {
      for (const dy of [1.5, 2.5]) svg.circle(dx, base - dy * s, s * 0.22);
    }
  };
  const thick = ` style="stroke-width:${r(S.thickBarlineThickness * s)}"`;
  if (marks.forward) {
    svg.line(x + s * 0.18, systemTop, x + s * 0.18, systemBottom, "bar", thick);
    dots(x + s * 0.95);
  }
  if (marks.backward_times > 0) {
    svg.line(x + mWidth - s * 0.18, systemTop, x + mWidth - s * 0.18, systemBottom, "bar", thick);
    dots(x + mWidth - s * 0.95);
  }
  if (marks.ending_start.length > 0 || marks.ending_stop || marks.ending_discontinue) {
    const y = systemTop - s * 1.6;
    svg.line(x, y, x + mWidth, y, "bar");
    if (marks.ending_start.length > 0) {
      svg.line(x, y, x, y + s * 1.1, "bar");
      svg.label(`${marks.ending_start.join(".")}.`, x + s * 0.4, y + s * 1.1, s * 1.1);
    }
    if (marks.ending_stop) svg.line(x + mWidth, y, x + mWidth, y + s * 1.1, "bar");
  }
  if (marks.measure_repeat_of != null) {
    svg.glyph(
      marks.measure_repeat_slashes >= 2 ? S.repeat2Bars : S.repeat1Bar,
      x + mWidth / 2,
      trebleBottom - 2 * s,
      s,
      "ink",
      true,
    );
  }
  if (marks.segno || marks.coda) {
    svg.glyph(marks.segno ? S.segno : S.coda, x + s * 1.2, systemTop - s * 1.8, s * 1.1, "ink", true);
  }
}

function drawStaffLines(svg: Svg, bottom: number, width: number): void {
  for (let i = 0; i < 5; i++) {
    const y = bottom - i * s;
    svg.line(0, y, width, y, "staff");
  }
}

function drawClef(svg: Svg, clef: Clef, x: number, staffBottom: number, size: number): void {
  // The percussion clef is centred on the staff (middle line), whatever `line`
  // the exporter declared — its SMuFL registration is the staff centre.
  const line = clef.sign === "percussion" ? 3 : clef.line;
  const baselineY = staffBottom - (line - 1) * size;
  svg.glyph(S.clefGlyph(clef.sign), x, baselineY, size);
}

function drawKeySignature(svg: Svg, xStart: number, staffBottom: number, fifths: number, isBass: boolean): number {
  if (fifths === 0) return 0;
  const sharps = fifths > 0;
  const steps = sharps ? S.sharpSteps : S.flatSteps;
  const glyph = sharps ? S.accidentalSharp : S.accidentalFlat;
  const count = Math.min(Math.abs(fifths), 7);
  const adv = 0.95;
  for (let i = 0; i < count; i++) {
    const step = steps[i] - (isBass ? 2 : 0);
    svg.glyph(glyph, xStart + i * adv * s, staffBottom - step * (s / 2), s);
  }
  return count * adv * s + s * 0.4;
}

// A key change from oldFifths to newFifths: a natural cancels every accidental
// that leaves the signature (on its old staff position), then the new signature
// follows. Returns the width it consumed.
function drawKeyChange(
  svg: Svg,
  xStart: number,
  staffBottom: number,
  oldFifths: number,
  newFifths: number,
  isBass: boolean,
): number {
  const oldSharp = oldFifths > 0;
  const oldSteps = oldSharp ? S.sharpSteps : S.flatSteps;
  const oldCount = Math.min(Math.abs(oldFifths), 7);
  const newSharp = newFifths > 0;
  const newSteps = newSharp ? S.sharpSteps : S.flatSteps;
  const newCount = Math.min(Math.abs(newFifths), 7);
  const kept = new Set<number>();
  if (oldSharp === newSharp) for (let i = 0; i < newCount; i++) kept.add(newSteps[i]);

  const adv = 0.95;
  let x = xStart;
  for (let i = 0; i < oldCount; i++) {
    const step = oldSteps[i];
    if (kept.has(step)) continue;
    svg.glyph(S.accidentalNatural, x, staffBottom - (step - (isBass ? 2 : 0)) * (s / 2), s);
    x += adv * s;
  }
  const newWidth = drawKeySignature(svg, x, staffBottom, newFifths, isBass);
  return x - xStart + newWidth;
}

function drawTimeSignature(svg: Svg, xStart: number, staffBottom: number, beats: number, beatType: number): number {
  const midY = staffBottom - 2 * s;
  const cx = xStart + s * 0.9;
  svg.glyph(S.timeSigNumber(beats), cx, midY - s, s, "ink", true);
  svg.glyph(S.timeSigNumber(beatType), cx, midY + s, s, "ink", true);
  return s * 2.2;
}

interface StemNote {
  x: number;
  y: number;
  up: boolean;
  note: NoteEvent;
}

function paintMeasure(
  ctx: Ctx,
  measureIdx: number,
  measureX: number,
  measureWidth: number,
  trebleBottom: number,
  bassBottom: number,
  previousKeyFifths: number | null,
): void {
  const measure = ctx.doc.measures[measureIdx];
  const clefs = ctx.clefAt[measureIdx];
  const beamGroups = new Map<string, StemNote[]>();

  // A mid-system key change (modulation): cancelling naturals + the new
  // signature at the measure start, reserving space before the first note.
  // Never on a percussion staff (no armature, no cancelling naturals).
  let keyLead = 0;
  if (!ctx.percussion && previousKeyFifths !== null && measure.key_fifths !== previousKeyFifths) {
    keyLead = drawKeyChange(ctx.svg, measureX + s * 0.3, trebleBottom, previousKeyFifths, measure.key_fifths, false);
    if (ctx.twoStaff)
      drawKeyChange(ctx.svg, measureX + s * 0.3, bassBottom, previousKeyFifths, measure.key_fifths, true);
  }

  // Voice census for the percussion two-voice rules: rests displace by voice only
  // in a measure where both voices are present (single-voice rests keep the
  // ordinary midline).
  const voices = new Set<number>();
  for (const n of measure.notes) voices.add(n.voice);
  const multiVoice = voices.size > 1;

  // Shared-onset collision (percussion): when both voices strike the same instant
  // on the same written position, the later head is offset horizontally so neither
  // hides. Chords within one voice already offset via `is_chord`/grace handling.
  const collided = new Set<number>();
  if (ctx.percussion) {
    const seen = new Map<string, number>(); // "position:y" → voice of the first head
    for (let ni = 0; ni < measure.notes.length; ni++) {
      const n = measure.notes[ni];
      if (n.is_rest || n.unpitched == null) continue;
      const key = `${n.position_divisions}:${yForUnpitched(n.unpitched, trebleBottom)}`;
      const first = seen.get(key);
      if (first === undefined) seen.set(key, n.voice);
      else if (first !== n.voice) collided.add(ni);
    }
  }

  const xForPosition = (position: number): number => {
    const frac = ctx.divPerMeasure > 0 ? clamp(position / ctx.divPerMeasure, 0, 1) : 0;
    return measureX + s + keyLead + frac * (measureWidth - keyLead - 2.4 * s);
  };

  for (let ni = 0; ni < measure.notes.length; ni++) {
    const note = measure.notes[ni];
    // A grace note shares its principal's position (duration 0): offset it left
    // of the principal's column so the two heads don't engrave on top of each
    // other. A cross-voice same-position head offsets right instead.
    paintNote(ctx, {
      note,
      ni,
      measureIdx,
      x:
        xForPosition(note.position_divisions) -
        (note.is_grace ? S.noteheadWidth * s * 1.4 : 0) +
        (collided.has(ni) ? S.noteheadWidth * s : 0),
      trebleBottom,
      bassBottom,
      clefs,
      beamGroups,
      multiVoice,
    });
  }
  // Any beam group left open at the barline (defensive).
  for (const group of beamGroups.values()) drawBeamGroup(ctx.svg, group);
}

interface PaintNoteOpts {
  note: NoteEvent;
  ni: number;
  measureIdx: number;
  x: number;
  trebleBottom: number;
  bassBottom: number;
  clefs: Map<number, Clef>;
  beamGroups: Map<string, StemNote[]>;
  /** Both voices present in this measure (drives percussion rest displacement). */
  multiVoice: boolean;
}

function paintNote(ctx: Ctx, o: PaintNoteOpts): void {
  const { svg } = ctx;
  const { note, x } = o;
  const isBass = note.staff >= 2 && ctx.twoStaff;
  const staffBottom = isBass ? o.bassBottom : o.trebleBottom;
  const midY = staffBottom - 2 * s;

  if (note.is_rest) {
    // On a two-voice percussion measure rests displace by voice — voice 1 above
    // the middle line, voice 2 below — so a rest never sits on the midline where
    // the other voice's material runs; single-voice measures keep the midline.
    const restY = percussionRestY(ctx.percussion && o.multiVoice, note.voice, midY, s);
    svg.glyph(S.restGlyph(note), x, restY, s, "ink", true);
    return;
  }

  const u = note.unpitched;
  if (u != null) {
    // Percussion head: placed by its WRITTEN position (display step/octave) through
    // the same diatonic machinery as pitched notes, with the treble reference — the
    // MusicXML display-placement convention. Never placed from the GM number, and a
    // note whose GM number is unresolved is still engraved (omission-when-unresolved
    // is a playback rule, not an engraving rule).
    const y = yForUnpitched(u, staffBottom);
    drawLedgerLines(svg, x, y, staffBottom);

    const head = S.unpitchedHeadGlyph(note, u.head_class ?? "Oval", ctx.doc.attributes.divisions);
    const headLeft = x - (S.noteheadWidth * s) / 2;
    svg.glyph(head, headLeft, y, s, "ink", false, ` data-note="${o.measureIdx}:${o.ni}"`);
    drawDots(svg, x, y, note.dots);

    // Stems: the file's explicit <stem> wins; a bare note follows the drum
    // convention — voice 1 (hands) up, voice 2 (feet) down.
    const up = note.stem != null ? note.stem === "Up" : note.voice <= 1;
    if (u.head_class === "XOpen") {
      // The conventional open mark of the open hi-hat (GM 46): a small circle above
      // the head — clear of the stem when it points up.
      const markY = up && !note.is_chord ? y - (stemLen + 0.9) * s : y - 1.1 * s;
      svg.openMark(x, markY, s * 0.28);
    }
    queueStem(svg, { x, y, up: false, note }, head, o.beamGroups, up);
    return;
  }

  const pitch = note.pitch;
  if (pitch == null) return;

  const y = yForPitch(pitch.step, pitch.octave, staffBottom, clefFor(o.clefs, note.staff));
  drawLedgerLines(svg, x, y, staffBottom);

  const head = S.headGlyph(note, ctx.doc.attributes.divisions);
  const headLeft = x - (S.noteheadWidth * s) / 2;
  // data-note correlates this head with its scheduled note so the playhead can
  // highlight it while it sounds.
  svg.glyph(head, headLeft, y, s, "ink", false, ` data-note="${o.measureIdx}:${o.ni}"`);

  if (note.accidental) {
    const glyph = S.accidentalGlyph(note.accidental);
    if (glyph) svg.glyph(glyph, headLeft - s * 1.5, y, s);
  }
  drawDots(svg, x, y, note.dots);

  queueStem(svg, { x, y, up: false, note }, head, o.beamGroups, y >= midY);
}

// Stem + beam grouping (chord members share the principal note's stem; whole notes —
// oval or x-form — have no stem). Draws a lone stem/flag immediately, or accumulates
// a beam group and flushes it on the group's End note. `fallbackUp` is the direction
// when the file carries no explicit <stem>: below-midline for pitched notes, the
// voice convention for percussion. Beam groups stay keyed on staff_voice, so beams
// never merge notes of different voices.
function queueStem(
  svg: Svg,
  n: StemNote,
  head: string,
  beamGroups: Map<string, StemNote[]>,
  fallbackUp: boolean,
): void {
  if (head === S.noteheadWhole || head === S.noteheadXWhole || n.note.is_chord) return;
  const stem: StemNote = { ...n, up: n.note.stem != null ? n.note.stem === "Up" : fallbackUp };

  if (n.note.beams.length === 0) {
    drawStemAndFlag(svg, stem);
    return;
  }
  const key = `${n.note.staff}_${n.note.voice}`;
  const group = beamGroups.get(key) ?? [];
  group.push(stem);
  beamGroups.set(key, group);
  if (n.note.beams.includes("End")) {
    drawBeamGroup(svg, group);
    beamGroups.delete(key);
  }
}

function yForPitch(step: string, octave: number, bottomLineY: number, clef: Clef): number {
  const diatonic = octave * 7 + (stepOrder[step] ?? 0);
  return bottomLineY - (diatonic - clefBottomDiatonic(clef)) * (s / 2);
}

// An unpitched note's written display position maps to lines and spaces exactly as
// a treble (G, line 2) staff maps pitched spellings — the MusicXML convention for
// unpitched display placement (snare C5 third space, kick F4 bottom space) —
// regardless of the percussion clef's declared `line`.
const trebleReference: Clef = { staff: 1, sign: "G", line: 2 };

function yForUnpitched(u: Unpitched, bottomLineY: number): number {
  return yForPitch(u.display_step, u.display_octave, bottomLineY, trebleReference);
}

// Diatonic value of a clef's bottom staff line (G→G4, F→F3, C→C4; each line = 2 steps).
function clefRefDiatonic(sign: string): number {
  if (sign === "F") return 3 * 7 + 3;
  if (sign === "C") return 4 * 7 + 0;
  return 4 * 7 + 4;
}

function clefBottomDiatonic(clef: Clef): number {
  return clefRefDiatonic(clef.sign) - (clef.line - 1) * 2;
}

function stemAnchor(n: StemNote): { x: number; y: number } {
  const baseX = n.x - (S.noteheadWidth * s) / 2;
  return n.up
    ? { x: baseX + S.stemUpAnchorX * s, y: n.y - S.stemUpAnchorY * s }
    : { x: baseX + S.stemDownAnchorX * s, y: n.y - S.stemDownAnchorY * s };
}

function drawStemAndFlag(svg: Svg, n: StemNote): void {
  const a = stemAnchor(n);
  const tipY = n.up ? a.y - stemLen * s : a.y + stemLen * s;
  svg.line(a.x, a.y, a.x, tipY, "stem");
  const flag = S.flagGlyph(n.note, n.up);
  if (flag) svg.glyph(flag, a.x, tipY, s);
}

function drawBeamGroup(svg: Svg, group: StemNote[]): void {
  if (group.length === 0) return;
  if (group.length === 1) {
    drawStemAndFlag(svg, group[0]);
    return;
  }
  const up = group[0].up;
  const anchors = group.map(stemAnchor);
  const ys = anchors.map((a) => a.y);
  const beamY = up ? Math.min(...ys) - stemLen * s : Math.max(...ys) + stemLen * s;
  for (const a of anchors) svg.line(a.x, a.y, a.x, beamY, "stem");
  const last = anchors.at(-1)!;
  svg.line(anchors[0].x, beamY, last.x, beamY, "beam");

  // Secondary beam for consecutive 16th-or-shorter notes.
  const off = (up ? 1 : -1) * (S.beamThickness + 0.2) * s;
  for (let i = 0; i < group.length - 1; i++) {
    if (S.flagCount(group[i].note) >= 2 && S.flagCount(group[i + 1].note) >= 2) {
      svg.line(anchors[i].x, beamY + off, anchors[i + 1].x, beamY + off, "beam");
    }
  }
}

function drawDots(svg: Svg, x: number, y: number, dots: number): void {
  for (let i = 0; i < dots; i++) {
    svg.glyph(S.augmentationDot, x + (S.noteheadWidth * s) / 2 + s * (0.3 + i * 0.5), y, s);
  }
}

function drawLedgerLines(svg: Svg, x: number, y: number, bottomLineY: number): void {
  const topLineY = bottomLineY - staffHeight;
  const ext = S.legerLineExtension * s;
  const half = (S.noteheadWidth * s) / 2 + ext;
  for (let ly = bottomLineY + s; ly <= y + 0.5; ly += s) svg.line(x - half, ly, x + half, ly, "ledger");
  for (let ly = topLineY - s; ly >= y - 0.5; ly -= s) svg.line(x - half, ly, x + half, ly, "ledger");
}

function clamp(v: number, lo: number, hi: number): number {
  return Math.min(Math.max(v, lo), hi);
}

// Round to 2 decimals to keep the SVG compact and stable across platforms.
function r(v: number): string {
  return (Math.round(v * 100) / 100).toString();
}

function escapeText(t: string): string {
  return t.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}
