use std::time::Duration;
use dbus::blocking::Connection;
use dbus::channel::MatchingReceiver;
use dbus::Message;
use dbus::message::MatchRule;
use gtk::prelude::*;
use std::process::Command;
use gtk::gdk::Display;
use gtk::Window;
use gtk::{Application,CssProvider,StyleContext};
const APP_ID: &str = "org.swaynoti.com";
fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = CssProvider::new();
    provider.load_from_data(include_bytes!("style.css"));

    // Add the provider to the default screen
    StyleContext::add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
fn build_ui(app: &Application) {

   
    // Create a window
    let window = Window::builder()
        .application(app)
        .focusable(true)
        .valign(gtk::Align::End)
        .halign(gtk::Align::End)
        .deletable(false)
        .can_focus(true)
        .resizable(false)
        .build();
    window.set_decorated(false);
    window.add_css_class("application");
    // Presset_focusableent window
    window.show();
}


// This programs implements the equivalent of running the "dbus-monitor" tool
fn main() {
    // First open up a connection to the session bus.
    let conn = Connection::new_session().expect("D-Bus connection failed");

    // Second create a rule to match messages we want to receive; in this example we add no
    // further requirements, so all messages will match
    let mut rule = MatchRule::new();
    rule = rule.with_path("/org/freedesktop/Notifications");
    rule = rule.with_member("Notify");

    // Try matching using new scheme
    let proxy = conn.with_proxy("org.freedesktop.DBus", "/org/freedesktop/Notifications", Duration::from_millis(5000));
    let result: Result<(), dbus::Error> = proxy.method_call("org.freedesktop.Notifications", "Notify", (vec!(rule.match_str()), 0u32));

    if result.is_ok() {
        // Start matching using new scheme
        conn.start_receive(rule, Box::new(|msg, _| {
            handle_message(&msg);
            true
        }));
    } else {
        // Start matching using old scheme
        rule.eavesdrop = true; // this lets us eavesdrop on *all* session messages, not just ours
        conn.add_match(rule, |_: (), _, msg| {
            handle_message(&msg);
            true
        }).expect("add_match failed");
    }

    // Loop and print out all messages received (using handle_message()) as they come.
    // Some can be quite large, e.g. if they contain embedded images..
    loop { conn.process(Duration::from_millis(1000)).unwrap(); };
}

fn handle_message(msg: &Message) { 
    let data = msg.iter_init();
    for value in data {   
      println!("value is {:?}",value);  
    }
    Command::new("pkill").arg("-RTMIN+8").arg("waybar").spawn().expect("Can't run this command");
        // Create a new application
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| load_css());
     // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);
    // Run the application
    app.run();

     
}
