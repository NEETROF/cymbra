<script setup lang="ts">
import { computed, nextTick, onMounted, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import { type CampaignKind, membersCsv, usePlansStore } from "@/stores/plans";
import { useRolesStore } from "@/stores/roles";
import { useToastsStore } from "@/stores/toasts";
import { saveTextAsFile } from "@/lib/download";
import type { Async } from "@/lib/async";
import type { CampaignMsg, MembershipMsg } from "@/gen/plans_pb";
import AppTag from "@/components/AppTag.vue";
import IdBadge from "@/components/IdBadge.vue";

// Music-admin campaign console (change: restructure-back-office-users-console): campaign
// lifecycle, code minting (clear text shown ONCE), the per-campaign member list and its
// export. It holds nothing about an individual account's subscription — that work starts
// in the users directory and ends on `/users/{user_id}`; an account-lookup field here
// would be a second door to the same room. All API work lives in the plans store; this
// view only matches on the Async unions and toasts each mutation's outcome (localized —
// never a raw error).
const store = usePlansStore();
const roles = useRolesStore();
const toasts = useToastsStore();
const { t } = useI18n();

const acting = computed(() => store.op.status === "loading");

const fmt = (iso?: string | null): string => {
  if (!iso) return "—";
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? iso : d.toLocaleDateString();
};

/** A destructive action waiting for the operator: the localized question and what to run
 *  once confirmed. In-app dialogs, never `window.prompt/confirm` — a native dialog blocks
 *  the renderer, which puts the action out of reach of the e2e suite and of browser
 *  automation. */
const pendingConfirm = shallowRef<{ message: string; run: () => Promise<void> } | null>(null);

/** Ask a plain yes/no, then run the action. */
function askConfirm(message: string, run: () => Promise<void>) {
  pendingConfirm.value = { message, run };
  modal.value = "confirm";
}
async function submitConfirm() {
  const pending = pendingConfirm.value;
  if (!pending) return;
  await pending.run();
  pendingConfirm.value = null;
  modal.value = null;
}
/** Toast a mutation outcome (localized), returning whether it succeeded. */
function report(outcome: Async<void>, okMsg: string): boolean {
  if (outcome.status === "error") toasts.error(outcome.error);
  else toasts.success(okMsg);
  return outcome.status === "success";
}

// ---- dialogs (create / mint / minted / confirm) ----
type Modal = "mint" | "minted" | "create" | "confirm" | null;
const modal = ref<Modal>(null);

/** Focus moves INTO the dialog when one opens. `aria-modal` requires it, and the Escape
 *  handler lives on the dialog element — with focus left on the trigger button the keydown
 *  never reached it, so Escape silently did nothing. The container takes the focus (not a
 *  button), so Enter cannot fire a destructive action. */
const dialogEl = ref<HTMLDialogElement | null>(null);
watch(modal, async (open) => {
  if (!open) return;
  await nextTick();
  dialogEl.value?.focus();
});
const mintForm = ref({ campaignKey: "", count: 10, hint: "" });
const createForm = ref<{ key: string; name: string; kind: CampaignKind; durationDays: number }>({
  key: "",
  name: "",
  kind: "premium_trial",
  durationDays: 90,
});

// ---- (b) campaigns ----
const campaignsVm = computed(() =>
  match(store.campaigns)
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null as string | null, rows: data }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, rows: [] as CampaignMsg[] }))
    .otherwise(() => ({ loading: true, error: null as string | null, rows: [] as CampaignMsg[] })),
);
function openCreate() {
  createForm.value = { key: "", name: "", kind: "premium_trial", durationDays: 90 };
  modal.value = "create";
}
const createValid = computed(() => {
  const f = createForm.value;
  return f.key.trim() !== "" && f.name.trim() !== "" && (f.kind !== "premium_trial" || f.durationDays > 0);
});
async function submitCreate() {
  const f = createForm.value;
  const ok = report(
    await store.createCampaign({
      key: f.key.trim(),
      name: f.name.trim(),
      kind: f.kind,
      durationDays: f.kind === "premium_trial" ? f.durationDays : undefined,
    }),
    t("plans.created"),
  );
  if (ok) modal.value = null;
}
/** On a TRIAL campaign, closing enrolment is the only lever the console offers
 *  (a trial campaign has no close-campaign button), and it means less than it
 *  looks: it stops new trials, it takes back none of the ones already granted —
 *  those run to their own end date, and reopening returns nothing either. An
 *  admin must not believe they stopped something they did not. */
