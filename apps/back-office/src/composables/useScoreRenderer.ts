// Turns score bytes into a rendered notation SVG, as one reactive `Async` value.
//
// This is the isolated renderer seam (design D4/D5): the leaf component stays
// presentational and reads this state; the heavy wasm module is lazy-loaded here, not
// in a component. A load/instantiate/render failure folds into `error` (never throws),
// so the preview degrades to a placeholder rather than crashing the page.

import { ref, watch, type Ref } from "vue";
import { type Async, failure, idle, loading, success } from "@/lib/async";
import { renderNotation } from "@/lib/notation/painter";
import { loadNotationWasm } from "@/lib/notation/wasm";

// Layout width handed to the engine (and thus the SVG's intrinsic width). The SVG
// scales down responsively (max-width:100%); this only sets where system line breaks
// fall, matching the app's own justification.
const RENDER_WIDTH = 1000;

/**
 * @param bytes reactive score bytes (null while absent/loading upstream)
 * @returns `notation` — Async<string> of the rendered SVG markup
 */
export function useScoreRenderer(bytes: Ref<Uint8Array | null | undefined>): {
  notation: Ref<Async<string>>;
} {
  const notation = ref<Async<string>>(idle);

  watch(
    bytes,
    async (value) => {
      if (value == null) {
        notation.value = idle;
        return;
      }
      notation.value = loading;
      // Snapshot the bytes we started on, so a newer request wins if the row changes
      // mid-load (avoids a stale render clobbering a fresh one).
      const started = value;
      try {
        const wasm = await loadNotationWasm();
        const geometry = wasm.render(started, RENDER_WIDTH);
        const svg = renderNotation(geometry, RENDER_WIDTH);
        if (bytes.value === started) notation.value = success(svg);
      } catch {
        // The error is a stable code the component localizes — never a raw wasm
        // string shown to the user (see the no-raw-technical-errors rule).
        if (bytes.value === started) notation.value = failure("render_failed");
      }
    },
    { immediate: true },
  );

  return { notation };
}
