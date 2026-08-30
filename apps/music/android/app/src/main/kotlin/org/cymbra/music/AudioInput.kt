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

import android.Manifest
import android.app.Activity
import android.content.Context
import android.content.pm.PackageManager
import android.media.AudioDeviceCallback
import android.media.AudioDeviceInfo
import android.media.AudioManager
import android.os.Handler
import android.os.Looper
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import io.flutter.plugin.common.BinaryMessenger
import io.flutter.plugin.common.MethodChannel

/**
 * Microphone input support (change: add-acoustic-piano-input): the RECORD_AUDIO
 * runtime permission, the capture configuration report, and the *input* route —
 * mirrored on [AudioRouter], which owns the output side.
 *
 * Capture itself runs through the engine (cpal → AAudio), which uses AAudio's
 * default input preset — VOICE_RECOGNITION, the least-processed guaranteed
 * source and exactly the spec's fallback. UNPROCESSED support is *probed* here
 * and recorded for diagnostics, so the fleet answer arrives as data even while
 * the effective source stays the fallback.
 */
class AudioInput(private val activity: Activity) {
    companion object {
        private const val INPUT_CHANNEL = "org.cymbra.music/audio_input"

        /** Request code for the RECORD_AUDIO runtime permission. */
        const val RECORD_AUDIO_REQUEST = 0xCA9

        /** [AudioDeviceInfo] type → the stable token the engine classifies.
         *  Tokens, never display names (spec: Input Route Classification). */
        private fun typeToken(type: Int): String = when (type) {
            AudioDeviceInfo.TYPE_BUILTIN_MIC -> "TYPE_BUILTIN_MIC"
            AudioDeviceInfo.TYPE_WIRED_HEADSET -> "TYPE_WIRED_HEADSET"
            AudioDeviceInfo.TYPE_USB_DEVICE -> "TYPE_USB_DEVICE"
            AudioDeviceInfo.TYPE_USB_HEADSET -> "TYPE_USB_HEADSET"
            AudioDeviceInfo.TYPE_USB_ACCESSORY -> "TYPE_USB_ACCESSORY"
            AudioDeviceInfo.TYPE_BLUETOOTH_SCO -> "TYPE_BLUETOOTH_SCO"
            AudioDeviceInfo.TYPE_BLE_HEADSET -> "TYPE_BLE_HEADSET"
            else -> "TYPE_UNKNOWN"
        }

        /** Capture-candidate priority: what AAudio's default routing prefers
         *  when the device is present. SCO/BLE mics are listed so the app can
         *  *refuse* them explicitly, but they are never started (the app never
         *  calls startBluetoothSco), so they rank below everything wired. */
        private fun priority(type: Int): Int = when (type) {
            AudioDeviceInfo.TYPE_WIRED_HEADSET -> 0
            AudioDeviceInfo.TYPE_USB_HEADSET,
            AudioDeviceInfo.TYPE_USB_DEVICE,
            AudioDeviceInfo.TYPE_USB_ACCESSORY -> 1
            AudioDeviceInfo.TYPE_BUILTIN_MIC -> 2
            else -> 3
        }
    }

    private var channel: MethodChannel? = null
    private var deviceCallback: AudioDeviceCallback? = null
    private var pendingPermission: MethodChannel.Result? = null

    private val audioManager: AudioManager =
        activity.getSystemService(Context.AUDIO_SERVICE) as AudioManager

    fun attach(messenger: BinaryMessenger) {
        val channel = MethodChannel(messenger, INPUT_CHANNEL)
        channel.setMethodCallHandler { call, result ->
            when (call.method) {
                "permissionStatus" -> result.success(permissionStatus())
                "requestPermission" -> requestPermission(result)
                "activeInputRoute" -> result.success(activeInputRoute())
                "inputConfig" -> result.success(inputConfig())
                // The AVAudioSession shape only exists on iOS; Android capture
                // needs no session flip, so both are cheap no-ops here.
                "beginCaptureSession" -> result.success(true)
                "endCaptureSession" -> result.success(null)
                else -> result.notImplemented()
            }
        }
        this.channel = channel

        val callback = object : AudioDeviceCallback() {
            override fun onAudioDevicesAdded(added: Array<out AudioDeviceInfo>) = pushRoute()
            override fun onAudioDevicesRemoved(removed: Array<out AudioDeviceInfo>) = pushRoute()
        }
        audioManager.registerAudioDeviceCallback(callback, Handler(Looper.getMainLooper()))
        deviceCallback = callback
    }

    fun dispose() {
        deviceCallback?.let(audioManager::unregisterAudioDeviceCallback)
        deviceCallback = null
        channel?.setMethodCallHandler(null)
        channel = null
        pendingPermission = null
    }

    /** Forwarded by the activity; answers the pending "requestPermission". */
    fun onRequestPermissionsResult(requestCode: Int, grantResults: IntArray) {
        if (requestCode != RECORD_AUDIO_REQUEST) return
        val granted =
            grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED
        pendingPermission?.success(granted)
        pendingPermission = null
    }

    private fun permissionStatus(): String =
        if (ContextCompat.checkSelfPermission(activity, Manifest.permission.RECORD_AUDIO) ==
            PackageManager.PERMISSION_GRANTED
        ) "granted" else "undetermined"

    private fun requestPermission(result: MethodChannel.Result) {
        if (permissionStatus() == "granted") {
            result.success(true)
            return
        }
        // One request at a time; a second caller gets the same answer path.
        pendingPermission?.success(false)
        pendingPermission = result
        ActivityCompat.requestPermissions(
            activity, arrayOf(Manifest.permission.RECORD_AUDIO), RECORD_AUDIO_REQUEST
        )
    }

    /** The input capture would use right now: the highest-priority candidate,
     *  as `{name, token}` — or null when the platform lists no inputs. */
    private fun activeInputRoute(): Map<String, String>? {
        val device = audioManager.getDevices(AudioManager.GET_DEVICES_INPUTS)
            .minByOrNull { priority(it.type) } ?: return null
        return mapOf(
            "name" to device.productName.toString(),
            "token" to typeToken(device.type),
        )
    }

    /** The capture configuration report (spec: Unprocessed Capture
     *  Configuration — the obtained configuration is recorded and available
     *  to diagnostics). */
    private fun inputConfig(): Map<String, Any> = mapOf(
        "unprocessedSupported" to
            ("true" == audioManager.getProperty(
                AudioManager.PROPERTY_SUPPORT_AUDIO_SOURCE_UNPROCESSED
            )),
        // cpal opens AAudio with its default input preset — VOICE_RECOGNITION,
        // the least-processed guaranteed source. UNPROCESSED needs a builder
        // preset cpal does not expose; when it becomes reachable, this report
        // is where the change shows up.
        "source" to "VOICE_RECOGNITION",
    )

    private fun pushRoute() {
        channel?.invokeMethod("inputRouteChanged", activeInputRoute())
    }
}
