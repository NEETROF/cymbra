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

import android.app.Activity
import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.media.AudioDeviceCallback
import android.media.AudioDeviceInfo
import android.media.AudioManager
import android.os.Handler
import android.os.Looper
import android.provider.Settings
import io.flutter.plugin.common.BinaryMessenger
import io.flutter.plugin.common.MethodChannel

/**
 * Audio routing on Android (change: add-audio-output-routing): enumerates the
 * platform's outputs, applies the user's selection to [EngineOutput], reports
 * the active route, and pushes route changes — all over one method channel.
 *
 * Owned by the activity ([attach] in `configureFlutterEngine`, [dispose] in
 * `onDestroy`); everything audio-routing lives here so the activity stays
 * lifecycle-only.
 */
class AudioRouter(private val activity: Activity) {
    companion object {
        /** Channel carrying the audio route to Dart. */
        private const val ROUTING_CHANNEL = "org.cymbra.music/audio_routing"

        /**
         * The system output switcher panel, for platforms where the app reports
         * the route rather than owning it.
         */
        private const val MEDIA_OUTPUT_PANEL = "com.android.settings.panel.action.MEDIA_OUTPUT"
        private const val MEDIA_OUTPUT_PACKAGE_EXTRA =
            "com.android.settings.panel.extra.PACKAGE_NAME"
    }

    private var channel: MethodChannel? = null
    private var deviceCallback: AudioDeviceCallback? = null

    /** The output the user explicitly pinned, or null when they follow the
     *  (policy-resolved) default — which then re-resolves on device changes. */
    private var userPinnedId: Int? = null

    private val audioManager: AudioManager
        get() = activity.getSystemService(Context.AUDIO_SERVICE) as AudioManager

    /** Wires the method channel, the reopen fallback and the device observer. */
    fun attach(messenger: BinaryMessenger) {
        val channel = MethodChannel(messenger, ROUTING_CHANNEL)
        channel.setMethodCallHandler { call, result ->
            when (call.method) {
                "activeRoute" -> result.success(activeRoute())
                "allOutputs" -> result.success(allOutputs())
                "presentRoutePicker" -> {
                    presentRoutePicker()
                    result.success(null)
                }
                // --- Output ownership on Android (see EngineOutput) -----------
                // The platform plays and the engine renders, the inverse of the
                // `cpal` model used everywhere else.
                "startOutput" -> {
                    // "Make sure the output runs": never re-targets a live
                    // stream. On a cold start the "default" is policy-resolved
                    // so it cannot land on a broken USB route.
                    val requested = outputById(call.argument<Int>("deviceId"))
                    val device =
                        if (requested == null && !EngineOutput.isPlaying) safeDefaultDevice()
                        else requested
                    result.success(EngineOutput.start(device, rateFor(device)))
                }
                "stopOutput" -> {
                    EngineOutput.stop()
                    result.success(null)
                }
                "selectOutput" -> {
                    val id = call.argument<Int>("deviceId")
                    val device = outputById(id)
                    if (id == null || (id >= 0 && device == null)) {
                        // Refuse a device that is gone (ids die on unplug) — and a
                        // malformed call with no id at all. Un-pinning is an
                        // explicit act (-1), never a fallback: pinning to null here
                        // would silently move the sound to the system default while
                        // reporting success.
                        result.success(false)
                    } else if (device != null) {
                        // A full rebuild, not a live re-pin: on this hardware a
                        // track whose device went away sits on an invalidated
                        // output that consumes off-clock without erroring, and
                        // `setPreferredDevice` does not resurrect it — only a
                        // fresh track does. start() is still free when already
                        // running on this very device at this rate.
                        userPinnedId = device.id
                        result.success(EngineOutput.start(device, rateFor(device)))
                    } else {
                        userPinnedId = null
                        val target = safeDefaultDevice()
                        result.success(EngineOutput.restartOn(target, rateFor(target)))
                    }
                }
                "outputState" -> result.success(
                    mapOf(
                        "playing" to EngineOutput.isPlaying,
                        "engineReady" to EngineOutput.nativeIsReady(),
                    )
                )
                else -> result.notImplemented()
            }
        }
        this.channel = channel
        // Where a repeatedly-dying stream retreats to: the best output the app
        // may actually play on — never "the default", which re-resolves to the
        // broken USB route that killed the stream in the first place.
        EngineOutput.fallbackProvider = { playableOutputs().firstOrNull() }
        observeRouteChanges()
    }

