pub mod models;
pub mod views;

use crate::models::fan_model::FanModel;
use crate::views::fan_view::create_fan_list;
use libadwaita::gtk::{
    Align, Box, Button, Grid, Label, ListBox, Orientation, ProgressBar, StringList,
};
use libadwaita::prelude::*;
use libadwaita::{
    ActionRow, Application, ApplicationWindow, ComboRow, HeaderBar, NavigationPage,
    NavigationSplitView, Sidebar, SidebarItem, SidebarSection, ToolbarView,
};
use nvml_wrapper::enum_wrappers::device::{Clock, ClockId};
use nvml_wrapper::enums::device::FanControlPolicy;
use nvml_wrapper::{Device, Nvml};

fn main() -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    let application = Application::builder()
        .application_id("com.mads256h.NvidiaOC")
        .build();

    application.connect_activate(move |app| {
        let nvml = Nvml::init().unwrap();

        let mut devices = vec![];
        let mut device_names = vec![];

        for i in 0..nvml.device_count().unwrap() {
            let device = nvml.device_by_index(i).unwrap();
            let name = device.name().unwrap();

            eprintln!("Found {i}: {name}");
            devices.push(device);
            device_names.push(name);
        }

        let header = HeaderBar::new();

        let sidebar_item_overclock = SidebarItem::builder()
            .title("Overclock")
            .icon_name("power-profile-performance-symbolic")
            .build();

        let sidebar_item_fan = SidebarItem::builder()
            .title("Fan")
            .icon_name("find-location-symbolic")
            .build();

        let sidebar_item_statistics = SidebarItem::builder()
            .title("Statistics")
            .icon_name("computer-fail-symbolic")
            .build();

        let sidebar_content = SidebarSection::new();
        sidebar_content.append(sidebar_item_overclock);
        sidebar_content.append(sidebar_item_fan);
        sidebar_content.append(sidebar_item_statistics);

        let sidebar = Sidebar::builder().build();
        sidebar.append(sidebar_content);

        let sidebar_navigation_page = NavigationPage::builder()
            .title("Sidebar")
            .child(&sidebar)
            .build();

        let grid = Grid::builder()
            .column_spacing(12)
            .row_spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        add_metric(&grid, 0, "GPU", "1950 MHz", 0.78);
        add_metric(&grid, 1, "Memory", "1200 MHz", 0.55);
        add_metric(&grid, 2, "Voltage", "0.95 V", 0.64);
        add_metric(&grid, 3, "Temperature", "68 C", 0.68);

        let (fan_list_view, fan_model) = create_fan_list();

        // HACK: Use selected device.
        let device = devices.get(0).unwrap();

        print_clock(device, Clock::Graphics).unwrap();
        print_clock(device, Clock::Memory).unwrap();
        print_clock(device, Clock::Video).unwrap();
        print_clock(device, Clock::SM).unwrap();

        let constraints = device.power_management_limit_constraints().unwrap();
        let min = constraints.min_limit / 1000;
        let max = constraints.max_limit / 1000;
        let power_limit = device.enforced_power_limit().unwrap() / 1000;

        println!("Power limit: {power_limit} W, min: {min} W, max: {max} W");

        for index in 0..device.num_fans().unwrap() {
            let speed = device.fan_speed(index).unwrap();
            let speed_rpm = device.fan_speed_rpm(index).unwrap();
            let automatic = device.fan_control_policy(index).unwrap()
                == FanControlPolicy::TemperatureContinousSw;

            println!("Fan {index}, speed: {speed}, speed_rpm: {speed_rpm}, automatic: {automatic}");
            fan_model.append(&FanModel::new(index, automatic, speed_rpm, speed_rpm));
        }

        let content_box = Box::new(Orientation::Vertical, 0);
        content_box.append(&grid);
        content_box.append(&fan_list_view);
        content_box.set_vexpand(true);

        let content_navigation_page = NavigationPage::builder()
            .title("Overclock")
            .child(&content_box)
            .build();

        let navigation_split_view = NavigationSplitView::builder()
            .sidebar(&sidebar_navigation_page)
            .content(&content_navigation_page)
            .vexpand(true)
            .build();

        let gpu_string_list =
            StringList::new(&device_names.iter().map(String::as_str).collect::<Vec<_>>());

        let gpu_selector = ComboRow::builder()
            .title("GPU")
            .model(&gpu_string_list)
            .build();

        let apply_button = Button::builder()
            .child(&Label::new(Some("Apply")))
            .halign(Align::End)
            .build();

        let footer = Box::new(Orientation::Horizontal, 0);
        footer.append(&gpu_selector);
        // spacer
        footer.append(
            &Box::builder()
                .orientation(Orientation::Horizontal)
                .hexpand(true)
                .build(),
        );
        footer.append(&apply_button);

        let content = Box::new(Orientation::Vertical, 0);
        content.append(&header);
        content.append(&navigation_split_view);
        content.append(&footer);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Nvidia OC")
            .default_width(350)
            .content(&content)
            .build();

        window.present();
    });

    application.run();

    Ok(())
}

fn add_metric(grid: &Grid, row: i32, name: &str, value: &str, fraction: f64) {
    let name_label = Label::new(Some(name));
    name_label.set_xalign(0.0);

    let bar = ProgressBar::new();
    bar.set_fraction(fraction.clamp(0.0, 1.0));
    bar.set_hexpand(true);
    bar.set_show_text(false);
    bar.set_valign(Align::Center);

    let value_label = Label::new(Some(value));
    value_label.set_xalign(1.0);
    value_label.set_width_chars(10);

    grid.attach(&name_label, 0, row, 1, 1);
    grid.attach(&bar, 1, row, 1, 1);
    grid.attach(&value_label, 2, row, 1, 1);
}

fn print_device(device: &Device) -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    let fan_control_policy = device.fan_control_policy(0)?;

    Ok(())
}

fn print_clock(
    device: &Device,
    clock: Clock,
) -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    let current = device.clock(clock, ClockId::Current)?;

    let offset = match clock {
        Clock::Graphics => device.gpc_clock_vf_offset()?,
        Clock::SM => 0,
        Clock::Memory => device.mem_clock_vf_offset()?,
        Clock::Video => 0,
    };
    println!("{clock:?}: Current: {current} MHz, Offset: {offset}");

    Ok(())
}
