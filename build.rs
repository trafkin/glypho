use std::path::Path;
use std::process::Command;

fn main() -> eyre::Result<()> {
    let npm = which::which("npm")
        .map_err(|_| eyre::eyre!("npm not found on PATH; install Node.js to build glypho"))?;

    let status = Command::new(npm)
        .current_dir("./glypho-web/")
        .args(["run", "build"])
        .status()?;
    if !status.success() {
        eyre::bail!("frontend build failed: `npm run build` exited with {status}");
    }

    let dist = Path::new("./glypho-web/dist/index.html");
    if !dist.exists() {
        eyre::bail!("frontend build output missing: {}", dist.display());
    }
    std::fs::copy(dist, "./src/template.html")?;

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=./glypho-web/src/*");
    println!("cargo::rerun-if-changed=./glypho-web/dist/index.html");

    Ok(())
}
