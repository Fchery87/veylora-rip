//! Headless workflow commands. Parsing and printing only: every decision is made by
//! `veylora-workflow`. Inputs are bounded, outputs are JSON, exports never overwrite.
use serde::Serialize;
use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    process::ExitCode,
};
use veylora_device::{
    mode::ModeRecord,
    setup::{EvidenceRecord, SetupRecord},
};
use veylora_diagnostics::targets::{PhysicalSize, TargetKind};
use veylora_domain::{errors::ErrorCode, ids::JobId, recipe::PrintRecipe};
use veylora_image::UntaggedPolicy;
use veylora_workflow::{
    inspect_artifacts, inspect_asset, JobSpec, PreflightOptions, RenderOptions, ReviewResolution,
    SetupDraft, Workspace,
};

const MAX_INPUT_BYTES: u64 = 64 * 1024 * 1024;
const USAGE: &str = "usage: veylora-cli --workspace <dir> <command>
  setup import --input <setup.json>
  mode import --input <mode.json>
  recipe import --input <recipe.json>
  asset import --input <file.png>
  asset inspect --input <file.png> [--untagged reject|assume-srgb]
  job create --spec <job.json>
  job preflight --job <id> [--min-ppi N] [--margin-um N] [--max-page-pixels N] [--disk-budget-bytes N]
  setup create --input <draft.json>
  setup verify --setup <hash> --evidence <evidence.json>
  target generate --kind <kind> --width-mm N --height-mm N --output <new.json|new.svg>
  job status --job <id>
  job hold --job <id> | job cancel --job <id> | job recover --job <id>
  job review --job <id> [--resolve finished|not-printed --operator <name> --note <text>]
  job submit | job reprint   (refused: no encoder evidence, physical submission disabled)
  job render --job <id> [--min-ppi N] [--margin-um N] [--max-page-pixels N]
      [--disk-budget-bytes N] [--acknowledge <code>[:<subject>]]...
  artifact inspect --dir <run directory>
  report export --job <id> --output <new.json>
Output is file-sink diagnostic output only. Submission, review and reprint are not available.";

/// True when the arguments are a workspace command rather than another subcommand.
pub fn handles(first: &str) -> bool {
    first == "--workspace"
}

