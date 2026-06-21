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

  override func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
    return true
  }

  override func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
    return true
  }
}
