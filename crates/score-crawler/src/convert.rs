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

//! Conversion to validated, spec-compliant compressed MusicXML (`.mxl`).
//!
//! Native MusicXML is validated (genuinely MusicXML, not arbitrary XML) via the
//! shared [`cymbra_musicxml_core`] and compressed into a proper `.mxl`
//! container (`META-INF/container.xml` → internal `score.musicxml`), then every
//! produced `.mxl` is re-opened and re-parsed to confirm it round-trips. The
//! external converters (MuseScore CLI, Verovio, `python-ly`) attach here as
//! subprocess steps; MIDI is never treated as a score source.

use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

/// Wall-clock cap on any single external converter invocation.
const CONVERTER_TIMEOUT: Duration = Duration::from_secs(120);

/// How external converters are invoked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConverterBackend {
    /// Run the binary directly (must be on `PATH`).
    Local,
    /// Run each converter inside a Docker image (a temp dir is bind-mounted at
    /// `/work`), so nothing heavy needs installing on the host.
    Docker,
}

/// External-converter configuration: the backend and, for Docker, the image per
/// tool. Set once at startup via [`init_converters`]; defaults to `Local`.
#[derive(Debug, Clone)]
pub struct Converters {
    pub backend: ConverterBackend,
    pub musescore_image: String,
    pub verovio_image: String,
    pub lilypond_image: String,
}

impl Default for Converters {
    fn default() -> Self {
        // Docker image names are placeholders — override them in config to real
        // images that carry `mscore` / `verovio` / `ly` on PATH.
        Self {
            backend: ConverterBackend::Local,
            musescore_image: "cymbra/musescore".to_string(),
            verovio_image: "cymbra/verovio".to_string(),
            lilypond_image: "cymbra/python-ly".to_string(),
        }
    }
}

static CONVERTERS: std::sync::OnceLock<Converters> = std::sync::OnceLock::new();

/// Installs the converter configuration (idempotent; first call wins).
pub fn init_converters(converters: Converters) {
    let _ = CONVERTERS.set(converters);
}

/// The active converter configuration (defaults to `Local` if never set).
fn converters() -> &'static Converters {
    CONVERTERS.get_or_init(Converters::default)
}

/// Internal member name for the score inside the `.mxl` container.
const INNER_NAME: &str = "score.musicxml";

/// The spec `META-INF/container.xml` pointing at [`INNER_NAME`].
const CONTAINER_XML: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
    "<container>\n",
    "  <rootfiles>\n",
    "    <rootfile full-path=\"score.musicxml\" ",
    "media-type=\"application/vnd.recordare.musicxml+xml\"/>\n",
    "  </rootfiles>\n",
    "</container>\n",
);

/// The origin format of a harvested item, before conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OriginFormat {
    MusicXml,
    Mxl,
    MuseScore,
    LilyPond,
    Mei,
}

/// The conversion outcome recorded per item in the manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversionStatus {
    /// Produced a validated `.mxl`.
    Converted,
    /// Conversion to MusicXML failed/degraded; the original source was kept
    /// instead (e.g. LilyPond `.ly` + PDF) rather than emitting dubious output.
    FailedKeptSource,
    /// Conversion failed and nothing usable was produced.
    Failed,
    /// Not converted by policy (e.g. a MIDI-only item).
    Skipped,
}

/// A successful conversion: the `.mxl` bytes plus its recorded status.
#[derive(Debug, Clone)]
pub struct Converted {
    pub mxl: Vec<u8>,
    pub status: ConversionStatus,
}

/// Validates native MusicXML (rejecting non-MusicXML XML), compresses it to a
/// spec-compliant `.mxl`, and verifies the result re-parses.
pub fn convert_native(musicxml: &[u8]) -> Result<Converted> {
    // Reject well-formed-but-not-MusicXML input up front.
    cymbra_musicxml_core::validate(musicxml)
        .map_err(|r| anyhow!("input is not valid playable MusicXML: {r}"))?;
    let mxl = compress_to_mxl(musicxml)?;
    verify_mxl(&mxl).context("produced .mxl failed re-parse verification")?;
    Ok(Converted {
        mxl,
        status: ConversionStatus::Converted,
    })
}

