//
//  ViewController.swift
//  Shared (App)
//
//  Created by fortin guillaume on 14/09/2026.
//

import LinguaSignIn
import WebKit

#if os(iOS)
import UIKit
typealias PlatformViewController = UIViewController
#elseif os(macOS)
import Cocoa
import SafariServices
typealias PlatformViewController = NSViewController
#endif

let extensionBundleIdentifier = "com.cymbra.lingua.Extension"

/// The app's one page (add-lingua-apple-review): the review of the deck the Safari extension keeps
/// once the extension has shared it through the App Group — the extension's own Révision, served
/// from the app's bundle — and, until then, how to turn the extension on.
class ViewController: PlatformViewController, WKNavigationDelegate, WKScriptMessageHandler, WKScriptMessageHandlerWithReply {

    @IBOutlet var webView: WKWebView!

    /// Whether the review is shown, rather than how to turn the extension on.
    private var showingReview = false
    /// A shared state the page said it cannot show (an extension newer than the app): the app
    /// explains the extension instead, until the extension shares another.
    private var unusableState: String?

    override func viewDidLoad() {
        super.viewDidLoad()

        // The scheme the review is served under must be on the configuration the web view is built
        // with: the storyboard's view is replaced by one built here, in its place.
        let configuration = WKWebViewConfiguration()
        configuration.setURLSchemeHandler(AppPageSchemeHandler(), forURLScheme: AppPageResource.scheme)
        configuration.userContentController.add(self, name: "controller")
        configuration.userContentController.addScriptMessageHandler(self, contentWorld: .page, name: "lingua")
        // SPIKE 1.4 (throwaway): the spike page's log lines, printed to the console.
        configuration.userContentController.add(SpikeLogger(), name: "spike")

        // The page's language (localise-lingua-apple-host, D2): the extension's interface language
        // once the extension has run — its native handler keeps it in the App Group — and the
        // bundle's preferred localisation before, among the languages the app declares. Said to the
        // page before it loads; copy.js fills it on DOMContentLoaded, and the review reads it when
        // the shared state names none. The value is one of the closed `SignInLanguage` cases, never
        // a string from outside.
        let language = SignInLanguage.shown(InterfaceLanguageStore.shared()?.current(), in: .main)
        configuration.userContentController.addUserScript(WKUserScript(
            source: "window.linguaLanguage = \"\(language.rawValue)\";",
            injectionTime: .atDocumentStart,
            forMainFrameOnly: true
        ))

        let built = WKWebView(frame: webView.frame, configuration: configuration)
#if os(iOS)
        built.autoresizingMask = [.flexibleWidth, .flexibleHeight]
#elseif os(macOS)
        built.autoresizingMask = [.width, .height]
#endif
        webView.superview?.addSubview(built)
        webView.removeFromSuperview()
        webView = built
        webView.navigationDelegate = self
        if #available(iOS 16.4, macOS 13.3, *) { webView.isInspectable = true }

#if os(iOS)
        NotificationCenter.default.addObserver(
            self, selector: #selector(foreground), name: UIApplication.willEnterForegroundNotification, object: nil
        )
#elseif os(macOS)
        NotificationCenter.default.addObserver(
            self, selector: #selector(foreground), name: NSApplication.didBecomeActiveNotification, object: nil
        )
#endif

        load()
    }

    /// The review when the extension has shared a state the page can show, the explanation otherwise.
    private func load() {
        // SPIKE 1.4 (throwaway): always the spike page, with SPIKE_QUERY (env) as its query.
        if ProcessInfo.processInfo.environment["SPIKE_OFF"] == nil {
            let query = ProcessInfo.processInfo.environment["SPIKE_QUERY"].map { "?" + $0 } ?? ""
            showingReview = true
#if os(macOS)
            sizeWindowForReview()
#endif
            webView.load(URLRequest(url: URL(string: "\(AppPageResource.scheme)://app/spike.html\(query)")!))
            DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { [weak self] in
                let pid = (self?.webView.value(forKey: "_webProcessIdentifier") as? NSNumber)?.intValue ?? -1
                let line = "[spike] INFO web content pid: \(pid) (app pid \(ProcessInfo.processInfo.processIdentifier))"
                NSLog("%@", line)
                print(line)
                fflush(stdout)
            }
            return
        }
        let state = ReviewRelayStore.shared()?.state()
        if let state, state != unusableState {
            showingReview = true
#if os(iOS)
            webView.scrollView.isScrollEnabled = true
#elseif os(macOS)
            sizeWindowForReview()
#endif
            webView.load(URLRequest(url: AppPageResource.page))
        } else {
            showingReview = false
#if os(iOS)
            webView.scrollView.isScrollEnabled = false
#endif
            webView.loadFileURL(Bundle.main.url(forResource: "Main", withExtension: "html")!, allowingReadAccessTo: Bundle.main.resourceURL!)
        }
    }

