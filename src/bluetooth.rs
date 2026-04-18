use crate::types::{DeviceDescription, DeviceStatus, get_icon};

use bluer::{Adapter, AdapterEvent};
use futures::StreamExt;
use std::error::Error;

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

pub async fn connect_device(
    device_description: &mut DeviceDescription,
    adapter: &Adapter,
) -> Result<(), Box<dyn Error>> {
    let device = adapter.device(device_description.addr)?;
    if !device_description.status.connected {
        let mut retries = 2;
        loop {
            match device.connect().await {
                Ok(()) => break,
                Err(err) if retries > 0 => {
                    println!("    Connect error: {}", &err);
                    retries -= 1;
                }
                Err(err) => return Err(Box::new(err)),
            }
        }
        device_description.status.toogle_connect();
        println!(" Connected");
    } else {
        println!("Already connected");
    }
    Ok(())
}
