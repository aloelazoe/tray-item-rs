use crate::TIError;
use cocoa::{
    appkit::{
        NSApp, NSApplication, NSImage, NSMenu, NSMenuItem, NSStatusBar,
        NSStatusItem, NSWindow, NSApplicationActivationPolicyAccessory,
    },
    base::{nil},
    foundation::{NSAutoreleasePool, NSString},
};
use objc::{msg_send, sel, sel_impl};
use std::thread::JoinHandle;

mod callback;
use callback::*;

pub struct TrayItemMacOS {
    name: String,
    menu: *mut objc::runtime::Object,
    _pool: *mut objc::runtime::Object,
    icon: Option<*mut objc::runtime::Object>,
    main_thread: Option<JoinHandle<()>>,
    app_delegate: Option<*mut objc::runtime::Object>,
}

impl TrayItemMacOS {
    pub fn new(title: &str, icon: &str) -> Result<Self, TIError> {
        unsafe {
            let pool = NSAutoreleasePool::new(nil);

            let icon = Some(icon).filter(|icon| !icon.is_empty());
            let icon = icon.map(|icon_name| {
                let icon_name = NSString::alloc(nil).init_str(icon_name);
                NSImage::imageNamed_(NSImage::alloc(nil), icon_name)
            });

            let t = TrayItemMacOS {
                name: title.to_string(),
                _pool: pool,
                icon,
                menu: NSMenu::new(nil).autorelease(),
                main_thread: None,
                app_delegate: None,
            };

            // t.display();

            Ok(t)
        }
    }

    pub fn set_icon(&mut self, icon: &str) -> Result<(), TIError> {
        unsafe {
            let icon_name = NSString::alloc(nil).init_str(icon);
            self.icon = Some(NSImage::imageNamed_(NSImage::alloc(nil), icon_name));
        }
        Ok(())
    }

    pub fn add_label(&mut self, label: &str) -> Result<(), TIError> {
        unsafe {
            let no_key = NSString::alloc(nil).init_str(""); // TODO want this eventually
            let itemtitle = NSString::alloc(nil).init_str(label);
            let action = sel!(call);
            let item = NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(itemtitle, action, no_key);
            let _: () = msg_send![item, setTitle: itemtitle];

            NSMenu::addItem_(self.menu, item);
        }

        Ok(())
    }

    pub fn add_menu_item<F>(&mut self, label: &str, cb: F) -> Result<(), TIError>
    where
        F: FnMut() -> () + 'static,
    {
        let cb_obj = Callback::from(Box::new(cb));

        unsafe {
            let no_key = NSString::alloc(nil).init_str(""); // TODO want this eventually
            let itemtitle = NSString::alloc(nil).init_str(label);
            let action = sel!(call);
            let item = NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(itemtitle, action, no_key);
            let _: () = msg_send![item, setTarget: cb_obj];

            NSMenu::addItem_(self.menu, item);
        }

        Ok(())
    }

    // private

    pub fn add_quit_item(&mut self, label: &str) {
        unsafe {
            let no_key = NSString::alloc(nil).init_str("");
            let pref_item = NSString::alloc(nil).init_str(label);
            let pref_action = sel!(terminate:);
            let menuitem = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
                pref_item,
                pref_action,
                no_key,
            );

            self.menu.addItem_(menuitem);
        }
    }

    /// should take the output of `cocoa::delegate` macro as an argument
    /// https://docs.rs/cocoa/0.24.0/cocoa/macro.delegate.html
    // pub unsafe fn set_app_delegate(&mut self, delegate: *mut objc::runtime::Object) {
    //     self.app_delegate = Some(delegate);
    // }

    pub fn display(&mut self) {
        unsafe {
            // let app = NSApp();
            // start without a dock icon
            // app.setActivationPolicy_(NSApplicationActivationPolicyAccessory);

            // if let Some(delegate) = self.app_delegate {
            //     app.setDelegate_(delegate);
            // }

            let item = NSStatusBar::systemStatusBar(nil).statusItemWithLength_(-1.0);
            let title = NSString::alloc(nil).init_str(&self.name);
            if let Some(icon) = self.icon {
                let _: () = msg_send![item, setImage: icon];
            } else {
                item.setTitle_(title);
            }
            item.setMenu_(self.menu);

            // app.run();
        }
    }
}

impl Drop for TrayItemMacOS {
    fn drop(&mut self) {
        match self.main_thread.take() {
            Some(t) => t.join(),
            None => Ok(()),
        }
        .unwrap()
    }
}
