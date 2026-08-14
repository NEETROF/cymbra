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

import android.media.AudioAttributes
import android.media.AudioDeviceInfo
import android.media.AudioFormat
import android.media.AudioTrack
import android.util.Log
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Android's audio output: an [AudioTrack] fed by the Rust engine.
 *
 * On every other platform `cpal` owns the stream and calls the engine back. That
 * model does not work here — measured on device, `cpal`'s AAudio path cannot
 * enumerate Android's outputs (so a USB-audio piano can never be offered to the
 * user) and its stream reports itself healthy while delivering almost nothing to
 * that route. A plain `AudioTrack` keeps perfect time on the same route, lists
 * every output, and can pin playback to a chosen one.
 *
 * So the direction is inverted: this class owns the stream and **pulls** samples
 * from the engine. `write` blocks until the device accepts the data, which is
 * exactly what paces the producer — the piece AAudio failed to provide.
 */
object EngineOutput {
    private const val TAG = "cymbra-audio"

    /** Fallback rate when the platform reports nothing usable. */
    const val DEFAULT_RATE = 44_100

    private const val CHANNELS = 2

    /** Frames per pull. ~23 ms at 44.1 kHz: responsive, and a comfortable block
     *  for the synth to render in one go. */
    private const val FRAMES_PER_PULL = 1024

    /** Pause before reopening after a writer death, letting the collapsing
     *  route finish tearing down. */
    private const val REOPEN_DELAY_MS = 500L

    /** Two deaths closer than this = the audio system itself is down. */
    private const val REOPEN_BACKOFF_MS = 5_000L

    /** Deaths further apart than this reset the strike count: the route is
     *  merely unlucky, not broken. */
    private const val REOPEN_RETRY_WINDOW_MS = 30_000L

    /** Largest client buffer to ask for: with a blocking writer the full buffer
     *  is key-to-ear latency, so this is a latency budget, not a safety knob. */
    private const val BUFFER_CAP_MS = 120

    /** Clock-watchdog measurement window. */
    private const val CLOCK_WINDOW_NS = 3_000_000_000L

    /** How far the loop rate may stray from the track's rate before a window
     *  counts as broken. Generous: a refill burst is ~10%, the observed zombie
     *  +87%. */
    private const val CLOCK_TOLERANCE = 0.25

    /** The rates real-world sinks run at, for the rate-snap correction. */
    private val STANDARD_RATES = listOf(44_100, 48_000)

    /** How close the measured loop rate must sit to *another* standard rate to
     *  count as "the sink actually clocks at that rate". Tight on purpose: a
     *  buffer-refill burst reads at most ~+4% over a window and must not
     *  qualify (44.1k × 1.04 = 45.9k is nowhere near 48k × 0.96 = 46.1k). */
    private const val RATE_SNAP_TOLERANCE = 0.04

    init {
        // The engine's shared library — already loaded by MainActivity for MIDI,
        // and loading twice is a no-op.
        System.loadLibrary("rust_lib_music")
    }

    private var track: AudioTrack? = null
    private var writer: Thread? = null
    private val running = AtomicBoolean(false)

    /**
     * The *current* writer generation's liveness. Each writer loops on its own
     * flag, not on [running]: a writer stuck in a blocking write can outlive
     * stop()'s bounded join, and if the shared flag has already been set true
     * again by the next start() the orphan would keep pulling frames from the
     * engine and writing them to a released track.
     */
    private var writerAlive = AtomicBoolean(false)

    /** The device the live stream is pinned to, so a redundant start is free. */
    private var currentDeviceId: Int? = null

    /** The descriptor the pin was made with, so a clock-watchdog rebuild can try
     *  the same device again before falling back to the system default. */
    private var currentDevice: AudioDeviceInfo? = null

    /**
     * The device this stream is pinned to, or null when it follows the system
     * default. The only trustworthy answer to "where is our sound going" — the
     * platform has no permission-free query for it, and inferring it from the
     * connected outputs produces confident lies.
     */
    val pinnedDeviceId: Int?
        @Synchronized get() = currentDeviceId

    val isPlaying: Boolean
        get() = running.get()

    /** The rate the live track was opened at. Survives stop() so a rebuild after
     *  a writer death can reuse it. */
    private var currentRate = DEFAULT_RATE

