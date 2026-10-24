//! net/minecraft/client/gui/components/TabOrderedElement.java

pub trait TabOrderedElement {
    fn get_tab_order_group(&self) -> i32 {
        0
    }
}
