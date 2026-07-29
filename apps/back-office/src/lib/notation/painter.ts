// A read-only SVG notation painter, mirroring the Flutter app's PartitionPainter so
// the moderation console draws the same engraving. It consumes the geometry the wasm
// module returns (the app's own `layout_systems`) and emits an SVG string: staff
// lines, the grand-staff bracket, clefs, key/time signatures, barlines, note heads,
// stems, flags, beams, accidentals, augmentation dots, rests and ledger lines.
//
// Intentionally NOT drawn in v1 (best-effort later, not required by the preview
// contract): expression/dynamics directions, lyrics, ties, slurs, tuplet
// numbers/brackets, and the playback cursor (playback is out of scope here).

import type { Clef, NoteEvent, RenderedScore, ScoreDocument, System } from "./geometry";
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
  private parts: string[] = [];

  line(x1: number, y1: number, x2: number, y2: number, cls: string, extra = ""): void {
    this.parts.push(`<line x1="${r(x1)}" y1="${r(y1)}" x2="${r(x2)}" y2="${r(y2)}" class="${cls}"${extra}/>`);
  }

  glyph(g: string, x: number, y: number, size: number, cls = "ink", center = false): void {
    if (!g) return;
    const anchor = center ? ` text-anchor="middle"` : "";
    this.parts.push(
      `<text x="${r(x)}" y="${r(y)}" class="glyph ${cls}" font-size="${r(size * 4)}"${anchor}>${escapeText(g)}</text>`,
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
      `</style>` +
      this.parts.join("") +
      `</svg>`
    );
  }
}

/** Render laid-out geometry to an SVG string sized to `width` (the same width handed
 *  to the wasm layout, so the systems' line breaks match). */
