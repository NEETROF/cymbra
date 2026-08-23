// A read-only SVG notation painter, mirroring the Flutter app's PartitionPainter so
// the moderation console draws the same engraving. It consumes the geometry the wasm
// module returns (the app's own `layout_systems`) and emits an SVG string: staff
// lines, the grand-staff bracket, clefs, key/time signatures, barlines, note heads,
// stems, flags, beams, accidentals, augmentation dots, rests and ledger lines.
//
// Intentionally NOT drawn in v1 (best-effort later, not required by the preview
// contract): expression/dynamics directions, lyrics, ties, slurs, tuplet
// numbers/brackets, and the playback cursor (playback is out of scope here).

import type { Clef, NoteEvent, RenderedScore, RepeatMarks, ScoreDocument, System } from "./geometry";
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
export interface RenderResult {
  svg: string;
  layout: { width: number; height: number; measures: MeasureRect[] };
}

/** Invariant render state threaded through the paint routines (keeps their parameter
 *  lists small). `measures` is accumulated as systems are painted. */
interface Ctx {
  svg: Svg;
  doc: ScoreDocument;
  width: number;
  divPerMeasure: number;
  twoStaff: boolean;
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
  const twoStaff = doc.staves >= 2;
  const systemHeight = topPad + staffHeight + (twoStaff ? interStaff + staffHeight : 0) + bottomPad;
  const height = systems.length * (systemHeight + systemGap) + systemGap;
  const measures: MeasureRect[] = [];
  if (systems.length === 0) {
    return { svg: svg.toString(width, systemHeight), layout: { width, height: systemHeight, measures } };
  }

  const ctx: Ctx = {
    svg,
    doc,
    width,
    divPerMeasure: divisionsPerMeasure(doc),
    twoStaff,
    clefAt: computeClefAt(doc),
    measures,
  };

  let y = systemGap;
  for (let i = 0; i < systems.length; i++) {
    paintSystem(ctx, systems[i], y, i === 0);
    y += systemHeight + systemGap;
  }
  return { svg: svg.toString(width, height), layout: { width, height, measures } };
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

  // Header: clef, key signature, and (first system only) time signature.
  drawClef(svg, clefFor(headerClefs, 1), s * 0.4, trebleBottom, s);
  if (twoStaff) drawClef(svg, clefFor(headerClefs, 2), s * 0.4, bassBottom, s);
  let hx = s * 3.0;

  // Armure of THIS system: the key in force at its first measure, not one
  // document-wide value — so a piece that modulates shows the right signature on
  // every line, and a key change at the system boundary shows the change
  // (cancelling naturals + new signature).
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
  const baselineY = staffBottom - (clef.line - 1) * size;
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
  let keyLead = 0;
  if (previousKeyFifths !== null && measure.key_fifths !== previousKeyFifths) {
    keyLead = drawKeyChange(ctx.svg, measureX + s * 0.3, trebleBottom, previousKeyFifths, measure.key_fifths, false);
    if (ctx.twoStaff)
      drawKeyChange(ctx.svg, measureX + s * 0.3, bassBottom, previousKeyFifths, measure.key_fifths, true);
  }

  const xForPosition = (position: number): number => {
    const frac = ctx.divPerMeasure > 0 ? clamp(position / ctx.divPerMeasure, 0, 1) : 0;
    return measureX + s + keyLead + frac * (measureWidth - keyLead - 2.4 * s);
  };

  for (let ni = 0; ni < measure.notes.length; ni++) {
    const note = measure.notes[ni];
    // A grace note shares its principal's position (duration 0): offset it left
    // of the principal's column so the two heads don't engrave on top of each
    // other.
    paintNote(ctx, {
      note,
      ni,
      measureIdx,
      x: xForPosition(note.position_divisions) - (note.is_grace ? S.noteheadWidth * s * 1.4 : 0),
      trebleBottom,
      bassBottom,
      clefs,
      beamGroups,
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
}

function paintNote(ctx: Ctx, o: PaintNoteOpts): void {
  const { svg } = ctx;
  const { note, x } = o;
  const isBass = note.staff >= 2 && ctx.twoStaff;
  const staffBottom = isBass ? o.bassBottom : o.trebleBottom;

  if (note.is_rest) {
    svg.glyph(S.restGlyph(note), x, staffBottom - 2 * s, s, "ink", true);
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

  queueStem(svg, { x, y, up: false, note }, staffBottom, head, o.beamGroups);
}

// Stem + beam grouping (chord members share the principal note's stem; whole notes
// have no stem). Draws a lone stem/flag immediately, or accumulates a beam group and
// flushes it on the group's End note.
function queueStem(
  svg: Svg,
  n: StemNote,
  staffBottom: number,
  head: string,
  beamGroups: Map<string, StemNote[]>,
): void {
  if (head === S.noteheadWhole || n.note.is_chord) return;
  const midY = staffBottom - 2 * s;
  const stem: StemNote = { ...n, up: n.note.stem != null ? n.note.stem === "Up" : n.y >= midY };

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
