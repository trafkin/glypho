use std::process::Command;

fn main() -> eyre::Result<()> {
    // let dest_path = Path::new("./src/").join("template.html");
    Command::new("npm")
        .current_dir("./glypho-web/")
        .args(["run", "build"])
        .spawn()?;

    Command::new("cp")
        .current_dir("./glypho-web/")
        .args(["./dist/index.html", "../src/template.html"])
        .spawn()?;

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=./glypho-web/src");
    println!("cargo::rerun-if-changed=./template.html/");

    Ok(())
}