/// Accepts an already-compressed `.mxl`: verifies it re-parses (and is genuine
/// MusicXML inside) rather than trusting the source blindly.
pub fn accept_mxl(mxl: &[u8]) -> Result<Converted> {
    verify_mxl(mxl).context(".mxl failed re-parse verification")?;
    Ok(Converted {
        mxl: mxl.to_vec(),
        status: ConversionStatus::Converted,
    })
}

/// Central conversion dispatch by origin format. Native MusicXML and `.mxl` are
/// handled in-process; the external-binary formats (MuseScore, LilyPond, MEI)
/// are wired as subprocess steps in a later slice and currently return an error
/// the orchestrator isolates as a per-item failure. MIDI is never a score
/// source and must not reach this function.
pub fn convert_any(origin: OriginFormat, bytes: &[u8]) -> Result<Converted> {
    match origin {
        OriginFormat::MusicXml => convert_native(bytes),
        OriginFormat::Mxl => accept_mxl(bytes),
        OriginFormat::MuseScore => musescore_to_mxl(bytes),
        OriginFormat::Mei => verovio_to_mxl(bytes),
        OriginFormat::LilyPond => lilypond_to_mxl(bytes),
    }
}

/// MuseScore `.mscx`/`.mscz` → `.mxl` via the MuseScore CLI (headless), then
/// re-parse verification. The CLI emits `.mxl` directly.
pub fn musescore_to_mxl(bytes: &[u8]) -> Result<Converted> {
    let mxl = convert_via_files(
        bytes,
        converters().backend,
        "mscx",
        "mxl",
        "mscore",
        &converters().musescore_image,
        |input, output| vec!["-o".into(), path(output), path(input)],
        &[("QT_QPA_PLATFORM", "offscreen")],
    )?;
    accept_mxl(&mxl).context("MuseScore output failed verification")
}

/// MEI → MusicXML via Verovio, then compress + verify.
pub fn verovio_to_mxl(bytes: &[u8]) -> Result<Converted> {
    let musicxml = convert_via_files(
        bytes,
        converters().backend,
        "mei",
        "xml",
        "verovio",
        &converters().verovio_image,
        |input, output| {
            vec![
                "-t".into(),
                "musicxml".into(),
                "-o".into(),
                path(output),
                path(input),
            ]
        },
        &[],
    )?;
    convert_native(&musicxml).context("Verovio output failed verification")
}

/// LilyPond `.ly` → MusicXML via `python-ly`. Conversion is imperfect; on any
/// failure (including a missing binary/image) the error is surfaced so the
/// orchestrator keeps the item as a failure rather than emitting dubious output.
/// The `failed_kept_source` refinement (persisting the `.ly`/PDF) is a follow-up
/// once the writer stores non-`.mxl` artefacts.
pub fn lilypond_to_mxl(bytes: &[u8]) -> Result<Converted> {
    let musicxml = convert_via_files(
        bytes,
        converters().backend,
        "ly",
        "xml",
        "ly",
        &converters().lilypond_image,
        |input, output| vec!["musicxml".into(), "-o".into(), path(output), path(input)],
        &[],
    )
    .context("LilyPond conversion failed")?;
    convert_native(&musicxml).context("LilyPond output failed verification")
}

