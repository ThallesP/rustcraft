//! net/minecraft/client/gui/screens/TitleScreen.java
//!
//! The main menu: the panorama, the logo, the menu's buttons (singleplayer, multiplayer, Realms,
//! the friends, language and accessibility icons, the options and quitting), the version in the
//! bottom left and the copyright in the bottom right. Shown after the game loads, it fades in over
//! two seconds of the wall clock, the widgets in the second.
//!
//! Not ported yet: the splash text (`SplashManager`, `SplashRenderer`: a random line of
//! `texts/splashes.txt` or a date's, pulsing by the wall clock), the Realms notifications (an
//! offline user's Realms client isn't valid, so vanilla draws none), the demo's menu, and the
//! screens the singleplayer, Realms, friends, language and accessibility buttons open, which say
//! so. `checkModStatus` is the unmodified game's: the version reads "Minecraft 26.3".

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use crate::net::minecraft::client::gui::components::abstract_widget::WidgetRef;
use crate::net::minecraft::client::gui::components::button::{Button, OnPress, PlainTextButton};
use crate::net::minecraft::client::gui::components::common_buttons::CommonButtons;
use crate::net::minecraft::client::gui::components::logo_renderer::LogoRenderer;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::gui::screens::options::options_screen::OptionsScreen;
use crate::net::minecraft::client::gui::screens::screen::{self, Screen, ScreenClass, ScreenRef};
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::shared_constants::SharedConstants;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::util::Util;

pub struct TitleScreen {
    screen: Screen,
    /// The widgets the screen added, Java's `children()` that are `AbstractWidget`s, which
    /// `fadeWidgets` sets the alpha of.
    widgets: Vec<WidgetRef>,
    fading: bool,
    fade_in_start: i64,
    logo_renderer: LogoRenderer,
}

impl TitleScreen {
    /// `TitleScreen(boolean)`: the panorama spins again.
    pub fn new(minecraft: &Rc<Minecraft>, fading: bool) -> TitleScreen {
        minecraft.panorama.borrow_mut().start_spin();
        TitleScreen {
            screen: Screen::new(
                minecraft,
                Component::translatable("narrator.screen.title").into(),
            ),
            widgets: Vec::new(),
            fading,
            fade_in_start: 0,
            logo_renderer: LogoRenderer::new(false),
        }
    }

    /// The screen as the game shows it.
    pub fn screen_ref(minecraft: &Rc<Minecraft>, fading: bool) -> ScreenRef {
        Rc::new(RefCell::new(TitleScreen::new(minecraft, fading)))
    }

    fn add(&mut self, widget: Button) -> WidgetRef {
        let widget: WidgetRef = Rc::new(RefCell::new(widget));
        self.screen.add_renderable_widget_ref(&widget);
        self.widgets.push(Rc::clone(&widget));
        widget
    }

    fn get_horizontal_position(
        &self,
        current_button: i32,
        number_of_buttons: i32,
        button_width: i32,
    ) -> i32 {
        let total_width = number_of_buttons * button_width + (number_of_buttons - 1) * 4;
        self.screen.width / 2 - total_width / 2 + (current_button - 1) * (button_width + 4)
    }

    /// `createNormalMenuOptions`: singleplayer, multiplayer and Realms, a row each; where the next
    /// row goes.
    fn create_normal_menu_options(&mut self, top_pos: i32, spacing: i32) -> i32 {
        let width = self.screen.width;
        self.add(
            Button::builder(
                Component::translatable("menu.singleplayer").into(),
                unported("the world selection screen"),
            )
            .bounds(width / 2 - 100, top_pos, 200, 20)
            .build(),
        );
        // An offline user may play on servers (`UserApiService.OFFLINE`'s properties allow it), so
        // there's no reason, no tooltip, and the buttons are active.
        let multiplayer_y = top_pos + spacing;
        self.add(
            Button::builder(
                Component::translatable("menu.multiplayer").into(),
                unported("the server list"),
            )
            .bounds(width / 2 - 100, multiplayer_y, 200, 20)
            .build(),
        );
        let realms_y = multiplayer_y + spacing;
        self.add(
            Button::builder(
                Component::translatable("menu.online").into(),
                unported("the Realms screen"),
            )
            .bounds(width / 2 - 100, realms_y, 200, 20)
            .build(),
        );
        realms_y
    }
}

/// A button opening what the port hasn't got: it says so.
fn unported(what: &'static str) -> OnPress {
    Box::new(move |_| eprintln!("{what} isn't ported yet"))
}

