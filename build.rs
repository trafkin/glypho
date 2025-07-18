use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use handlebars::Handlebars;
use walkdir::DirEntry;

fn is_css(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".css"))
        .unwrap_or(false)
}

fn is_js(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".js"))
        .unwrap_or(false)
}

fn main() -> eyre::Result<()> {
    // let out_dir = env::var_os("OUT_DIR").unwrap_or(OsString::from("./src"));
    let mut hb = Handlebars::new();
    let css = fs::read_to_string("./src/assets/style.css")?;
    let prism_css = fs::read_to_string("./src/assets/prism.css")?;
    let js = fs::read_to_string("./src/assets/prism.js")?;

    let index = fs::read_to_string("./src/assets/index.html")?;

    hb.register_template_string("index.html", index.clone())?;

    let all_css = [css, prism_css].join("\n");

    let mut data = BTreeMap::new();
    data.insert("css".to_string(), all_css.clone());
    data.insert("js".to_string(), js.clone());
    let t = hb.render("index.html", &data)?;
    let dest_path = Path::new("./src/").join("template.html");
    fs::write(&dest_path, format!("{t}"))?;
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=./assets/");
    println!("cargo::rerun-if-changed=./template.html/");

    Ok(())
}
