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

import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine

class MainActivity : FlutterActivity() {
    companion object {
        init {
            // Loads the Rust lib on the JVM side so that `JNI_OnLoad` is called
            // and initializes `ndk_context` (the JavaVM). flutter_rust_bridge
            // then loads the same lib via dlopen, but dlopen does not trigger
            // JNI_OnLoad — hence this explicit load, required for midir's
            // Android MIDI backend (AMidi).
            System.loadLibrary("rust_lib_music")
        }
    }

    /** Everything audio-routing (change: add-audio-output-routing) lives in
     *  [AudioRouter]; the activity only owns its lifecycle. */
    private var audioRouter: AudioRouter? = null

    /** Microphone input (change: add-acoustic-piano-input): permission,
     *  capture configuration and input route — same ownership shape. */
    private var audioInput: AudioInput? = null

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        audioRouter = AudioRouter(this).also {
            it.attach(flutterEngine.dartExecutor.binaryMessenger)
        }
        audioInput = AudioInput(this).also {
            it.attach(flutterEngine.dartExecutor.binaryMessenger)
        }
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray,
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        audioInput?.onRequestPermissionsResult(requestCode, grantResults)
    }

    override fun onDestroy() {
        audioInput?.dispose()
        audioInput = null
        audioRouter?.dispose()
        audioRouter = null
        super.onDestroy()
    }
}
