//
//  SceneDelegate.swift
//  iOS (App)
//
//  Created by fortin guillaume on 14/09/2026.
//

import LinguaSignIn
import SwiftUI
import UIKit

class SceneDelegate: UIResponder, UIWindowSceneDelegate {

    var window: UIWindow?

    func scene(_ scene: UIScene, willConnectTo session: UISceneSession, options connectionOptions: UIScene.ConnectionOptions) {
        guard let _ = (scene as? UIWindowScene) else { return }
        // Launched by the extension's sign-in link: wait for the storyboard's root to be on screen.
        if let url = connectionOptions.urlContexts.first?.url {
            DispatchQueue.main.async { self.open(url) }
        }
    }

    func scene(_ scene: UIScene, openURLContexts URLContexts: Set<UIOpenURLContext>) {
        if let url = URLContexts.first?.url {
            open(url)
        }
    }

    /// Show the sign-in sheet for `cymbra-lingua://signin?provider=…`, replacing one already open.
    private func open(_ url: URL) {
        guard let provider = SignInLink.provider(from: url), let root = window?.rootViewController else { return }
        root.dismiss(animated: false)
        let sheet = UIHostingController(
            rootView: SignInView(requested: provider, google: nil, handoff: IdTokenHandoff.shared()) { [weak root] in
                root?.dismiss(animated: true)
            }
        )
        root.present(sheet, animated: true)
    }

}
