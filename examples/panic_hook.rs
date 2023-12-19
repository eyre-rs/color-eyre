use nocolor_eyre::eyre::Report;
use tracing::instrument;

#[instrument]
fn main() -> Result<(), Report> {
    nocolor_eyre::config::HookBuilder::default()
        .panic_section("consider reporting the bug on github")
        .install()?;

    read_config();

    Ok(())
}

#[instrument]
fn read_file(path: &str) {
    if let Err(e) = std::fs::read_to_string(path) {
        panic!("{}", e);
    }
}

#[instrument]
fn read_config() {
    read_file("fake_file")
}
