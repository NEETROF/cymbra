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

package org.cymbra.music

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.media.AudioDeviceCallback
import android.media.AudioDeviceInfo
import android.media.AudioManager
import android.os.Handler
import android.os.Looper
import android.provider.Settings
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity : FlutterActivity() {
    companion object {
        /** Channel carrying the audio route to Dart (change: add-audio-output-routing). */
        private const val ROUTING_CHANNEL = "org.cymbra.music/audio_routing"

        /**
         * The system output switcher panel. Android gives an app no way to pick an
         * output device itself, so the app presents the OS panel and reports what
         * came out of it.
         */
        private const val MEDIA_OUTPUT_PANEL = "com.android.settings.panel.action.MEDIA_OUTPUT"
        private const val MEDIA_OUTPUT_PACKAGE_EXTRA =
            "com.android.settings.panel.extra.PACKAGE_NAME"

        init {
            // Loads the Rust lib on the JVM side so that `JNI_OnLoad` is called
            // and initializes `ndk_context` (the JavaVM). flutter_rust_bridge
            // then loads the same lib via dlopen, but dlopen does not trigger
            // JNI_OnLoad — hence this explicit load, required for midir's
            // Android MIDI backend (AMidi).
            System.loadLibrary("rust_lib_music")
        }
    }

    private var routingChannel: MethodChannel? = null
    private var deviceCallback: AudioDeviceCallback? = null

    private val audioManager: AudioManager
        get() = getSystemService(Context.AUDIO_SERVICE) as AudioManager

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        val channel = MethodChannel(flutterEngine.dartExecutor.binaryMessenger, ROUTING_CHANNEL)
        channel.setMethodCallHandler { call, result ->
            when (call.method) {
                "activeRoute" -> result.success(activeRoute())
                "presentRoutePicker" -> {
                    presentRoutePicker()
                    result.success(null)
                }
                else -> result.notImplemented()
            }
        }
        routingChannel = channel
        observeRouteChanges()
    }

    override fun onDestroy() {
        deviceCallback?.let { audioManager.unregisterAudioDeviceCallback(it) }
        deviceCallback = null
        routingChannel = null
        super.onDestroy()
    }

    /**
     * The output media audio is going to right now, as `{name, kind}`.
     *
     * Android exposes no public "which device is media routed to" query below
     * API 31 that does not need a system permission, so the connected outputs are
     * ranked the way the platform itself routes media: an external output wins
     * over the built-in speaker, and the most specific one wins among those.
     * Close enough to be honest, and it degrades to the speaker.
     */
    private fun activeRoute(): Map<String, String>? {
        val outputs = audioManager.getDevices(AudioManager.GET_DEVICES_OUTPUTS)
        if (outputs.isEmpty()) return null
        val device = outputs.minByOrNull { routePriority(it.type) } ?: return null
        val name = device.productName?.toString()?.takeIf { it.isNotBlank() }
            ?: fallbackName(device.type)
        return mapOf("name" to name, "kind" to routeKind(device.type))
    }

    /** Lower is preferred: an external output takes precedence over the speaker. */
    private fun routePriority(type: Int): Int = when (routeKind(type)) {
        "bluetooth" -> 0
        "usb" -> 1
        "headphones" -> 2
        "builtin" -> 4
        else -> 3
    }

    /**
     * Maps a device type onto the five kinds the app reasons about. The **kind**,
     * never the device's name, is what drives the wireless warning; an
     * unrecognized type degrades to "other" rather than breaking the section.
     */
    private fun routeKind(type: Int): String = when (type) {
        AudioDeviceInfo.TYPE_BLUETOOTH_A2DP,
        AudioDeviceInfo.TYPE_BLUETOOTH_SCO,
        AudioDeviceInfo.TYPE_BLE_HEADSET,
        AudioDeviceInfo.TYPE_BLE_SPEAKER,
        AudioDeviceInfo.TYPE_BLE_BROADCAST -> "bluetooth"

        AudioDeviceInfo.TYPE_USB_DEVICE,
        AudioDeviceInfo.TYPE_USB_HEADSET,
        AudioDeviceInfo.TYPE_USB_ACCESSORY -> "usb"

        AudioDeviceInfo.TYPE_WIRED_HEADSET,
        AudioDeviceInfo.TYPE_WIRED_HEADPHONES -> "headphones"

        AudioDeviceInfo.TYPE_BUILTIN_SPEAKER,
        AudioDeviceInfo.TYPE_BUILTIN_EARPIECE -> "builtin"

        else -> "other"
    }

    /** A readable label for a device the platform did not name. */
    private fun fallbackName(type: Int): String = when (routeKind(type)) {
        "bluetooth" -> "Bluetooth"
        "usb" -> "USB"
        "headphones" -> "Headphones"
        "builtin" -> "Speaker"
        else -> "Audio output"
    }

    /**
     * Opens the system output switcher. Falls back to the sound settings screen
     * on a build that has no panel, and stays silent if neither exists — the app
     * still reports the active route.
     */
    private fun presentRoutePicker() {
        val panel = Intent(MEDIA_OUTPUT_PANEL).putExtra(MEDIA_OUTPUT_PACKAGE_EXTRA, packageName)
        try {
            startActivity(panel)
            return
        } catch (_: ActivityNotFoundException) {
            // Older or trimmed builds ship no output panel.
        }
        try {
            startActivity(Intent(Settings.ACTION_SOUND_SETTINGS))
        } catch (_: ActivityNotFoundException) {
            // Nothing to open; the reported route stays accurate anyway.
        }
    }

    /** Pushes the new route to Dart whenever an output appears or disappears. */
    private fun observeRouteChanges() {
        val callback = object : AudioDeviceCallback() {
            override fun onAudioDevicesAdded(addedDevices: Array<out AudioDeviceInfo>?) =
                notifyRouteChanged()

            override fun onAudioDevicesRemoved(removedDevices: Array<out AudioDeviceInfo>?) =
                notifyRouteChanged()
        }
        audioManager.registerAudioDeviceCallback(callback, Handler(Looper.getMainLooper()))
        deviceCallback = callback
    }

    private fun notifyRouteChanged() {
        routingChannel?.invokeMethod("routeChanged", activeRoute())
    }
}
