use color_eyre::eyre;
use eyre::eyre;

#[test]
fn disabled() {
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .install()
        .unwrap();

    let report = eyre!("error occured");

    panic!("Error: {:?}", report);
}
