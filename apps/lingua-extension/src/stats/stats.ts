import { createLinguaPort } from "../analyzer/create-port.ts";
import { type AsyncStorageArea, hydrateEngine } from "../state/storage.ts";
import { mountStats } from "./view.ts";

// Standalone stats tab. The primary surface is now the side panel (same view via
// mountStats); this tab remains reachable and hydrates its own engine from the shared
// backup. Excluded from coverage (DOM wiring; the model + chart are unit-tested).

const area: AsyncStorageArea = {
  get: (keys) => chrome.storage.local.get(keys),
  set: (items) => chrome.storage.local.set(items),
};

async function main(): Promise<void> {
  const root = document.getElementById("stats-root");
  if (!root) return;
  const port = createLinguaPort();
  await hydrateEngine(port, area);
  await mountStats(root, port, area);
}

void main();
