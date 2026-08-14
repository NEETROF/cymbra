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
import AVKit
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

  /// Channel carrying the audio route to Dart (change:
  /// add-audio-output-routing). iOS gives the app no device to pick: the route
  /// belongs to the system, so the app only *reports* it and *presents* the
  /// system's own picker.
  private static let routingChannelName = "org.cymbra.music/audio_routing"
  private var routingChannel: FlutterMethodChannel?

  /// The system route picker, kept off-screen for the whole session.
  /// `AVRoutePickerView` has no imperative "present" API — the sanctioned way to
  /// show it is to tap its own button — so the view has to live in the hierarchy
  /// rather than be built on demand.
  private lazy var routePicker: AVRoutePickerView = {
    let picker = AVRoutePickerView(frame: .zero)
    picker.isHidden = true
    return picker
  }()

  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    configureAudioSession()
    MIDIClientCreateWithBlock("CymbraMidiRefresh" as CFString, &midiRefreshClient) { _ in }
    if let controller = window?.rootViewController as? FlutterViewController {
      configureAudioRouting(controller: controller)
    }
    GeneratedPluginRegistrant.register(with: self)
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }

  // The cpal/rustysynth output (a RemoteIO AudioUnit) stays silent on iOS unless
  // an AVAudioSession is configured and activated. `.playback` routes to the
  // speaker and keeps playing when the device is muted with the ring/silent
  // switch (musical output, like a piano app should).
  //
  // `.playback` is also the right category for route selection: it follows
  // whatever output the user picks (including Bluetooth A2DP and USB) and the
  // system re-routes the running stream in place, so a route change never stops
  // the engine.
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

  /// Wires the route channel and starts observing the system's route changes.
  private func configureAudioRouting(controller: FlutterViewController) {
    let channel = FlutterMethodChannel(
      name: AppDelegate.routingChannelName,
      binaryMessenger: controller.binaryMessenger
    )
    channel.setMethodCallHandler { [weak self] call, result in
      switch call.method {
      case "activeRoute":
        result(AppDelegate.currentRoute())
      case "presentRoutePicker":
        self?.presentRoutePicker()
        result(nil)
      default:
        result(FlutterMethodNotImplemented)
      }
    }
    routingChannel = channel
    controller.view.addSubview(routePicker)
    NotificationCenter.default.addObserver(
      self,
      selector: #selector(audioRouteChanged),
      name: AVAudioSession.routeChangeNotification,
      object: nil
    )
  }

  /// The output the session is using right now, as `{name, kind}` — or nil when
  /// the session reports no output at all.
  private static func currentRoute() -> [String: String]? {
    guard let output = AVAudioSession.sharedInstance().currentRoute.outputs.first else {
      return nil
    }
    return ["name": output.portName, "kind": routeKind(of: output.portType)]
  }

  /// Maps a port type onto the five kinds the app reasons about. Wireless is
  /// decided by the *port type*, never by the port's name, and an unrecognized
  /// type degrades to "other" rather than breaking the settings section.
  private static func routeKind(of portType: AVAudioSession.Port) -> String {
    switch portType {
    case .bluetoothA2DP, .bluetoothLE, .bluetoothHFP:
      return "bluetooth"
    case .headphones, .headsetMic:
      return "headphones"
    case .usbAudio:
      return "usb"
    case .builtInSpeaker, .builtInReceiver:
      return "builtin"
    default:
      return "other"
    }
  }

  /// Shows the system route picker by tapping the off-screen
  /// `AVRoutePickerView`'s own button — the only way UIKit offers to present it.
  private func presentRoutePicker() {
    for case let button as UIButton in routePicker.subviews {
      button.sendActions(for: .touchUpInside)
      return
    }
    NSLog("[cymbra-audio] route picker button unavailable")
  }

  /// The user (or the system) changed the route: push the new one to Dart so the
  /// settings section shows what is actually in use.
  @objc private func audioRouteChanged(_ notification: Notification) {
    DispatchQueue.main.async { [weak self] in
      self?.routingChannel?.invokeMethod("routeChanged", arguments: AppDelegate.currentRoute())
    }
  }
}
