pub fn main() {
    println!("cargo::rerun-if-changed=fonts/comet-icons.toml");
    iced_fontello::build(format!("fonts/comet-icons.toml")).expect("Build icons font");
}
