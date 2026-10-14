import Foundation

/// Where the app's review page finds its files (add-lingua-apple-review, D2). The app serves the
/// extension's `app` build from its own bundle under `lingua-app://app/…`, so the page's module
/// scripts and WebAssembly load under one origin, never `file://`. What the Safari extension already
/// carries is not copied twice: it is read from the extension the app embeds — the packs, and the
/// translation engine's files (add-lingua-review-translation-fallback, D7): its worker, the model
/// download's worker, the engine's glue and WebAssembly, and the models' catalogue. Served under the
/// page's origin, the workers read and fill the page's IndexedDB.
public enum AppPageResource {
    public static let scheme = "lingua-app"
    /// The page the app opens on.
    public static let page = URL(string: "\(scheme)://app/app.html")!
    /// The folder of the app bundle the `app` build is copied into (the _Copy Lingua app page_ phase).
    public static let folder = "LinguaApp"
    static let packs = ["assets", "packs"]
    /// The engine's files at the extension's root, as the Safari build writes them.
    static let engineFiles: Set<String> = ["engine-worker.js", "model-worker.js", "model-manifest.json"]
    /// The engine's folder in the extension: its glue and its WebAssembly, one level deep.
    static let engineFolder = "engine"

    /// The file `url` names: under `extensionRoot` for a pack or an engine file, under `pageRoot`
    /// otherwise; nil for another scheme or host, an empty path, or a path that would climb out of its
    /// root.
    public static func file(for url: URL, pageRoot: URL, extensionRoot: URL?) -> URL? {
        guard url.scheme == scheme, url.host == "app" else { return nil }
        let parts = url.path.split(separator: "/").map(String.init)
        guard !parts.isEmpty, !parts.contains(where: { $0 == ".." || $0 == "." }) else { return nil }
        guard let root = fromExtension(parts) ? extensionRoot : pageRoot else { return nil }
        return parts.reduce(root) { $0.appendingPathComponent($1) }
    }

    /// Whether `parts` name a file the extension carries: a pack, or one of the engine's files.
    static func fromExtension(_ parts: [String]) -> Bool {
        if parts.count == packs.count + 1 && Array(parts.prefix(packs.count)) == packs { return true }
        if parts.count == 1 && engineFiles.contains(parts[0]) { return true }
        return parts.count == 2 && parts[0] == engineFolder
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
