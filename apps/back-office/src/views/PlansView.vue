<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import { type CampaignKind, isRevocableSource, membersCsv, usePlansStore } from "@/stores/plans";
import { useRolesStore } from "@/stores/roles";
import { useToastsStore } from "@/stores/toasts";
import { saveTextAsFile } from "@/lib/download";
import type { Async } from "@/lib/async";
import type { CampaignMsg, EntitlementRowMsg, MembershipMsg } from "@/gen/plans_pb";
import AppTag from "@/components/AppTag.vue";
import IdBadge from "@/components/IdBadge.vue";

// Music-admin plan console (change: add-premium-subscription): account lookup →
// effective plan + entitlement rows + memberships, nominative grant/enrol/revoke with a
// reason, campaign lifecycle, code minting (clear text shown ONCE) and member export.
// All API work lives in the plans store; this view only matches on the Async unions
// and toasts each mutation's outcome (localized — never a raw error).
const store = usePlansStore();
const roles = useRolesStore();
const toasts = useToastsStore();
const { t } = useI18n();

// ---- (a) account lookup ----
const query = ref("");
const lookupVm = computed(() =>
  match(store.lookupResult)
    .with({ status: "idle" }, () => ({ loading: false, error: null as string | null, data: null }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, data: null }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, data: null }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, data }))
    .exhaustive(),
);
const acting = computed(() => store.op.status === "loading");
/** The looked-up account as the RPCs address it (id once resolved, else the typed handle). */
const target = computed(() =>
  lookupVm.value.data?.userId ? { userId: lookupVm.value.data.userId } : { handle: query.value.trim() },
);
const targetLabel = computed(() => store.lastLookup ?? "");

function lookup() {
  const q = query.value.trim();
  if (q) void store.lookup(q);
}
function clear() {
  query.value = "";
  store.clearLookup();
}

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

/** Ask a free-text reason (audited); null = cancelled. */
function askReason(message: string): string | null {
  const r = globalThis.prompt ? globalThis.prompt(message, "") : "";
  return r === null ? null : r;
}
function ask(message: string): boolean {
  return !globalThis.confirm || globalThis.confirm(message);
}
/** Toast a mutation outcome (localized), returning whether it succeeded. */
function report(outcome: Async<void>, okMsg: string): boolean {
  if (outcome.status === "error") toasts.error(outcome.error);
  else toasts.success(okMsg);
  return outcome.status === "success";
}

async function revokeRow(r: EntitlementRowMsg) {
  const reason = askReason(t("plans.revokeRowConfirm", { source: r.source }));
  if (reason === null) return;
  report(await store.revokeEntitlement(r.id, reason), t("plans.revokedRow"));
}
async function revokeMembership(m: MembershipMsg) {
  const reason = askReason(t("plans.revokeMembershipConfirm", { campaign: m.campaignKey }));
  if (reason === null) return;
  report(
    await store.revokeMembership({ target: { userId: m.userId }, campaignKey: m.campaignKey, reason }),
    t("plans.membershipRevoked"),
  );
}

// ---- dialogs (grant / enrol / mint / minted) ----
type Modal = "grant" | "enrol" | "mint" | "minted" | "create" | null;
const modal = ref<Modal>(null);
const grantForm = ref({ endDate: "", confirmOpenEnded: false, reason: "" });
const enrolForm = ref({ campaignKey: "", reason: "" });
const mintForm = ref({ campaignKey: "", count: 10, hint: "" });
const createForm = ref<{ key: string; name: string; kind: CampaignKind; durationDays: number }>({
  key: "",
  name: "",
  kind: "premium_trial",
  durationDays: 90,
});

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

function openEnrol() {
  enrolForm.value = { campaignKey: enrollable.value[0]?.key ?? "", reason: "" };
  modal.value = "enrol";
}
const enrolValid = computed(() => enrolForm.value.campaignKey !== "" && enrolForm.value.reason.trim() !== "");
async function submitEnrol() {
  const f = enrolForm.value;
  const ok = report(
    await store.enrolHandle({ target: target.value, campaignKey: f.campaignKey, reason: f.reason.trim() }),
    t("plans.enrolled_ok"),
  );
  if (ok) modal.value = null;
}

