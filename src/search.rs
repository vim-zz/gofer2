// src/search.rs
use cocoa::appkit::{NSApp, NSWindowStyleMask};
use cocoa::base::{NO, YES, id, nil};
use cocoa::foundation::{NSPoint, NSRect, NSSize, NSString};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use log::info;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use std::sync::Mutex;

use crate::data;

lazy_static::lazy_static! {
    static ref SEARCH_RESULTS: Mutex<Vec<SearchResult>> = Mutex::new(Vec::new());
}

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub source: String,
    pub target: String,
    pub score: i64,
}

// Register our delegate classes
pub fn register_search_delegates() -> *const Class {
    unsafe {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("SearchDelegate", superclass).unwrap();

        // Add method for handling text changes
        decl.add_method(
            sel!(controlTextDidChange:),
            text_did_change as extern "C" fn(&Object, Sel, id),
        );

        // Add required table view data source methods
        decl.add_method(
            sel!(numberOfRowsInTableView:),
            number_of_rows as extern "C" fn(&Object, Sel, id) -> i64,
        );

        decl.add_method(
            sel!(tableView:objectValueForTableColumn:row:),
            table_view_value as extern "C" fn(&Object, Sel, id, id, i64) -> id,
        );

        decl.register()
    }
}

// Add these methods to handle the table view data
extern "C" fn number_of_rows(_this: &Object, _cmd: Sel, _table_view: id) -> i64 {
    SEARCH_RESULTS.lock().unwrap().len() as i64
}

extern "C" fn table_view_value(
    _this: &Object,
    _cmd: Sel,
    _table_view: id,
    column: id,
    row: i64,
) -> id {
    unsafe {
        let results = SEARCH_RESULTS.lock().unwrap();
        if row >= 0 && (row as usize) < results.len() {
            let result = &results[row as usize];
            let identifier: id = msg_send![column, identifier];
            let id_str = NSString::UTF8String(identifier);

            if !id_str.is_null() {
                let column_id = std::ffi::CStr::from_ptr(id_str).to_string_lossy();

                let value = if column_id == "source" {
                    &result.source
                } else {
                    &result.target
                };

                return NSString::alloc(nil).init_str(value);
            }
        }
        nil
    }
}

pub extern "C" fn show_search_window(_this: &Object, _cmd: Sel, _sender: id) {
    unsafe {
        // Create window
        let window: id = msg_send![class!(NSWindow), alloc];
        let frame = NSRect::new(NSPoint::new(0., 0.), NSSize::new(400., 300.));
        let style_mask = NSWindowStyleMask::NSTitledWindowMask
            | NSWindowStyleMask::NSClosableWindowMask
            | NSWindowStyleMask::NSMiniaturizableWindowMask;

        let window: id = msg_send![window,
            initWithContentRect:frame
            styleMask:style_mask
            backing:2
            defer:NO
        ];

        // Set window properties
        let title = NSString::alloc(nil).init_str("Gofer2 Search");
        let _: () = msg_send![window, setTitle:title];
        let _: () = msg_send![window, center];
        // Make the window float above others
        let _: () = msg_send![window, setLevel: 3];

        // Create text field
        let text_frame = NSRect::new(NSPoint::new(20., 260.), NSSize::new(360., 25.));
        let text_field: id = msg_send![class!(NSTextField), alloc];
        let text_field: id = msg_send![text_field, initWithFrame:text_frame];

        // Set up text field delegate
        let text_delegate: id = msg_send![class!(SearchDelegate), new];
        let _: () = msg_send![text_field, setDelegate:text_delegate];

        // Create scroll view
        let scroll_frame = NSRect::new(NSPoint::new(20., 20.), NSSize::new(360., 220.));
        let scroll_view: id = msg_send![class!(NSScrollView), alloc];
        let scroll_view: id = msg_send![scroll_view, initWithFrame:scroll_frame];

        // Create table view
        let table_view: id = msg_send![class!(NSTableView), alloc];
        let table_view: id = msg_send![table_view, initWithFrame:scroll_frame];

        // Set the delegate (which is also our data source)
        let delegate: id = msg_send![class!(SearchDelegate), new];
        let _: () = msg_send![table_view, setDataSource:delegate];
        let _: () = msg_send![table_view, setDelegate:delegate];

        // Add columns
        let column1: id = msg_send![class!(NSTableColumn), alloc];
        let column1: id =
            msg_send![column1, initWithIdentifier:NSString::alloc(nil).init_str("source")];
        let _: () = msg_send![column1, setWidth:160.0];
        let _: () = msg_send![column1, setMinWidth:50.0];
        let _: () = msg_send![column1, setMaxWidth:1000.0];
        let _: () = msg_send![column1, setResizingMask:2]; // NSTableColumnUserResizingMask
        let _: () = msg_send![column1, setTitle:NSString::alloc(nil).init_str("Source")];
        let _: () = msg_send![table_view, addTableColumn:column1];

        let column2: id = msg_send![class!(NSTableColumn), alloc];
        let column2: id =
            msg_send![column2, initWithIdentifier:NSString::alloc(nil).init_str("target")];
        let _: () = msg_send![column2, setWidth:160.0];
        let _: () = msg_send![column2, setMinWidth:50.0];
        let _: () = msg_send![column2, setMaxWidth:1000.0];
        let _: () = msg_send![column2, setResizingMask:2]; // NSTableColumnUserResizingMask
        let _: () = msg_send![column2, setTitle:NSString::alloc(nil).init_str("Target")];
        let _: () = msg_send![table_view, addTableColumn:column2];

        // After adding columns, set up auto-resizing
        let _: () = msg_send![table_view, setColumnAutoresizingStyle:1]; // NSTableViewUniformColumnAutoresizingStyle

        // Make sure table view uses the full width of the scroll view
        let _: () = msg_send![scroll_view, setAutoresizingMask:18]; // NSViewWidthSizable | NSViewHeightSizable

        // Configure scroll view
        let _: () = msg_send![scroll_view, setDocumentView:table_view];
        let _: () = msg_send![scroll_view, setHasVerticalScroller:YES];
        let _: () = msg_send![scroll_view, setHasHorizontalScroller:NO];
        let _: () = msg_send![scroll_view, setBorderType:2];

        // Configure table view
        let _: () = msg_send![table_view, setAllowsMultipleSelection:NO];
        let _: () = msg_send![table_view, setAllowsEmptySelection:YES];

        // Add views to window
        let content_view: id = msg_send![window, contentView];
        let _: () = msg_send![content_view, addSubview:text_field];
        let _: () = msg_send![content_view, addSubview:scroll_view];

        // Activate the app and bring window to front
        let app: id = msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![app, activateIgnoringOtherApps:YES];
        let _: () = msg_send![window, makeKeyAndOrderFront:nil];
    }
}

