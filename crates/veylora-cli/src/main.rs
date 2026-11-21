use std::process::ExitCode;

mod probe;
mod release;
mod workflow;

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    match args
        .next()
        .and_then(|arg| arg.into_string().ok())
        .as_deref()
    {
        Some("--version") if args.next().is_none() => {
            println!(
                "{} {}",
                veylora_domain::PRODUCT_NAME,
                veylora_domain::version()
            );
            ExitCode::SUCCESS
        }
        Some("probe") => probe::run(args.collect()),
        Some("release") => release::run(args.collect()),
        Some(first) if workflow::handles(first) => {
            let mut all = vec![std::ffi::OsString::from(first)];
            all.extend(args);
            workflow::run(all)
        }
        _ => {
            eprintln!("Usage: veylora-cli --version | probe --help | release --help | --workspace <dir> <command>");
            ExitCode::from(2)
        }
    }
}
