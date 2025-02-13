use log::info;
use mac_notification_sys::*;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn show_notification(title: &str, message: &str) {
    // Initialize only once
    INIT.call_once(|| {
        let bundle = get_bundle_identifier_or_default("com.1000ants.gofer2");
        if let Err(e) = set_application(&bundle) {
            info!("Failed to set application bundle: {:?}", e);
        }
    });

    match send_notification("Gofer2", Some(title), message, None) {
        Ok(_) => info!("Notification sent successfully"),
        Err(e) => info!("Failed to send notification: {:?}", e),
    }
}

pub fn request_notification_permission() {
    // Not needed for this crate as it handles permissions internally
    info!("Notification system initialized");
}
