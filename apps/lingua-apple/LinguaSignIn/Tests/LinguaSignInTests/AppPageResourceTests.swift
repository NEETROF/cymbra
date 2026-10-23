import XCTest
@testable import LinguaSignIn

final class AppPageResourceTests: XCTestCase {
    private let page = URL(fileURLWithPath: "/App.app/LinguaApp")
    private let ext = URL(fileURLWithPath: "/App.app/PlugIns/Ext.appex")

    private func file(_ address: String, extensionRoot: URL? = nil) -> String? {
        AppPageResource.file(for: URL(string: address)!, pageRoot: page, extensionRoot: extensionRoot ?? ext)?.path
    }

    func testThePageAndItsFilesComeFromTheAppBundle() {
        XCTAssertEqual(AppPageResource.file(for: AppPageResource.page, pageRoot: page, extensionRoot: ext)?.path, "/App.app/LinguaApp/app.html")
        XCTAssertEqual(file("lingua-app://app/wasm/lingua_wasm_bg.wasm"), "/App.app/LinguaApp/wasm/lingua_wasm_bg.wasm")
    }

    func testThePacksComeFromTheEmbeddedExtension() {
        XCTAssertEqual(file("lingua-app://app/assets/packs/en-fr.lingua"), "/App.app/PlugIns/Ext.appex/assets/packs/en-fr.lingua")
        XCTAssertNil(AppPageResource.file(for: URL(string: "lingua-app://app/assets/packs/en-fr.lingua")!, pageRoot: page, extensionRoot: nil))
        // Deeper than a pack is the page's own folder, never the extension's.
        XCTAssertEqual(file("lingua-app://app/assets/packs/x/y"), "/App.app/LinguaApp/assets/packs/x/y")
    }

    /// ship-lingua-pack-bases-once D1: a pair's pack is its layer and its studied language's base, both
    /// the extension's; the page's engine fetches the two.
    func testTheBasesComeFromTheEmbeddedExtension() {
        XCTAssertEqual(file("lingua-app://app/assets/bases/en.lingua"), "/App.app/PlugIns/Ext.appex/assets/bases/en.lingua")
        XCTAssertNil(AppPageResource.file(for: URL(string: "lingua-app://app/assets/bases/en.lingua")!, pageRoot: page, extensionRoot: nil))
        XCTAssertEqual(file("lingua-app://app/assets/bases/x/y"), "/App.app/LinguaApp/assets/bases/x/y")
        // Another folder of `assets/` stays the page's.
        XCTAssertEqual(file("lingua-app://app/assets/other/en.lingua"), "/App.app/LinguaApp/assets/other/en.lingua")
    }

    /// add-lingua-review-translation-fallback D7: the engine's files are the extension's, served once.
    func testTheEngineFilesComeFromTheEmbeddedExtension() {
        for name in [
            "engine-worker.js", "model-worker.js", "model-manifest.json",
            "engine/bergamot-translator.js", "engine/bergamot-translator.wasm",
        ] {
            XCTAssertEqual(file("lingua-app://app/\(name)"), "/App.app/PlugIns/Ext.appex/\(name)", name)
            XCTAssertNil(
                AppPageResource.file(for: URL(string: "lingua-app://app/\(name)")!, pageRoot: page, extensionRoot: nil),
                "\(name): no extension, no engine"
            )
        }
    }

    /// Anything else of the extension stays out of the page's reach: its background, its manifest,
    /// a deeper path under `engine/`, the page's own files of the same names elsewhere.
    func testNothingElseOfTheExtensionIsServed() {
        for name in ["background.js", "manifest.json", "engine/x/y.js", "engine", "sub/engine-worker.js", "app.js"] {
            XCTAssertEqual(file("lingua-app://app/\(name)"), "/App.app/LinguaApp/\(name)", name)
        }
    }

    func testNothingOutsideItsRootNorAnotherOrigin() {
        XCTAssertNil(file("lingua-app://app/../secret"))
        XCTAssertNil(file("lingua-app://app/./app.html"))
        XCTAssertNil(file("lingua-app://app/engine/../background.js"))
        XCTAssertNil(file("lingua-app://app/"))
        XCTAssertNil(file("lingua-app://other/app.html"))
        XCTAssertNil(file("https://app/app.html"))
    }

    func testEachFileIsServedWithItsType() {
        let type = { (name: String) in AppPageResource.mimeType(of: URL(fileURLWithPath: name)) }
        XCTAssertEqual(type("app.js"), "text/javascript; charset=utf-8")
        XCTAssertEqual(type("e.WASM"), "application/wasm")
        XCTAssertEqual(type("app.html"), "text/html; charset=utf-8")
        XCTAssertEqual(type("app.css"), "text/css; charset=utf-8")
        XCTAssertEqual(type("m.json"), "application/json")
        XCTAssertEqual(type("i.png"), "image/png")
        XCTAssertEqual(type("i.svg"), "image/svg+xml")
        XCTAssertEqual(type("en-fr.lingua"), "application/octet-stream")
    }
}
