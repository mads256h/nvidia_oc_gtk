use relm4::SimpleComponent;
use relm4::adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct ScaleValueWidget {
    scale_value_config: ScaleValueConfig,

    current_value: f64,
}

#[derive(Debug)]
pub struct ScaleValueConfig {
    name: String,

    default_value: f64,
    minimum_value: f64,
    maximum_value: f64,
}

impl ScaleValueConfig {
    pub fn new(
        name: &str,
        default_value: f64,
        minimum_value: f64,
        maximum_value: f64,
    ) -> ScaleValueConfig {
        ScaleValueConfig {
            name: name.to_owned(),
            default_value,
            minimum_value,
            maximum_value,
        }
    }
}

#[derive(Debug)]
pub enum ScaleValueInputMessage {
    ModelChanged(ScaleValueConfig),
    ValueChanged(f64),
    SetDefaultValue,
}

#[derive(Debug)]
pub enum ScaleValueOutputMessage {
    ValueChanged(f64),
}

impl ScaleValueWidget {
    fn refresh(&mut self, scale_value_config: ScaleValueConfig) {
        self.scale_value_config = scale_value_config;
    }
}

#[relm4::component(pub)]
impl SimpleComponent for ScaleValueWidget {
    type Init = ScaleValueConfig;
    type Input = ScaleValueInputMessage;
    type Output = ScaleValueOutputMessage;

    view! {
        adw::ActionRow {
            #[watch]
            set_title: &model.scale_value_config.name,

            add_suffix = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 4,

                #[name(scale)]
                gtk::Scale {
                    set_orientation: gtk::Orientation::Horizontal,

                    set_hexpand: true,

                    #[watch]
                    set_range: (model.scale_value_config.minimum_value, model.scale_value_config.maximum_value),

                    #[watch]
                    #[block_signal(scale_value_changed)]
                    set_value: model.current_value,

                    connect_value_changed[sender] => move |scale| {
                        let pending_value = scale.value();
                        sender.input(ScaleValueInputMessage::ValueChanged(pending_value));
                    } @scale_value_changed,
                },

                gtk::SpinButton {
                    set_width_chars: 4,
                    set_numeric: true,
                    set_digits: 0,

                    set_increments: (1.0, 5.0),

                    #[watch]
                    set_range: (model.scale_value_config.minimum_value, model.scale_value_config.maximum_value),

                    #[watch]
                    #[block_signal(spinbutton_value_changed)]
                    set_value: model.current_value,

                    connect_value_changed[sender] => move |spin_button| {
                        let pending_value = spin_button.value();
                        sender.input(ScaleValueInputMessage::ValueChanged(pending_value));
                    } @spinbutton_value_changed,
                },

                gtk::Button {
                    #[watch]
                    set_sensitive: model.current_value != model.scale_value_config.default_value,


                    gtk::Image {
                        set_icon_name: Some("edit-undo-symbolic"),
                    },

                    connect_clicked[sender] => move |_| {
                        sender.input(ScaleValueInputMessage::SetDefaultValue);
                    }
                }
            }
        }
    }

    fn init(
        scale_value_config: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let default_value = scale_value_config.default_value;
        let model = ScaleValueWidget {
            scale_value_config,
            current_value: default_value,
        };

        // TODO: Make default mark updatable.
        let widgets = view_output!();

        widgets
            .scale
            .add_mark(default_value, gtk::PositionType::Bottom, None);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            ScaleValueInputMessage::ModelChanged(scale_value_config) => {
                self.refresh(scale_value_config)
            }
            ScaleValueInputMessage::ValueChanged(value) => {
                self.current_value = value;

                sender
                    .output(ScaleValueOutputMessage::ValueChanged(value))
                    .unwrap();
            }
            ScaleValueInputMessage::SetDefaultValue => sender.input(
                ScaleValueInputMessage::ValueChanged(self.scale_value_config.default_value),
            ),
        }
    }
}