    /**
     * Opens the output at [rate], optionally pinned to [device], and starts
     * pulling from the engine. Restarts cleanly when already running on a
     * *different* device or rate, so a device change is one call. Returns false
     * when the track could not be built.
     *
     * [rate] should be the route's own rate: AudioFlinger's output thread runs
     * at whatever the HAL negotiated with the sink, and a track at any other
     * rate goes through a resampler stage first.
     */
    @Synchronized
    fun start(device: AudioDeviceInfo?, rate: Int): Boolean {
        // Idempotent: the engine's Dart-side provider is auto-disposed, so init —
        // and therefore this — can be reached several times per session. Rebuilding
        // the track each time left retired `AudioTrack`s stacked on the device,
        // which is both wasteful and a good way to upset a USB output.
        //
        // A null device here means "no specific device", NOT "the system default":
        // init calls this long after the user may have pinned an output, and
        // restarting unpinned would silently destroy that choice (rate included —
        // whatever plays already plays at the right rate for its route).
        // Un-pinning is an explicit act — [restartUnpinned].
        if (running.get() &&
            (device == null || (currentDeviceId == device.id && rate == currentRate))
        ) {
            Log.i(TAG, "output already running — keeping the current device")
            return true
        }
        stop()
        currentRate = rate
        // The engine renders at the track's rate — no resample between the synth
        // and the sink. Safe before the SoundFont is even parsed: the rate is
        // remembered and applied at install.
        nativeSetRate(rate)
        val minBuffer = AudioTrack.getMinBufferSize(
            rate,
            AudioFormat.CHANNEL_OUT_STEREO,
            AudioFormat.ENCODING_PCM_16BIT,
        )
        // Headroom above the minimum so a slow render does not underrun — but
        // capped: a blocking-write producer keeps the client buffer FULL, so the
        // whole buffer is key-to-ear latency. On a USB route whose minimum is
        // already ~80 ms, "×4" meant a 320 ms buffer — a third of a second
        // between key press and sound. [BUFFER_CAP_MS] is the compromise;
        // the writer logs AudioTrack underruns, so a too-tight cap is visible.
        val capBytes = rate * BUFFER_CAP_MS / 1000 * CHANNELS * 2
        val bufferBytes =
            if (minBuffer > 0) maxOf(minBuffer, minOf(minBuffer * 4, capBytes))
            else 8192 * 4

        val built = try {
            AudioTrack.Builder()
                .setAudioAttributes(
                    AudioAttributes.Builder()
                        .setUsage(AudioAttributes.USAGE_MEDIA)
                        .setContentType(AudioAttributes.CONTENT_TYPE_MUSIC)
                        .build()
                )
                .setAudioFormat(
                    AudioFormat.Builder()
                        .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
                        .setSampleRate(rate)
                        .setChannelMask(AudioFormat.CHANNEL_OUT_STEREO)
                        .build()
                )
                .setTransferMode(AudioTrack.MODE_STREAM)
                .setBufferSizeInBytes(bufferBytes)
                .build()
        } catch (e: Exception) {
            Log.w(TAG, "could not open AudioTrack: $e")
            return false
        }

        // A refused pin must not masquerade as pinned: the recorded device id —
        // what activeRoute reports — follows what the platform actually accepted.
        val pinned = device == null || built.setPreferredDevice(device)
        if (!pinned) {
            Log.w(TAG, "pin refused for ${device?.productName}; following the system default")
        }
        track = built
        currentDeviceId = if (pinned) device?.id else null
        currentDevice = if (pinned) device else null
        running.set(true)
        built.play()
        Log.i(
            TAG,
            "android output started on ${device?.productName ?: "system default"} " +
                "($rate Hz, buffer $bufferBytes B)",
        )

        val alive = AtomicBoolean(true)
        writerAlive = alive
        writer = Thread({
            // Audio priority, or the renderer starves behind UI work: a default-
            // priority writer crackles (underruns) as soon as anything else is
            // busy. This is the one knob every media pipeline sets.
            android.os.Process.setThreadPriority(
                android.os.Process.THREAD_PRIORITY_URGENT_AUDIO
            )
            val buffer = ShortArray(FRAMES_PER_PULL * CHANNELS)
            // Clock watchdog. A blocking write paces this loop at the device's
            // consumption rate, so the loop rate IS the output's clock — and on
            // this hardware a track whose pinned USB device vanished can land on
            // an invalidated output that "consumes" far off real time without a
            // single write error (measured: +87%, silent, forever). No error
            // means no signal, so the clock itself is the tripwire.
            var windowFrames = 0L
            var windowStartNs = System.nanoTime()
            var badWindows = 0
            var rateSnapWindows = 0
            while (alive.get()) {
                val frames = nativeRender(buffer, FRAMES_PER_PULL)
                if (frames <= 0) {
                    // Engine not ready yet (the SoundFont is still parsing): write
                    // silence rather than tearing the stream down.
                    buffer.fill(0)
                }
                val wrote = built.write(buffer, 0, buffer.size, AudioTrack.WRITE_BLOCKING)
                if (wrote < 0) {
                    // The stream is dead (route torn down mid-write, mediaserver
                    // restart — routine when a USB device re-enumerates). Report
                    // not-running and reopen, off this thread: stop() joins the
                    // writer, so it cannot restart itself.
                    Log.w(TAG, "AudioTrack write failed ($wrote); scheduling a reopen")
                    running.set(false)
                    scheduleReopen(alive)
                    break
                }
                windowFrames += FRAMES_PER_PULL
                val elapsedNs = System.nanoTime() - windowStartNs
                if (elapsedNs >= CLOCK_WINDOW_NS) {
                    // Client-side starvation counter, straight from the platform:
                    // non-zero growth here means the writer failed to keep the
                    // track fed — the app's fault; zero while the ear still hears
                    // crackle points at the far (HAL/device) side instead.
                    val underruns = built.underrunCount
                    if (underruns > 0) {
                        Log.w(TAG, "AudioTrack underruns so far: $underruns")
                    }
                    val loopRate = windowFrames * 1_000_000_000.0 / elapsedNs
                    // Rate-snap correction. A sink can clock at a different
                    // standard rate than the track claims to need: this Samsung
                    // USB HAL consumes a 44.1 kHz track 1:1 on its 48 kHz clock
                    // instead of resampling — measured +9.7%, heard as crackle
                    // and a sharp pitch. The blocking write makes the sink's
                    // true clock measurable here; two consecutive windows at
                    // the OTHER standard rate → reopen the track (and the
                    // synth) at the rate the route actually runs.
                    val snapped = STANDARD_RATES.minByOrNull {
                        kotlin.math.abs(loopRate - it)
                    }
                    if (snapped != null && snapped != rate &&
                        kotlin.math.abs(loopRate - snapped) < snapped * RATE_SNAP_TOLERANCE
                    ) {
                        rateSnapWindows++
                        if (rateSnapWindows >= 2) {
                            Log.w(
                                TAG,
                                "route clocks at $snapped Hz but the track was opened " +
                                    "at $rate Hz (measured ${loopRate.toInt()} f/s); " +
                                    "reopening at $snapped Hz",
                            )
                            running.set(false)
                            scheduleRateCorrection(alive, snapped)
                            break
                        }
                    } else {
                        rateSnapWindows = 0
                    }
                    // One bad window can be a legitimate burst (buffer refill
                    // after a stall); a broken output is bad every window.
                    if (kotlin.math.abs(loopRate - rate) > rate * CLOCK_TOLERANCE) {
                        badWindows++
                    } else {
                        badWindows = 0
                    }
                    if (badWindows >= 2) {
                        Log.w(
                            TAG,
                            "output clock broken (${loopRate.toInt()} frames/s on a " +
                                "$rate Hz track); rebuilding",
                        )
                        running.set(false)
                        scheduleReopen(alive)
                        break
                    }
                    windowFrames = 0
                    windowStartNs = System.nanoTime()
                }
            }
        }, "cymbra-audio-writer").also { it.start() }
        return true
    }

