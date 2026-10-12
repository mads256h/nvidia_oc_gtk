use nvogue_protocol::service::NVogueProxy;
use std::error::Error;
use zbus::Connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;

    let proxy = NVogueProxy::new(&connection).await?;

    let devices = proxy.get_devices().await?;

    for device in devices.iter() {
        let uuid = &device.id.uuid;
        let name = &device.name;
        println!("{name}: {uuid}");

        let overclock = proxy.get_overclock(device.id.clone()).await?;
        println!("{overclock:?}");
    }

    Ok(())
}
