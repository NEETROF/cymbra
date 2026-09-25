import { describe, expect, it } from "vitest";
import { groupByMap, groupByObject, installGroupBy } from "@/reader/polyfill.ts";

// foliate-js needs Object.groupBy and Map.groupBy, which the oldest supported browsers lack.

describe("groupBy for the reader page", () => {
  it("groups into an object, as Object.groupBy does", () => {
    const grouped = groupByObject(["apple", "avocado", "banana"], (w) => w[0]);
    expect({ ...grouped }).toEqual({ a: ["apple", "avocado"], b: ["banana"] });
    expect(Object.getPrototypeOf(grouped)).toBeNull();
  });

  it("groups into a map, as Map.groupBy does, keys of any kind", () => {
    const a = { id: 1 };
    const grouped = groupByMap([a, { id: 2 }, a], (x) => x);
    expect(grouped.get(a)).toHaveLength(2);
    expect(groupByMap([1, 2, 3], (_, i) => i % 2).get(0)).toEqual([1, 3]);
  });

  it("installs only where missing", () => {
    const bareObject: { groupBy?: unknown } = {};
    const bareMap: { groupBy?: unknown } = {};
    installGroupBy(bareObject, bareMap);
    expect(bareObject.groupBy).toBe(groupByObject);
    expect(bareMap.groupBy).toBe(groupByMap);
    const native = () => ({});
    const withIt: { groupBy?: unknown } = { groupBy: native };
    installGroupBy(withIt, withIt);
    expect(withIt.groupBy).toBe(native);
  });
});
