use std::{path::Path, process::Command};

fn main() -> eyre::Result<()> {
    let web_dir = Path::new("./glypho-web/");
    let node_modules = web_dir.join("node_modules");

    if node_modules.exists() {
        let status = Command::new("npm")
            .current_dir(web_dir)
            .args(["run", "build"])
            .status()?;

        if !status.success() {
            eyre::bail!("npm run build failed");
        }

        std::fs::copy("./glypho-web/dist/index.html", "./src/template.html")?;
    } else {
        println!(
            "cargo::warning=glypho-web/node_modules not found; using committed src/template.html"
        );
    }

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=./glypho-web/src/*");
    println!("cargo::rerun-if-changed=./glypho-web/package-lock.json");
    println!("cargo::rerun-if-changed=./glypho-web/package.json");
    println!("cargo::rerun-if-changed=./glypho-web/dist/index.html");

    Ok(())
}