export function renderNotation(rendered: RenderedScore, width: number): string {
  const svg = new Svg();
  const { document: doc, systems } = rendered;
  const twoStaff = doc.staves >= 2;
  const systemHeight = topPad + staffHeight + (twoStaff ? interStaff + staffHeight : 0) + bottomPad;
  const height = systems.length * (systemHeight + systemGap) + systemGap;
  if (systems.length === 0) return svg.toString(width, systemHeight);

  const divPerMeasure = divisionsPerMeasure(doc);
  const clefAt = computeClefAt(doc);

  let y = systemGap;
  for (let i = 0; i < systems.length; i++) {
    paintSystem(svg, doc, systems[i], width, y, divPerMeasure, i === 0, twoStaff, clefAt);
    y += systemHeight + systemGap;
  }
  return svg.toString(width, height);
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

function paintSystem(
  svg: Svg,
  doc: ScoreDocument,
  system: System,
  width: number,
  yTop: number,
  divPerMeasure: number,
  isFirst: boolean,
  twoStaff: boolean,
  clefAt: Map<number, Clef>[],
): void {
  const trebleBottom = yTop + topPad + staffHeight;
  const bassBottom = trebleBottom + interStaff + staffHeight;
  const systemBottom = twoStaff ? bassBottom : trebleBottom;
  const systemTop = trebleBottom - staffHeight;
  const headerClefs = clefAt[system.measures[0]];

  drawStaffLines(svg, trebleBottom, width);
  if (twoStaff) drawStaffLines(svg, bassBottom, width);
  // Left system bracket connecting the grand staff.
  svg.line(1.2, systemTop, 1.2, systemBottom, "bracket");

  // Header: clef, key signature, and (first system only) time signature.
  drawClef(svg, clefFor(headerClefs, 1), s * 0.4, trebleBottom, s);
  if (twoStaff) drawClef(svg, clefFor(headerClefs, 2), s * 0.4, bassBottom, s);
  let hx = s * 3.0;

  const fifths = doc.attributes.key_fifths;
  const keyWidth = drawKeySignature(svg, hx, trebleBottom, fifths, false);
  if (twoStaff) drawKeySignature(svg, hx, bassBottom, fifths, true);
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
  for (const idx of system.measures) {
    const measure = doc.measures[idx];
    const mWidth = measure.min_width * scale;
    svg.line(x + mWidth, systemTop, x + mWidth, systemBottom, "bar");
    paintMeasure(svg, doc, idx, x, mWidth, divPerMeasure, trebleBottom, bassBottom, clefAt[idx], twoStaff);
    x += mWidth;
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
  svg: Svg,
  doc: ScoreDocument,
  measureIdx: number,
  measureX: number,
  measureWidth: number,
  divPerMeasure: number,
  trebleBottom: number,
  bassBottom: number,
  clefs: Map<number, Clef>,
  twoStaff: boolean,
): void {
  const measure = doc.measures[measureIdx];

  const xForPosition = (position: number): number => {
    const frac = divPerMeasure > 0 ? clamp(position / divPerMeasure, 0, 1) : 0;
    const left = measureX + s;
    return left + frac * (measureWidth - 2.4 * s);
  };

  const beamGroups = new Map<string, StemNote[]>();

  for (const note of measure.notes) {
    const isBass = note.staff >= 2 && twoStaff;
    const staffBottom = isBass ? bassBottom : trebleBottom;
    const x = xForPosition(note.position_divisions);

    if (note.is_rest) {
      svg.glyph(S.restGlyph(note), x, staffBottom - 2 * s, s, "ink", true);
      continue;
    }
    const pitch = note.pitch;
    if (pitch == null) continue;

    const y = yForPitch(pitch.step, pitch.octave, staffBottom, clefFor(clefs, note.staff));
    drawLedgerLines(svg, x, y, staffBottom);

    const head = S.headGlyph(note, doc.attributes.divisions);
    const headLeft = x - (S.noteheadWidth * s) / 2;
    svg.glyph(head, headLeft, y, s);

    if (note.accidental) {
      const glyph = S.accidentalGlyph(note.accidental);
      if (glyph) svg.glyph(glyph, headLeft - s * 1.5, y, s);
    }
    drawDots(svg, x, y, note.dots);

    // Stem + beam grouping (chord members share the principal note's stem; whole
    // notes have no stem).
    if (head !== S.noteheadWhole && !note.is_chord) {
      const midY = staffBottom - 2 * s;
      const up = note.stem != null ? note.stem === "Up" : y >= midY;
      const n: StemNote = { x, y, up, note };
      if (note.beams.length === 0) {
        drawStemAndFlag(svg, n);
      } else {
        const key = `${note.staff}_${note.voice}`;
        const group = beamGroups.get(key) ?? [];
        group.push(n);
        beamGroups.set(key, group);
        if (note.beams.includes("End")) {
          drawBeamGroup(svg, group);
          beamGroups.delete(key);
        }
      }
    }
  }
  // Any beam group left open at the barline (defensive).
  for (const group of beamGroups.values()) drawBeamGroup(svg, group);
}

function yForPitch(step: string, octave: number, bottomLineY: number, clef: Clef): number {
  const diatonic = octave * 7 + (stepOrder[step] ?? 0);
  return bottomLineY - (diatonic - clefBottomDiatonic(clef)) * (s / 2);
}

function clefBottomDiatonic(clef: Clef): number {
  const refOnLine = clef.sign === "F" ? 3 * 7 + 3 : clef.sign === "C" ? 4 * 7 + 0 : 4 * 7 + 4;
  return refOnLine - (clef.line - 1) * 2;
}

function stemAnchor(n: StemNote): { x: number; y: number } {
  return n.up
    ? {
        x: n.x - (S.noteheadWidth * s) / 2 + S.stemUpAnchorX * s,
        y: n.y - S.stemUpAnchorY * s,
      }
    : {
        x: n.x - (S.noteheadWidth * s) / 2 + S.stemDownAnchorX * s,
        y: n.y - S.stemDownAnchorY * s,
      };
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
  const beamY = up
    ? Math.min(...anchors.map((a) => a.y)) - stemLen * s
    : Math.max(...anchors.map((a) => a.y)) + stemLen * s;
  for (const a of anchors) svg.line(a.x, a.y, a.x, beamY, "stem");
  svg.line(anchors[0].x, beamY, anchors[anchors.length - 1].x, beamY, "beam");

  // Secondary beam for consecutive 16th-or-shorter notes.
  const dir = up ? 1 : -1;
  const off = dir * (S.beamThickness + 0.2) * s;
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
  return v < lo ? lo : v > hi ? hi : v;
}

// Round to 2 decimals to keep the SVG compact and stable across platforms.
function r(v: number): string {
  return (Math.round(v * 100) / 100).toString();
}

function escapeText(t: string): string {
  return t.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
