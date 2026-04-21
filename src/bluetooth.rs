use crate::types::{DeviceDescription, DeviceStatus, get_icon};

use bluer::{Adapter, AdapterEvent, Device};
use futures::StreamExt;
use notify_rust::{Notification, Timeout, Urgency};
use std::error::Error;

fn show_notficiation(body: String, is_error: bool) {
    let mut notification = Notification::new();
    notification
        .summary("Bluetooth")
        .body(body.as_str())
        .icon("network-bluetooth")
        .timeout(Timeout::Milliseconds(2000));
    if is_error {
        notification.urgency(Urgency::Critical);
    } else {
        notification.urgency(Urgency::Normal);
    }
    let _ = notification.show();
}

pub async fn scan_device(
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
                    let device_description = DeviceDescription {
                        icon: String::from(get_icon(&icon_name)),
                        name,
                        addr: addr,
                        status: DeviceStatus {
                            connected: device.is_connected().await?,
                            paired: device.is_paired().await?,
                            trusted: device.is_trusted().await?,
                        },
                    };
                    if !devices.iter().any(|d| d.addr == device_description.addr) {
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
                let device_description = DeviceDescription {
                    icon: String::from(get_icon(&icon_name)),
                    name,
                    addr: addr,
                    status: DeviceStatus {
                        connected: device.is_connected().await?,
                        paired: device.is_paired().await?,
                        trusted: device.is_trusted().await?,
                    },
                };
                if !devices.iter().any(|d| d.addr == device_description.addr) {
                    devices.push(device_description);
                }
            }
        }
    }
    Ok(())
}

pub async fn connect_device(device_description: &mut DeviceDescription, device: &mut Device) {
    let mut retries = 2;
    loop {
        match device.connect().await {
            Ok(()) => break,
            Err(_) if retries > 0 => {
                retries -= 1;
            }
            Err(err) => {
                let msg = format!("Connection to {} failed {}", device_description.name, err);
                show_notficiation(msg, true);
                return;
            }
        }
    }
    device_description.status.toogle_connect();
    let msg = format!("Connected: {}", device_description.name);
    show_notficiation(msg, false);
    return;
}

pub async fn disconnect_device(device_description: &mut DeviceDescription, device: &mut Device) {
    match device.disconnect().await {
        Ok(()) => {
            let msg = format!("Disconnected: {}", device_description.name);
            device_description.status.toogle_connect();
            show_notficiation(msg, false);
        }
        Err(err) => {
            let msg = format!(
                "Disconnection from {} failed {}",
                device_description.name, err
            );
            show_notficiation(msg, true);
        }
    }
}
