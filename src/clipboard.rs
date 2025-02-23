// src/clipboard.rs
use crate::data;
use crate::menu;
use crate::notification;
use cocoa::appkit::NSPasteboard;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSDefaultRunLoopMode, NSString};
use log::info;
use objc::declare::ClassDecl;
use objc::runtime::{Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use std::sync::Mutex;
use std::time::{Duration, Instant};

// Structure to keep track of clipboard state
struct ClipboardState {
    last_change_count: i64,
    last_content: String,
    last_copy_time: Instant,
    consecutive_copies: u32,
}

// Use a Mutex for thread-safe interior mutability
lazy_static::lazy_static! {
    static ref CLIPBOARD_STATE: Mutex<ClipboardState> = Mutex::new(ClipboardState {
        last_change_count: 0,
        last_content: String::new(),
        last_copy_time: Instant::now(),
        consecutive_copies: 0,
    });
}

/// Get the current clipboard text content
unsafe fn get_clipboard_text(pasteboard: id) -> Option<String> {
    unsafe {
        let type_str = NSString::alloc(nil).init_str("public.utf8-plain-text");
        let copied_text: id = msg_send![pasteboard, stringForType: type_str];
        if copied_text != nil {
            let c_str = NSString::UTF8String(copied_text);
            if !c_str.is_null() {
                return Some(
                    std::ffi::CStr::from_ptr(c_str)
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
        None
    }
}

extern "C" fn check_pasteboard(_this: &Object, _cmd: Sel, _timer: id) {
    unsafe {
        let mut state = CLIPBOARD_STATE.lock().unwrap();
        let pasteboard: id = NSPasteboard::generalPasteboard(nil);
        let current_count: i64 = msg_send![pasteboard, changeCount];

        // If the pasteboard has changed...
        if current_count != state.last_change_count {
            state.last_change_count = current_count;

            if let Some(current_text) = get_clipboard_text(pasteboard) {
                let now = Instant::now();
                let time_since_last_copy = now.duration_since(state.last_copy_time);

                if current_text == state.last_content
                    && time_since_last_copy < Duration::from_secs(1)
                {
                    state.consecutive_copies += 1;

                    if state.consecutive_copies == 2 {
                        info!("Double copy detected! Text: {}", current_text);

                        // Look up the target text
                        if let Some(mapping) = data::find_target(&current_text) {
                            // Add new menu item with translation
                            menu::add_menu_item(&current_text, &mapping.value);

                            // Show notification
                            notification::show_notification(
                                format!("{} to {}", mapping.source_name, mapping.target_name)
                                    .as_str(),
                                format!("{} → {}", current_text, mapping.value).as_str(),
                            );
                        } else {
                            // No mapping found
                            notification::show_notification(
                                "No mapping found",
                                &format!("No target text found for: {}", current_text),
                            );
                        }

                        // Reset consecutive copies after processing
                        state.consecutive_copies = 0;
                    }
                } else {
                    // Reset if it's different text or too much time has passed
                    state.consecutive_copies = 1;
                }

                state.last_content = current_text;
                state.last_copy_time = now;
            } else {
                // Reset if no text content
                state.consecutive_copies = 0;
                state.last_content.clear();
            }
        }
    }
}

pub fn start_clipboard_monitor() {
    unsafe {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("ClipboardMonitor", superclass).unwrap();
        decl.add_method(
            sel!(checkPasteboard:),
            check_pasteboard as extern "C" fn(&Object, Sel, id),
        );
        let cls = decl.register();

        let monitor: id = msg_send![cls, new];

        let _timer: id = msg_send![class!(NSTimer),
            scheduledTimerWithTimeInterval: 0.1  // Check more frequently (every 100ms)
            target: monitor
            selector: sel!(checkPasteboard:)
            userInfo: nil
            repeats: 1];

        let run_loop: id = msg_send![class!(NSRunLoop), currentRunLoop];
        let _: () = msg_send![run_loop, addTimer: _timer forMode: NSDefaultRunLoopMode];
    }
}
