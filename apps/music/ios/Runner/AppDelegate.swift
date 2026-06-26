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

import AVFoundation
import CoreMIDI
import Flutter
import UIKit

@main
@objc class AppDelegate: FlutterAppDelegate {
  // CoreMIDI "refresher" client: created on the main thread so that the
  // process handles MIDI configuration change notifications (hot plug).
  // Without it, enumeration on the Rust side does not see devices connected
  // after startup (CoreMIDI only delivers these notifications to the main run
  // loop). Empty block on purpose.
  private var midiRefreshClient = MIDIClientRef()

  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    configureAudioSession()
    MIDIClientCreateWithBlock("CymbraMidiRefresh" as CFString, &midiRefreshClient) { _ in }
    GeneratedPluginRegistrant.register(with: self)
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }

  // The cpal/rustysynth output (a RemoteIO AudioUnit) stays silent on iOS unless
  // an AVAudioSession is configured and activated. `.playback` routes to the
  // speaker and keeps playing when the device is muted with the ring/silent
  // switch (musical output, like a piano app should).
  private func configureAudioSession() {
    let session = AVAudioSession.sharedInstance()
    do {
      try session.setCategory(.playback, mode: .default)
      try session.setActive(true)
    } catch {
      // Non-fatal: the engine degrades to silence; the rest of the app works.
      NSLog("[cymbra-audio] AVAudioSession setup failed: \(error)")
    }
  }
}