    /// Back in the foreground, the extension may have shared its state, or saved a newer one.
    @objc private func foreground() {
        if showingReview {
            webView.evaluateJavaScript("window.linguaForeground && window.linguaForeground()")
        } else if let state = ReviewRelayStore.shared()?.state(), state != unusableState {
            load()
        }
    }

#if os(macOS)
    override func viewDidAppear() {
        super.viewDidAppear()
        // The window exists only now: size it for the review a launch already chose.
        if showingReview {
            sizeWindowForReview()
            view.window?.center()
        }
    }

    /// A review wants a column the reader can widen, not the explanation's small fixed window.
    private func sizeWindowForReview() {
        guard let window = view.window else { return }
        window.styleMask.insert(.resizable)
        if window.frame.width < 480 { window.setContentSize(NSSize(width: 520, height: 720)) }
    }
#endif

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        guard !showingReview else { return }
#if os(iOS)
        webView.evaluateJavaScript("show('ios')")
#elseif os(macOS)
        webView.evaluateJavaScript("show('mac')")

        SFSafariExtensionManager.getStateOfSafariExtension(withIdentifier: extensionBundleIdentifier) { (state, error) in
            guard let state = state, error == nil else {
                // Insert code to inform the user that something went wrong.
                return
            }

            DispatchQueue.main.async {
                if #available(macOS 13, *) {
                    webView.evaluateJavaScript("show('mac', \(state.isEnabled), true)")
                } else {
                    webView.evaluateJavaScript("show('mac', \(state.isEnabled), false)")
                }
            }
        }
#endif
    }

    /// The review page asking for the relay (`src/app/bridge.ts`): the shared state, the changes
    /// waiting for the extension, the changes of each answer. Its messages are never logged.
    func userContentController(
        _ userContentController: WKUserContentController,
        didReceive message: WKScriptMessage,
        replyHandler: @escaping (Any?, String?) -> Void
    ) {
        guard message.frameInfo.isMainFrame, message.frameInfo.securityOrigin.protocol == AppPageResource.scheme else {
            replyHandler(nil, "refused")
            return
        }
        let store = ReviewRelayStore.shared()
        if (message.body as? [String: Any])?["type"] as? String == "review.unavailable" {
            unusableState = store?.state()
            replyHandler([String: Any](), nil)
            load()
            return
        }
        replyHandler(AppBridgeMessage.reply(to: message.body, store: store), nil)
    }

    func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
#if os(macOS)
        if (message.body as? String != "open-preferences") {
            return
        }

        SFSafariApplication.showPreferencesForExtension(withIdentifier: extensionBundleIdentifier) { error in
            guard error == nil else {
                // Insert code to inform the user that something went wrong.
                return
            }

            DispatchQueue.main.async {
                NSApp.terminate(self)
            }
        }
#endif
    }

}

/// SPIKE 1.4 (throwaway): prints the spike page's lines.
final class SpikeLogger: NSObject, WKScriptMessageHandler {
    func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
        let line = "[spike] \(message.body)"
        NSLog("%@", line)
        print(line)
        fflush(stdout)
    }
}

/// Serves the review page and its files from the app's bundle, and the packs from the Safari
/// extension the app embeds (`AppPageResource`, add-lingua-apple-review D2).
final class AppPageSchemeHandler: NSObject, WKURLSchemeHandler {
    private let pageRoot = Bundle.main.resourceURL!.appendingPathComponent(AppPageResource.folder, isDirectory: true)
    private let packsRoot: URL? = Bundle.main.builtInPlugInsURL
        .map { $0.appendingPathComponent("Cymbra Lingua Extension.appex") }
        .flatMap { Bundle(url: $0)?.resourceURL }

    func webView(_ webView: WKWebView, start urlSchemeTask: WKURLSchemeTask) {
        guard let url = urlSchemeTask.request.url else {
            urlSchemeTask.didFailWithError(URLError(.badURL))
            return
        }
        guard let file = AppPageResource.file(for: url, pageRoot: pageRoot, packsRoot: packsRoot),
              let data = try? Data(contentsOf: file) else {
            urlSchemeTask.didReceive(HTTPURLResponse(url: url, statusCode: 404, httpVersion: "HTTP/1.1", headerFields: [:])!)
            urlSchemeTask.didFinish()
            return
        }
        let headers = ["Content-Type": AppPageResource.mimeType(of: file), "Content-Length": String(data.count)]
        urlSchemeTask.didReceive(HTTPURLResponse(url: url, statusCode: 200, httpVersion: "HTTP/1.1", headerFields: headers)!)
        urlSchemeTask.didReceive(data)
        urlSchemeTask.didFinish()
    }

    func webView(_ webView: WKWebView, stop urlSchemeTask: WKURLSchemeTask) {}
}
