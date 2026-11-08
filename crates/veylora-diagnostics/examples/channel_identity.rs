use veylora_diagnostics::targets::{generate_target, PhysicalSize, TargetKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = generate_target(
        TargetKind::ChannelIdentity,
        PhysicalSize::new(110.0, 150.0)?,
    );
    eprintln!("Synthetic target SHA-256: {}", target.stable_hash());
    println!("{}", serde_json::to_string_pretty(&target)?);
    Ok(())
}
