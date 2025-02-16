// src/menu.rs
use cocoa::appkit::{NSEventModifierFlags, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem};
use cocoa::base::{id, nil, BOOL, NO, YES};
use cocoa::foundation::{NSAutoreleasePool, NSSize, NSString};
use log::info;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use std::path::PathBuf;
use std::sync::Once;

static mut STATUS_ITEM: Option<id> = None;
static INIT: Once = Once::new();
static mut MENU: Option<id> = None;
static mut HANDLER: Option<id> = None;

fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

extern "C" fn order_front_standard_about_panel(_this: &Object, _cmd: Sel, _sender: id) {
    unsafe {
        let app: id = msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![app, orderFrontStandardAboutPanel: nil];
    }
}

extern "C" fn show_help_panel(_this: &Object, _cmd: Sel, _sender: id) {
    unsafe {
        let alert: id = msg_send![class!(NSAlert), new];

        // Set alert properties
        let title = NSString::alloc(nil).init_str("Gofer2 Help");
        let _: () = msg_send![alert, setMessageText:title];

        let message = NSString::alloc(nil).init_str(
            "How to use Gofer2:\n\n\
            1. Add CSV files to ~/.gofer2/ directory with two columns:\n\
               English,French\n\
               hello,bonjour\n\
               goodbye,au revoir\n\n\
            2. Double-copy (⌘C twice quickly) any text to look up its translation\n\n\
            3. When a translation is found:\n\
               • A notification will appear\n\
               • The translation will be added to this menu\n\n\
            4. Click any translation in the menu to copy it to clipboard",
        );
        let _: () = msg_send![alert, setInformativeText:message];

        // Show the alert
        let _: () = msg_send![alert, runModal];
    }
}

pub fn register_selector() -> *const Class {
    unsafe {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("MenuHandler", superclass).unwrap();

        decl.add_method(
            sel!(doAction:),
            do_action as extern "C" fn(&Object, Sel, id),
        );

        decl.add_method(
            sel!(applicationWillTerminate:),
            application_will_terminate as extern "C" fn(&Object, Sel, id),
        );

        decl.add_method(
            sel!(orderFrontStandardAboutPanel:),
            order_front_standard_about_panel as extern "C" fn(&Object, Sel, id),
        );

        decl.add_method(
            sel!(showHelpPanel:),
            show_help_panel as extern "C" fn(&Object, Sel, id),
        );

        decl.register()
    }
}

fn copy_to_clipboard(text: &str) {
    unsafe {
        let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
        let _: () = msg_send![pasteboard, clearContents];

        let ns_string = NSString::alloc(nil).init_str(text);
        let _: BOOL = msg_send![pasteboard,
            setString:ns_string
            forType:NSString::alloc(nil).init_str("public.utf8-plain-text")
        ];

        info!("Copied to clipboard: {}", text);
    }
}

extern "C" fn do_action(_this: &Object, _cmd: Sel, item: id) {
    unsafe {
        let title: id = msg_send![item, title];
        if title != nil {
            let c_str = NSString::UTF8String(title);
            if !c_str.is_null() {
                let text = std::ffi::CStr::from_ptr(c_str)
                    .to_string_lossy()
                    .into_owned();
                copy_to_clipboard(&text);
            }
        }
    }
}

extern "C" fn application_will_terminate(_this: &Object, _cmd: Sel, _notification: id) {
    info!("Application will terminate – cleaning up if necessary.");
}

pub fn create_menu(handler: id) -> id {
    unsafe {
        let menu = NSMenu::new(nil).autorelease();

        // Store menu reference
        MENU = Some(menu);

        // Add Separator before About
        let separator1 = NSMenuItem::separatorItem(nil);
        menu.addItem_(separator1);

        // Add About item
        let about_title =
            NSString::alloc(nil).init_str(&format!("Gofer2 (v{})", get_app_version()));
        let about_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            about_title,
            sel!(orderFrontStandardAboutPanel:),
            NSString::alloc(nil).init_str(""),
        );
        about_item.setTarget_(handler);
        menu.addItem_(about_item);

        // Add Help item
        let help_title = NSString::alloc(nil).init_str("Help");
        let help_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            help_title,
            sel!(showHelpPanel:),
            NSString::alloc(nil).init_str("h"),
        );
        help_item.setTarget_(handler);
        menu.addItem_(help_item);

        // Add Separator after About
        let separator2 = NSMenuItem::separatorItem(nil);
        menu.addItem_(separator2);

        // Add Quit item
        let quit_title = NSString::alloc(nil).init_str("Quit");
        let quit_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            quit_title,
            sel!(terminate:),
            NSString::alloc(nil).init_str("q"),
        );

        // Set Command modifier for Quit shortcut
        let _: () = msg_send![quit_item, setKeyEquivalentModifierMask: NSEventModifierFlags::NSCommandKeyMask];

        menu.addItem_(quit_item);

        menu
    }
}

pub fn add_menu_item(source: &str, target: &str) {
    unsafe {
        if let Some(menu) = MENU {
            if let Some(handler) = HANDLER {
                // Create parent menu item with source text
                let source_title = NSString::alloc(nil).init_str(source);
                let source_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
                    source_title,
                    sel!(doAction:),
                    NSString::alloc(nil).init_str(""),
                );

                // Enable the menu item and set its action
                let _: () = msg_send![source_item, setEnabled:YES];
                let _: () = msg_send![source_item, setTarget:handler];
                let _: () = msg_send![source_item, setAction:sel!(doAction:)];

                // Create submenu for the translation
                let submenu = NSMenu::new(nil).autorelease();

                // Create translation menu item
                let target_title = NSString::alloc(nil).init_str(target);
                let target_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
                    target_title,
                    sel!(doAction:),
                    NSString::alloc(nil).init_str(""),
                );

                // Enable the submenu item and set its action
                let _: () = msg_send![target_item, setEnabled:YES];
                let _: () = msg_send![target_item, setTarget:handler];
                let _: () = msg_send![target_item, setAction:sel!(doAction:)];

                // Add translation to submenu
                submenu.addItem_(target_item);

                // Set submenu to parent item
                let _: () = msg_send![source_item, setSubmenu:submenu];

                // Insert at the top (index 0), before the Quit item
                let _: () = msg_send![menu, insertItem:source_item atIndex:0];

                // Limit the number of items (optional)
                let count: i64 = msg_send![menu, numberOfItems];
                if count > 10 {
                    // Keep last 10 items plus Quit
                    let _: () = msg_send![menu, removeItemAtIndex:count - 2];
                }
            }
        }
    }
}

pub fn create_status_item(handler: id) -> id {
    unsafe {
        HANDLER = Some(handler); // Store handler reference
        let status_bar = NSStatusBar::systemStatusBar(nil);
        let status_item: id = msg_send![status_bar, statusItemWithLength: -1.0];

        INIT.call_once(|| {
            STATUS_ITEM = Some(status_item);
        });

        let button: id = msg_send![status_item, button];

        let image = load_status_bar_image();
        let _: () = msg_send![button, setImage: image];

        status_item.setMenu_(create_menu(handler));

        status_item
    }
}

fn load_status_bar_image() -> id {
    unsafe {
        let image: id = msg_send![class!(NSImage), new];
        let size = NSSize::new(16.0, 16.0);
        let _: () = msg_send![image, setSize:size];

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("images")
            .join("icon_16x16.png");

        let path_str = path.to_str().unwrap();
        let path_ns = NSString::alloc(nil).init_str(path_str);

        let _: () = msg_send![image, initWithContentsOfFile:path_ns];
        let _: () = msg_send![image, setTemplate:YES as BOOL];

        image
    }
}