/// Runs an external converter over temp files and returns the output bytes.
///
/// In `Local` mode `program` is run directly with host paths. In `Docker` mode
/// the temp dir is bind-mounted at `/work` and `docker run --rm -v <dir>:/work
/// <image> <program> <args-with-/work-paths>` is invoked (env vars passed via
/// `-e`). The temp dir is always removed.
#[allow(clippy::too_many_arguments)] // an internal seam; grouping would obscure it
fn convert_via_files(
    input: &[u8],
    backend: ConverterBackend,
    input_ext: &str,
    output_ext: &str,
    program: &str,
    image: &str,
    args: impl Fn(&Path, &Path) -> Vec<String>,
    envs: &[(&str, &str)],
) -> Result<Vec<u8>> {
    let dir = unique_temp_dir()?;
    let in_host = dir.join(format!("input.{input_ext}"));
    let out_host = dir.join(format!("output.{output_ext}"));
    let write = std::fs::write(&in_host, input)
        .with_context(|| format!("writing converter input {}", in_host.display()));

    let result = write.and_then(|()| match backend {
        ConverterBackend::Local => {
            let built = args(&in_host, &out_host);
            run_external(program, &refs(&built), envs, CONVERTER_TIMEOUT)
        }
        ConverterBackend::Docker => {
            // Build the tool command with the in-container paths.
            let in_c = PathBuf::from("/work").join(format!("input.{input_ext}"));
            let out_c = PathBuf::from("/work").join(format!("output.{output_ext}"));
            let tool = args(&in_c, &out_c);
            let mut cmd = vec!["run".to_string(), "--rm".to_string()];
            for (k, v) in envs {
                cmd.push("-e".into());
                cmd.push(format!("{k}={v}"));
            }
            cmd.push("-v".into());
            cmd.push(format!("{}:/work", dir.display()));
            cmd.push(image.to_string());
            cmd.push(program.to_string());
            cmd.extend(tool);
            run_external("docker", &refs(&cmd), &[], CONVERTER_TIMEOUT)
        }
    });

    let output = result.and_then(|()| {
        std::fs::read(&out_host)
            .with_context(|| format!("reading converter output {}", out_host.display()))
    });
    let _ = std::fs::remove_dir_all(&dir);
    output
}

/// `&str` view over owned args.
fn refs(v: &[String]) -> Vec<&str> {
    v.iter().map(|s| s.as_str()).collect()
}

/// Spawns `program`, enforcing `timeout` and checking the exit status. A missing
/// binary is a clear, non-panicking error the caller can degrade on.
pub fn run_external(
    program: &str,
    args: &[&str],
    envs: &[(&str, &str)],
    timeout: Duration,
) -> Result<()> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(anyhow!("converter '{program}' not found on PATH"));
        }
        Err(e) => return Err(anyhow!("spawning {program}: {e}")),
    };

    let start = Instant::now();
    loop {
        match child.try_wait().context("waiting for converter")? {
            Some(status) if status.success() => return Ok(()),
            Some(status) => return Err(anyhow!("{program} exited with {status}")),
            None => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(anyhow!("{program} timed out after {timeout:?}"));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

/// Creates a unique temp directory (process id + a monotonic counter) to hold a
/// converter's input/output — one dir so Docker can bind-mount it at `/work`.
fn unique_temp_dir() -> Result<PathBuf> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("score_crawler_conv_{}_{n}", std::process::id()));
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("creating temp dir {}", dir.display()))?;
    Ok(dir)
}

/// Lossy path→String for building converter args.
fn path(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// Builds a spec-compliant `.mxl` (ZIP: `META-INF/container.xml` → internal
/// `score.musicxml`) from raw MusicXML bytes.
pub fn compress_to_mxl(musicxml: &[u8]) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opts: zip::write::FileOptions<()> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("META-INF/container.xml", opts)
            .context("mxl: start container.xml")?;
        zip.write_all(CONTAINER_XML.as_bytes())
            .context("mxl: write container.xml")?;
        zip.start_file(INNER_NAME, opts)
            .context("mxl: start score entry")?;
        zip.write_all(musicxml).context("mxl: write score entry")?;
        zip.finish().context("mxl: finalize archive")?;
    }
    Ok(buf)
}