// ---- (b) campaigns ----
const campaignsVm = computed(() =>
  match(store.campaigns)
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null as string | null, rows: data }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, rows: [] as CampaignMsg[] }))
    .otherwise(() => ({ loading: true, error: null as string | null, rows: [] as CampaignMsg[] })),
);
/** Campaigns an admin may still enrol someone in (open AND accepting enrolment). */
const enrollable = computed(() => store.openCampaigns.filter((c) => c.acceptsEnrolment));

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
async function closeEnrolment(c: CampaignMsg) {
  if (!ask(t("plans.closeEnrolmentConfirm", { key: c.key }))) return;
  report(await store.closeEnrollment(c.key), t("plans.enrolmentClosed"));
}
async function closeCampaign(c: CampaignMsg) {
  if (!ask(t("plans.closeCampaignConfirm", { key: c.key }))) return;
  report(await store.closeCampaign(c.key), t("plans.campaignClosed"));
}
async function revokeCodes(c: CampaignMsg) {
  if (!ask(t("plans.revokeCodesConfirm", { key: c.key }))) return;
  report(await store.revokeCodes({ campaignKey: c.key }), t("plans.codesRevoked"));
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
/** Best-effort handle for a member: the directory page (if loaded) or the last lookup. */
function handleFor(userId: string): string | undefined {
  const dir = roles.directory.status === "success" ? roles.directory.data.accounts : [];
  const hit = dir.find((a) => a.userId === userId)?.handle;
  if (hit) return hit;
  const looked = lookupVm.value.data;
  if (looked?.userId === userId && store.lastLookup && !/^[0-9a-f-]{36}$/i.test(store.lastLookup)) {
    return store.lastLookup.replace(/^@/, "");
  }
  return undefined;
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
  <h1 class="page-title">{{ t("plans.title") }}</h1>
  <p class="muted">{{ t("plans.intro") }}</p>

  <!-- (a) account lookup -->
  <section class="block">
    <h2>{{ t("plans.lookupTitle") }}</h2>
    <div class="filter">
      <input
        v-model="query"
        type="search"
        :placeholder="t('plans.lookupPlaceholder')"
        :aria-label="t('plans.lookupPlaceholder')"
        @keyup.enter="lookup"
      />
      <button type="button" class="btn-primary" :disabled="lookupVm.loading" @click="lookup">
        {{ t("plans.lookup") }}
      </button>
      <button v-if="lookupVm.data" type="button" @click="clear">{{ t("plans.clear") }}</button>
    </div>
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
          <button type="button" :disabled="acting || enrollable.length === 0" @click="openEnrol">
            {{ t("plans.enrol") }}
          </button>
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
              <td>{{ m.revokedAt ? t("plans.revoked") : fmt(m.endsAt) }}</td>
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

  <!-- (b) campaigns -->
  <section class="block">
    <div class="page-head">
      <h2>{{ t("plans.campaigns") }}</h2>
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

  <!-- (c) members of the selected campaign -->
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
          <tr v-for="m in membersVm.rows" :key="m.userId" :class="{ inactive: !!m.revokedAt }">
            <td>
              <span v-if="handleFor(m.userId)" class="handle">{{ handleFor(m.userId) }}</span>
              <IdBadge v-else :id="m.userId" />
            </td>
            <td>{{ fmt(m.enrolledAt) }}</td>
            <td>{{ m.revokedAt ? t("plans.revoked") : fmt(m.endsAt) }}</td>
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
    <dialog class="modal" open aria-modal="true" @keydown.esc="modal = null">
      <!-- grant premium -->
      <template v-if="modal === 'grant'">
        <h2>{{ t("plans.grantTitle", { handle: targetLabel }) }}</h2>
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
        <h2>{{ t("plans.enrolTitle", { handle: targetLabel }) }}</h2>
        <label>
          {{ t("plans.campaign") }}
          <select v-model="enrolForm.campaignKey" :aria-label="t('plans.campaign')" :disabled="acting">
            <option value="" disabled>{{ t("plans.chooseCampaign") }}</option>
            <option v-for="c in enrollable" :key="c.key" :value="c.key">
              {{ c.name }} ({{ t(`plans.kind.${c.kind}`) }})
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
.block h2 {
  font-size: 1.15rem;
  margin: 0 0 0.75rem;
}
.block h3 {
  font-size: 0.95rem;
  margin: 1.25rem 0 0.5rem;
  color: var(--muted);
}
.filter {
  display: flex;
  gap: 0.5rem;
  margin: 0.75rem 0;
  flex-wrap: wrap;
}
.filter input[type="search"] {
  min-width: 18rem;
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
.kv.actions {
  flex-direction: row;
  align-items: flex-end;
  gap: 0.5rem;
}
.mono {
  font-family: var(--mono);
  font-size: 0.85rem;
}
.handle {
  font-weight: 600;
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
