use cocoa::appkit::{NSApplication, NSApplicationActivationPolicy};
use cocoa::base::{id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use log::info;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};

mod clipboard; // <-- import the new clipboard module
mod logger;
mod menu;
mod notification;

fn main() {
    // Initialize our logger early on.
    logger::init_logger();
    info!("Starting Basic Menu Bar App");

    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let app = NSApplication::sharedApplication(nil);
        app.setActivationPolicy_(
            NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory,
        );

        // Request notification permissions
        notification::request_notification_permission();

        // Register our Objective‑C handler class for menu events.
        let handler_class = menu::register_selector();
        let handler: id = msg_send![handler_class, new];

        // Create the status bar item with our custom menu.
        let _status_item = menu::create_status_item(handler);

        // Optionally, you can also listen for app termination notifications.
        let notification_center: id = msg_send![class!(NSNotificationCenter), defaultCenter];
        let quit_notification =
            NSString::alloc(nil).init_str("NSApplicationWillTerminateNotification");
        let _: () = msg_send![notification_center,
            addObserver: handler
            selector: sel!(applicationWillTerminate:)
            name: quit_notification
            object: nil
        ];

        // Start monitoring the clipboard.
        clipboard::start_clipboard_monitor();

        // Run the application.
        app.run();
    }
}
