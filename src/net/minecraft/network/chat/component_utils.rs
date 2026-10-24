//! net/minecraft/network/chat/ComponentUtils.java
//!
//! Only `mergeStyles` of a `Component`, which the GUI's greyed messages use. The resolving of
//! selectors, scores and NBT, the joins and the formatting of lists come with their callers.

use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::style::Style;

pub struct ComponentUtils;

impl ComponentUtils {
    /// `mergeStyles(Component, Style)`: the component's own style over `style`, the component
    /// itself when that changes nothing.
    pub fn merge_styles(component: &Component, style: &Style) -> Component {
        if style.is_empty() {
            return component.clone();
        }
        let inner = component.get_style();
        if inner.is_empty() {
            let mut copy = component.copy();
            copy.set_style(style.clone());
            copy.into()
        } else if inner == style {
            component.clone()
        } else {
            let mut copy = component.copy();
            copy.set_style(inner.apply_to(style));
            copy.into()
        }
    }
}
