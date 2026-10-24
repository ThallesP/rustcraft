//! net/minecraft/client/gui/screens/PauseScreen.java
//!
//! The game menu Escape opens: back to the game, the screens a player opens from it, the icon
//! buttons (bugs, feedback, friends, player reporting), and disconnecting, over the blurred level.
//! On a server the game doesn't pause (`Minecraft.pauseGame`).
//!
//! Not ported yet: the screens its buttons open but the options (advancements, statistics, the
//! world options, the confirmed links, the friends overlay, social interactions), which do
//! nothing; the server's custom dialog buttons (`getCustomAdditions`: the `pause_screen_additions`
//! dialog tag, the server links), which a vanilla server has none of; the now playing toast (the
//! oracle's music is off); the draft report's icon and the player social manager's friend list.

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use crate::net::minecraft::client::gui::components::abstract_widget::WidgetRef;
use crate::net::minecraft::client::gui::components::button::{Button, OnPress};
use crate::net::minecraft::client::gui::components::common_buttons::CommonButtons;
use crate::net::minecraft::client::gui::components::sprite_icon_button::SpriteIconButton;
use crate::net::minecraft::client::gui::components::string_widget::StringWidget;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::gui::layouts::frame_layout::FrameLayout;
use crate::net::minecraft::client::gui::layouts::grid_layout::GridLayout;
use crate::net::minecraft::client::gui::layouts::layout::Layout;
use crate::net::minecraft::client::gui::layouts::layout_element::LayoutElementRef;
use crate::net::minecraft::client::gui::layouts::linear_layout::LinearLayout;
use crate::net::minecraft::client::gui::screens::options::options_screen::OptionsScreen;
use crate::net::minecraft::client::gui::screens::screen::{self, Screen, ScreenClass};
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;

pub struct PauseScreen {
    screen: Screen,
    show_pause_menu: bool,
    disconnect_button: Option<Rc<RefCell<Button>>>,
}

const MENU_PADDING_TOP: i32 = 50;
const BUTTON_PADDING: i32 = 4;
const BUTTON_WIDTH_FULL: i32 = 204;
const BUTTON_WIDTH_HALF: i32 = 98;

impl PauseScreen {
    pub fn new(minecraft: &Rc<Minecraft>, show_pause_menu: bool) -> PauseScreen {
        let title = if show_pause_menu {
            "menu.game"
        } else {
            "menu.paused"
        };
        PauseScreen {
            screen: Screen::new(minecraft, Component::translatable(title).into()),
            show_pause_menu,
            disconnect_button: None,
        }
    }

    pub fn shows_pause_menu(&self) -> bool {
        self.show_pause_menu
    }

