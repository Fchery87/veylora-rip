//! `release` commands: build, verify, install, roll back and recover side-by-side releases.
//! The decisions live in `veylora-release`; this only parses arguments and prints results.

use serde::Serialize;
use std::{ffi::OsString, path::Path, process::ExitCode};
use veylora_release::{
    build_manifest, verify_release, write_manifest, Blocker, InstallStore, ReleaseManifest,
    UpdateGuard, UpdateRequest,
};
use veylora_workflow::Workspace;

const USAGE: &str = "usage: veylora-cli release <command>
  manifest --dir <release dir> --version <X.Y.Z> --max-data-schema <N> [--smoke-test <program> [--smoke-arg <arg>]...]
  verify --dir <release dir>
  status --install-root <dir>
  install --install-root <dir> --from <release dir> (--workspace <dir> | --no-workspace)
  rollback --install-root <dir> (--workspace <dir> | --no-workspace)
  recover --install-root <dir>
  blockers --workspace <dir>
An update or rollback is refused while a submission or render is in flight, or when that cannot be
read. --no-workspace says there is no workspace on this machine to protect.";

pub fn run(args: Vec<OsString>) -> ExitCode {
    match execute(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

struct Options {
    values: Vec<(String, String)>,
    no_workspace: bool,
}

impl Options {
    fn parse(args: &[String], allowed: &[&str]) -> Result<Self, String> {
        let mut values = Vec::new();
        let mut no_workspace = false;
        let mut i = 0;
        while i < args.len() {
            let key = args[i].as_str();
            if key == "--no-workspace" && allowed.contains(&key) {
                no_workspace = true;
                i += 1;
                continue;
            }
            let value = args.get(i + 1);
            match value {
                Some(v)
                    if allowed.contains(&key) && (key == "--smoke-arg" || !v.starts_with("--")) =>
                {
                    values.push((key.to_string(), v.clone()));
                    i += 2;
                }
                _ => return Err(format!("unsupported or malformed option {key}\n{USAGE}")),
            }
        }
        Ok(Self {
            values,
            no_workspace,
        })
    }
    fn all(&self, key: &str) -> Vec<&str> {
        self.values
            .iter()
            .filter(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .collect()
    }
    fn one(&self, key: &str) -> Result<Option<&str>, String> {
        match self.all(key).as_slice() {
            [] => Ok(None),
            [v] => Ok(Some(v)),
            _ => Err(format!("{key} was given more than once")),
        }
    }
    fn need(&self, key: &str) -> Result<&str, String> {
        self.one(key)?
            .ok_or_else(|| format!("{key} is required\n{USAGE}"))
    }
}

fn print(value: &impl Serialize) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    println!("{text}");
    Ok(())
}

/// Reads job state from a real workspace; never creates one.
struct WorkspaceGuard(Workspace);
impl UpdateGuard for WorkspaceGuard {
    fn blockers(&self) -> Result<Vec<Blocker>, String> {
        Ok(self
            .0
            .update_blockers()
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|b| Blocker {
                subject: format!("job {}", b.job_id),
                reason: b.reason.to_string(),
            })
            .collect())
    }
}

/// Used only when the operator states there is no workspace to protect.
struct NoWorkspace;
impl UpdateGuard for NoWorkspace {
    fn blockers(&self) -> Result<Vec<Blocker>, String> {
        Ok(vec![])
    }
}

fn open_workspace(path: &str) -> Result<Workspace, String> {
    if !Path::new(path).is_dir() {
        return Err(format!(
            "workspace {path} does not exist; use --no-workspace if there is none"
        ));
    }
    Workspace::open(path).map_err(|e| e.to_string())
}

enum Protect {
    Workspace(Box<Workspace>),
    Nothing,
}

fn protect(o: &Options) -> Result<Protect, String> {
    match (o.one("--workspace")?, o.no_workspace) {
        (Some(_), true) => Err("give --workspace or --no-workspace, not both".into()),
        (Some(path), false) => Ok(Protect::Workspace(Box::new(open_workspace(path)?))),
        (None, true) => Ok(Protect::Nothing),
        (None, false) => Err(format!(
            "an update cannot be shown safe without a workspace: give --workspace <dir> or --no-workspace\n{USAGE}"
        )),
    }
}

fn smoke(release_dir: &Path, manifest: &ReleaseManifest) -> Result<(), String> {
    let Some(cmd) = &manifest.smoke_test else {
        return Ok(());
    };
    let program = release_dir.join(&cmd[0]);
    let out = std::process::Command::new(&program)
        .args(&cmd[1..])
        .output()
        .map_err(|e| format!("cannot run {}: {e}", cmd[0]))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{} exited with {}: {}",
            cmd[0],
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
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
    let verb = args.first().ok_or(USAGE)?.as_str();
    let rest = &args[1..];
    let e = |x: veylora_release::ReleaseError| x.to_string();
    match verb {
        "manifest" => {
            let o = Options::parse(
                rest,
                &[
                    "--dir",
                    "--version",
                    "--max-data-schema",
                    "--smoke-test",
                    "--smoke-arg",
                ],
            )?;
            let max: u32 = o
                .need("--max-data-schema")?
                .parse()
                .map_err(|_| "--max-data-schema must be a number")?;
            let smoke_test = o.one("--smoke-test")?.map(|program| {
                let mut cmd = vec![program.to_string()];
                cmd.extend(o.all("--smoke-arg").into_iter().map(str::to_string));
                cmd
            });
            if smoke_test.is_none() && !o.all("--smoke-arg").is_empty() {
                return Err("--smoke-arg needs --smoke-test".into());
            }
            let dir = Path::new(o.need("--dir")?);
            let manifest = build_manifest(dir, o.need("--version")?, max, smoke_test).map_err(e)?;
            write_manifest(dir, &manifest).map_err(e)?;
            print(&manifest)
        }
        "verify" => {
            let o = Options::parse(rest, &["--dir"])?;
            print(&verify_release(Path::new(o.need("--dir")?)).map_err(e)?)
        }
        "status" => {
            let o = Options::parse(rest, &["--install-root"])?;
            let store = InstallStore::open(o.need("--install-root")?).map_err(e)?;
            let verified = store.verify_current().map_err(e)?;
            print(&serde_json::json!({
                "state": store.state().map_err(e)?,
                "current_verified": verified.is_some(),
                "current_version": verified.map(|m| m.version),
            }))
        }
        "install" => {
            let o = Options::parse(
                rest,
                &["--install-root", "--from", "--workspace", "--no-workspace"],
            )?;
            let store = InstallStore::open(o.need("--install-root")?).map_err(e)?;
            let source = Path::new(o.need("--from")?);
            let guard = protect(&o)?;
            let (data_schema, guard): (Option<u32>, Box<dyn UpdateGuard>) = match guard {
                Protect::Workspace(ws) => (
                    Some(ws.data_schema_version().map_err(|x| x.to_string())?),
                    Box::new(WorkspaceGuard(*ws)),
                ),
                Protect::Nothing => (None, Box::new(NoWorkspace)),
            };
            let outcome = store
                .install(&UpdateRequest {
                    source,
                    data_schema,
                    guard: guard.as_ref(),
                    smoke_test: &smoke,
                })
                .map_err(e)?;
            print(&serde_json::json!({
                "installed": outcome.installed,
                "previous": outcome.previous,
            }))
        }
        "rollback" => {
            let o = Options::parse(rest, &["--install-root", "--workspace", "--no-workspace"])?;
            let store = InstallStore::open(o.need("--install-root")?).map_err(e)?;
            let guard: Box<dyn UpdateGuard> = match protect(&o)? {
                Protect::Workspace(ws) => Box::new(WorkspaceGuard(*ws)),
                Protect::Nothing => Box::new(NoWorkspace),
            };
            let outcome = store.rollback(guard.as_ref()).map_err(e)?;
            print(&serde_json::json!({ "installed": outcome.installed }))
        }
        "recover" => {
            let o = Options::parse(rest, &["--install-root"])?;
            let store = InstallStore::open(o.need("--install-root")?).map_err(e)?;
            let r = store.recover().map_err(e)?;
            print(&serde_json::json!({
                "abandoned_update": r.abandoned_update,
                "fell_back_from_damaged_current": r.fell_back_from_damaged_current,
                "removed": r.removed,
            }))
        }
        "blockers" => {
            let o = Options::parse(rest, &["--workspace"])?;
            let ws = open_workspace(o.need("--workspace")?)?;
            let blockers = ws.update_blockers().map_err(|x| x.to_string())?;
            print(
                &serde_json::json!({ "safe_to_update": blockers.is_empty(), "blockers": blockers }),
            )
        }
        _ => Err(format!("unknown release command\n{USAGE}")),
    }
}