    /** Releases the channel, the observer and the output. Idempotent. */
    fun dispose() {
        EngineOutput.fallbackProvider = null // holds the activity via audioManager
        EngineOutput.stop()
        deviceCallback?.let { audioManager.unregisterAudioDeviceCallback(it) }
        deviceCallback = null
        channel = null
    }

    /**
     * The output *our* audio is going to right now, as `{name, kind}`.
     *
     * When the stream is pinned to a device we know the answer exactly, because we
     * are the ones who pinned it — report that and nothing else. Guessing here was
     * actively harmful: the screen claimed the sound was on the USB piano while the
     * mixer had it on the built-in speaker, which sent the search for a routing bug
     * that did not exist.
     *
     * Only when the stream follows the system default is a guess needed: Android
     * exposes no public "which device is media routed to" query below API 31 that
     * does not need a system permission, so the connected outputs are ranked the
     * way the platform itself routes media — an external output wins over the
     * built-in speaker, the most specific one wins among those.
     */
    private fun activeRoute(): Map<String, String>? {
        val outputs = audioManager.getDevices(AudioManager.GET_DEVICES_OUTPUTS)
        if (outputs.isEmpty()) return null
        val pinned = outputById(EngineOutput.pinnedDeviceId)
        val device = pinned ?: outputs.minByOrNull { routePriority(it.type) } ?: return null
        val name = device.productName?.toString()?.takeIf { it.isNotBlank() }
            ?: fallbackName(device.type)
        return mapOf("name" to name, "kind" to routeKind(device.type))
    }

    /**
     * Every connected output, as `{id, name, kind}`, external ones first.
     *
     * This is the list `cpal` cannot produce on Android: its own enumeration fails
     * and it offers a single placeholder, which is why the piano never appeared in
     * the picker. `AudioManager` has always had it.
     */
    private fun allOutputs(): List<Map<String, String>> =
        audioManager.getDevices(AudioManager.GET_DEVICES_OUTPUTS)
            // The earpiece is not a music output: media cannot be pinned to it
            // (writes fail with -4), and it shares the speaker's product name, so
            // listing it made the by-name dedupe keep the wrong sibling.
            //
            // USB outputs ARE listed, but selecting one is an **experimental**
            // opt-in (the Dart side labels them): both Androids we tested have a
            // USB path broken beneath any app's reach — a HAL clocking the route
            // at random per session, another crackling in every app, YouTube
            // included. What is NOT negotiable is the default: the app never
            // *lands* on USB by itself (see [safeDefaultDevice]), and a USB
            // route that keeps killing tracks is abandoned by the strike ladder
            // rather than hammered — the rebuild churn is what used to knock the
            // composite instrument (its MIDI included) off the bus.
            .filter { it.type != AudioDeviceInfo.TYPE_BUILTIN_EARPIECE }
            .sortedBy { routePriority(it.type) }
            .map { device ->
                val name = device.productName?.toString()?.takeIf { it.isNotBlank() }
                    ?: fallbackName(device.type)
                mapOf(
                    // `AudioTrack.setPreferredDevice` needs the id, so it travels to
                    // Dart and comes back with the user's choice. Ids are **not**
                    // stable across replugs, so the name is what gets persisted.
                    "id" to device.id.toString(),
                    "name" to name,
                    "kind" to routeKind(device.type),
                )
            }
            // One entry per name: a USB instrument can expose several output
            // AudioDeviceInfo under the same product name, and the selection
            // travels by name — duplicates made the picker show the device
            // several times and broke the dropdown (two items, one value). The
            // priority sort ran first, so the best-ranked entry survives.
            .distinctBy { it["name"] }

