use bluer::mesh::element;
use bluer::{Adapter, AdapterEvent, Session};
use futures::stream::StreamExt;
use notify_rust::Notification;
use rofi::Rofi;
use std::error::Error;
use std::time::Duration;

struct DeviceDescription {
    icon: String,
    name: String,
    mac_addr: String,
}

struct Options {
    name: String,
    aktive: bool,
}

impl ToString for DeviceDescription {
    fn to_string(&self) -> String {
        format!("{} {} | {}", self.icon, self.name, self.mac_addr)
    }
}

fn get_icon(icon_name: &str) -> &'static str {
    match icon_name {
        "audio-headphones" | "audio-headset" => "󰋋",
        "audio-card" | "audio-speakers" => "󰓃",
        "input-mouse" => "󰍽",
        "input-keyboard" => "󰌌",
        "input-gaming" => "󰊴",
        "phone" => "󰏲",
        "computer" | "laptop" => "󰟀",
        "video-display" | "tv" => "󰗑",
        "camera-video" => "󰄀",
        _ => "",
    }
}

async fn get_device_names(
    adapter: &Adapter,
    devices: &mut Vec<DeviceDescription>,
    scan: bool,
) -> Result<(), Box<dyn Error>> {
    if scan {
        let mut discover = adapter.discover_devices().await?;
        while let Some(evt) = discover.next().await {
            match evt {
                AdapterEvent::DeviceAdded(addr) => {
                    let device = adapter.device(addr)?;
                    let name = device.name().await?.unwrap_or_else(|| addr.to_string());
                    let icon_name = device.icon().await?.unwrap_or_default();
                    println!("{icon_name} {name}");
                    let device_description = DeviceDescription {
                        icon: String::from(get_icon(&icon_name)),
                        name,
                        mac_addr: addr.to_string(),
                    };
                    devices.push(device_description);
                }
                _ => (),
            }
        }
    } else {
        let devices_addrs = adapter.device_addresses().await?;
        for addr in devices_addrs {
            if let Ok(device) = adapter.device(addr) {
                let name = device.name().await?.unwrap_or_else(|| addr.to_string());
                let icon_name = device.icon().await?.unwrap_or_default();
                println!("{icon_name} {name}");
                let device_description = DeviceDescription {
                    icon: String::from(get_icon(&icon_name)),
                    name,
                    mac_addr: addr.to_string(),
                };
                devices.push(device_description);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let session = Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let mut devices = Vec::new();

    let _ = tokio::time::timeout(
        Duration::from_secs(5),
        get_device_names(&adapter, &mut devices, true),
    )
    .await;

    let mut element_strings: Vec<String> = devices.iter().map(|d| d.to_string()).collect();
    element_strings.reverse();
    element_strings.push("------".to_string());
    element_strings.push("Power: On".to_string());
    element_strings.push("Scan: On".to_string());

    match Rofi::new(&element_strings).prompt(" Bluetooth").run() {
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
