fn main() {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .theme(color_eyre::config::Theme::new())
        .into_hooks();

    eyre_hook
        .install()
        .expect("there won't be any conflicting eyre hooks previously installed");

    std::panic::set_hook(Box::new(move |panic_info| {
        let panic_report = panic_hook.panic_report(panic_info);
        rfd::MessageDialog::new()
            .set_title("App Crash")
            .set_description(&format!("{}", &panic_report))
            .set_level(rfd::MessageLevel::Error)
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
    }));

    panic!("oh no I forgot to write the rest of my program!");
}
