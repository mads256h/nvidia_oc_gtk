use nvml_wrapper::error::NvmlError;

#[derive(zbus::DBusError, Debug)]
pub enum Error {
    #[zbus(error)]
    ZBus(zbus::Error),
    NvmlError(String),
}

impl From<NvmlError> for Error {
    fn from(value: NvmlError) -> Self {
        Error::NvmlError(value.to_string())
    }
}
