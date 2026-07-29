// The playback overlay for the rendered notation SVG: a moving cursor line, the
// highlight on currently-sounding note heads, and click-to-seek per measure. It
// manipulates the injected SVG DOM directly (the SVG is set once via v-html and is
// static during playback), driven by the pure playhead maths in schedule.ts. Isolated
// here so ScorePreview stays presentational.
//
// No auto-scroll: this is a back-office validation view, so the moderator keeps control
// of the viewport. Clicking a measure seeks playback to that measure's start.

import { nextTick, watch, type Ref } from "vue";
import { type PlaybackSchedule, measureAt, playingNoteIds } from "@/lib/notation/schedule";
import type { MeasureRect } from "@/lib/notation/painter";

const SVG_NS = "http://www.w3.org/2000/svg";

interface Options {
  container: Ref<HTMLElement | null>;
  svg: Ref<string | null>;
  layout: Ref<{ measures: MeasureRect[] } | null>;
  schedule: Ref<PlaybackSchedule | null>;
  elapsedMs: Ref<number>;
  playing: Ref<boolean>;
  /** Called when a measure is clicked, with its index — the caller seeks to it. */
  onSeekMeasure?: (measureIndex: number) => void;
}

export function usePlayhead(opts: Options): void {
  let svgEl: SVGSVGElement | null = null;
  let cursor: SVGLineElement | null = null;
  let heads: Map<string, SVGElement> = new Map();
  let lit = new Set<string>();

  // (Re)bind to the freshly-injected SVG whenever the notation changes.
  watch(
    opts.svg,
    async () => {
      await nextTick();
      svgEl = opts.container.value?.querySelector("svg") ?? null;
      cursor = null;
      heads = new Map();
      lit = new Set();
      if (!svgEl) return;

      // Transparent per-measure hit areas for click-to-seek (added first, so they sit
      // under the glyphs visually but still receive clicks over their whole box).
      for (const m of opts.layout.value?.measures ?? []) {
        const hit = document.createElementNS(SVG_NS, "rect");
        hit.setAttribute("x", String(m.x));
        hit.setAttribute("y", String(m.top));
        hit.setAttribute("width", String(m.width));
        hit.setAttribute("height", String(m.bottom - m.top));
        hit.setAttribute("fill", "transparent");
        hit.setAttribute("class", "measure-hit");
        hit.dataset.measure = String(m.index);
        hit.style.cursor = "pointer";
        svgEl.appendChild(hit);
      }
      svgEl.addEventListener("click", onClick);

      for (const el of Array.from(svgEl.querySelectorAll<SVGElement>("[data-note]"))) {
        const id = el.dataset.note;
        if (id) heads.set(id, el);
      }

      // Cursor last, so it paints on top of the notes; pointer-events:none so it never
      // swallows a measure click.
      cursor = document.createElementNS(SVG_NS, "line");
      cursor.setAttribute("class", "cursor");
      cursor.style.display = "none";
      cursor.style.pointerEvents = "none";
      svgEl.appendChild(cursor);
      update();
    },
    { immediate: true },
  );

  watch([opts.elapsedMs, opts.playing], update);

  function onClick(e: Event): void {
    const el = (e.target as Element | null)?.closest<SVGElement>("[data-measure]");
    const raw = el?.dataset.measure;
    if (raw == null) return;
    const idx = Number(raw);
    if (Number.isFinite(idx)) opts.onSeekMeasure?.(idx);
  }

  function update(): void {
    if (!svgEl || !cursor) return;
    const schedule = opts.schedule.value;
    const measures = opts.layout.value?.measures ?? [];
    const at = schedule ? measureAt(schedule, opts.elapsedMs.value) : null;
    const rect = at ? measures.find((m) => m.index === at.index) : undefined;

    if (!rect || !at) {
      cursor.style.display = "none";
      setLit(new Set());
      return;
    }
    const x = rect.x + at.fraction * rect.width;
    cursor.setAttribute("x1", String(x));
    cursor.setAttribute("x2", String(x));
    cursor.setAttribute("y1", String(rect.top));
    cursor.setAttribute("y2", String(rect.bottom));
    cursor.style.display = "";

    setLit(schedule ? playingNoteIds(schedule, opts.elapsedMs.value) : new Set());
  }

  function setLit(ids: Set<string>): void {
    for (const id of lit) if (!ids.has(id)) heads.get(id)?.classList.remove("playing");
    for (const id of ids) if (!lit.has(id)) heads.get(id)?.classList.add("playing");
    lit = ids;
  }
}
