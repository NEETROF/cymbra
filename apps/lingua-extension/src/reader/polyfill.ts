// foliate-js groups the package's metadata with `Object.groupBy` and `Map.groupBy`, which
// arrived in Chrome 117, Firefox 119 and Safari 17.4 — after the oldest browsers the extension
// supports (Chrome 116, and Safari 17.2 on the iOS the Apple app targets). The reader page
// installs them where missing, before any book is opened; nothing else in the extension uses them.

type KeyOf<T, K> = (item: T, index: number) => K;

export function groupByObject<T, K extends PropertyKey>(
  items: Iterable<T>,
  keyOf: KeyOf<T, K>,
): Partial<Record<K, T[]>> {
  const out = Object.create(null) as Partial<Record<K, T[]>>;
  let i = 0;
  for (const item of items) {
    const key = keyOf(item, i++);
    (out[key] ??= []).push(item);
  }
  return out;
}

export function groupByMap<T, K>(items: Iterable<T>, keyOf: KeyOf<T, K>): Map<K, T[]> {
  const out = new Map<K, T[]>();
  let i = 0;
  for (const item of items) {
    const key = keyOf(item, i++);
    const list = out.get(key);
    if (list) list.push(item);
    else out.set(key, [item]);
  }
  return out;
}

/** Install the two where the browser lacks them. */
export function installGroupBy(
  objectCtor: { groupBy?: unknown } = Object as { groupBy?: unknown },
  mapCtor: { groupBy?: unknown } = Map as { groupBy?: unknown },
): void {
  if (typeof objectCtor.groupBy !== "function") objectCtor.groupBy = groupByObject;
  if (typeof mapCtor.groupBy !== "function") mapCtor.groupBy = groupByMap;
}
