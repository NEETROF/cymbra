import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import CatalogTable from "@/components/CatalogTable.vue";
import FiltersBar from "@/components/FiltersBar.vue";
import TablePager from "@/components/TablePager.vue";
import { i18n } from "@/i18n";

// The components use vue-i18n (useI18n), so the plugin must be installed. Default
// locale is `en`, and the English strings match the selectors below.
const withI18n = { plugins: [i18n] };

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
    expect(w.findAll(".badge.pending")).toHaveLength(2);
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
