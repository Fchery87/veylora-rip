use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() == Some(std::ffi::OsStr::new("--version")) && args.next().is_none() {
        println!(
            "{} {}",
            veylora_domain::PRODUCT_NAME,
            veylora_domain::version()
        );
        ExitCode::SUCCESS
    } else {
        eprintln!("Usage: veylora-cli --version");
        ExitCode::from(2)
    }
}
