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

struct BltSetting {
    name: String,
    active: bool,
}

impl BltSetting {
    fn toggle(&mut self) {
        self.active = !self.active;
    }
}

impl ToString for BltSetting {
    fn to_string(&self) -> String {
        if self.active {
            format!("{}: on", self.name)
        } else {
            format!("{}: off", self.name)
        }
    }
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

async fn scan_device_names(
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
                    if !devices
                        .iter()
                        .any(|d| d.mac_addr == device_description.mac_addr)
                    {
                        devices.push(device_description);
                    }
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
                if !devices
                    .iter()
                    .any(|d| d.mac_addr == device_description.mac_addr)
                {
                    devices.push(device_description);
                }
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

    let mut options: Vec<BltSetting> = vec!["Power", "Scan", "Pairable", "Discoverable"]
        .into_iter()
        .map(|o| BltSetting {
            name: o.to_string(),
            active: false,
        })
        .collect();

    options[0].toggle();

    let mut devices: Vec<DeviceDescription> = Vec::new();
    let _ = tokio::time::timeout(
        Duration::from_secs(5),
        scan_device_names(&adapter, &mut devices, options[1].active),
    )
    .await;

    loop {
        let mut options_names: Vec<String> =
            options.iter().map(|o: &BltSetting| o.to_string()).collect();

        let mut element_names: Vec<String> = devices
            .iter()
            .map(|d: &DeviceDescription| d.to_string())
            .collect();
        element_names.reverse();

        let seperator_index = element_names.len();
        element_names.push("------".to_string());

        element_names.append(&mut options_names);
        element_names.push("Exit".to_string());

        match Rofi::new(&element_names).prompt(" Bluetooth").run_index() {
            Ok(index) => {
                if index < seperator_index {
                    let device_idx = seperator_index - 1 - index;
                    let selected_device = &devices[device_idx];
                    println!("Connect: {}", selected_device.name);
                } else if index > seperator_index {
                    let opt_idx = index - seperator_index - 1;
                    let selected_option = &mut options[opt_idx];
                    selected_option.toggle();
                    println!(
                        "{} ist toggled to {}",
                        selected_option.name, selected_option.active
                    );
                    match selected_option.name.as_str() {
                        "Scan" => {
                            let _ = tokio::time::timeout(
                                Duration::from_secs(2),
                                scan_device_names(&adapter, &mut devices, selected_option.active),
                            )
                            .await;
                        }
                        _ => (),
                    };
                    element_names[index] = selected_option.to_string();
                } else if index == element_names.len() - 1 {
                    println!("Exit");
                    break;
                }
            }
            Err(rofi::Error::Interrupted) => {
                println!("Abgebrochen");
                break;
            }
            Err(e) => {
                eprintln!("Fehler: {}", e);
                break;
            }
        }
    }
    Ok(())
}
