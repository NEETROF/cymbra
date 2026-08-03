import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import CatalogTable from "@/components/CatalogTable.vue";
import FiltersBar from "@/components/FiltersBar.vue";
import StatCards, { type StatItem } from "@/components/StatCards.vue";
import TablePager from "@/components/TablePager.vue";
import { i18n } from "@/i18n";

// The components use vue-i18n (useI18n), so the plugin must be installed. Default
// locale is `en`, and the English strings match the selectors below.
const withI18n = { plugins: [i18n] };

describe("StatCards", () => {
  const items: StatItem[] = [
    { id: "total", label: "TOTAL", value: "155", accent: "accent", icon: "M4 7h16" },
    { id: "accepted", label: "ACCEPTED", value: "32", accent: "green", icon: "M20 6" },
    { id: "pending", label: "PENDING", value: "118", accent: "amber", icon: "M12 7" },
  ];

  it("renders a card per item with its label, value and accent", () => {
    const w = mount(StatCards, { props: { items } });
    expect(w.findAll(".stat").length).toBe(3);
    const accepted = w.get('[data-testid="stat-accepted"]');
    expect(accepted.text()).toContain("ACCEPTED");
    expect(accepted.text()).toContain("32");
    expect(accepted.classes()).toContain("green");
  });
});

describe("CatalogTable", () => {
  const hits = [
    { id: "a", title: "Alpha", composer: "Bach", level: "beginner", source: "pdmx", noteCount: 100, tempoBpm: 90 },
    { id: "b", title: "Beta", composer: "Satie", level: "advanced", source: "pdmx", noteCount: 500, tempoBpm: 60 },
  ];

  it("renders a row per hit with the active status badge", () => {
    const w = mount(CatalogTable, { global: withI18n, props: { hits: hits as never, status: "pending", sort: [] } });
    const rows = w.findAll("tbody tr");
    expect(rows).toHaveLength(2);
    expect(w.text()).toContain("Alpha");
    expect(w.findAll(".tag.pending")).toHaveLength(2);
  });

  it("emits a sort field when a sortable header is clicked", async () => {
    const w = mount(CatalogTable, { global: withI18n, props: { hits: hits as never, status: "pending", sort: [] } });
    await w.get('[aria-label="sort by Notes"]').trigger("click");
    expect(w.emitted("sort")?.[0]).toEqual(["note_count"]);
  });

  it("emits the row id when a row is clicked", async () => {
    const w = mount(CatalogTable, { global: withI18n, props: { hits: hits as never, status: "pending", sort: [] } });
    await w.findAll("tbody tr")[1].trigger("click");
    expect(w.emitted("select")?.[0]).toEqual(["b"]);
  });

  it("shows an empty state when there are no hits", () => {
    const w = mount(CatalogTable, { global: withI18n, props: { hits: [] as never, status: "accepted", sort: [] } });
    expect(w.text()).toContain("No scores.");
  });

  it("hides the download control when the operator cannot download (non-moderator)", () => {
    const w = mount(CatalogTable, { global: withI18n, props: { hits: hits as never, status: "pending", sort: [] } });
    expect(w.find(".dl-btn").exists()).toBe(false);
    // No Actions header either.
    expect(w.text()).not.toContain("Actions");
  });

  it("renders a per-row download control for an authorized operator and emits the hit", async () => {
    const w = mount(CatalogTable, {
      global: withI18n,
      props: { hits: hits as never, status: "pending", sort: [], canDownload: true },
    });
    const buttons = w.findAll(".dl-btn");
    expect(buttons).toHaveLength(2);
    await buttons[1].trigger("click");
    const emitted = w.emitted("download");
    expect(emitted).toBeTruthy();
    expect((emitted![0][0] as { id: string }).id).toBe("b");
    // Clicking the download must NOT also select the row (the button stops it).
    expect(w.emitted("select")).toBeFalsy();
  });

  it("reflects a row's download loading and error state without touching other rows", () => {
    const downloads = {
      a: { status: "loading" as const },
      b: { status: "error" as const, error: "Not available yet. Try again later." },
    };
    const w = mount(CatalogTable, {
      global: withI18n,
      props: { hits: hits as never, status: "pending", sort: [], canDownload: true, downloads },
    });
    const buttons = w.findAll(".dl-btn");
    // Row A is loading → its button is disabled; row B is not loading → enabled.
    expect(buttons[0].attributes("disabled")).toBeDefined();
    expect(buttons[1].attributes("disabled")).toBeUndefined();
    // Row B's error message is shown (localized), row A shows none.
    const errors = w.findAll(".dl-error");
    expect(errors).toHaveLength(1);
    expect(errors[0].text()).toContain("Not available yet");
  });

  it("shows each row's own status and flags community re-reviews (mixed queue)", () => {
    // A mixed review queue: a pending score and an accepted score flagged for
    // re-review. Each row shows its OWN status, and the flagged one gets a badge.
    const mixed = [
      { id: "p", title: "Pending One", source: "pdmx", moderationStatus: "pending", needsReview: false },
      { id: "f", title: "Flagged One", source: "pdmx", moderationStatus: "accepted", needsReview: true },
    ];
    const w = mount(CatalogTable, { global: withI18n, props: { hits: mixed as never, status: "pending", sort: [] } });
    // Per-row status, not the single filter status.
    expect(w.findAll(".tag.pending")).toHaveLength(1);
    expect(w.findAll(".tag.accepted")).toHaveLength(1);
    // Only the flagged accepted row carries the re-review marker.
    const reviewBadges = w.findAll(".tag.review");
    expect(reviewBadges).toHaveLength(1);
    expect(reviewBadges[0].text()).toBe("Re-review");
  });
});

