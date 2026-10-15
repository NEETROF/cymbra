import Foundation

/// What the app tells its review page about its own life (add-lingua-app-deck-sync D5).
public enum AppPageHooks {
    /// Going to the background: the body of a `callAsyncJavaScript`, which the app awaits within the
    /// time the system gives (`beginBackgroundTask` on iOS). The page pushes the reviews its
    /// synchronisation still holds; a page without the hook answers at once.
    public static let background = "if (window.linguaBackground) { await window.linguaBackground(); } return true;"
}
