// The Lingua leaving page's state + its one call (rule: components never call an API —
// composables do, behind the injectable `lingua-leave.ts` seam). Change:
// add-lingua-usage-counts, D12.
//
// Loading the page sends nothing: `boot` only reads the address. Tapping an answer sends
// it once; the reader is thanked from that tap on, whatever the backend answers — a
// refusal or a network failure is never shown, it changes nothing for the reader.
import { computed, ref } from "vue";
import { type Async, idle, run } from "@cymbra/web-auth";
import {
  leaveContextFromQuery,
  useLinguaLeaveClient,
  type LeaveContext,
  type LeaveReason,
} from "../lib/lingua-leave";

/** What the page shows for a state of the answer: the six answers, or the thanks. */
export interface LeaveView {
  /** The answers are disabled and the thanks shown. */
  answered: boolean;
}

/** Folds the answer's state into the view; exhaustive, so a new state is a compile error. */
export function leaveView(state: Async<void>): LeaveView {
  switch (state.status) {
    case "idle":
      return { answered: false };
    // Thanked as soon as the answer leaves, and still thanked if it fails.
    case "loading":
    case "success":
    case "error":
      return { answered: true };
    default: {
      const unreachable: never = state;
      return unreachable;
    }
  }
}

export function useLeaveReason() {
  /** The one answer of this visit. */
  const answer = ref<Async<void>>(idle);
  /** The answer the reader tapped, to mark its button. */
  const chosen = ref<LeaveReason | null>(null);
  const context = ref<LeaveContext>({ browser: "unknown", version: "unknown" });
  const view = computed(() => leaveView(answer.value));

  /** Reads the browser family and version from the page's address. Sends nothing. */
  function boot(search: string): void {
    context.value = leaveContextFromQuery(search);
  }

  /** Sends the tapped answer — at most one per visit; a second tap sends nothing. */
  async function choose(reason: LeaveReason): Promise<void> {
    if (answer.value.status !== "idle") return;
    chosen.value = reason;
    const { browser, version } = context.value;
    await run(answer, () => useLinguaLeaveClient().leave({ reason, browser, version }));
  }

  return { answer, chosen, view, boot, choose };
}
