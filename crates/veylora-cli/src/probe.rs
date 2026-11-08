use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};
use veylora_domain::provenance::ContentHash;

mod svg;

const MAX_INPUT_BYTES: u64 = 8 * 1024 * 1024;
const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

pub fn run(args: Vec<OsString>) -> ExitCode {
    match execute(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

fn execute(args: &[OsString]) -> Result<(), String> {
    let action = args
        .first()
        .and_then(|s| s.to_str())
        .ok_or("use probe --help")?;
    if matches!(action, "decode-candidate" | "submit") {
        return Err("no established protocol or encoder evidence; submission refused with and without --experimental (necessary, not sufficient)".into());
    }
    let allowed: &[&str] = match action {
        "--help" => &[],
        "inspect-reference" => &["--input"],
        "record-observation" => &["--input", "--output"],
        "render-bounded-target" => &["--output"],
        _ => return Err("unknown probe action; use probe --help".into()),
    };
    let options = parse_options(&args[1..], allowed)?;
    match action {
        "--help" => println!("veylora-cli probe\n  inspect-reference --input <file>\n  record-observation --input <request.json> --output <new.json>\n  render-bounded-target --output <new.json|new.svg>\n    SVG also creates <new.svg>.manifest.json\n  decode-candidate [--experimental]\nFile-only diagnostic investigation. NOT printer-ready. Input limit 8 MiB, output limit 1 MiB per file. No established protocol. No hardware operations."),
        "inspect-reference" => {
            let input = &options["--input"];
            let bytes = read_bounded(input)?;
            let value = serde_json::json!({"schema_version":1, "software_version":veylora_domain::version(), "action":"inspect-reference", "path":input, "byte_count":bytes.len(), "sha256":ContentHash::digest(&bytes), "decoded_protocol":false, "interpretation":"exact file bytes only; no protocol decoded"});
            let output = encode(&value)?;
            std::io::stdout().write_all(&output).map_err(|e| e.to_string())?;
        }
        "record-observation" => record(&options["--input"], &options["--output"] )?,
        "render-bounded-target" => {
            let output = &options["--output"];
            if output.exists() { return Err("refusing to overwrite existing output".into()); }
            if !matches!(output.extension().and_then(|s| s.to_str()), Some("json" | "svg")) {
                return Err("diagnostic export supports .json or .svg only, not printer-ready encoding".into());
            }
            use veylora_diagnostics::targets::{generate_target, PhysicalSize, TargetKind};
            let target = generate_target(TargetKind::ChannelIdentity, PhysicalSize::new(100.0, 100.0).map_err(|e| e.to_string())?);
            let artifact = serde_json::json!({"schema_version":1, "software_version":veylora_domain::version(), "label":"DIAGNOSTIC ONLY - NOT PRINTER-READY", "printer_ready":false, "target_sha256":target.stable_hash(), "target":target});
            if output.extension().and_then(|s| s.to_str()) == Some("svg") {
                let rendered = svg::render(&target)?;
                let mut manifest = artifact;
                manifest["artifact_sha256"] = serde_json::json!(ContentHash::digest(rendered.as_bytes()));
                manifest["artifact_byte_count"] = serde_json::json!(rendered.len());
                manifest["visualization_version"] = serde_json::json!(1);
                manifest["visualization_policy"] = serde_json::json!("approximate RGB from CMYK; white on gray; K+W black; viewer sans-serif text; not physical output");
                let encoded = encode(&manifest)?;
                let mut sidecar = output.as_os_str().to_os_string();
                sidecar.push(".manifest.json");
                let sidecar = PathBuf::from(sidecar);
                if sidecar.exists() { return Err("refusing to overwrite existing manifest".into()); }
                // Reserve both names before writing. Failures retain incomplete files, never replace them.
                let mut svg_file = OpenOptions::new().write(true).create_new(true).open(output).map_err(|e| format!("refusing to overwrite SVG: {e}"))?;
                let mut manifest_file = OpenOptions::new().write(true).create_new(true).open(&sidecar).map_err(|e| format!("refusing to overwrite manifest; empty SVG reservation retained: {e}"))?;
                svg_file.write_all(rendered.as_bytes()).and_then(|()| svg_file.sync_all()).map_err(|e| format!("incomplete SVG/manifest retained: {e}"))?;
                manifest_file.write_all(&encoded).and_then(|()| manifest_file.sync_all()).map_err(|e| format!("incomplete manifest retained: {e}"))?;
            } else {
                write_new(output, &encode(&artifact)?)?;
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn parse_options(args: &[OsString], allowed: &[&str]) -> Result<BTreeMap<String, PathBuf>, String> {
    if args.len() != allowed.len() * 2 {
        return Err("missing, duplicate or unsupported flags; use probe --help".into());
    }
    let mut result = BTreeMap::new();
    for pair in args.as_chunks::<2>().0 {
        let key = pair[0].to_str().ok_or("option names must be UTF-8")?;
        if !allowed.contains(&key)
            || result.contains_key(key)
            || pair[1].is_empty()
            || pair[1].to_string_lossy().starts_with("--")
        {
            return Err("missing, duplicate or unsupported flags; use probe --help".into());
        }
        result.insert(key.to_owned(), PathBuf::from(&pair[1]));
    }
    Ok(result)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Observation {
    operator: String,
    source: String,
    permission: String,
    command_family: String,
    framing: Option<String>,
    notes: String,
    baseline: Reference,
    candidate: Reference,
    changed_variable: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    evidence: PathBuf,
    settings: BTreeMap<String, String>,
}

fn record(input: &Path, output: &Path) -> Result<(), String> {
    let bytes = read_bounded(input)?;
    let observation: Observation =
        serde_json::from_slice(&bytes).map_err(|e| format!("invalid observation JSON: {e}"))?;
    for value in [
        &observation.operator,
        &observation.source,
        &observation.permission,
        &observation.command_family,
        &observation.notes,
        &observation.changed_variable,
    ] {
        if value.trim().is_empty() || value.len() > 4096 {
            return Err("observation text must be nonempty and at most 4096 bytes".into());
        }
    }
    if observation
        .framing
        .as_ref()
        .is_some_and(|s| s.trim().is_empty() || s.len() > 4096)
    {
        return Err("framing must be null or nonempty text at most 4096 bytes".into());
    }
    for reference in [&observation.baseline, &observation.candidate] {
        if reference.evidence.as_os_str().is_empty() || reference.evidence.as_os_str().len() > 4096
        {
            return Err("evidence path must be nonempty and bounded".into());
        }
    }
    veylora_device::probe::validate_comparison(
        &observation.baseline.settings,
        &observation.candidate.settings,
        &observation.changed_variable,
    )?;
    let parent = input.parent().unwrap_or(Path::new("."));
    let mut evidence = Vec::new();
    for reference in [&observation.baseline, &observation.candidate] {
        let data = read_bounded(&parent.join(&reference.evidence))?;
        evidence.push(serde_json::json!({"path":reference.evidence, "sha256":ContentHash::digest(&data), "byte_count":data.len()}));
    }
    let value = serde_json::json!({"schema_version":1, "software_version":veylora_domain::version(), "request_sha256":ContentHash::digest(&bytes), "observation":observation, "evidence_files":evidence, "protocol_established":false, "interpretation":"operator-reported observation, not decoded protocol or physical verification"});
    write_new(output, &encode(&value)?)
}

fn encode(value: &impl Serialize) -> Result<Vec<u8>, String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_OUTPUT_BYTES {
        return Err("output exceeds 1 MiB bound".into());
    }
    Ok(bytes)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > MAX_OUTPUT_BYTES {
        return Err("output exceeds 1 MiB bound".into());
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("cannot create output (refusing to overwrite): {e}"))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("output write failed; incomplete file retained: {e}"))
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| format!("cannot open evidence: {e}"))?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_INPUT_BYTES {
        return Err("input must be a regular file at most 8 MiB".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_INPUT_BYTES {
        return Err("input exceeds 8 MiB bound".into());
    }
    Ok(bytes)
}
