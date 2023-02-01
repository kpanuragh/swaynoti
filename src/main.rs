use std::time::Duration;

use dbus::blocking::Connection;
use dbus::channel::MatchingReceiver;
use dbus::Message;
use dbus::message::MatchRule;
use gtk::prelude::*;

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("First GTK+ Program");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(350, 70);

    let button = gtk::Button::with_label("Click me!");

    window.add(&button);

    window.show_all();
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
    println!("Got message: {:?}", msg.get_items()[3]);
     let application =
        gtk::Application::new(Some("com.github.gtk-rs.examples.basi"), Default::default());

    application.connect_activate(build_ui);

    application.run();
}
