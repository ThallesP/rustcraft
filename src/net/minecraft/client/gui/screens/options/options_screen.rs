//! net/minecraft/client/gui/screens/options/OptionsScreen.java
//!
//! The options the pause menu's Options… opens: the field of view's slider and the online
//! options' button under the title, the buttons to the options' screens in two columns, and Done,
//! which goes back to the screen it came from.
//!
//! Not ported yet: the screens its buttons open (skin customization, sounds, video, controls,
//! language, chat, resource packs, accessibility, telemetry, credits), which say so, and the
//! `options.txt` that saving the options when the screen closes writes (`removed`): the port reads
//! and writes none, so saving only tells the server about the options (`broadcastOptions`).

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use crate::net::minecraft::client::gui::components::abstract_widget::{
    AbstractWidgetClass, WidgetRef,
};
use crate::net::minecraft::client::gui::components::button::{Button, OnPress};
use crate::net::minecraft::client::gui::components::string_widget::StringWidget;
use crate::net::minecraft::client::gui::components::tooltip::Tooltip;
use crate::net::minecraft::client::gui::layouts::grid_layout::GridLayout;
use crate::net::minecraft::client::gui::layouts::header_and_footer_layout::HeaderAndFooterLayout;
use crate::net::minecraft::client::gui::layouts::layout::Layout;
use crate::net::minecraft::client::gui::layouts::layout_element::LayoutElementRef;
use crate::net::minecraft::client::gui::layouts::linear_layout::LinearLayout;
use crate::net::minecraft::client::gui::screens::screen::{Screen, ScreenClass, ScreenRef};
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::network::chat::common_components::CommonComponents;
use crate::net::minecraft::network::chat::component::Component;

pub struct OptionsScreen {
    screen: Screen,
    layout: Rc<RefCell<HeaderAndFooterLayout>>,
    last_screen: Option<ScreenRef>,
}

impl OptionsScreen {
    const COLUMNS: i32 = 2;

    /// `OptionsScreen(Screen, Options)`: the game's options, which the screen reaches through the
    /// game.
    pub fn new(minecraft: &Rc<Minecraft>, last_screen: Option<ScreenRef>) -> OptionsScreen {
        OptionsScreen {
            screen: Screen::new(minecraft, Component::translatable("options.title").into()),
            layout: Rc::new(RefCell::new(HeaderAndFooterLayout::with_heights(61, 33))),
            last_screen,
        }
    }

    /// `createOnlineButton`: its bounds are the layout's in the end.
    fn create_online_button(&self) -> Button {
        Button::builder(
            Component::translatable("options.online").into(),
            unported("options.online"),
        )
        .bounds(
            self.screen.width / 2 + 5,
            self.screen.height / 6 - 12 + 24,
            150,
            20,
        )
        .build()
    }

    /// `openScreenButton`: a button to a screen that isn't ported, which says so.
    fn open_screen_button(key: &'static str) -> Button {
        Button::builder(Component::translatable(key).into(), unported(key)).build()
    }
}

fn unported(key: &'static str) -> OnPress {
    Box::new(move |_| eprintln!("the screen of {key} isn't ported yet"))
}

fn widget(widget: impl AbstractWidgetClass) -> LayoutElementRef {
    let widget: WidgetRef = Rc::new(RefCell::new(widget));
    LayoutElementRef::widget(widget)
}

impl ScreenClass for OptionsScreen {
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
        let minecraft = self.screen.minecraft();
        let font = Rc::clone(&self.screen.font);
        let title = self.screen.title.clone();
        let mut header = LinearLayout::vertical().spacing(8);
        let title_settings = header.new_cell_settings().align_horizontally_center();
        header.add_child_with(widget(StringWidget::of(title, font)), title_settings);
        let mut sub_header = LinearLayout::horizontal().spacing(8);
        let fov = minecraft.options.borrow().fov();
        sub_header.add_child(LayoutElementRef::widget(
            fov.create_button(&self.screen.minecraft_weak()),
        ));
        sub_header.add_child(widget(self.create_online_button()));
        header.add_child(LayoutElementRef::layout(Rc::new(RefCell::new(sub_header))));
        self.layout
            .borrow_mut()
            .add_to_header(LayoutElementRef::layout(Rc::new(RefCell::new(header))));

        let mut grid_layout = GridLayout::new();
        grid_layout
            .default_cell_setting()
            .padding_horizontal(4)
            .padding_bottom(4)
            .align_horizontally_center();
        let mut helper = grid_layout.create_row_helper(OptionsScreen::COLUMNS);
        for key in [
            "options.skinCustomisation",
            "options.sounds",
            "options.video",
            "options.controls",
            "options.language",
            "options.chat",
            "options.resourcepack",
            "options.accessibility",
        ] {
            helper.add_child(
                &mut grid_layout,
                widget(OptionsScreen::open_screen_button(key)),
            );
        }
        let mut telemetry_button = OptionsScreen::open_screen_button("options.telemetry");
        if !minecraft.allows_telemetry() {
            telemetry_button.widget_mut().active = false;
            telemetry_button.set_tooltip(Some(Tooltip::create(
                Component::translatable("options.telemetry.disabled").into(),
            )));
        }
        helper.add_child(&mut grid_layout, widget(telemetry_button));
        helper.add_child(
            &mut grid_layout,
            widget(OptionsScreen::open_screen_button(
                "options.credits_and_attribution",
            )),
        );
        self.layout
            .borrow_mut()
            .add_to_contents(LayoutElementRef::layout(Rc::new(RefCell::new(grid_layout))));

        let weak = self.screen.minecraft_weak();
        let last_screen = self.last_screen.clone();
        let done = Button::builder(
            CommonComponents::gui_done(),
            Box::new(move |_| {
                // onClose.
                if let Some(minecraft) = weak.upgrade() {
                    minecraft.set_screen(last_screen.clone());
                }
            }),
        )
        .width(200)
        .build();
        self.layout.borrow_mut().add_to_footer(widget(done));
        let screen = &mut self.screen;
        self.layout
            .borrow()
            .visit_widgets(&mut |widget| screen.add_renderable_widget_ref(widget));
        self.reposition_elements();
    }

    /// `repositionElements`: the layout arranged again at the screen's size.
    fn reposition_elements(&mut self) {
        let mut layout = self.layout.borrow_mut();
        layout.set_screen_size(self.screen.width, self.screen.height);
        layout.arrange_elements();
    }

    fn on_close(&mut self) {
        self.screen.minecraft().set_screen(self.last_screen.clone());
    }

    /// `removed`: the options saved, which tells the server about them.
    fn removed(&mut self) {
        let minecraft = self.screen.minecraft();
        minecraft.options.borrow().save(&minecraft);
    }
}