    fn create_pause_menu(&mut self) {
        let minecraft = self.screen.minecraft();
        let weak = self.screen.minecraft_weak();
        let grid = Rc::new(RefCell::new(GridLayout::new()));
        let mut helper;
        {
            let grid_layout = grid.borrow_mut();
            grid_layout.default_cell_setting().padding_ltrb(
                BUTTON_PADDING,
                BUTTON_PADDING,
                BUTTON_PADDING,
                0,
            );
            helper = grid_layout.create_row_helper(2);
        }
        let mut add = |child: LayoutElementRef, columns: i32, settings: Option<_>| {
            let mut grid_layout = grid.borrow_mut();
            match settings {
                Some(settings) => {
                    helper.add_child_columns_with(&mut grid_layout, child, columns, settings);
                }
                None => {
                    helper.add_child_columns(&mut grid_layout, child, columns);
                }
            }
        };

        let return_to_game = {
            let weak = weak.clone();
            Button::builder(
                Component::translatable("menu.returnToGame").into(),
                Box::new(move |_| {
                    if let Some(minecraft) = weak.upgrade() {
                        minecraft.set_screen(None);
                        minecraft.grab_mouse();
                    }
                }),
            )
            .width(BUTTON_WIDTH_FULL)
            .build()
        };
        let top_settings = grid
            .borrow()
            .new_cell_settings()
            .padding_top(MENU_PADDING_TOP);
        add(widget(return_to_game), 2, Some(top_settings));
        add(
            widget(PauseScreen::open_screen_button(
                Component::translatable("gui.advancements").into(),
            )),
            1,
            None,
        );
        add(
            widget(PauseScreen::open_screen_button(
                Component::translatable("gui.stats").into(),
            )),
            1,
            None,
        );

        let mut icon_button_row = LinearLayout::horizontal().spacing(4);
        // Active, since a release isn't a side series' version.
        let report_bugs_button = SpriteIconButton::builder(
            Component::translatable("menu.reportBugs").into(),
            unported("the bug report link's confirmation"),
            true,
        )
        .width(20)
        .sprite(identifier("pause_menu/bug"), 15, 15)
        .with_tooltip()
        .build();
        icon_button_row.add_child(widget(report_bugs_button));
        let feedback_button = SpriteIconButton::builder(
            Component::translatable("menu.sendFeedback").into(),
            unported("the feedback link's confirmation"),
            true,
        )
        .width(20)
        .sprite(identifier("pause_menu/social_interactions"), 15, 15)
        .with_tooltip()
        .build();
        icon_button_row.add_child(widget(feedback_button));
        // Friends are available unless the game is a demo or in offline developer mode, which the
        // port never is.
        let friends = CommonButtons::friends(20, unported("the friends overlay"), true);
        icon_button_row.add_child(widget(friends));
        let player_reporting_button = SpriteIconButton::builder(
            Component::translatable("menu.playerReporting").into(),
            unported("the social interactions screen"),
            true,
        )
        .width(20)
        .sprite(identifier("pause_menu/player_reporting"), 15, 15)
        .with_tooltip()
        .build();
        // A singleplayer game with no one else greys it out; the port joins servers only.
        icon_button_row.add_child(widget(player_reporting_button));
        let center_settings = grid
            .borrow()
            .new_cell_settings()
            .align_horizontally_center();
        add(
            LayoutElementRef::layout(Rc::new(RefCell::new(icon_button_row))),
            2,
            Some(center_settings),
        );

        if minecraft.level.borrow().is_some() {
            let options_screen = {
                let weak = weak.clone();
                Button::builder(
                    Component::translatable("menu.options").into(),
                    Box::new(move |_| {
                        // `new OptionsScreen(this, …)`: this screen, the game's while it's shown.
                        if let Some(minecraft) = weak.upgrade() {
                            let options_screen = OptionsScreen::new(&minecraft, minecraft.screen());
                            minecraft.set_screen(Some(Rc::new(RefCell::new(options_screen))));
                        }
                    }),
                )
                .width(BUTTON_WIDTH_HALF)
                .build()
            };
            add(widget(options_screen), 1, None);
            add(
                widget(PauseScreen::open_screen_button(
                    Component::translatable("options.worldOptions.button").into(),
                )),
                1,
                None,
            );
        }

        let disconnect_button = Rc::new(RefCell::new(
            Button::builder(
                Component::translatable(if minecraft.is_local_server() {
                    "menu.returnToMenu"
                } else {
                    "menu.disconnect"
                })
                .into(),
                Box::new(move |button| {
                    button.active = false;
                    if let Some(minecraft) = weak.upgrade() {
                        minecraft.disconnect_from_world();
                    }
                }),
            )
            .width(BUTTON_WIDTH_FULL)
            .build(),
        ));
        self.disconnect_button = Some(disconnect_button.clone());
        add(LayoutElementRef::widget(disconnect_button), 2, None);

        grid.borrow_mut().arrange_elements();
        let grid_ref = LayoutElementRef::layout(grid.clone());
        FrameLayout::align_in_rectangle(
            &grid_ref,
            0,
            0,
            self.screen.width,
            self.screen.height,
            0.5,
            0.25,
        );
        let screen = &mut self.screen;
        grid.borrow()
            .visit_widgets(&mut |widget| screen.add_renderable_widget_ref(widget));
    }

    /// `openScreenButton`: a half-width button opening a screen, which isn't ported.
    fn open_screen_button(message: Component) -> Button {
        let name = message.get_string();
        Button::builder(message, unported_owned(name))
            .width(BUTTON_WIDTH_HALF)
            .build()
    }
}

fn identifier(path: &str) -> crate::net::minecraft::resources::identifier::Identifier {
    crate::net::minecraft::resources::identifier::Identifier::with_default_namespace(path)
}

fn widget(button: Button) -> LayoutElementRef {
    let widget: WidgetRef = Rc::new(RefCell::new(button));
    LayoutElementRef::widget(widget)
}

/// A button opening what the port hasn't got: it says so.
fn unported(what: &'static str) -> OnPress {
    Box::new(move |_| eprintln!("{what} isn't ported yet"))
}

fn unported_owned(what: String) -> OnPress {
    Box::new(move |_| eprintln!("{what}'s screen isn't ported yet"))
}

impl ScreenClass for PauseScreen {
    fn screen(&self) -> &Screen {
        &self.screen
    }

    fn screen_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn init(&mut self) {
        if self.show_pause_menu {
            self.create_pause_menu();
        }
        let font = Rc::clone(&self.screen.font);
        let title = self.screen.title.clone();
        let text_width = font.width_text(&title);
        let y = if self.show_pause_menu { 40 } else { 10 };
        let title_widget = StringWidget::new(
            self.screen.width / 2 - text_width / 2,
            y,
            text_width,
            9,
            title,
            font,
        );
        self.screen
            .add_renderable_widget(Rc::new(RefCell::new(title_widget)));
    }

    /// `extractBackground`: the blurred level, when this is the screen shown, and the menu
    /// background, for the menu (the subtitles aren't ported).
    fn extract_background(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        _mouse_x: i32,
        _mouse_y: i32,
        _a: f32,
    ) {
        if self.show_pause_menu {
            // isTopmostScreen: the game's screen is this one while it draws.
            screen::extract_blurred_background(&self.screen, graphics);
            screen::extract_menu_background(&self.screen, graphics);
        }
    }
}
