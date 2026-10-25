//! com/mojang/blaze3d/platform/MacosUtil.java
//!
//! What the game asks of SDL only on macOS: whether the menu bar shows over a fullscreen window,
//! and whether a click with Ctrl held is a right click, from the options (both off by default).
//! `disableCloseWindowMenuItem`, which hides the Window menu's Close and Close All items (Cmd+W)
//! through Objective-C messages to `NSApplication` (java-objc-bridge in Java), isn't ported: no
//! Mac has run it here, so on a Mac Cmd+W closes the window, where vanilla's menu has no such item.

use crate::org::lwjgl::sdl::sdl_hints::SDLHints;
use crate::org::lwjgl::system::platform::Platform;

pub struct MacosUtil;

impl MacosUtil {
    pub const IS_MACOS: bool = matches!(Platform::get(), Platform::MACOSX);

    pub fn set_fullscreen_menu_visibility(value: bool) {
        if MacosUtil::IS_MACOS {
            SDLHints::SDL_SetHint(
                "SDL_VIDEO_MAC_FULLSCREEN_MENU_VISIBILITY",
                if value { "1" } else { "0" },
            );
        }
    }

    pub fn set_ctrl_click_emulates_right_click(value: bool) {
        if MacosUtil::IS_MACOS {
            SDLHints::SDL_SetHint(
                "SDL_MAC_CTRL_CLICK_EMULATE_RIGHT_CLICK",
                if value { "1" } else { "0" },
            );
        }
    }
}
