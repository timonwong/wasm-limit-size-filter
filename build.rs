const REQUIRED_MAJOR: usize = 1;
const REQUIRED_MINOR: usize = 52;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ac = autocfg::AutoCfg::new()?;

    if !ac.probe_rustc_version(REQUIRED_MAJOR, REQUIRED_MINOR) {
        println!(
            "cargo:warning=rustc version {}.{} or greater required, compilation might fail",
            REQUIRED_MAJOR, REQUIRED_MINOR
        );
    }

    autocfg::rerun_path("build.rs");

    Ok(())
}
