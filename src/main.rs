use std::process::exit;

use args::Args;
use clap::Parser;
use config::{get_configuration_path, Configuration};
use gtk::{
    glib::Propagation,
    prelude::{ApplicationExt, ApplicationExtManual, EditableExt, GtkWindowExt},
    Application, ApplicationWindow, SearchEntry,
};
use pinned::APP_ID;
use pm::PluginManager;
mod args;
mod config;
mod pinned;
mod plugin;
mod pm;

fn main() {
    let args = Args::parse();
    let configuration_path = get_configuration_path(args);
    let configuration_file_contents =
        std::fs::read_to_string(configuration_path).expect("Could not read configuration file:");
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(move |app: &Application| {
        let configuration: Configuration = ron::from_str(&configuration_file_contents)
            .expect("Could not parse the configuration file:");
        let mut pm = PluginManager::from_configuration(configuration)
            .expect("Could not instantiate Plugin Manager via configuration file.");
        pm.start_enabled_plugins().unwrap();
        build_ui(app, pm);
    });
    app.run();
}

fn handle_search(se: &SearchEntry) {
    let entry = se.text().to_string();
}

fn build_ui(app: &Application, pm: PluginManager) {
    let search_entry = SearchEntry::builder()
        .placeholder_text("{Plugin Name}")
        .margin_top(0)
        .margin_start(0)
        .margin_end(0)
        .build();
    search_entry.connect_changed(|se: &SearchEntry| handle_search(&se));
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Runixeer")
        // Interestingly resizable makes window poped up at the center of the curr monitor in
        // hyprland
        .resizable(false)
        .child(&search_entry)
        .focus_widget(&search_entry)
        .build();
    window.connect_close_request(move |_| {
        pm.exit_all().unwrap();
        exit(-1);
    });
    window.present();
}
