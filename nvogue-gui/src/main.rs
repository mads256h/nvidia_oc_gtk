mod widgets;

use crate::widgets::clock_offset_widget::{
    ClockOffsetInputMsg, ClockOffsetOutputMsg, ClockOffsetWidget,
};
use nvogue_protocol::device::Device;
use nvogue_protocol::overclock::{ClockInfo, ClockType, OverclockInfo};
use nvogue_protocol::service::NVogueProxy;
use nvogue_protocol::units::MHz;
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

    graphics_offset_widget: Controller<ClockOffsetWidget>,
    memory_offset_widget: Controller<ClockOffsetWidget>,

    pending_graphics_offset: MHz,
    pending_memory_offset: MHz,
}

#[derive(Debug)]
enum AppMsg {
    DeviceSelected(usize),
    ClockOffsetChanged(ClockType, MHz),
}

impl AppModel {
    fn get_selected_device(&self) -> &Device {
        let index = self.selected_device_index;
        &self.devices[index]
    }

    fn get_overclock_info(&self) -> &OverclockInfo {
        self.overclock_info.as_ref().unwrap()
    }

    async fn refresh_overclock_info(&mut self) -> Result<(), nvogue_protocol::error::Error> {
        self.overclock_info = Some(
            self.proxy
                .get_overclock(self.get_selected_device().id.clone())
                .await?,
        );

        self.graphics_offset_widget
            .emit(ClockOffsetInputMsg::ClockInfoChanged(
                self.get_overclock_info().graphics_clock.clone(),
            ));
        self.memory_offset_widget
            .emit(ClockOffsetInputMsg::ClockInfoChanged(
                self.get_overclock_info().memory_clock.clone(),
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

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,

                            append = model.graphics_offset_widget.widget(),
                            append = model.memory_offset_widget.widget(),
                        }
                    },
                },

                add_bottom_bar = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,

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

        let graphics_offset_widget = ClockOffsetWidget::builder()
            .launch(ClockInfo::new_empty(ClockType::Graphics))
            .forward(sender.input_sender(), |msg| match msg {
                ClockOffsetOutputMsg::ClockOffsetChanged(offset) => {
                    AppMsg::ClockOffsetChanged(ClockType::Graphics, offset)
                }
            });

        let memory_offset_widget = ClockOffsetWidget::builder()
            .launch(ClockInfo::new_empty(ClockType::Memory))
            .forward(sender.input_sender(), |msg| match msg {
                ClockOffsetOutputMsg::ClockOffsetChanged(offset) => {
                    AppMsg::ClockOffsetChanged(ClockType::Memory, offset)
                }
            });

        let mut model = AppModel {
            proxy,
            devices,
            selected_device_index: 0,
            overclock_info: None,
            graphics_offset_widget,
            memory_offset_widget,
            pending_graphics_offset: MHz(0),
            pending_memory_offset: MHz(0),
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
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = RelmApp::new("org.mads256h.nvouge");
    app.run_async::<AppModel>(());

    Ok(())
}
