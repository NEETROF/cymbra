import Foundation

/// Where the app's review page finds its files (add-lingua-apple-review, D2). The app serves the
/// extension's `app` build from its own bundle under `lingua-app://app/…`, so the page's module
/// scripts and WebAssembly load under one origin, never `file://`. The packs are not copied twice:
/// they are read from the Safari extension the app embeds, the same files the extension reads.
public enum AppPageResource {
    public static let scheme = "lingua-app"
    /// The page the app opens on.
    public static let page = URL(string: "\(scheme)://app/app.html")!
    /// The folder of the app bundle the `app` build is copied into (the _Copy Lingua app page_ phase).
    public static let folder = "LinguaApp"
    static let packs = ["assets", "packs"]

    /// The file `url` names: under `packsRoot` for a pack, under `pageRoot` otherwise; nil for
    /// another scheme or host, an empty path, or a path that would climb out of its root.
    public static func file(for url: URL, pageRoot: URL, packsRoot: URL?) -> URL? {
        guard url.scheme == scheme, url.host == "app" else { return nil }
        let parts = url.path.split(separator: "/").map(String.init)
        guard !parts.isEmpty, !parts.contains(where: { $0 == ".." || $0 == "." }) else { return nil }
        let isPack = parts.count == packs.count + 1 && Array(parts.prefix(packs.count)) == packs
        guard let root = isPack ? packsRoot : pageRoot else { return nil }
        return parts.reduce(root) { $0.appendingPathComponent($1) }
    }

    /// The Content-Type a file is served with: module scripts must be JavaScript to run, and the
    /// engine compiles fastest from a response typed as WebAssembly.
    public static func mimeType(of file: URL) -> String {
        switch file.pathExtension.lowercased() {
        case "html": return "text/html; charset=utf-8"
        case "js": return "text/javascript; charset=utf-8"
        case "css": return "text/css; charset=utf-8"
        case "json": return "application/json"
        case "wasm": return "application/wasm"
        case "png": return "image/png"
        case "svg": return "image/svg+xml"
        default: return "application/octet-stream"
        }
    }
}
