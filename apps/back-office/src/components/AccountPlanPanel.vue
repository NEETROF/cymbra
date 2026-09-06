<script setup lang="ts">
import { computed, nextTick, onMounted, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import { isRevocableSource, usePlansStore } from "@/stores/plans";
import { useToastsStore } from "@/stores/toasts";
import type { Async } from "@/lib/async";
import type { EntitlementRowMsg, MembershipMsg } from "@/gen/plans_pb";
import AppTag from "@/components/AppTag.vue";

// One account's subscription, on that account's own page (change:
// restructure-back-office-users-console): effective plan, entitlement rows, beta
// memberships, and the nominative gestures (grant premium, enrol, revoke) with their
// audited reason. Rendered ONLY for a music-scope admin — the parent gates it, so a
// `live`-only admin never sees the block and no plan RPC is issued for them.
//
// The account is addressed by id: there is no lookup field here, because the page
// already knows whose account it is. All API work stays in the plans store.
const props = defineProps<{ userId: string; handle: string }>();

const store = usePlansStore();
const toasts = useToastsStore();
const { t } = useI18n();

const lookupVm = computed(() =>
  match(store.lookupResult)
    .with({ status: "idle" }, () => ({ loading: true, error: null as string | null, data: null }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, data: null }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, data: null }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, data }))
    .exhaustive(),
);
const acting = computed(() => store.op.status === "loading");
/** Always by id — the account is the one being viewed, never a retyped handle. */
const target = computed(() => ({ userId: props.userId }));

const fmt = (iso?: string | null): string => {
  if (!iso) return "—";
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? iso : d.toLocaleDateString();
};
const isActive = (r: EntitlementRowMsg) => r.status === "active";
const canRevoke = (r: EntitlementRowMsg) => isActive(r) && isRevocableSource(r.source);
const openEnded = (r: EntitlementRowMsg) => isActive(r) && !r.endsAt;

async function copyRef(r: EntitlementRowMsg) {
  try {
    await navigator.clipboard.writeText(r.providerRef);
    toasts.success(t("plans.refCopied"));
  } catch {
    // Clipboard blocked (insecure context): nothing to surface — the ref stays hidden.
  }
}

/** A destructive action waiting for the operator: the localized question and
 *  what to run once confirmed. In-app dialogs, never `window.prompt/confirm` —
 *  a native dialog blocks the renderer, which puts the action out of reach of
 *  the e2e suite and of browser automation. */
const pendingReason = shallowRef<{ message: string; run: (reason: string) => Promise<void> } | null>(null);
const reasonForm = ref({ reason: "" });
const reasonValid = computed(() => reasonForm.value.reason.trim() !== "");

/** Ask a free-text reason (audited), then run the action. */
function askReason(message: string, run: (reason: string) => Promise<void>) {
  reasonForm.value = { reason: "" };
  pendingReason.value = { message, run };
  modal.value = "reason";
}
async function submitReason() {
  const pending = pendingReason.value;
  if (!pending) return;
  await pending.run(reasonForm.value.reason);
  pendingReason.value = null;
  modal.value = null;
}
/** Toast a mutation outcome (localized), returning whether it succeeded. */
function report(outcome: Async<void>, okMsg: string): boolean {
  if (outcome.status === "error") toasts.error(outcome.error);
  else toasts.success(okMsg);
  return outcome.status === "success";
}

function revokeRow(r: EntitlementRowMsg) {
  askReason(t("plans.revokeRowConfirm", { source: r.source }), async (reason) => {
    report(await store.revokeEntitlement(r.id, reason), t("plans.revokedRow"));
  });
}
function revokeMembership(m: MembershipMsg) {
  askReason(t("plans.revokeMembershipConfirm", { campaign: m.campaignKey }), async (reason) => {
    report(
      await store.revokeMembership({ target: { userId: m.userId }, campaignKey: m.campaignKey, reason }),
      t("plans.membershipRevoked"),
    );
  });
}

// ---- dialogs (grant / enrol / reason) ----
type Modal = "grant" | "enrol" | "reason" | null;
const modal = ref<Modal>(null);

/** Focus moves INTO the dialog when one opens. `aria-modal` requires it, and the
 *  Escape handler lives on the dialog element — with focus left on the trigger
 *  button the keydown never reached it, so Escape silently did nothing. The
 *  container takes the focus (not a button), so Enter cannot fire a destructive
 *  action; the reason field is still one Tab away. */
const dialogEl = ref<HTMLDialogElement | null>(null);
watch(modal, async (open) => {
  if (!open) return;
  await nextTick();
  dialogEl.value?.focus();
});
const grantForm = ref({ endDate: "", confirmOpenEnded: false, reason: "" });
const enrolForm = ref({ campaignKey: "", reason: "" });

