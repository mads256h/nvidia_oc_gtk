use libadwaita::glib;

mod imp {
    use libadwaita::glib;
    use libadwaita::glib::Properties;
    use libadwaita::prelude::*;
    use libadwaita::subclass::prelude::*;
    use std::cell::Cell;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::FanModel)]
    pub struct FanModel {
        #[property(get, set)]
        pub fan_index: Cell<u32>,

        #[property(get, set)]
        pub automatic: Cell<bool>,

        #[property(get, set)]
        pub current_speed: Cell<u32>,

        #[property(get, set)]
        pub target_speed: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FanModel {
        const NAME: &'static str = "FanModel";
        type Type = super::FanModel;
    }

    #[glib::derived_properties]
    impl ObjectImpl for FanModel {}
}

glib::wrapper! {
    pub struct FanModel(ObjectSubclass<imp::FanModel>);
}

impl FanModel {
    pub fn new(fan_index: u32, automatic: bool, current_speed: u32, target_speed: u32) -> Self {
        glib::Object::builder()
            .property("fan_index", fan_index)
            .property("automatic", automatic)
            .property("current_speed", current_speed)
            .property("target_speed", target_speed)
            .build()
    }
}