    /** The connected output with this id, or null to let the system choose. */
    private fun outputById(id: Int?): AudioDeviceInfo? {
        if (id == null || id < 0) return null
        return audioManager.getDevices(AudioManager.GET_DEVICES_OUTPUTS)
            .firstOrNull { it.id == id }
    }

    /** Connected outputs the app may actually play on (no earpiece, no USB —
     *  see [allOutputs] for why), best-ranked first. */
    private fun playableOutputs(): List<AudioDeviceInfo> =
        audioManager.getDevices(AudioManager.GET_DEVICES_OUTPUTS)
            .filter {
                it.type != AudioDeviceInfo.TYPE_BUILTIN_EARPIECE &&
                    routeKind(it.type) != "usb"
            }
            .sortedBy { routePriority(it.type) }

    /**
     * Where "follow the system default" should actually land. Normally null —
     * the true system default — but when the system would route media to a USB
     * output (it does so as soon as one is plugged), the app pins the
     * best-ranked non-USB output instead: the platform's USB path is broken on
     * every Android we tested, and playing into it both loses the audio and,
     * through the resulting rebuild churn, can take the composite instrument's
     * MIDI down with it.
     */
    private fun safeDefaultDevice(): AudioDeviceInfo? {
        val ranked = audioManager.getDevices(AudioManager.GET_DEVICES_OUTPUTS)
            .filter { it.type != AudioDeviceInfo.TYPE_BUILTIN_EARPIECE }
            .minByOrNull { routePriority(it.type) } ?: return null
        if (routeKind(ranked.type) != "usb") return null
        return playableOutputs().firstOrNull()
    }

    /**
     * The rate to open the output at for [device] (null = the system route):
     * the rate the route itself runs at, so AudioFlinger has nothing to
     * resample between the track and the sink. The platform's answer is a
     * *preference* — a rate the device does not list falls back through the
     * common rates rather than trusting it blindly.
     */
    private fun rateFor(device: AudioDeviceInfo?): Int {
        val system = audioManager
            .getProperty(AudioManager.PROPERTY_OUTPUT_SAMPLE_RATE)
            ?.toIntOrNull()
            ?.takeIf { it > 0 }
            ?: EngineOutput.DEFAULT_RATE
        val supported = device?.sampleRates ?: intArrayOf()
        return when {
            // Empty means "any rate" per the AudioDeviceInfo contract.
            supported.isEmpty() -> system
            supported.contains(system) -> system
            supported.contains(48_000) -> 48_000
            supported.contains(EngineOutput.DEFAULT_RATE) -> EngineOutput.DEFAULT_RATE
            else -> supported.max()
        }
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
        val panel = Intent(MEDIA_OUTPUT_PANEL)
            .putExtra(MEDIA_OUTPUT_PACKAGE_EXTRA, activity.packageName)
        try {
            activity.startActivity(panel)
            return
        } catch (_: ActivityNotFoundException) {
            // Older or trimmed builds ship no output panel.
        }
        try {
            activity.startActivity(Intent(Settings.ACTION_SOUND_SETTINGS))
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
        // A device came or went. When the user follows the default, re-resolve
        // the policy target: the USB device may have appeared (move off the
        // route the system now prefers) or gone away (drop the policy pin and
        // follow the true default again). start() is free when nothing changes.
        if (userPinnedId == null && EngineOutput.isPlaying) {
            val target = safeDefaultDevice()
            EngineOutput.start(target, rateFor(target))
        }
        channel?.invokeMethod("routeChanged", activeRoute())
    }
}
