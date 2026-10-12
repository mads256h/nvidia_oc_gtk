use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

#[derive(
    Deserialize, Serialize, Type, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Copy, Debug,
)]
pub struct MHz(pub i32);

#[derive(
    Deserialize, Serialize, Type, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Copy, Debug,
)]
pub struct Watt(pub u32);
