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
use veylora_device::{mode::ModeRecord, setup::SetupRecord};
use veylora_domain::{errors::ErrorCode, ids::JobId, recipe::PrintRecipe};
use veylora_image::UntaggedPolicy;
use veylora_workflow::{
    inspect_artifacts, inspect_asset, JobSpec, PreflightOptions, RenderOptions, Workspace,
};

const MAX_INPUT_BYTES: u64 = 64 * 1024 * 1024;
const USAGE: &str = "usage: veylora-cli --workspace <dir> <command>
  setup import --input <setup.json>
  mode import --input <mode.json>
  recipe import --input <recipe.json>
  asset import --input <file.png>
  asset inspect --input <file.png> [--untagged reject|assume-srgb]
  job create --spec <job.json>
  job preflight --job <id> [--min-ppi N] [--margin-um N]
  job render --job <id> [--min-ppi N] [--margin-um N] [--acknowledge <code>[:<subject>]]...
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
    let allowed: &[&str] = match (noun.as_str(), verb.as_str()) {
        ("setup" | "mode" | "recipe" | "asset", "import") => &["--input"],
        ("asset", "inspect") => &["--input", "--untagged"],
        ("job", "create") => &["--spec"],
        ("job", "preflight") => &["--job", "--min-ppi", "--margin-um"],
        ("job", "render") => &["--job", "--min-ppi", "--margin-um", "--acknowledge"],
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
