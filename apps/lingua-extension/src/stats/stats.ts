import { createLinguaPort } from "../analyzer/create-port.ts";
import { type AsyncStorageArea, hydrateEngine } from "../state/storage.ts";
import { messagedArea } from "../state/store.ts";
import { mountStats } from "./view.ts";

// Standalone stats tab. The primary surface is now the side panel (same view via
// mountStats); this tab remains reachable and hydrates its own engine from the shared
// backup. Excluded from coverage (DOM wiring; the model + chart are unit-tested).

/** The reader's data, owned by the background — the same area every other surface asks. */
const store: AsyncStorageArea = messagedArea();

async function main(): Promise<void> {
  const root = document.getElementById("stats-root");
  if (!root) return;
  const port = createLinguaPort();
  await hydrateEngine(port, store);
  await mountStats(root, port, store);
}

void main();
