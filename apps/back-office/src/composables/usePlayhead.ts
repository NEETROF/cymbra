// The playback overlay for the rendered notation SVG: a moving cursor line, the
// highlight on currently-sounding note heads, and auto-scroll to follow the playhead.
// It manipulates the injected SVG DOM directly (the SVG is set once via v-html and is
// static during playback), driven by the pure playhead maths in schedule.ts. Isolated
// here so ScorePreview stays presentational.

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
      cursor = document.createElementNS(SVG_NS, "line");
      cursor.setAttribute("class", "cursor");
      cursor.style.display = "none";
      svgEl.appendChild(cursor);
      for (const el of Array.from(svgEl.querySelectorAll<SVGElement>("[data-note]"))) {
        const id = el.getAttribute("data-note");
        if (id) heads.set(id, el);
      }
      update();
    },
    { immediate: true },
  );

  watch([opts.elapsedMs, opts.playing], update);

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
    if (opts.playing.value) scrollIntoView();
  }

  function setLit(ids: Set<string>): void {
    for (const id of lit) if (!ids.has(id)) heads.get(id)?.classList.remove("playing");
    for (const id of ids) if (!lit.has(id)) heads.get(id)?.classList.add("playing");
    lit = ids;
  }

  // Keep the cursor's line comfortably inside the scroll container.
  function scrollIntoView(): void {
    const box = opts.container.value;
    if (!box || !cursor) return;
    const c = cursor.getBoundingClientRect();
    const b = box.getBoundingClientRect();
    const margin = 24;
    if (c.top < b.top + margin) box.scrollTop -= b.top + margin - c.top;
    else if (c.bottom > b.bottom - margin) box.scrollTop += c.bottom - (b.bottom - margin);
  }
}
