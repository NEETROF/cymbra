import LinguaSignIn
import SafariServices
import os.log

/// The extension's native side (add-lingua-connected-clients, design D6): it hands over the
/// id_token the host app's sign-in sheet left in the App Group, once. It has no UI and never
/// touches the network or the Cymbra session, which stay in the extension.
class SafariWebExtensionHandler: NSObject, NSExtensionRequestHandling {

    func beginRequest(with context: NSExtensionContext) {
        let request = context.inputItems.first as? NSExtensionItem
        let message = request?.userInfo?[SFExtensionMessageKey]

        // Never log the message or the reply: the reply can carry an id_token.
        let reply = NativeMessage.reply(
            to: message,
            handoff: IdTokenHandoff.shared(),
            googleClientId: Bundle.main.object(forInfoDictionaryKey: "LinguaGoogleClientId") as? String
        )
        if reply["error"] != nil {
            os_log(.error, "Refused an unknown native message from the extension")
        }

        let response = NSExtensionItem()
        response.userInfo = [SFExtensionMessageKey: reply]
        context.completeRequest(returningItems: [response], completionHandler: nil)
    }

}