function openGrant() {
  grantForm.value = { endDate: "", confirmOpenEnded: false, reason: "" };
  modal.value = "grant";
}
const grantValid = computed(
  () => grantForm.value.reason.trim() !== "" && (grantForm.value.endDate !== "" || grantForm.value.confirmOpenEnded),
);
async function submitGrant() {
  const f = grantForm.value;
  const endsAt = f.endDate ? new Date(`${f.endDate}T23:59:59Z`).toISOString() : undefined;
  const ok = report(
    await store.grantPremium({
      target: target.value,
      endsAt,
      confirmOpenEnded: !f.endDate && f.confirmOpenEnded,
      reason: f.reason.trim(),
    }),
    t("plans.granted"),
  );
  if (ok) modal.value = null;
}

/** Campaigns an admin may still enrol this account in (open AND accepting enrolment),
 *  each flagged when the account is already a LIVE member of it. Such a campaign stays
 *  listed but disabled with the reason: hiding it leaves the operator hunting for a
 *  campaign that is right there on the page, and offering it plainly walks them into a
 *  refusal the console could see coming. A REVOKED membership does not disable it —
 *  putting someone back in is exactly what an admin re-enrols for. */
const enrollable = computed(() => {
  const live = new Set((lookupVm.value.data?.memberships ?? []).filter((m) => !m.revokedAt).map((m) => m.campaignKey));
  return store.openCampaigns
    .filter((c) => c.acceptsEnrolment)
    .map((c) => ({ key: c.key, name: c.name, kind: c.kind, taken: live.has(c.key) }));
});
const joinable = computed(() => enrollable.value.filter((c) => !c.taken));
function openEnrol() {
  enrolForm.value = { campaignKey: joinable.value[0]?.key ?? "", reason: "" };
  modal.value = "enrol";
}
const enrolValid = computed(
  () =>
    enrolForm.value.campaignKey !== "" &&
    enrolForm.value.reason.trim() !== "" &&
    joinable.value.some((c) => c.key === enrolForm.value.campaignKey),
);
async function submitEnrol() {
  const f = enrolForm.value;
  const ok = report(
    await store.enrolHandle({ target: target.value, campaignKey: f.campaignKey, reason: f.reason.trim() }),
    t("plans.enrolled_ok"),
  );
  if (ok) modal.value = null;
}

/** What a membership's end column says: revoked wins, then a closed campaign (paused,
 *  restorable), then the trial's own end date. The campaign list is already loaded for
 *  the enrol selector, so the closure is read per membership — an operator must not have
 *  to remember a campaign's state to read this row correctly. */
const closedCampaigns = computed(
  () =>
    new Set(
      store.campaigns.status === "success" ? store.campaigns.data.filter((c) => c.closedAt).map((c) => c.key) : [],
    ),
);
function memberState(m: MembershipMsg): string {
  if (m.revokedAt) return t("plans.revoked");
  if (closedCampaigns.value.has(m.campaignKey)) return t("plans.pausedByClosure");
  return fmt(m.endsAt);
}

/** The aggregator's customer page for this account (D5) — built by the server, which
 *  owns the project id; empty when the aggregator is unconfigured. */
const revenueCatUrl = computed(() => lookupVm.value.data?.aggregatorCustomerUrl || undefined);

// The lookup slot is shared store state: clear it BEFORE loading, so a switch from one
// account to another can never paint account A's rights under account B's name. The
// parent keys this component by user id, so this runs on every account.
onMounted(() => {
  store.clearLookup();
  void store.lookupUser(props.userId);
  void store.loadCampaigns(true);
});
</script>