    /** Hands the rebuild to a fresh thread: stop() joins the writer, so the dying
     *  writer cannot run it, and the pause lets the collapsing route tear down. */
    private fun scheduleReopen(generation: AtomicBoolean) {
        Thread({
            Thread.sleep(REOPEN_DELAY_MS)
            reopenAfterDeath(generation)
        }, "cymbra-audio-reopen").start()
    }

    /** Same hand-off for a rate-snap correction — not a death, so it keeps the
     *  device and stays out of the strike accounting. */
    private fun scheduleRateCorrection(generation: AtomicBoolean, rate: Int) {
        Thread({
            Thread.sleep(REOPEN_DELAY_MS)
            reopenAtRate(generation, rate)
        }, "cymbra-audio-rerate").start()
    }

    /** Rebuilds the output on the same device at the rate the route was measured
     *  to actually clock at. Generation-checked like [reopenAfterDeath]. */
    @Synchronized
    private fun reopenAtRate(generation: AtomicBoolean, rate: Int) {
        if (writerAlive !== generation || !generation.get()) return
        val device = currentDevice
        Log.i(TAG, "reopening ${device?.productName ?: "system default"} at $rate Hz")
        start(device, rate)
    }

    /**
     * Where to send the sound when its route proved broken. Set by the activity
     * (it owns `AudioManager`); returns the best-ranked output the app may
     * actually play on. Deliberately never "the system default": with a USB
     * device plugged the default re-resolves straight back to the broken route,
     * which is how the reopen loop once churned tracks every 6 s until the
     * whole composite instrument fell off the bus.
     */
    @Volatile
    var fallbackProvider: (() -> AudioDeviceInfo?)? = null

