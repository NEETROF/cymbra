import { ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, failure, idle, loading, run, success } from "@/lib/async";
import { humanError } from "@/lib/errors";
import { t } from "@/i18n";
import type { AdminUserScore } from "@/gen/score_pb";

// Private-score takedown (change: add-private-score-catalog) — the in-repo half of
// notice-and-takedown. A music-scope admin looks a reported score up and removes it
// with a mandatory reason; the server writes an audit row before deleting anything.
//
// Both the search and the removal are `Async` unions the view matches on: a refusal
// (not an admin, no reason, unknown score) lands in the union as an error, never as
// a throw. The store never serves score bytes — the lookup carries metadata only.

const PAGE_SIZE = 25;

export const useTakedownsStore = defineStore("takedowns", () => {
  const results = ref<Async<AdminUserScore[]>>(idle);
  const op = ref<Async<void>>(idle);
  const ownerId = ref("");
  const title = ref("");
  const offset = ref(0);

  /** Whether the current criteria can be searched at all. The server refuses a
   *  criterion-less lookup (it answers a notice, it is not a corpus browser), so
   *  the button is disabled rather than provoking that refusal. */
  function canSearch(): boolean {
    return ownerId.value.trim() !== "" || title.value.trim() !== "";
  }

  /** Look up private scores by owner and/or title fragment. */
  async function search(opts: { ownerId?: string; title?: string; offset?: number } = {}) {
    if (opts.ownerId !== undefined) ownerId.value = opts.ownerId;
    if (opts.title !== undefined) title.value = opts.title;
    offset.value = opts.offset ?? 0;
    if (!canSearch()) {
      results.value = failure(t("takedowns.criterionRequired"));
      return;
    }
    await run(results, async () => {
      const resp = await api().score.adminSearchUserScores({
        ownerId: ownerId.value.trim() || undefined,
        title: title.value.trim() || undefined,
        limit: PAGE_SIZE,
        offset: offset.value,
      });
      return resp.scores;
    });
  }

  /** Remove one private score with a mandatory reason, then re-run the search so
   *  the table reflects what is left. Irreversible — the view confirms first. */
  async function remove(id: string, reason: string) {
    if (reason.trim() === "") {
      op.value = failure(t("takedowns.reasonRequired"));
      return;
    }
    op.value = loading;
    try {
      await api().score.adminRemoveUserScore({ id, reason: reason.trim() });
      op.value = success(undefined);
      await search();
    } catch (e) {
      op.value = failure(humanError(e)); // humanError logs the cause
    }
  }

  return { results, op, ownerId, title, offset, canSearch, search, remove };
});
