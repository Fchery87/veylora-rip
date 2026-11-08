use std::process::ExitCode;

mod probe;

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
        _ => {
            eprintln!("Usage: veylora-cli --version | probe --help");
            ExitCode::from(2)
        }
    }
}
