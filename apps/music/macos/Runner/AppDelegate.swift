// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import Cocoa
import CoreMIDI
import FirebaseCore
import FirebaseMessaging
import FlutterMacOS

@main
class AppDelegate: FlutterAppDelegate {
  // CoreMIDI "refresher" client. Created on the main thread so that the
  // process handles MIDI configuration change notifications (hot
  // plug/unplug). Without it, enumeration on the Rust side does not see
  // devices connected AFTER startup: CoreMIDI only delivers these
  // notifications to the main run loop. The block can stay empty — simply
  // receiving it is enough to refresh the process's MIDI view. The property
  // keeps the client alive.
  private var midiRefreshClient = MIDIClientRef()

  override func applicationDidFinishLaunching(_ notification: Notification) {
    let status = MIDIClientCreateWithBlock("CymbraMidiRefresh" as CFString, &midiRefreshClient) { msg in
      NSLog("[cymbra-swift] MIDI notification messageID=\(msg.pointee.messageID.rawValue)")
    }
    NSLog("[cymbra-swift] MIDIClientCreateWithBlock status=\(status) client=\(midiRefreshClient)")

    // Ask APNs for a device token ourselves (change: add-push-notifications).
    // firebase_messaging does this from an NSApplicationDidFinishLaunching
    // observer, which only fires if the plugin was registered before the
    // notification was posted — a race we do not control. Registering here is
    // idempotent and makes the request unconditional. `delegate` is logged
    // because Firebase's GULAppDelegateSwizzler may proxy it, and a proxy that
    // fails to forward is indistinguishable from APNs never answering.
    NSApplication.shared.registerForRemoteNotifications()
    NSLog(
      "[cymbra-swift] requested APNs registration; delegate="
        + String(describing: type(of: NSApplication.shared.delegate!)))
    super.applicationDidFinishLaunching(notification)
  }

  // --- APNs device token (change: add-push-notifications) --------------------
  // FCM derives its registration token from the APNs one, so without these two
  // callbacks `getToken()` fails forever with `apns-token-not-set` and no macOS
  // device ever registers.
  //
  // NEVER call `super` from these two. `NSApplicationDelegate` declares them, so
  // Swift demands `override`, but `FlutterAppDelegate` does not implement them:
  // the super call forwards to a missing selector and kills the app the instant
  // the token arrives (NSInvalidArgumentException, unrecognized selector).
  override func application(
    _ application: NSApplication,
    didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data
  ) {
    NSLog("[cymbra-swift] APNs device token received (\(deviceToken.count) bytes)")
    // Firebase is configured lazily from Dart, so it may not be up yet when the
    // token lands. FirebaseApp.app() being nil means the plugin will pick the
    // token up itself once configured, so dropping it here is safe.
    if FirebaseApp.app() != nil {
      Messaging.messaging().apnsToken = deviceToken
    } else {
      NSLog("[cymbra-swift] Firebase not configured yet; leaving the token to the plugin")
    }
  }

  override func application(
    _ application: NSApplication,
    didFailToRegisterForRemoteNotificationsWithError error: Error
  ) {
    // Surfaced deliberately: the Dart side can only report "no token after 10s",
    // which does not say whether APNs refused us or never answered.
    NSLog("[cymbra-swift] APNs registration failed: \(error.localizedDescription)")
  }

  override func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
    return true
  }

  override func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
    return true
  }
}
