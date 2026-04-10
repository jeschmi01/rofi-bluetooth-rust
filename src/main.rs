use btleplug::api::{
    Central, CentralEvent, Manager as _, Peripheral as _, PeripheralProperties as _, ScanFilter,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use notify_rust::Notification;
use rofi::Rofi;
use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;
use tokio::time;

async fn update_device_names(
    central: &Adapter,
    device_names: &mut HashSet<String>,
) -> Result<(), Box<dyn Error>> {
    let mut devices = central.events().await?;
    while let Some(device) = devices.next().await {
        match device {
            CentralEvent::DeviceDiscovered(id) => {
                let periphal = central.peripheral(&id).await?;
                if let Some(props) = periphal.properties().await? {
                    let name = props.local_name.unwrap_or_else(|| "Unknown".to_string());
                    println!("{}", name);
                    device_names.insert(name);
                }
            }
            _ => (),
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut device_names: HashSet<String> = HashSet::new();

    let manager = Manager::new().await.unwrap();
    let adapters = manager.adapters().await?;
    let central = adapters
        .into_iter()
        .nth(0)
        .ok_or("No Bluetooth Adapter found")?;

    tokio::time::timeout(
        Duration::from_secs(5),
        update_device_names(&central, &mut device_names),
    )
    .await;

    let names: Vec<String> = device_names.into_iter().collect();

    if names.is_empty() {
        println!("Keine Geräte gefunden. Scannt der Adapter noch?");
        return Ok(());
    }

    let devices = vec![
        "󰋋 Kopfhörer | 00:11:22:33:44:55".to_string(),
        "󰍽 Maus | AA:BB:CC:DD:EE:FF".to_string(),
    ];

    match Rofi::new(&names).prompt(" Bluetooth").run() {
        Ok(choice) => {
            Notification::new()
                .summary(" Bluetooth")
                .body(&format!("Connected to device: {}", choice))
                .show()
                .unwrap();
        }
        Err(rofi::Error::Interrupted) => println!("Abgebrochen"),
        Err(e) => eprintln!("Fehler: {}", e),
    }

    Ok(())
}
