import Foundation

/// The relay between the Safari extension and the app's review (add-lingua-apple-review, D3), kept
/// in the App Group they share: the extension's state, written by its native handler after each
/// save, and the changes a review in the app made, written by the app after every answer and taken
/// by the extension. Both are strings this store never reads — the extension's own JSON — kept in
/// files rather than in the group's defaults, which load whole into every process that opens them.
/// Nothing here leaves the device.
public final class ReviewRelayStore {
    static let stateFile = "review-state.json"
    static let opsFile = "review-ops.json"

    private let directory: URL
    private let now: () -> Date

    public init(directory: URL, now: @escaping () -> Date = { Date() }) {
        self.directory = directory
        self.now = now
    }

    /// The App Group's relay, or nil when the App Group is not available to this process.
    public static func shared() -> ReviewRelayStore? {
        FileManager.default
            .containerURL(forSecurityApplicationGroupIdentifier: IdTokenHandoff.appGroup)
            .map { ReviewRelayStore(directory: $0.appendingPathComponent("Review", isDirectory: true)) }
    }

    /// Keep the extension's state, replacing the one kept before.
    public func saveState(_ state: String) {
        write(Data(state.utf8), to: Self.stateFile)
    }

    /// The extension's state, or nil before the extension has shared one.
    public func state() -> String? {
        guard let data = try? Data(contentsOf: url(Self.stateFile)), !data.isEmpty else { return nil }
        return String(data: data, encoding: .utf8)
    }

    /// Replace the changes waiting for the extension; returns their stamp, always later than the
    /// one replaced, so a clear of what was read before never drops what is written after.
    @discardableResult
    public func writeOps(_ ops: String) -> Int64 {
        let millis = Int64((now().timeIntervalSince1970 * 1000).rounded(.down))
        let stamp = max(millis, (pendingOps()?.writtenAt ?? 0) + 1)
        write((try? JSONEncoder().encode(Pending(ops: ops, writtenAt: stamp))) ?? Data(), to: Self.opsFile)
        return stamp
    }

    /// The changes waiting for the extension and their stamp; nil when none wait or what is kept
    /// cannot be read.
    public func pendingOps() -> (ops: String, writtenAt: Int64)? {
        guard let data = try? Data(contentsOf: url(Self.opsFile)),
              let pending = try? JSONDecoder().decode(Pending.self, from: data) else { return nil }
        return (pending.ops, pending.writtenAt)
    }

    /// Drop the changes stamped `writtenAt` — the ones the extension read and saved — and nothing
    /// written since. Unreadable changes are dropped too: nobody could ever read them.
    public func clearOps(writtenAt: Int64) {
        guard FileManager.default.fileExists(atPath: url(Self.opsFile).path) else { return }
        if let pending = pendingOps(), pending.writtenAt != writtenAt { return }
        try? FileManager.default.removeItem(at: url(Self.opsFile))
    }

    private func url(_ file: String) -> URL {
        directory.appendingPathComponent(file)
    }

    private func write(_ data: Data, to file: String) {
        try? FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        try? data.write(to: url(file), options: .atomic)
    }

    private struct Pending: Codable {
        let ops: String
        let writtenAt: Int64
    }
}

/// The app's page asking for the relay (`window.webkit.messageHandlers.lingua`, answered by
/// `ViewController`). Kept here so the protocol is unit-tested; the page's side is
/// `apps/lingua-extension/src/app/bridge.ts`.
public enum AppBridgeMessage {
    /// - `review.state` → `{state}`, or `{}` before the extension has shared one;
    /// - `review.pendingOps` → `{ops, writtenAt}` for changes the extension has not taken, or `{}`;
    /// - `review.writeOps` `{ops}` → `{writtenAt}`, the changes kept for the extension;
    /// - anything else → `{error: "unknownMessage"}`.
    public static func reply(to message: Any?, store: ReviewRelayStore?) -> [String: Any] {
        let body = message as? [String: Any]
        switch body?["type"] as? String {
        case "review.state":
            guard let state = store?.state() else { return [:] }
            return ["state": state]
        case "review.pendingOps":
            guard let pending = store?.pendingOps() else { return [:] }
            return ["ops": pending.ops, "writtenAt": pending.writtenAt]
        case "review.writeOps":
            guard let ops = body?["ops"] as? String, let store else { return ["error": "badOps"] }
            return ["writtenAt": store.writeOps(ops)]
        default:
            return ["error": "unknownMessage"]
        }
    }
}