impl ScreenClass for TitleScreen {
    fn screen(&self) -> &Screen {
        &self.screen
    }

    fn screen_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_pause_screen(&self) -> bool {
        false
    }

    fn should_close_on_esc(&self) -> bool {
        false
    }

    fn can_interrupt_with_another_screen(&self) -> bool {
        true
    }

    fn init(&mut self) {
        self.widgets.clear();
        let font = Rc::clone(&self.screen.font);
        let copyright_text: Component = Component::translatable("title.credits").into();
        let copyright_width = font.width_text(&copyright_text);
        let copyright_x = self.screen.width - copyright_width - 2;
        let spacing = 24;
        let mut top_pos = self.screen.height / 4 + 48;
        top_pos = self.create_normal_menu_options(top_pos, spacing);
        top_pos += 24;
        // Friends are available unless the game is a demo or in offline developer mode, which the
        // port never is.
        let friends = self.add(CommonButtons::friends(
            20,
            unported("the friends overlay"),
            true,
        ));
        let x = self.get_horizontal_position(1, 3, 20);
        friends.borrow_mut().set_position(x, top_pos);
        let language = self.add(CommonButtons::language(
            20,
            unported("the language screen"),
            true,
        ));
        let x = self.get_horizontal_position(2, 3, 20);
        language.borrow_mut().set_position(x, top_pos);
        let accessibility = self.add(CommonButtons::accessibility(
            20,
            unported("the accessibility options"),
            true,
        ));
        let x = self.get_horizontal_position(3, 3, 20);
        accessibility.borrow_mut().set_position(x, top_pos);
        top_pos += 24;
        let width = self.screen.width;
        let weak = self.screen.minecraft_weak();
        self.add(
            Button::builder(
                Component::translatable("menu.options").into(),
                Box::new(move |_| {
                    if let Some(minecraft) = weak.upgrade() {
                        let options_screen = OptionsScreen::new(&minecraft, minecraft.screen());
                        minecraft.set_screen(Some(Rc::new(RefCell::new(options_screen))));
                    }
                }),
            )
            .bounds(width / 2 - 100, top_pos, 98, 20)
            .build(),
        );
        let weak = self.screen.minecraft_weak();
        self.add(
            Button::builder(
                Component::translatable("menu.quit").into(),
                Box::new(move |_| {
                    if let Some(minecraft) = weak.upgrade() {
                        minecraft.stop();
                    }
                }),
            )
            .bounds(width / 2 + 2, top_pos, 98, 20)
            .build(),
        );
        self.add(PlainTextButton::create(
            copyright_x,
            self.screen.height - 10,
            copyright_width,
            10,
            copyright_text,
            unported("the credits and attribution screen"),
            font,
        ));
    }

    /// `extractRenderState`: the fade by the wall clock, the panorama, the widgets, the logo and
    /// the version.
    fn extract_render_state(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        mouse_x: i32,
        mouse_y: i32,
        a: f32,
    ) {
        if self.fade_in_start == 0 && self.fading {
            self.fade_in_start = Util::get_millis();
        }
        let mut widget_fade = 1.0;
        if self.fading {
            let mut fade = (Util::get_millis() - self.fade_in_start) as f32 / 2000.0;
            if fade > 1.0 {
                self.fading = false;
            } else {
                fade = Mth::clamp_f32(fade, 0.0, 1.0);
                widget_fade = Mth::clamped_map_f32(fade, 0.5, 1.0, 0.0, 1.0);
            }
            // fadeWidgets.
            for widget in &self.widgets {
                widget.borrow_mut().abstract_widget_mut().alpha = widget_fade;
            }
        }
        screen::extract_panorama(&self.screen, graphics, a);
        self.screen
            .extract_renderables(graphics, mouse_x, mouse_y, a);
        let logo_alpha = if self.logo_renderer.keep_logo_through_fade() {
            1.0
        } else {
            widget_fade
        };
        self.logo_renderer
            .extract_render_state(graphics, self.screen.width, logo_alpha);
        let version_string = format!("Minecraft {}", SharedConstants::get_current_version_id());
        let font = Rc::clone(&self.screen.font);
        graphics.text_str(
            &font,
            Some(&version_string),
            2,
            self.screen.height - 10,
            ARGB::white_f32(widget_fade),
            true,
        );
    }

    fn extract_background(
        &mut self,
        _graphics: &mut GuiGraphicsExtractor,
        _mouse_x: i32,
        _mouse_y: i32,
        _a: f32,
    ) {
    }
}
