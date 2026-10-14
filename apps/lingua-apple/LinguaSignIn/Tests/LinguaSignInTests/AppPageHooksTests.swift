import XCTest
@testable import LinguaSignIn

final class AppPageHooksTests: XCTestCase {
    /// add-lingua-app-deck-sync D5: the background hook is awaited, and a page without it answers.
    func testTheBackgroundHookIsAwaitedAndOptional() {
        let script = AppPageHooks.background
        XCTAssertTrue(script.contains("await window.linguaBackground()"))
        XCTAssertTrue(script.hasPrefix("if (window.linguaBackground)"))
        XCTAssertTrue(script.hasSuffix("return true;"))
    }
}
