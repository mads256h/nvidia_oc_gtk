use nvogue_protocol::overclock::ClockInfo;
use nvogue_protocol::units::MHz;
use relm4::SimpleComponent;
use relm4::adw::prelude::*;
use relm4::prelude::*;

pub struct ClockOffsetWidget {
    clock_info: ClockInfo,

    pending_offset: MHz,
}

#[derive(Debug)]
pub enum ClockOffsetInputMsg {
    ClockInfoChanged(ClockInfo),
    ClockOffsetChanged(MHz),
}

#[derive(Debug)]
pub enum ClockOffsetOutputMsg {
    ClockOffsetChanged(MHz),
}

impl ClockOffsetWidget {
    fn refresh(&mut self, clock_info: ClockInfo) {
        self.clock_info = clock_info;
        self.pending_offset = self.clock_info.current_offset;
    }
}

#[relm4::component(pub)]
impl SimpleComponent for ClockOffsetWidget {
    type Init = ClockInfo;
    type Input = ClockOffsetInputMsg;
    type Output = ClockOffsetOutputMsg;

    view! {
        adw::PreferencesGroup {
            set_title: format!("{0:?} Clock", model.clock_info.clock_type).as_str(),
            set_description: Some("Placeholder"),

            set_margin_vertical: 12,

            adw::ActionRow {
                set_title: "Offset",

                add_suffix = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,

                    #[name(scale)]
                    gtk::Scale {
                        set_orientation: gtk::Orientation::Horizontal,

                        set_hexpand: true,

                        #[watch]
                        set_range: (model.clock_info.min_clock_offset.0 as f64, model.clock_info.max_clock_offset.0 as f64),

                        #[watch]
                        set_value: model.pending_offset.0 as f64,

                        connect_value_changed[sender] => move |scale| {
                            let pending_offset = scale.value() as i32;
                            sender.input(ClockOffsetInputMsg::ClockOffsetChanged(MHz(pending_offset)));
                        }
                    },

                    gtk::SpinButton {
                        set_width_chars: 4,
                        set_numeric: true,
                        set_digits: 0,

                        set_increments: (1.0, 5.0),

                        #[watch]
                        set_range: (model.clock_info.min_clock_offset.0 as f64, model.clock_info.max_clock_offset.0 as f64),

                        #[watch]
                        set_value: model.pending_offset.0 as f64,

                        connect_value_changed[sender] => move |spin_button| {
                            let pending_offset = spin_button.value() as i32;
                            sender.input(ClockOffsetInputMsg::ClockOffsetChanged(MHz(pending_offset)));
                        }
                    }
                }
            }
        }
    }

    fn init(
        clock_info: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let pending_offset = clock_info.current_offset;
        let model = ClockOffsetWidget {
            clock_info,
            pending_offset,
        };

        let widgets = view_output!();

        widgets.scale.add_mark(
            model.clock_info.current_offset.0.into(),
            gtk::PositionType::Bottom,
            None,
        );

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            ClockOffsetInputMsg::ClockInfoChanged(clock_info) => self.refresh(clock_info),
            ClockOffsetInputMsg::ClockOffsetChanged(offset) => {
                self.pending_offset = offset;

                sender
                    .output(ClockOffsetOutputMsg::ClockOffsetChanged(offset))
                    .unwrap();
            }
        }
    }
}
