mod widgets;

use crate::widgets::scale_value_widget::{
    ScaleValueConfig, ScaleValueInputMessage, ScaleValueOutputMessage, ScaleValueWidget,
};
use nvogue_protocol::device::Device;
use nvogue_protocol::overclock::{ClockInfo, ClockType, OverclockInfo};
use nvogue_protocol::service::NVogueProxy;
use nvogue_protocol::units::{MHz, Watt};
use relm4::adw::prelude::*;
use relm4::loading_widgets::LoadingWidgets;
use relm4::prelude::*;
use relm4::{RelmApp, adw, gtk, view};
use std::error::Error;

struct AppModel {
    proxy: NVogueProxy<'static>,
    devices: Vec<Device>,
    selected_device_index: usize,

    overclock_info: Option<OverclockInfo>,

    graphics_offset_widget: Controller<ScaleValueWidget>,
    memory_offset_widget: Controller<ScaleValueWidget>,
    power_limit_widget: Controller<ScaleValueWidget>,

    pending_graphics_offset: MHz,
    pending_memory_offset: MHz,
    pending_power_limit: Watt,
}

#[derive(Debug)]
enum AppMsg {
    DeviceSelected(usize),
    ClockOffsetChanged(ClockType, MHz),
    PowerLimitChanged(Watt),
}

impl AppModel {
    fn get_selected_device(&self) -> &Device {
        let index = self.selected_device_index;
        &self.devices[index]
    }

    fn get_overclock_info(&self) -> &OverclockInfo {
        self.overclock_info.as_ref().unwrap()
    }

    fn clock_info_to_scale_value_config(name: &str, clock_info: &ClockInfo) -> ScaleValueConfig {
        // TODO: set default and current_offset
        ScaleValueConfig::new(
            name,
            clock_info.current_offset.0 as f64,
            clock_info.min_clock_offset.0 as f64,
            clock_info.max_clock_offset.0 as f64,
        )
    }

    async fn refresh_overclock_info(&mut self) -> Result<(), nvogue_protocol::error::Error> {
        self.overclock_info = Some(
            self.proxy
                .get_overclock(self.get_selected_device().id.clone())
                .await?,
        );

        self.graphics_offset_widget
            .emit(ScaleValueInputMessage::ModelChanged(
                Self::clock_info_to_scale_value_config(
                    "Graphics",
                    &self.get_overclock_info().graphics_clock,
                ),
            ));
        self.memory_offset_widget
            .emit(ScaleValueInputMessage::ModelChanged(
                Self::clock_info_to_scale_value_config(
                    "Memory",
                    &self.get_overclock_info().memory_clock,
                ),
            ));

        self.power_limit_widget
            .emit(ScaleValueInputMessage::ModelChanged(ScaleValueConfig::new(
                "Limit",
                self.get_overclock_info().power_limit.default_limit.0 as f64,
                self.get_overclock_info().power_limit.min_limit.0 as f64,
                self.get_overclock_info().power_limit.max_limit.0 as f64,
            )));
        self.power_limit_widget
            .emit(ScaleValueInputMessage::ValueChanged(
                self.get_overclock_info().power_limit.current_limit.0 as f64,
            ));

        Ok(())
    }
}

#[relm4::component(async)]
impl SimpleAsyncComponent for AppModel {
    type Init = ();

    type Input = AppMsg;
    type Output = ();

