fn main() -> eyre::Result<()> {
    // let dest_path = Path::new("./src/").join("template.html");

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=./glypho-web/src");
    println!("cargo::rerun-if-changed=./template.html/");

    Ok(())
}
