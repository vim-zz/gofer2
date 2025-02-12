use cocoa::appkit::NSPasteboard;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSDefaultRunLoopMode, NSString};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use log::info;

// A (static) variable to keep track of the last seen change count.
// Because the timer runs on the main thread, access here is safe.
static mut LAST_CHANGE_COUNT: i64 = 0;

/// This is the callback method that will be called by NSTimer.
/// It checks the pasteboard’s changeCount, and if the value has changed,
/// it reads the text (if any) and logs it.
extern "C" fn check_pasteboard(this: &Object, _cmd: Sel, _timer: id) {
    unsafe {
        // Get the general pasteboard.
        let pasteboard: id = NSPasteboard::generalPasteboard(nil);
        // Get the current change count (an NSInteger, here we use i64).
        let current_count: i64 = msg_send![pasteboard, changeCount];
        // If the pasteboard has changed...
        if current_count != LAST_CHANGE_COUNT {
            LAST_CHANGE_COUNT = current_count;
            // We try to extract the string data.
            // (We use "public.utf8-plain-text" as the type; older apps might use "NSStringPboardType".)
            let type_str = NSString::alloc(nil).init_str("public.utf8-plain-text");
            let copied_text: id = msg_send![pasteboard, stringForType: type_str];
            if copied_text != nil {
                let c_str = NSString::UTF8String(copied_text);
                if !c_str.is_null() {
                    let text = std::ffi::CStr::from_ptr(c_str).to_string_lossy();
                    info!("Clipboard changed – new text: {}", text);
                }
            } else {
                info!("Clipboard changed – but no text found.");
            }
        }
    }
}

/// Starts the clipboard monitor: this function creates an Objective‑C class
/// named "ClipboardMonitor" implementing a method to check the pasteboard.
/// Then it schedules an NSTimer to call that method every second.
pub fn start_clipboard_monitor() {
    unsafe {
        // Create a new Objective‑C class "ClipboardMonitor" that subclasses NSObject.
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("ClipboardMonitor", superclass).unwrap();
        // Add our check_pasteboard: method.
        decl.add_method(sel!(checkPasteboard:), check_pasteboard as extern "C" fn(&Object, Sel, id));
        let cls = decl.register();

        // Create an instance of ClipboardMonitor.
        let monitor: id = msg_send![cls, new];

        // Schedule a repeating NSTimer that fires every 1 second.
        let _timer: id = msg_send![class!(NSTimer),
            scheduledTimerWithTimeInterval: 1.0
            target: monitor
            selector: sel!(checkPasteboard:)
            userInfo: nil
            repeats: 1]; // YES is 1 in Objective‑C

        // Optionally add the timer to the run loop (though scheduledTimerWithTimeInterval does
        // this automatically).
        let run_loop: id = msg_send![class!(NSRunLoop), currentRunLoop];
        let _: () = msg_send![run_loop, addTimer: _timer forMode: NSDefaultRunLoopMode];
    }
}