extern "C" fn text_did_change(_this: &Object, _cmd: Sel, notification: id) {
    unsafe {
        let text_field: id = msg_send![notification, object];
        let string_value: id = msg_send![text_field, stringValue];
        let chars = NSString::UTF8String(string_value);

        if !chars.is_null() {
            let query = std::ffi::CStr::from_ptr(chars)
                .to_string_lossy()
                .into_owned();

            info!("Search query: {}", query);
            if let Some(results) = search_mappings(&query) {
                info!("Found {} results", results.len());
                *SEARCH_RESULTS.lock().unwrap() = results;

                // Find and reload the table view
                if let Some(window) = find_window_with_title("Gofer2 Search") {
                    let content_view: id = msg_send![window, contentView];
                    let subviews: id = msg_send![content_view, subviews];
                    let count: usize = msg_send![subviews, count];

                    for i in 0..count {
                        let view: id = msg_send![subviews, objectAtIndex:i];
                        if msg_send![view, isKindOfClass:class!(NSScrollView)] {
                            let doc_view: id = msg_send![view, documentView];
                            if msg_send![doc_view, isKindOfClass:class!(NSTableView)] {
                                info!("Reloading table view");
                                let _: () = msg_send![doc_view, reloadData];
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn search_mappings(query: &str) -> Option<Vec<SearchResult>> {
    if query.is_empty() {
        return Some(Vec::new());
    }

    let matcher = SkimMatcherV2::default();
    let mut results = Vec::new();

    if let Some(mappings) = data::get_all_mappings() {
        for (source, mapping) in mappings {
            // Search in source
            if let Some(score) = matcher.fuzzy_match(&source, query) {
                results.push(SearchResult {
                    source: source.clone(),
                    target: mapping.value.clone(),
                    score,
                });
            }
            // Search in target
            if let Some(score) = matcher.fuzzy_match(&mapping.value, query) {
                results.push(SearchResult {
                    source: source.clone(),
                    target: mapping.value.clone(),
                    score,
                });
            }

            // Don't go over 10 results to save time
            if results.len() > 10 {
                break;
            }
        }
    }

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results.dedup_by(|a, b| a.source == b.source && a.target == b.target);
    Some(results)
}

unsafe fn find_window_with_title(title: &str) -> Option<id> {
    let windows: id = msg_send![NSApp(), windows];
    let count: usize = msg_send![windows, count];

    for i in 0..count {
        let window: id = msg_send![windows, objectAtIndex:i];
        let window_title: id = msg_send![window, title];
        let title_str = NSString::UTF8String(window_title);
        if !title_str.is_null() {
            let window_title = std::ffi::CStr::from_ptr(title_str).to_string_lossy();
            if window_title == title {
                return Some(window);
            }
        }
    }
    None
}