function closeEnrolment(c: CampaignMsg) {
  const key = c.kind === "premium_trial" ? "plans.closeTrialEnrolmentConfirm" : "plans.closeEnrolmentConfirm";
  askConfirm(t(key, { key: c.key }), async () => {
    report(await store.closeEnrollment(c.key), t("plans.enrolmentClosed"));
  });
}
/** Feature betas only — a trial campaign has no close button (see
 *  `closeEnrolment`, which carries the trial's own warning). */
function closeCampaign(c: CampaignMsg) {
  askConfirm(t("plans.closeCampaignConfirm", { key: c.key }), async () => {
    report(await store.closeCampaign(c.key), t("plans.campaignClosed"));
  });
}
/** Reopening restores people, so say how many BEFORE acting: an admin reopening
 *  a campaign for a new wave must not discover the previous cohort afterwards.
 *  The count comes from the server — the "active membership" rule lives there. */
async function reopenCampaign(c: CampaignMsg) {
  const n = await store.reactivatableMembers(c.key);
  askConfirm(t("plans.reopenCampaignConfirm", { key: c.key, n }), async () => {
    report(await store.reopenCampaign(c.key), t("plans.campaignReopened", { n }));
  });
}
/** Enrolment is its own act: closing a campaign closed it as a side effect, and
 *  a campaign can legitimately be live for its members and shut to newcomers. */
function reopenEnrolment(c: CampaignMsg) {
  askConfirm(t("plans.reopenEnrolmentConfirm", { key: c.key }), async () => {
    report(await store.reopenEnrollment(c.key), t("plans.enrolmentReopened"));
  });
}
function revokeCodes(c: CampaignMsg) {
  askConfirm(t("plans.revokeCodesConfirm", { key: c.key }), async () => {
    report(await store.revokeCodes({ campaignKey: c.key }), t("plans.codesRevoked"));
  });
}

/** Whether the campaign whose members are listed is closed — its rows are then
 *  shown as inactive rather than as live memberships, so an operator never has
 *  to remember the campaign's state to read them correctly. */
const membersCampaignClosed = computed(() =>
  campaignsVm.value.rows.some((c) => c.key === store.membersKey && !!c.closedAt),
);
/** What a member's end column says: revoked wins, then a closed campaign
 *  (paused, restorable), then the trial's own end date. */
function memberState(m: MembershipMsg): string {
  if (m.revokedAt) return t("plans.revoked");
  if (membersCampaignClosed.value) return t("plans.pausedByClosure");
  return fmt(m.endsAt);
}

// ---- codes: minted once ----
function openMint(c: CampaignMsg) {
  mintForm.value = { campaignKey: c.key, count: 10, hint: "" };
  modal.value = "mint";
}
const mintedCodes = computed(() =>
  match(store.minted)
    .with({ status: "success" }, ({ data }) => data)
    .otherwise(() => [] as string[]),
);
const minting = computed(() => store.minted.status === "loading");
async function submitMint() {
  const f = mintForm.value;
  const outcome = await store.mintCodes(f.campaignKey, Math.max(1, Math.floor(f.count)), f.hint.trim());
  if (outcome.status === "error") toasts.error(outcome.error);
  else modal.value = "minted";
}
function downloadCodes() {
  saveTextAsFile(mintedCodes.value.join("\n") + "\n", `${mintForm.value.campaignKey}-codes.txt`);
}
function doneMinted() {
  store.clearMinted();
  modal.value = null;
}

