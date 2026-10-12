use crate::models::fan_model::FanModel;
use libadwaita::gio;
use libadwaita::gio::ListStore;
use libadwaita::gtk::prelude::*;
use libadwaita::gtk::{
    Box, Label, ListView, Orientation, Scale, ScrolledWindow, SignalListItemFactory,
    SingleSelection,
};
use libadwaita::prelude::{BoxExt, CastNone, RangeExt, ScaleExt, WidgetExt};

pub fn create_fan_list() -> (ScrolledWindow, ListStore) {
    let model = gio::ListStore::new::<FanModel>();

    let factory = SignalListItemFactory::new();

    factory.connect_setup(|_, list_item| {
        let row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(12)
            .margin_end(12)
            .build();

        let name = Label::new(None);
        name.set_xalign(0.0);
        name.set_width_chars(8);

        let current = Label::new(None);
        current.set_width_chars(5);

        let scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
        scale.set_hexpand(true);
        scale.set_draw_value(true);

        row.append(&name);
        row.append(&current);
        row.append(&scale);

        let item = list_item.clone();

        scale.connect_value_changed(move |scale| {
            if let Some(fan_model) = item.item().and_downcast::<FanModel>() {
                // TODO: Implement
                // fan_model.set_target_speed(u32::from(scale.value()));
            }
        });

        list_item.set_child(Some(&row));
    });

    factory.connect_bind(|_, list_item| {
        let Some(fan) = list_item.item().and_downcast::<FanModel>() else {
            return;
        };

        let Some(row) = list_item.child().and_downcast::<Box>() else {
            return;
        };

        let Some(name) = row.first_child().and_downcast::<Label>() else {
            return;
        };

        let Some(current) = name.next_sibling().and_downcast::<Label>() else {
            return;
        };

        let Some(scale) = current.next_sibling().and_downcast::<Scale>() else {
            return;
        };

        name.set_text(&format!("Fan {}", fan.fan_index()));
        current.set_text(&format!("{}%", fan.current_speed()));

        // Avoid treating a programmatic update as a user edit.
        // TODO: FIX
        scale.set_value(fan.target_speed() as f64);
    });

    let selection = SingleSelection::new(Some(model.clone()));

    let list_view = ListView::new(Some(selection), Some(factory));

    let scrolled = ScrolledWindow::builder()
        .child(&list_view)
        .vexpand(true)
        .hexpand(true)
        .build();

    (scrolled, model)
}
