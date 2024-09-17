use std::{
    collections::HashMap,
    io::Read,
    os::unix::net::{UnixListener, UnixStream},
    process::exit,
};

use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use librunixeer::{Action, ListItem, SubListItem};

const SOCKET_NAME: &str = "desktopentries.sock";

#[derive(Debug, Default)]
struct App {
    pub items: HashMap<u64, ListItem>,
}

impl App {
    pub fn get_all_in_known_locations(&mut self) {
        let locales = get_languages_from_env();
        for (idx, entry) in Iter::new(default_paths())
            .entries(Some(&locales))
            .enumerate()
        {
            let name = entry.name(&locales).unwrap();
            if entry.exec().is_some() {
                let exec = entry.parse_exec().unwrap();
                let index = idx as u64;
                match entry.actions() {
                    Some(actions) => {
                        let mut subitems = vec![];
                        for (idx, action) in actions.iter().enumerate() {
                            if let Some(action_name) = entry.action_name(action, &locales) {
                                subitems
                                    .push(SubListItem::new(idx as u64, action_name.to_string()));
                            }
                        }
                        self.items.insert(
                            index,
                            ListItem::with_subitems(index, name.to_string(), subitems),
                        );
                    }
                    None => {
                        self.items
                            .insert(index, ListItem::new(index, name.to_string()));
                    }
                }
            }
        }
    }

    pub fn handle_stream(&self, mut unix_stream: UnixStream) {
        let mut message = String::new();
        unix_stream
            .read_to_string(&mut message)
            .expect("Could not read the message.");
        let action: Action = ron::from_str(&message).expect("Message is not valid.");
        match action {
            Action::GetList => {
                // TODO: reply w/ ListItems
                println!("Got it, getting desktop entries.");
            }
            Action::Refresh => {}
            Action::Exit => {
                match std::fs::remove_file(SOCKET_NAME) {
                    Ok(_) => exit(0),
                    Err(e) => {
                        eprintln!("Could not delete the socket file at {}. {e}", SOCKET_NAME);
                        exit(-1);
                    }
                };
            }
        };
    }
}

fn main() {
    if std::fs::metadata(SOCKET_NAME).is_ok() {
        eprintln!("{} is active, exiting.", SOCKET_NAME);
        exit(17);
    }
    let mut app = App::default();
    let unix_listener =
        UnixListener::bind(SOCKET_NAME).expect("Could not bind to the socket file.");
    loop {
        let (unix_stream, _socket_address) = unix_listener
            .accept()
            .expect("Could not accept connection.");
        app.handle_stream(unix_stream);
    }
}