    /**
     * Rebuilds the output after the writer died (write error) or was killed by
     * the clock watchdog. Generation-checked — a newer start() already owns the
     * output. Escalating: strike one retries the same target; strike two moves
     * to the [fallbackProvider]'s safe output; strike three gives up until the
     * next start() (any re-init or selection) — sound must survive a broken
     * route, but a route that keeps killing tracks must not be hammered: the
     * rebuild churn itself is what knocks a composite USB instrument (its MIDI
     * function included) off the bus.
     */
    @Synchronized
    private fun reopenAfterDeath(generation: AtomicBoolean) {
        // Superseded by a newer start(), or explicitly stopped (only stop()
        // clears the generation flag — a writer death leaves it set): in both
        // cases the reopen belongs to a stream nobody wants back.
        if (writerAlive !== generation || !generation.get()) return
        val now = android.os.SystemClock.elapsedRealtime()
        if (now - lastReopenAtMs >= REOPEN_RETRY_WINDOW_MS) strikes = 0
        if (now - lastReopenAtMs < REOPEN_BACKOFF_MS) {
            Log.w(TAG, "output died again within ${REOPEN_BACKOFF_MS}ms; staying stopped")
            return
        }
        strikes++
        lastReopenAtMs = now
        when {
            strikes >= 3 -> Log.w(
                TAG,
                "output keeps dying (strike $strikes); staying stopped until the next start",
            )
            strikes == 2 -> {
                val target = fallbackProvider?.invoke()
                Log.i(
                    TAG,
                    "reopening on the fallback output " +
                        "${target?.productName ?: "(none available)"} after writer death",
                )
                start(target, currentRate)
            }
            else -> {
                Log.i(
                    TAG,
                    "reopening output on ${currentDevice?.productName ?: "the system default"} " +
                        "after writer death",
                )
                start(currentDevice, currentRate)
            }
        }
    }

    private var lastReopenAtMs = 0L
    private var strikes = 0

    /**
     * Sends the output to [device] (null = the system default) — always a
     * **fresh** track. The old one is torn down both to drop any pin and to
     * shed whatever zombie output it may be stuck on: on this hardware a track
     * whose device vanished can sit on an invalidated output that consumes
     * off-clock without erroring, and a live `setPreferredDevice` does not
     * resurrect it.
     */
    @Synchronized
    fun restartOn(device: AudioDeviceInfo?, rate: Int): Boolean {
        stop()
        return start(device, rate)
    }

    @Synchronized
    fun stop() {
        running.set(false)
        writerAlive.set(false)
        writer?.join(500)
        writer = null
        track?.let {
            runCatching { it.stop() }
            it.release()
        }
        track = null
        currentDeviceId = null
        currentDevice = null
    }

    /** Whether the engine has a mixer installed and can be pulled from. */
    external fun nativeIsReady(): Boolean

    /**
     * Tells the engine what rate to render at, so the synth and the track agree
     * and AudioFlinger has nothing to resample. Remembered if the SoundFont is
     * still parsing; rebuilds the renderer if it is already installed.
     */
    private external fun nativeSetRate(rate: Int)

    /**
     * Fills `out` with `frames` interleaved stereo 16-bit frames and returns how
     * many it wrote (0 when the engine is not ready).
     */
    external fun nativeRender(out: ShortArray, frames: Int): Int
}