<template>
  <section class="block">
    <h2>{{ t("users.subscription") }}</h2>
    <p v-if="lookupVm.loading" class="muted">{{ t("common.loading") }}</p>
    <p v-if="lookupVm.error" class="error" role="alert">{{ lookupVm.error }}</p>

    <template v-if="lookupVm.data">
      <div class="summary" data-testid="effective-plan">
        <div class="kv">
          <span class="k">{{ t("plans.effective") }}</span>
          <span class="v">
            <AppTag :variant="lookupVm.data.snapshot?.plan === 'premium' ? 'accepted' : 'neutral'">
              {{ lookupVm.data.snapshot?.plan === "premium" ? t("plans.premium") : t("plans.free") }}
            </AppTag>
            <AppTag v-if="lookupVm.data.snapshot?.trialCampaignKey" variant="warn">{{ t("plans.trialTag") }}</AppTag>
            <AppTag v-if="lookupVm.data.snapshot?.plan === 'premium' && !lookupVm.data.snapshot?.endsAt" variant="warn">
              {{ t("plans.openEnded") }}
            </AppTag>
          </span>
        </div>
        <div class="kv">
          <span class="k">{{ t("plans.source") }}</span>
          <span class="v mono">{{ lookupVm.data.snapshot?.source || "—" }}</span>
        </div>
        <div class="kv">
          <span class="k">{{ t("plans.endsAt") }}</span>
          <span class="v">{{ fmt(lookupVm.data.snapshot?.endsAt) }}</span>
        </div>
        <div class="kv">
          <span class="k">{{ t("plans.trial") }}</span>
          <span class="v">
            <template v-if="lookupVm.data.snapshot?.trialCampaignKey">
              <code>{{ lookupVm.data.snapshot?.trialCampaignKey }}</code> ·
              {{ fmt(lookupVm.data.snapshot?.trialEndsAt) }}
            </template>
            <template v-else>—</template>
          </span>
        </div>
        <div class="kv">
          <span class="k">{{ t("plans.betas") }}</span>
          <span class="v chips">
            <AppTag v-for="b in lookupVm.data.snapshot?.betas ?? []" :key="b.campaignKey" variant="neutral" mono>
              {{ b.campaignKey }}
            </AppTag>
            <span v-if="(lookupVm.data.snapshot?.betas ?? []).length === 0">—</span>
          </span>
        </div>
        <div class="kv actions">
          <button type="button" class="btn-primary" :disabled="acting" @click="openGrant">
            {{ t("plans.grant") }}
          </button>
          <button type="button" :disabled="acting || joinable.length === 0" @click="openEnrol">
            {{ t("plans.enrol") }}
          </button>
          <!-- Store facts (transactions, renewals, refunds) and revenue live at the
               aggregator (change: swap-store-billing-to-revenuecat, D5): one link
               to the customer page; hidden when the project id is not configured. -->
          <a
            v-if="revenueCatUrl"
            :href="revenueCatUrl"
            target="_blank"
            rel="noopener noreferrer"
            data-testid="revenuecat-link"
          >
            {{ t("plans.openInRevenueCat") }} ↗
          </a>
        </div>
      </div>

      <h3>{{ t("plans.entitlements") }}</h3>
      <div class="table-card">
        <table data-testid="entitlements">
          <thead>
            <tr>
              <th>{{ t("plans.source") }}</th>
              <th>{{ t("plans.startsAt") }}</th>
              <th>{{ t("plans.endsAt") }}</th>
              <th>{{ t("plans.status") }}</th>
              <th>{{ t("plans.campaign") }}</th>
              <th>{{ t("plans.colActions") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="r in lookupVm.data.rows" :key="r.id" :class="{ inactive: !isActive(r) }">
              <td class="mono">{{ r.source }}</td>
              <td>{{ fmt(r.startsAt) }}</td>
              <td>
                <AppTag v-if="openEnded(r)" variant="warn">{{ t("plans.openEnded") }}</AppTag>
                <template v-else>{{ fmt(r.endsAt) }}</template>
              </td>
              <td>
                <AppTag :variant="isActive(r) ? 'accepted' : 'neutral'">{{ r.status }}</AppTag>
              </td>
              <td class="mono">{{ r.campaignId || "—" }}</td>
              <td class="row-actions">
                <button v-if="r.providerRef" type="button" class="btn-sm" @click="copyRef(r)">
                  {{ t("plans.copyRef") }}
                </button>
                <button
                  v-if="canRevoke(r)"
                  type="button"
                  class="btn-sm reject"
                  :disabled="acting"
                  @click="revokeRow(r)"
                >
                  {{ t("plans.revoke") }}
                </button>
              </td>
            </tr>
            <tr v-if="lookupVm.data.rows.length === 0">
              <td colspan="6" class="empty">{{ t("plans.noRows") }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <h3>{{ t("plans.memberships") }}</h3>
      <div class="table-card">
        <table data-testid="memberships">
          <thead>
            <tr>
              <th>{{ t("plans.campaign") }}</th>
              <th>{{ t("plans.kindCol") }}</th>
              <th>{{ t("plans.enrolled") }}</th>
              <th>{{ t("plans.endsAt") }}</th>
              <th>{{ t("plans.colSource") }}</th>
              <th>{{ t("plans.colActions") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="m in lookupVm.data.memberships" :key="m.campaignKey" :class="{ inactive: !!m.revokedAt }">
              <td>
                <code>{{ m.campaignKey }}</code> <span class="muted">{{ m.campaignName }}</span>
              </td>
              <td>{{ t(`plans.kind.${m.kind}`) }}</td>
              <td>{{ fmt(m.enrolledAt) }}</td>
              <td>{{ memberState(m) }}</td>
              <td class="mono">{{ m.source }}</td>
              <td class="row-actions">
                <button
                  v-if="!m.revokedAt"
                  type="button"
                  class="btn-sm reject"
                  :disabled="acting"
                  @click="revokeMembership(m)"
                >
                  {{ t("plans.revoke") }}
                </button>
              </td>
            </tr>
            <tr v-if="lookupVm.data.memberships.length === 0">
              <td colspan="6" class="empty">{{ t("plans.noMemberships") }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>
  </section>

  <!-- dialogs -->
  <div v-if="modal" class="overlay" @click.self="modal = null">
    <dialog ref="dialogEl" class="modal" open aria-modal="true" tabindex="-1" @keydown.esc="modal = null">
      <!-- a destructive action that needs an audited reason -->
      <template v-if="modal === 'reason'">
        <h2>{{ t("plans.confirmTitle") }}</h2>
        <p>{{ pendingReason?.message }}</p>
        <label>
          {{ t("plans.reason") }}
          <input
            v-model="reasonForm.reason"
            :placeholder="t('plans.reasonPlaceholder')"
            :aria-label="t('plans.reason')"
            :disabled="acting"
          />
        </label>
        <div class="modal-actions">
          <button type="button" class="btn-primary" :disabled="acting || !reasonValid" @click="submitReason">
            {{ t("plans.confirm") }}
          </button>
          <button type="button" :disabled="acting" @click="modal = null">{{ t("plans.cancel") }}</button>
        </div>
      </template>

      <!-- grant premium -->
      <template v-else-if="modal === 'grant'">
        <h2>{{ t("plans.grantTitle", { handle: props.handle }) }}</h2>
        <label>
          {{ t("plans.endDate") }}
          <input v-model="grantForm.endDate" type="date" :aria-label="t('plans.endDate')" :disabled="acting" />
        </label>
        <p class="hint">{{ t("plans.endDateHint") }}</p>
        <label v-if="!grantForm.endDate" class="check">
          <input v-model="grantForm.confirmOpenEnded" type="checkbox" :disabled="acting" />
          {{ t("plans.confirmOpenEnded") }}
        </label>
        <label>
          {{ t("plans.reason") }}
          <input
            v-model="grantForm.reason"
            :placeholder="t('plans.reasonPlaceholder')"
            :aria-label="t('plans.reason')"
            :disabled="acting"
          />
        </label>
        <div class="modal-actions">
          <button type="button" class="btn-primary" :disabled="acting || !grantValid" @click="submitGrant">
            {{ t("plans.confirm") }}
          </button>
          <button type="button" :disabled="acting" @click="modal = null">{{ t("plans.cancel") }}</button>
        </div>
      </template>

      <!-- enrol in campaign -->
      <template v-else-if="modal === 'enrol'">
        <h2>{{ t("plans.enrolTitle", { handle: props.handle }) }}</h2>
        <label>
          {{ t("plans.campaign") }}
          <select v-model="enrolForm.campaignKey" :aria-label="t('plans.campaign')" :disabled="acting">
            <option value="" disabled>{{ t("plans.chooseCampaign") }}</option>
            <option v-for="c in enrollable" :key="c.key" :value="c.key" :disabled="c.taken">
              {{ c.name }} ({{ t(`plans.kind.${c.kind}`) }})<template v-if="c.taken">
                — {{ t("plans.alreadyMember") }}</template
              >
            </option>
          </select>
        </label>
        <label>
          {{ t("plans.reason") }}
          <input
            v-model="enrolForm.reason"
            :placeholder="t('plans.reasonPlaceholder')"
            :aria-label="t('plans.reason')"
            :disabled="acting"
          />
        </label>
        <div class="modal-actions">
          <button type="button" class="btn-primary" :disabled="acting || !enrolValid" @click="submitEnrol">
            {{ t("plans.confirm") }}
          </button>
          <button type="button" :disabled="acting" @click="modal = null">{{ t("plans.cancel") }}</button>
        </div>
      </template>
    </dialog>
  </div>
</template>

<style scoped>
.block {
  margin-top: 1.75rem;
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
.summary {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
  gap: 0.75rem 1.25rem;
  padding: 1rem 1.25rem;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
}
.kv {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}
.kv .k {
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--muted);
}
.kv .v,
.chips {
  display: flex;
  gap: 0.35rem;
  flex-wrap: wrap;
  align-items: center;
}
/* The actions get their own full-width row: squeezed into a 14rem grid cell next to
   the read-only facts, "Grant premium" wrapped onto two lines. */
.kv.actions {
  grid-column: 1 / -1;
  flex-direction: row;
  align-items: center;
  gap: 0.5rem;
  padding-top: 0.25rem;
  border-top: 1px solid var(--border);
}
.mono {
  font-family: var(--mono);
  font-size: 0.85rem;
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
.modal-actions {
  display: flex;
  gap: 0.5rem;
  border-top: 1px solid var(--border);
  padding-top: 0.9rem;
}
</style>
