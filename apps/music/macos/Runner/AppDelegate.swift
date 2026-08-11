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
    super.applicationDidFinishLaunching(notification)
  }

  // --- APNs device token (change: add-push-notifications) --------------------
  // FCM derives its registration token from the APNs one, so without these two
  // callbacks `getToken()` fails forever with `apns-token-not-set` and no macOS
  // device ever registers.
  //
  // firebase_messaging does NOT hook the app delegate on macOS — its
  // `[_registrar addApplicationDelegate:self]` is compiled out under
  // `#if !TARGET_OS_OSX` — and relies solely on GULAppDelegateSwizzler, which
  // does not reach this delegate here. Handing the token over explicitly is
  // independent of that swizzling, and it is the only reason we override these.
  override func application(
    _ application: NSApplication,
    didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data
  ) {
    Messaging.messaging().apnsToken = deviceToken
    // Keep FlutterAppDelegate's own forwarding to plugins that DID register as
    // application delegates.
    super.application(
      application, didRegisterForRemoteNotificationsWithDeviceToken: deviceToken)
  }

  override func application(
    _ application: NSApplication,
    didFailToRegisterForRemoteNotificationsWithError error: Error
  ) {
    // Surfaced deliberately: the Dart side can only report "no token after 10s",
    // which does not say whether APNs refused us or never answered.
    NSLog("[cymbra-swift] APNs registration failed: \(error.localizedDescription)")
    super.application(application, didFailToRegisterForRemoteNotificationsWithError: error)
  }

  override func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
    return true
  }

  override func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
    return true
  }
}