/// Re-opens a `.mxl`, decodes its rootfile, and confirms it parses as MusicXML.
pub fn verify_mxl(mxl: &[u8]) -> Result<()> {
    if !cymbra_musicxml_core::mxl::is_mxl(mxl) {
        return Err(anyhow!("not a .mxl (missing zip magic)"));
    }
    let inner = cymbra_musicxml_core::mxl::decode(mxl).context("mxl: decode rootfile")?;
    cymbra_musicxml_core::parse(&inner).context("mxl: internal score does not parse")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCORE: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>1</divisions>
        <key><fifths>0</fifths></key>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><type>whole</type></note>
    </measure>
  </part>
</score-partwise>"#;

    #[test]
    fn native_musicxml_converts_and_verifies() {
        let c = convert_native(SCORE.as_bytes()).unwrap();
        assert_eq!(c.status, ConversionStatus::Converted);
        assert!(cymbra_musicxml_core::mxl::is_mxl(&c.mxl));
        // The produced .mxl round-trips back to a parseable score.
        verify_mxl(&c.mxl).unwrap();
    }

    #[test]
    fn container_points_at_internal_score() {
        let mxl = compress_to_mxl(SCORE.as_bytes()).unwrap();
        let inner = cymbra_musicxml_core::mxl::decode(&mxl).unwrap();
        assert_eq!(inner, SCORE.as_bytes());
    }

    #[test]
    fn non_musicxml_xml_is_rejected() {
        let not_score = b"<html><body>hello</body></html>";
        assert!(convert_native(not_score).is_err());
    }

    #[test]
    fn empty_or_garbage_is_rejected() {
        assert!(convert_native(b"").is_err());
        assert!(convert_native(b"not xml at all").is_err());
    }

    #[test]
    fn accept_mxl_verifies_roundtrip() {
        let mxl = compress_to_mxl(SCORE.as_bytes()).unwrap();
        let c = accept_mxl(&mxl).unwrap();
        assert_eq!(c.status, ConversionStatus::Converted);
    }

    #[test]
    fn verify_rejects_non_mxl() {
        assert!(verify_mxl(SCORE.as_bytes()).is_err());
    }

    #[test]
    fn run_external_reports_exit_status() {
        assert!(run_external("sh", &["-c", "exit 0"], &[], Duration::from_secs(5)).is_ok());
        assert!(run_external("sh", &["-c", "exit 3"], &[], Duration::from_secs(5)).is_err());
    }

    #[test]
    fn run_external_missing_binary_is_a_clean_error() {
        let err = run_external(
            "score-crawler-no-such-binary-xyz",
            &[],
            &[],
            Duration::from_secs(5),
        )
        .unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn run_external_enforces_timeout() {
        let err =
            run_external("sh", &["-c", "sleep 5"], &[], Duration::from_millis(100)).unwrap_err();
        assert!(err.to_string().contains("timed out"));
    }

    #[test]
    fn docker_backend_roundtrips_via_bind_mount() {
        // Opt-in (it pulls/runs a container). Proves the Docker path — bind-mount
        // the temp dir at /work, exec the tool, read back the output — using a
        // trivial `alpine` + `cp` instead of a real converter image.
        if std::env::var("SCORE_CRAWLER_DOCKER_TEST").is_err() {
            eprintln!("skip: set SCORE_CRAWLER_DOCKER_TEST=1 to run the docker roundtrip");
            return;
        }
        let out = convert_via_files(
            b"HELLO-DOCKER",
            ConverterBackend::Docker,
            "in",
            "out",
            "cp",
            "alpine",
            |input, output| vec![path(input), path(output)],
            &[],
        )
        .expect("docker convert_via_files roundtrip");
        assert_eq!(out, b"HELLO-DOCKER");
    }

    #[test]
    fn external_conversion_degrades_when_binary_absent() {
        // With no MuseScore/Verovio/python-ly installed, conversion is a clean
        // per-item error (the orchestrator isolates it), never a panic.
        if run_external("mscore", &["-v"], &[], Duration::from_secs(2)).is_err() {
            assert!(musescore_to_mxl(b"<museScore/>").is_err());
        }
    }
}
