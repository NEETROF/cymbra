// UUIDv7 — a time-ordered UUID (RFC 9562): a 48-bit big-endian millisecond
// timestamp, then random bits, with the version (7) and variant nibbles set. Used
// to mint catalog SoundFont ids on create (the operator no longer types an id).
// Time-ordered so new fonts sort after older ones, matching the backend's
// `uuid::Uuid::now_v7()` for the private library.
export function uuidv7(): string {
  const ts = Date.now();
  const bytes = new Uint8Array(16);
  // 48-bit big-endian timestamp (ms).
  bytes[0] = Math.floor(ts / 2 ** 40) & 0xff;
  bytes[1] = Math.floor(ts / 2 ** 32) & 0xff;
  bytes[2] = Math.floor(ts / 2 ** 24) & 0xff;
  bytes[3] = Math.floor(ts / 2 ** 16) & 0xff;
  bytes[4] = Math.floor(ts / 2 ** 8) & 0xff;
  bytes[5] = ts & 0xff;
  // Random for the remaining 10 bytes.
  crypto.getRandomValues(bytes.subarray(6));
  bytes[6] = (bytes[6] & 0x0f) | 0x70; // version 7
  bytes[8] = (bytes[8] & 0x3f) | 0x80; // variant (RFC 4122)
  const hex = Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}
