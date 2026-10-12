use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

#[derive(Deserialize, Serialize, Type, Eq, PartialEq, Hash, Debug, Clone)]
pub struct DeviceId {
    pub uuid: String,
}

#[derive(Deserialize, Serialize, Type, Eq, PartialEq, Hash, Debug)]
pub struct Device {
    pub id: DeviceId,
    pub name: String,
}

impl DeviceId {
    pub fn new(uuid: &str) -> Self {
        DeviceId {
            uuid: uuid.to_owned(),
        }
    }
}

impl Device {
    pub fn new(id: DeviceId, name: &str) -> Self {
        Device {
            id,
            name: name.to_owned(),
        }
    }
}