    view! {
        adw::Window {
            set_title: Some("NVogue"),

            adw::ToolbarView {
                add_top_bar = &adw::HeaderBar { },

                #[wrap(Some)]
                set_content = &adw::NavigationSplitView {
                    #[wrap(Some)]
                    set_sidebar = &adw::NavigationPage {
                        set_title: "Sidebar",
                        adw::Sidebar {
                            append = adw::SidebarSection {
                                set_title: Some("Tunables"),

                                append = adw::SidebarItem {
                                    set_title: Some("Overclock"),
                                    set_icon_name: Some("power-profile-performance-symbolic")
                                },
                                append = adw::SidebarItem {
                                    set_title: Some("Fan control"),
                                    set_icon_name: Some("weather-windy-symbolic")
                                }
                            },
                            append = adw::SidebarSection {
                                set_title: Some("Info"),

                                append = adw::SidebarItem {
                                    set_title: Some("Live metrics"),
                                    set_icon_name: Some("system-run-symbolic")
                                },
                                append = adw::SidebarItem {
                                    set_title: Some("GPU Info"),
                                    set_icon_name: Some("help-about-symbolic")
                                }
                            }
                        }
                    },

                    #[wrap(Some)]
                    set_content = &adw::NavigationPage {
                        set_title: "Content",

                        adw::Clamp {
                            set_maximum_size: 600,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 32,

                                adw::PreferencesGroup {
                                    set_title: "Clock offsets",
                                    set_description: Some("Specify clock offsets in MHz"),

                                    model.graphics_offset_widget.widget(),
                                    model.memory_offset_widget.widget(),
                                },

                                adw::PreferencesGroup {
                                    set_title: "Power limit",
                                    set_description: Some("Specify the power limit in Watt"),

                                    model.power_limit_widget.widget(),
                                }
                            }
                        }
                    },
                },

                add_bottom_bar = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_margin_all: 4,

                    gtk::Box {
                        set_margin_start: 8,
                        set_spacing: 6,

                        gtk::Image {
                            set_icon_name: Some("video-display-symbolic"),
                        },
                        gtk::DropDown {
                            set_model: Some(&gtk::StringList::new(&model.devices.iter().map(|device| device.name.clone()).collect::<Vec<_>>().iter().map(String::as_str).collect::<Vec<_>>())),
                            connect_selected_notify[sender] => move |dropdown| {
                                let index = dropdown.selected();
                                if index != gtk::INVALID_LIST_POSITION {
                                    sender.input(AppMsg::DeviceSelected(index as usize));
                                }
                            }
                        }
                    },

                    gtk::Box {
                        set_hexpand: true
                    },

                    gtk::Box {
                        gtk::Button {
                            adw::ButtonContent {
                                set_icon_name: "object-select-symbolic",
                                set_label: "Apply"
                            }
                        }
                    }

                }
            }
        }
    }
    async fn init(
        _input: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        // TODO: Show errors in ui
        let connection = zbus::Connection::system().await.unwrap();

        let proxy = NVogueProxy::new(&connection).await.unwrap();

        let devices = proxy.get_devices().await.unwrap();

        let graphics_offset_widget = ScaleValueWidget::builder()
            .launch(Self::clock_info_to_scale_value_config(
                "Graphics",
                &ClockInfo::new_empty(ClockType::Graphics),
            ))
            .forward(sender.input_sender(), |msg| match msg {
                ScaleValueOutputMessage::ValueChanged(offset) => {
                    AppMsg::ClockOffsetChanged(ClockType::Graphics, MHz(offset as i32))
                }
            });

        let memory_offset_widget = ScaleValueWidget::builder()
            .launch(Self::clock_info_to_scale_value_config(
                "Memory",
                &ClockInfo::new_empty(ClockType::Graphics),
            ))
            .forward(sender.input_sender(), |msg| match msg {
                ScaleValueOutputMessage::ValueChanged(offset) => {
                    AppMsg::ClockOffsetChanged(ClockType::Memory, MHz(offset as i32))
                }
            });

        let power_limit_widget = ScaleValueWidget::builder()
            .launch(ScaleValueConfig::new("Limit", 0.0, 0.0, 0.0))
            .forward(sender.input_sender(), |msg| match msg {
                ScaleValueOutputMessage::ValueChanged(limit) => {
                    AppMsg::PowerLimitChanged(Watt(limit as u32))
                }
            });

        let mut model = AppModel {
            proxy,
            devices,
            selected_device_index: 0,
            overclock_info: None,
            graphics_offset_widget,
            memory_offset_widget,
            power_limit_widget,
            pending_graphics_offset: MHz(0),
            pending_memory_offset: MHz(0),
            pending_power_limit: Watt(0),
        };

        model.refresh_overclock_info().await.unwrap();

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    fn init_loading_widgets(root: Self::Root) -> Option<LoadingWidgets> {
        view! {
            #[local]
            root {
                set_title: Some("NVogue"),
                set_default_size: (300, 100),

                // This will be removed automatically by
                // LoadingWidgets when the full view has loaded
                #[name(spinner)]
                gtk::Spinner {
                    start: (),
                    set_halign: gtk::Align::Center,
                }
            }
        }
        Some(LoadingWidgets::new(root, spinner))
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            AppMsg::DeviceSelected(index) => {
                self.selected_device_index = index;
                self.refresh_overclock_info().await.unwrap()
            }
            AppMsg::ClockOffsetChanged(clock_type, offset) => match clock_type {
                ClockType::Graphics => self.pending_graphics_offset = offset,
                ClockType::Memory => self.pending_memory_offset = offset,
            },
            AppMsg::PowerLimitChanged(limit) => self.pending_power_limit = limit,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = RelmApp::new("org.mads256h.nvouge");
    app.run_async::<AppModel>(());

    Ok(())
}