describe("FiltersBar", () => {
  it("emits the selected moderation status (BO-only filter)", async () => {
    const w = mount(FiltersBar, { global: withI18n, props: { status: "pending" } });
    await w.get('[aria-label="moderation status"]').setValue("accepted");
    const events = w.emitted("change");
    expect(events).toBeTruthy();
    const last = events!.at(-1)![0] as { moderationStatus: string };
    expect(last.moderationStatus).toBe("accepted");
  });

  it("emits the text query change", async () => {
    const w = mount(FiltersBar, { global: withI18n, props: { status: "pending" } });
    await w.get('[aria-label="search"]').setValue("chopin");
    const last = w.emitted("change")!.at(-1)![0] as { query: string };
    expect(last.query).toBe("chopin");
  });
});

describe("TablePager", () => {
  const buttons = (w: ReturnType<typeof mount>) => w.findAll(".pager button");

  it("is hidden when a single page holds every row", () => {
    const w = mount(TablePager, { global: withI18n, props: { offset: 0, limit: 50, total: 42 } });
    expect(w.find(".pager").exists()).toBe(false);
  });

  it("renders the current window and disables Previous on the first page", () => {
    const w = mount(TablePager, { global: withI18n, props: { offset: 0, limit: 50, total: 130 } });
    expect(w.text()).toContain("1–50 of 130");
    const [prev, next] = buttons(w);
    expect(prev.attributes("disabled")).toBeDefined();
    expect(next.attributes("disabled")).toBeUndefined();
  });

  it("emits the next offset when Next is clicked", async () => {
    const w = mount(TablePager, { global: withI18n, props: { offset: 0, limit: 50, total: 130 } });
    await buttons(w)[1].trigger("click");
    expect(w.emitted("page")?.[0]).toEqual([50]);
  });

  it("emits the previous offset and disables Next on the last page", async () => {
    const w = mount(TablePager, { global: withI18n, props: { offset: 100, limit: 50, total: 130 } });
    expect(w.text()).toContain("101–130 of 130");
    const [prev, next] = buttons(w);
    expect(next.attributes("disabled")).toBeDefined();
    await prev.trigger("click");
    expect(w.emitted("page")?.[0]).toEqual([50]);
  });
});
