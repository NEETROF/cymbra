import XCTest
@testable import LinguaSignIn

final class AppPageResourceTests: XCTestCase {
    private let page = URL(fileURLWithPath: "/App.app/LinguaApp")
    private let packs = URL(fileURLWithPath: "/App.app/PlugIns/Ext.appex")

    private func file(_ address: String, packsRoot: URL? = nil) -> String? {
        AppPageResource.file(for: URL(string: address)!, pageRoot: page, packsRoot: packsRoot ?? packs)?.path
    }

    func testThePageAndItsFilesComeFromTheAppBundle() {
        XCTAssertEqual(AppPageResource.file(for: AppPageResource.page, pageRoot: page, packsRoot: packs)?.path, "/App.app/LinguaApp/app.html")
        XCTAssertEqual(file("lingua-app://app/wasm/lingua_wasm_bg.wasm"), "/App.app/LinguaApp/wasm/lingua_wasm_bg.wasm")
    }

    func testThePacksComeFromTheEmbeddedExtension() {
        XCTAssertEqual(file("lingua-app://app/assets/packs/en-fr.lingua"), "/App.app/PlugIns/Ext.appex/assets/packs/en-fr.lingua")
        XCTAssertNil(AppPageResource.file(for: URL(string: "lingua-app://app/assets/packs/en-fr.lingua")!, pageRoot: page, packsRoot: nil))
        // Deeper than a pack is the page's own folder, never the extension's.
        XCTAssertEqual(file("lingua-app://app/assets/packs/x/y"), "/App.app/LinguaApp/assets/packs/x/y")
    }

    func testNothingOutsideItsRootNorAnotherOrigin() {
        XCTAssertNil(file("lingua-app://app/../secret"))
        XCTAssertNil(file("lingua-app://app/./app.html"))
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
