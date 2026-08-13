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

//! Engine log lines that reach the platform's own log.
//!
//! `eprintln!` goes nowhere on Android — a process's stderr is not wired to
//! logcat — which made the two subsystems that can *only* be diagnosed on a real
//! device (audio output and MIDI input) the two with no visibility at all. That
//! cost a full debugging session: the USB device was being dropped from the bus
//! and nothing in the app could say so.

use flutter_rust_bridge::frb;

/// Writes `message` under `tag` where the platform can show it: liblog on
/// Android, plain stderr everywhere else. Never called from a real-time audio
/// callback.
#[frb(ignore)]
pub(crate) fn log_line(tag: &str, message: &str) {
    #[cfg(target_os = "android")]
    {
        use std::ffi::{CString, c_char, c_int};

        /// `ANDROID_LOG_INFO` from `<android/log.h>`.
        const ANDROID_LOG_INFO: c_int = 4;

        #[link(name = "log")]
        unsafe extern "C" {
            fn __android_log_write(prio: c_int, tag: *const c_char, text: *const c_char) -> c_int;
        }

        if let (Ok(tag), Ok(text)) = (CString::new(tag), CString::new(message)) {
            // SAFETY: both pointers are valid, NUL-terminated, and only read for
            // the duration of the call.
            unsafe { __android_log_write(ANDROID_LOG_INFO, tag.as_ptr(), text.as_ptr()) };
        }
    }
    #[cfg(not(target_os = "android"))]
    eprintln!("[{tag}] {message}");
}
