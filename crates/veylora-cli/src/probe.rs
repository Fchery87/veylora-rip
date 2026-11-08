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
        "--help" => println!("veylora-cli probe\n  inspect-reference --input <file>\n  record-observation --input <request.json> --output <new.json>\n  render-bounded-target --output <new.json>\n  decode-candidate [--experimental]\nFile-only diagnostic investigation. NOT printer-ready. Input limit 8 MiB, output limit 1 MiB. No established protocol. No hardware operations."),
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
            return Err("diagnostic generator dependency not integrated yet; no artifact produced".into());
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
    for pair in args.chunks_exact(2) {
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
    let baseline = &observation.baseline.settings;
    let candidate = &observation.candidate.settings;
    if baseline.is_empty() || baseline.len() > 64 || baseline.keys().ne(candidate.keys()) {
        return Err("comparison must use the same 1..64 setting names".into());
    }
    let changed: Vec<_> = baseline
        .iter()
        .filter(|(k, v)| candidate.get(*k) != Some(*v))
        .map(|(k, _)| k)
        .collect();
    if changed != vec![&observation.changed_variable] {
        return Err("comparison must change exactly the declared one variable".into());
    }
    for (name, value) in baseline.iter().chain(candidate) {
        if name.trim().is_empty()
            || value.trim().is_empty()
            || name.len() > 128
            || value.len() > 4096
        {
            return Err("invalid comparison setting".into());
        }
    }
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
