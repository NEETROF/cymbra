//
//  AppDelegate.swift
//  macOS (App)
//
//  Created by fortin guillaume on 14/09/2026.
//

import Cocoa
import LinguaSignIn
import SwiftUI

@main
class AppDelegate: NSObject, NSApplicationDelegate {

    private var signInWindow: NSWindow?

    func applicationDidFinishLaunching(_ notification: Notification) {
        // Override point for customization after application launch.
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        return true
    }

    /// The extension's sign-in link, `cymbra-lingua://signin?provider=…`.
    func application(_ application: NSApplication, open urls: [URL]) {
        guard let provider = urls.lazy.compactMap(SignInLink.provider(from:)).first else { return }
        signInWindow?.close()
        let window = NSWindow(
            contentViewController: NSHostingController(
                rootView: SignInView(requested: provider, google: nil, handoff: IdTokenHandoff.shared()) { [weak self] in
                    self?.signInWindow?.close()
                }
            )
        )
        window.title = "Cymbra Lingua"
        window.isReleasedWhenClosed = false
        window.setContentSize(NSSize(width: 460, height: 340))
        window.center()
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        signInWindow = window
    }

}
