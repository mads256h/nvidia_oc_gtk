use crate::device::{Device, DeviceId};
use crate::error::Error;
use crate::overclock::OverclockInfo;
use zbus::interface;

pub struct NVogue<T: NVogueServiceImpl + 'static> {
    pub imp: T,
}

#[interface(
    name = "org.mads256h.NVogue1",
    proxy(
        default_path = "/org/mads256h/NVogue",
        default_service = "org.mads256h.NVogue"
    )
)]
impl<T> NVogue<T>
where
    T: NVogueServiceImpl,
{
    fn get_devices(&self) -> Result<Vec<Device>, Error> {
        self.imp.get_devices()
    }

    fn get_overclock(&self, device_id: DeviceId) -> Result<OverclockInfo, Error> {
        self.imp.get_overclock(device_id)
    }
}

pub trait NVogueServiceImpl: Send + Sync {
    fn get_devices(&self) -> Result<Vec<Device>, Error>;
    fn get_overclock(&self, device_id: DeviceId) -> Result<OverclockInfo, Error>;
}
