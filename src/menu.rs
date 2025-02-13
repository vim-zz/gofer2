use cocoa::appkit::{NSMenu, NSMenuItem, NSStatusBar, NSStatusItem};
use cocoa::base::{id, nil, NO};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use log::info;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use std::sync::Once;

static mut STATUS_ITEM: Option<id> = None;
static INIT: Once = Once::new();

/// Registers and returns the pointer to an Objective-C class named "MenuHandler".
/// The class implements two methods:
///    • doAction: – Responds to menu item selections.
///    • applicationWillTerminate: – Gets called when the app is terminating.
pub fn register_selector() -> *const Class {
    unsafe {
        // Start with NSObject as the superclass.
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("MenuHandler", superclass).unwrap();

        // Add the doAction: method.
        decl.add_method(
            sel!(doAction:),
            do_action as extern "C" fn(&Object, Sel, id),
        );

        // Add a simple applicationWillTerminate: method.
        decl.add_method(
            sel!(applicationWillTerminate:),
            application_will_terminate as extern "C" fn(&Object, Sel, id),
        );

        decl.register()
    }
}

/// The implementation for the doAction: Objective‑C method.
/// It grabs a string from the menu item’s representedObject and logs an info message.
extern "C" fn do_action(_this: &Object, _cmd: Sel, item: id) {
    unsafe {
        // Get the represented object from the menu item (a NSString pointer).
        let repr: id = msg_send![item, representedObject];
        let c_str = NSString::UTF8String(repr);
        if !c_str.is_null() {
            let action = std::ffi::CStr::from_ptr(c_str)
                .to_string_lossy()
                .into_owned();
            info!("Menu action invoked: {}", action);
        } else {
            info!("Menu action invoked with no represented object");
        }
    }
}

/// A simple application termination handler that logs a message.
extern "C" fn application_will_terminate(_this: &Object, _cmd: Sel, _notification: id) {
    info!("Application will terminate – cleaning up if necessary.");
}

/// Creates the NSMenu for our status bar item.
pub fn create_menu(handler: id) -> id {
    unsafe {
        let menu = NSMenu::new(nil).autorelease();

        // Create a menu item for "Say Hello".
        let hello_item = create_menu_item(handler, "Say Hello", "hello");
        // Create a menu item for "Say Goodbye".
        let goodbye_item = create_menu_item(handler, "Say Goodbye", "goodbye");
        // Create a Quit menu item.
        let quit_title = NSString::alloc(nil).init_str("Quit");
        let quit_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            quit_title,
            sel!(terminate:),
            NSString::alloc(nil).init_str("q"),
        );

        menu.addItem_(hello_item);
        menu.addItem_(goodbye_item);
        menu.addItem_(quit_item);

        menu
    }
}

/// Helper function to create a single NSMenuItem.
/// The menu item’s representedObject is set to the provided action identifier.
fn create_menu_item(handler: id, title: &str, action_id: &str) -> id {
    unsafe {
        let title_ns = NSString::alloc(nil).init_str(title);
        let item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            title_ns,
            sel!(doAction:),
            NSString::alloc(nil).init_str(""),
        );
        // Set the represented object so that the action identifier can be read.
        let action_ns = NSString::alloc(nil).init_str(action_id);
        let _: () = msg_send![item, setRepresentedObject: action_ns];
        // Set the target to our handler instance.
        let _: () = msg_send![item, setTarget: handler];

        // Set the initial state.
        let _: () = msg_send![item, setState: NO];
        item
    }
}

/// Creates a status bar item with the custom menu attached.
pub fn create_status_item(handler: id) -> id {
    unsafe {
        let status_bar = NSStatusBar::systemStatusBar(nil);
        let status_item: id = msg_send![status_bar, statusItemWithLength: -1.0];

        // Store the status_item reference
        INIT.call_once(|| {
            STATUS_ITEM = Some(status_item);
        });

        // Set a simple title for the status bar button.
        let title = NSString::alloc(nil).init_str("🌟");
        let button: id = msg_send![status_item, button];
        let _: () = msg_send![button, setTitle: title];

        // Set the status item's menu.
        status_item.setMenu_(create_menu(handler));

        status_item
    }
}

pub fn update_menubar_text(text: &str) {
    unsafe {
        if let Some(status_item) = STATUS_ITEM {
            let button: id = msg_send![status_item, button];
            let title = NSString::alloc(nil).init_str(text);
            let _: () = msg_send![button, setTitle: title];
            info!("updated menu: {}", text);
        } else {
            info!("Status item not initialized!");
        }
    }
}