// ---- (c) members ----
const membersVm = computed(() =>
  match(store.members)
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null as string | null, rows: data }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, rows: [] as MembershipMsg[] }))
    .otherwise(() => ({ loading: true, error: null as string | null, rows: [] as MembershipMsg[] })),
);
/** Best-effort handle for a member, from a directory page loaded earlier in the session.
 *  Without one, the row shows the id — and the link to the member's own page is what
 *  actually answers "who is this?", in one click. */
function handleFor(userId: string): string | undefined {
  const dir = roles.directory.status === "success" ? roles.directory.data.accounts : [];
  return dir.find((a) => a.userId === userId)?.handle || undefined;
}
function showMembers(c: CampaignMsg) {
  void store.loadMembers(c.key);
}
function exportCsv() {
  if (!store.membersKey) return;
  saveTextAsFile(
    membersCsv(membersVm.value.rows, handleFor),
    `${store.membersKey}-members.csv`,
    "text/csv;charset=utf-8",
  );
}

onMounted(() => void store.loadCampaigns(true));
</script>

<template>
  <h1 class="page-title">{{ t("campaigns.title") }}</h1>
  <p class="muted">{{ t("campaigns.intro") }}</p>

  <!-- campaigns -->
  <section class="block">
    <!-- No section heading: the page is the campaign list, and a second "Campaigns"
         under the page title says nothing the title has not. -->
    <div class="page-head actions-only">
      <button type="button" class="btn-primary" :disabled="acting" @click="openCreate">{{ t("plans.create") }}</button>
    </div>
    <p v-if="campaignsVm.error" class="error" role="alert">{{ campaignsVm.error }}</p>
    <div class="table-card">
      <table data-testid="campaigns">
        <thead>
          <tr>
            <th>{{ t("plans.colKey") }}</th>
            <th>{{ t("plans.colName") }}</th>
            <th>{{ t("plans.colKind") }}</th>
            <th>{{ t("plans.colDuration") }}</th>
            <th>{{ t("plans.colEnrolment") }}</th>
            <th>{{ t("plans.colClosed") }}</th>
            <th>{{ t("plans.colActions") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="c in campaignsVm.rows" :key="c.key" :class="{ inactive: !!c.closedAt }">
            <td>
              <code>{{ c.key }}</code>
            </td>
            <td>{{ c.name }}</td>
            <td>{{ t(`plans.kind.${c.kind}`) }}</td>
            <td>{{ c.durationDays ? t("plans.days", { n: c.durationDays }) : "—" }}</td>
            <td>
              <AppTag :variant="c.acceptsEnrolment ? 'accepted' : 'neutral'">
                {{ c.acceptsEnrolment ? t("plans.open") : t("plans.closed") }}
              </AppTag>
            </td>
            <td>{{ c.closedAt ? fmt(c.closedAt) : "—" }}</td>
            <td class="row-actions">
              <button type="button" class="btn-sm" @click="showMembers(c)">{{ t("plans.members") }}</button>
              <button
                v-if="c.acceptsEnrolment"
                type="button"
                class="btn-sm"
                :disabled="acting"
                @click="closeEnrolment(c)"
              >
                {{ t("plans.closeEnrolment") }}
              </button>
              <button
                v-if="c.kind === 'feature' && !c.closedAt"
                type="button"
                class="btn-sm reject"
                :disabled="acting"
                @click="closeCampaign(c)"
              >
                {{ t("plans.closeCampaign") }}
              </button>
              <button v-if="c.closedAt" type="button" class="btn-sm" :disabled="acting" @click="reopenCampaign(c)">
                {{ t("plans.reopenCampaign") }}
              </button>
              <button
                v-if="!c.closedAt && !c.acceptsEnrolment"
                type="button"
                class="btn-sm"
                :disabled="acting"
                @click="reopenEnrolment(c)"
              >
                {{ t("plans.reopenEnrolment") }}
              </button>
              <button v-if="!c.closedAt" type="button" class="btn-sm" :disabled="acting" @click="openMint(c)">
                {{ t("plans.mint") }}
              </button>
              <button v-if="!c.closedAt" type="button" class="btn-sm" :disabled="acting" @click="revokeCodes(c)">
                {{ t("plans.revokeCodes") }}
              </button>
            </td>
          </tr>
          <tr v-if="campaignsVm.rows.length === 0">
            <td colspan="7" class="empty">{{ campaignsVm.loading ? t("common.loading") : t("plans.noCampaigns") }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>

  <!-- members of the selected campaign -->
  <section v-if="store.membersKey" class="block">
    <div class="page-head">
      <h2>{{ t("plans.membersOf", { key: store.membersKey }) }}</h2>
      <button type="button" :disabled="membersVm.rows.length === 0" @click="exportCsv">
        {{ t("plans.exportCsv") }}
      </button>
    </div>
    <p v-if="membersVm.error" class="error" role="alert">{{ membersVm.error }}</p>
    <div class="table-card">
      <table data-testid="members">
        <thead>
          <tr>
            <th>{{ t("plans.colAccount") }}</th>
            <th>{{ t("plans.enrolled") }}</th>
            <th>{{ t("plans.endsAt") }}</th>
            <th>{{ t("plans.colSource") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="m in membersVm.rows" :key="m.userId" :class="{ inactive: !!m.revokedAt || membersCampaignClosed }">
            <td>
              <!-- A cohort is read here, but acted on there: one click opens the member's
                   own page instead of re-identifying them by hand. -->
              <RouterLink class="member" :to="{ name: 'user-detail', params: { userId: m.userId } }">
                <span v-if="handleFor(m.userId)" class="handle">{{ handleFor(m.userId) }}</span>
                <IdBadge v-else :id="m.userId" />
              </RouterLink>
            </td>
            <td>{{ fmt(m.enrolledAt) }}</td>
            <td>{{ memberState(m) }}</td>
            <td class="mono">{{ m.source }}</td>
          </tr>
          <tr v-if="membersVm.rows.length === 0">
            <td colspan="4" class="empty">{{ membersVm.loading ? t("common.loading") : t("plans.noMembers") }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>

  <!-- dialogs -->
  <div v-if="modal" class="overlay" @click.self="modal = null">
    <dialog ref="dialogEl" class="modal" open aria-modal="true" tabindex="-1" @keydown.esc="modal = null">
      <!-- a destructive action that only needs a yes -->
      <template v-if="modal === 'confirm'">
        <h2>{{ t("plans.confirmTitle") }}</h2>
        <p>{{ pendingConfirm?.message }}</p>
        <div class="modal-actions">
          <button type="button" class="btn-primary" :disabled="acting" @click="submitConfirm">
            {{ t("plans.confirm") }}
          </button>
          <button type="button" :disabled="acting" @click="modal = null">{{ t("plans.cancel") }}</button>
        </div>
      </template>

      <!-- create campaign -->
      <template v-else-if="modal === 'create'">
        <h2>{{ t("plans.createTitle") }}</h2>
        <label>
          {{ t("plans.colKey") }}
          <input v-model="createForm.key" :placeholder="t('plans.keyPlaceholder')" :aria-label="t('plans.colKey')" />
        </label>
        <label>
          {{ t("plans.colName") }}
          <input v-model="createForm.name" :placeholder="t('plans.namePlaceholder')" :aria-label="t('plans.colName')" />
        </label>
        <label>
          {{ t("plans.colKind") }}
          <select v-model="createForm.kind" :aria-label="t('plans.colKind')">
            <option value="premium_trial">{{ t("plans.kind.premium_trial") }}</option>
            <option value="feature">{{ t("plans.kind.feature") }}</option>
          </select>
        </label>
        <label v-if="createForm.kind === 'premium_trial'">
          {{ t("plans.durationDays") }}
          <input v-model.number="createForm.durationDays" type="number" min="1" :aria-label="t('plans.durationDays')" />
        </label>
        <div class="modal-actions">
          <button type="button" class="btn-primary" :disabled="acting || !createValid" @click="submitCreate">
            {{ t("plans.create") }}
          </button>
          <button type="button" :disabled="acting" @click="modal = null">{{ t("plans.cancel") }}</button>
        </div>
      </template>

      <!-- mint codes -->
      <template v-else-if="modal === 'mint'">
        <h2>{{ t("plans.mintTitle", { key: mintForm.campaignKey }) }}</h2>
        <label>
          {{ t("plans.count") }}
          <input v-model.number="mintForm.count" type="number" min="1" max="1000" :aria-label="t('plans.count')" />
        </label>
        <label>
          {{ t("plans.hint") }}
          <input v-model="mintForm.hint" :aria-label="t('plans.hint')" />
        </label>
        <div class="modal-actions">
          <button type="button" class="btn-primary" :disabled="minting || mintForm.count < 1" @click="submitMint">
            {{ t("plans.mint") }}
          </button>
          <button type="button" :disabled="minting" @click="modal = null">{{ t("plans.cancel") }}</button>
        </div>
      </template>

      <!-- minted: shown once -->
      <template v-else-if="modal === 'minted'">
        <h2>{{ t("plans.mintedTitle", { n: mintedCodes.length }) }}</h2>
        <p class="hint">{{ t("plans.mintedHint") }}</p>
        <pre class="codes" data-testid="minted-codes">{{ mintedCodes.join("\n") }}</pre>
        <div class="modal-actions">
          <button type="button" class="btn-primary" @click="downloadCodes">{{ t("plans.download") }}</button>
          <button type="button" @click="doneMinted">{{ t("plans.done") }}</button>
        </div>
      </template>
    </dialog>
  </div>
</template>

<style scoped>
.block {
  margin-top: 1.75rem;
}
.page-head.actions-only {
  justify-content: flex-end;
}
.block h2 {
  font-size: 1.15rem;
  margin: 0 0 0.75rem;
}
.block h3 {
  font-size: 0.95rem;
  margin: 1.25rem 0 0.5rem;
  color: var(--muted);
}
.mono {
  font-family: var(--mono);
  font-size: 0.85rem;
}
.handle {
  font-weight: 600;
}
.member {
  color: inherit;
  text-decoration: none;
}
.member:hover,
.member:focus-visible {
  color: var(--accent);
  text-decoration: underline;
}
.row-actions {
  display: flex;
  gap: 0.4rem;
  flex-wrap: wrap;
}
.btn-sm {
  padding: 0.32rem 0.6rem;
  font-size: 0.8rem;
  white-space: nowrap;
}
tr.inactive td {
  opacity: 0.55;
}
/* …except the actions. A closed campaign's row is dimmed to say the campaign is
   over, but its controls are the one thing on that row that still works — and
   the reopen button IS the row's reason to exist. Dimmed, they read as disabled
   while remaining clickable, which is worse than not dimming them at all. */
tr.inactive td.row-actions {
  opacity: 1;
}
.empty {
  color: var(--muted);
  text-align: center;
  padding: 2rem;
}
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 90;
  display: grid;
  place-items: center;
}
.modal {
  position: static;
  width: min(480px, 94vw);
  margin: 0;
  background: var(--panel, #1a1a24);
  color: inherit;
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 1.35rem 1.4rem;
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
}
.modal:focus {
  outline: none;
}
.modal h2 {
  font-size: 1.05rem;
  margin: 0;
}
.modal label {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  font-size: 0.85rem;
  color: var(--muted);
}
.modal label.check {
  flex-direction: row;
  align-items: center;
  color: var(--text);
}
.modal input,
.modal select {
  width: 100%;
}
.modal label.check input {
  width: auto;
}
.hint {
  margin: 0;
  color: var(--muted);
  font-size: 0.78rem;
}
.codes {
  max-height: 40vh;
  overflow: auto;
  padding: 0.75rem;
  background: var(--panel-2);
  border-radius: 10px;
  font-family: var(--mono);
  font-size: 0.85rem;
  user-select: all;
}
.modal-actions {
  display: flex;
  gap: 0.5rem;
  border-top: 1px solid var(--border);
  padding-top: 0.9rem;
}
</style>
