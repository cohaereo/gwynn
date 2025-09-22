use clap::Parser;

// Decompress a single file
fn main() -> anyhow::Result<()> {
    use gwynn_mpk::compression;
    use std::fs;

    let args = Args::parse();
    let mut buf = fs::read(&args.file)?;
    let start = args.offset.unwrap_or(0) as usize;
    let size = args.length.unwrap_or(buf.len() - start);
    let buf = &mut buf[start..start + size];
    let detected_type = compression::CompressionType::detect_from_slice(buf);
    if detected_type.is_none() {
        anyhow::bail!("Could not detect compression type or file is uncompressed");
    }
    println!("Guessed compression type: {:?}", detected_type);
    let decompressed = compression::decompress(buf)?;
    fs::write(&args.output, &decompressed)?;
    println!(
        "Successfully wrote {} bytes to {}",
        decompressed.len(),
        args.output
    );

    Ok(())
}

#[derive(clap::Parser, Debug)]
pub struct Args {
    file: String,

    #[arg(default_value = "decompressed.bin")]
    output: String,

    #[arg(short, long)]
    offset: Option<u64>,

    #[arg(short, long)]
    length: Option<usize>,
}
