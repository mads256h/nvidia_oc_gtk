use nvml_wrapper::enum_wrappers::device::{Clock, PerformanceState};
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::{Device, Nvml};
use nvogue_protocol::device::DeviceId;
use nvogue_protocol::overclock::{ClockInfo, ClockType, OverclockInfo, PowerLimitInfo};
use nvogue_protocol::service::{NVogue, NVogueServiceImpl};
use nvogue_protocol::units::{MHz, Watt};
use std::collections::HashMap;
use std::error::Error;
use std::future::pending;
use zbus::connection;

struct NVogueImpl {
    nvml: Nvml,
    device_map: HashMap<String, u32>,
}

impl NVogueImpl {
    fn find_device(&self, device_id: DeviceId) -> Result<Device, nvogue_protocol::error::Error> {
        // TODO: Wrap in error
        let device_idx = self.device_map.get(&device_id.uuid).unwrap();
        Ok(self.nvml.device_by_index(*device_idx)?)
    }
}

impl NVogueServiceImpl for NVogueImpl {
    fn get_devices(
        &self,
    ) -> Result<Vec<nvogue_protocol::device::Device>, nvogue_protocol::error::Error> {
        let mut result = vec![];
        for (_, index) in self.device_map.iter() {
            let device = self.nvml.device_by_index(*index)?;
            result.push(device_to_pdevice(&device)?)
        }

        Ok(result)
    }

    fn get_overclock(
        &self,
        device_id: DeviceId,
    ) -> Result<OverclockInfo, nvogue_protocol::error::Error> {
        let device = self.find_device(device_id)?;

        let graphics_clock = offset_to_clockinfo(&device, ClockType::Graphics)?;
        let memory_clock = offset_to_clockinfo(&device, ClockType::Memory)?;

        let constraints = device.power_management_limit_constraints()?;

        let default_limit = device.power_management_limit_default()?;

        let current_limit = device.power_management_limit()?;

        let power_limit = PowerLimitInfo {
            min_limit: Watt(constraints.min_limit / 1000),
            max_limit: Watt(constraints.max_limit / 1000),
            default_limit: Watt(default_limit / 1000),
            current_limit: Watt(current_limit / 1000),
        };

        Ok(OverclockInfo {
            graphics_clock,
            memory_clock,
            power_limit,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let nvml = Nvml::init()?;
    let device_map = create_device_map(&nvml)?;

    let connection = connection::Builder::system()?
        .name("org.mads256h.NVogue")?
        .serve_at(
            "/org/mads256h/NVogue",
            NVogue {
                imp: NVogueImpl { nvml, device_map },
            },
        )?
        .build()
        .await?;

    // Wait forever
    // TODO: Graceful shutdown
    pending::<()>().await;

    Ok(())
}

fn create_device_map(nvml: &Nvml) -> Result<HashMap<String, u32>, NvmlError> {
    let mut device_map = HashMap::new();
    for i in 0..nvml.device_count()? {
        let device = nvml.device_by_index(i)?;
        let uuid = device.uuid()?;
        device_map.insert(uuid, i);
    }

    Ok(device_map)
}

fn device_to_pdevice(device: &Device) -> Result<nvogue_protocol::device::Device, NvmlError> {
    let uuid = device.uuid()?;
    let device_id = DeviceId::new(&uuid);
    let name = device.name()?;

    Ok(nvogue_protocol::device::Device::new(device_id, &name))
}

fn offset_to_clockinfo(device: &Device, clock_type: ClockType) -> Result<ClockInfo, NvmlError> {
    let clock = match clock_type {
        ClockType::Graphics => Clock::Graphics,
        ClockType::Memory => Clock::Memory,
    };

    let clock_offset = device.clock_offset(Clock::Graphics, PerformanceState::Zero)?;

    Ok(ClockInfo {
        clock_type,
        min_clock_offset: MHz(clock_offset.min_clock_offset_mhz),
        max_clock_offset: MHz(clock_offset.max_clock_offset_mhz),
        current_offset: MHz(clock_offset.clock_offset_mhz),
    })
}