pub fn run(args: Vec<OsString>) -> ExitCode {
    match execute(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

struct Parsed {
    command: (String, String),
    values: Vec<(String, String)>,
}
impl Parsed {
    fn one(&self, key: &str) -> Result<Option<&str>, String> {
        let mut found = self.values.iter().filter(|(k, _)| k == key);
        let first = found.next();
        if found.next().is_some() {
            return Err(format!("{key} given more than once"));
        }
        Ok(first.map(|(_, v)| v.as_str()))
    }
    fn need(&self, key: &str) -> Result<&str, String> {
        self.one(key)?
            .ok_or_else(|| format!("{key} is required\n{USAGE}"))
    }
    fn all(&self, key: &str) -> Vec<&str> {
        self.values
            .iter()
            .filter(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .collect()
    }
    fn number<T: std::str::FromStr>(&self, key: &str) -> Result<Option<T>, String> {
        self.one(key)?
            .map(|v| {
                v.parse()
                    .map_err(|_| format!("{key} must be a whole number"))
            })
            .transpose()
    }
    fn job(&self) -> Result<JobId, String> {
        self.need("--job")?
            .parse()
            .map_err(|_| "--job is not a job id".to_string())
    }
}

fn parse(args: &[String], allowed: &[&str]) -> Result<Vec<(String, String)>, String> {
    if !args.len().is_multiple_of(2) {
        return Err(format!("every option needs a value\n{USAGE}"));
    }
    let mut out = Vec::new();
    for pair in args.chunks(2) {
        if !allowed.contains(&pair[0].as_str()) || pair[1].starts_with("--") {
            return Err(format!(
                "unsupported or malformed option {}\n{USAGE}",
                pair[0]
            ));
        }
        out.push((pair[0].clone(), pair[1].clone()));
    }
    Ok(out)
}

fn execute(args: &[OsString]) -> Result<(), String> {
    let args: Vec<String> = args
        .iter()
        .map(|a| {
            a.to_str()
                .map(str::to_owned)
                .ok_or("arguments must be UTF-8")
        })
        .collect::<Result<_, _>>()?;
    // args[0] is --workspace
    let root = args.get(1).ok_or(USAGE)?;
    let (noun, verb) = (args.get(2).ok_or(USAGE)?, args.get(3).ok_or(USAGE)?);
    // Refused before anything is read or created. There is no encoder, and the retained rasters
    // are not printer bytes, so there is nothing honest to submit or reprint.
    if noun == "job" && matches!(verb.as_str(), "submit" | "reprint") {
        return Err(format!(
            "job {verb} is refused: there is no encoder evidence for the ET-8550 and physical submission is disabled. \
             Retained artifacts are diagnostic rasters, not printer data."
        ));
    }
    let allowed: &[&str] = match (noun.as_str(), verb.as_str()) {
        ("setup", "create") => &["--input"],
        ("setup", "verify") => &["--setup", "--evidence"],
        ("target", "generate") => &["--kind", "--width-mm", "--height-mm", "--output"],
        ("job", "review") => &["--job", "--resolve", "--operator", "--note"],
        ("job", "status" | "hold" | "cancel" | "recover") => &["--job"],
        ("setup" | "mode" | "recipe" | "asset", "import") => &["--input"],
        ("asset", "inspect") => &["--input", "--untagged"],
        ("job", "create") => &["--spec"],
        ("job", "preflight") => &[
            "--job",
            "--min-ppi",
            "--margin-um",
            "--max-page-pixels",
            "--disk-budget-bytes",
        ],
        ("job", "render") => &[
            "--job",
            "--min-ppi",
            "--margin-um",
            "--max-page-pixels",
            "--disk-budget-bytes",
            "--acknowledge",
        ],
        ("artifact", "inspect") => &["--dir"],
        ("report", "export") => &["--job", "--output"],
        _ => return Err(format!("unknown command\n{USAGE}")),
    };
    let p = Parsed {
        command: (noun.clone(), verb.clone()),
        values: parse(&args[4..], allowed)?,
    };
    dispatch(Path::new(root), &p)
}

fn options(p: &Parsed) -> Result<PreflightOptions, String> {
    let mut o = PreflightOptions::default();
    if let Some(v) = p.number("--min-ppi")? {
        o.min_effective_ppi = v;
    }
    if let Some(v) = p.number("--margin-um")? {
        o.margin_um = v;
    }
    if let Some(v) = p.number("--max-page-pixels")? {
        o.max_page_pixels = v;
    }
    o.disk_budget_bytes = p.number("--disk-budget-bytes")?;
    Ok(o)
}

fn acknowledged(p: &Parsed) -> Result<Vec<(ErrorCode, Option<String>)>, String> {
    p.all("--acknowledge")
        .into_iter()
        .map(|a| {
            let (code, subject) = match a.split_once(':') {
                Some((c, s)) => (c, Some(s.to_string())),
                None => (a, None),
            };
            let code = serde_json::from_value(serde_json::Value::String(code.to_string()))
                .map_err(|_| format!("'{code}' is not a finding code"))?;
            Ok((code, subject))
        })
        .collect()
}

fn dispatch(root: &Path, p: &Parsed) -> Result<(), String> {
    let e = |x: veylora_workflow::WorkflowError| x.to_string();
    // Inspection commands never create a workspace.
    match (p.command.0.as_str(), p.command.1.as_str()) {
        ("asset", "inspect") => {
            let policy = match p.one("--untagged")? {
                None | Some("assume-srgb") => UntaggedPolicy::AssumeSrgb,
                Some("reject") => UntaggedPolicy::Reject,
                Some(_) => return Err("--untagged must be reject or assume-srgb".into()),
            };
            let bytes = read_bounded(Path::new(p.need("--input")?))?;
            return print(&inspect_asset(&bytes, policy).map_err(e)?);
        }
        ("target", "generate") => {
            let kind: TargetKind =
                serde_json::from_value(serde_json::Value::String(p.need("--kind")?.to_string()))
                    .map_err(|_| "--kind is not a target kind".to_string())?;
            let dim = |key: &str| -> Result<f64, String> {
                p.need(key)?
                    .parse()
                    .map_err(|_| format!("{key} must be a number"))
            };
            let size = PhysicalSize::new(dim("--width-mm")?, dim("--height-mm")?)
                .map_err(|x| x.to_string())?;
            return crate::probe::export_target(Path::new(p.need("--output")?), kind, size);
        }
        ("artifact", "inspect") => {
            return print(&inspect_artifacts(Path::new(p.need("--dir")?)).map_err(e)?);
        }
        _ => {}
    }
    let ws = Workspace::open(root).map_err(e)?;
    match (p.command.0.as_str(), p.command.1.as_str()) {
        ("setup", "import") => {
            let r: SetupRecord = read_json(p.need("--input")?)?;
            hash_result(ws.import_setup(&r).map_err(e)?)
        }
        ("mode", "import") => {
            let r: ModeRecord = read_json(p.need("--input")?)?;
            hash_result(ws.import_mode(&r).map_err(e)?)
        }
        ("recipe", "import") => {
            let r: PrintRecipe = read_json(p.need("--input")?)?;
            hash_result(ws.import_recipe(&r).map_err(e)?)
        }
        ("asset", "import") => {
            let bytes = read_bounded(Path::new(p.need("--input")?))?;
            hash_result(ws.import_asset(&bytes).map_err(e)?)
        }
        ("setup", "create") => {
            let draft: SetupDraft = read_json(p.need("--input")?)?;
            print(&ws.create_setup(&draft).map_err(e)?)
        }
        ("setup", "verify") => {
            let hash = p
                .need("--setup")?
                .parse()
                .map_err(|_| "--setup is not a content hash")?;
            let evidence: EvidenceRecord = read_json(p.need("--evidence")?)?;
            print(&ws.verify_setup(&hash, evidence).map_err(e)?)
        }
        ("job", "review") => {
            let job = p.job()?;
            let Some(decision) = p.one("--resolve")? else {
                if p.one("--operator")?.is_some() || p.one("--note")?.is_some() {
                    return Err("--operator and --note only apply with --resolve".into());
                }
                return print(&ws.review(job).map_err(e)?);
            };
            let resolution = match decision {
                "finished" => ReviewResolution::ConfirmedFinished,
                "not-printed" => ReviewResolution::NotPrinted,
                _ => return Err("--resolve must be finished or not-printed".into()),
            };
            let state = ws
                .resolve_review(job, resolution, p.need("--operator")?, p.need("--note")?)
                .map_err(e)?;
            print(&serde_json::json!({ "state": state }))
        }
        ("job", "status") => {
            let job = p.job()?;
            print(&serde_json::json!({
                "state": ws.state(job).map_err(e)?,
                "events": ws.lifecycle_events(job).map_err(e)?,
            }))
        }
        ("job", "hold") => print(&serde_json::json!({ "state": ws.hold(p.job()?).map_err(e)? })),
        ("job", "cancel") => {
            print(&serde_json::json!({ "state": ws.cancel(p.job()?).map_err(e)? }))
        }
        ("job", "recover") => print(&ws.recover(p.job()?).map_err(e)?),
        ("job", "create") => {
            let spec: JobSpec = read_json(p.need("--spec")?)?;
            print(&ws.create_job(&spec).map_err(e)?)
        }
        ("job", "preflight") => print(&ws.preflight(p.job()?, &options(p)?).map_err(e)?),
        ("job", "render") => {
            let opts = RenderOptions {
                layout: options(p)?,
                acknowledged: acknowledged(p)?,
                ..RenderOptions::default()
            };
            let out = ws.render(p.job()?, &opts).map_err(e)?;
            print(&serde_json::json!({
                "dir": out.dir,
                "manifest_hash": out.manifest.snapshot_hash,
                "label": "file-sink diagnostic output; not printer-ready",
            }))
        }
        ("report", "export") => {
            let report = ws.report(p.job()?).map_err(e)?;
            let mut bytes = serde_json::to_vec_pretty(&report).map_err(|x| x.to_string())?;
            bytes.push(b'\n');
            write_new(Path::new(p.need("--output")?), &bytes)
        }
        _ => unreachable!("validated above"),
    }
}

fn hash_result(h: veylora_domain::provenance::ContentHash) -> Result<(), String> {
    print(&serde_json::json!({ "content_hash": h }))
}

fn print(value: &impl Serialize) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    std::io::stdout()
        .write_all(&bytes)
        .map_err(|e| e.to_string())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let bytes = read_bounded(Path::new(path))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("invalid JSON in {path}: {e}"))
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    let meta = file.metadata().map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > MAX_INPUT_BYTES {
        return Err("input must be a regular file of at most 64 MiB".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_INPUT_BYTES {
        return Err("input exceeds the 64 MiB bound".into());
    }
    Ok(bytes)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("cannot create output (refusing to overwrite): {e}"))?;
    f.write_all(bytes)
        .and_then(|()| f.sync_all())
        .map_err(|e| format!("output write failed; incomplete file retained: {e}"))
}
