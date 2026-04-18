mod bluetooth;
mod types;

use crate::bluetooth::{connect_device, scan_device};
use crate::types::{BltSetting, DeviceDescription};

use bluer::{Adapter, Session};
use notify_rust::Notification;
use rofi::Rofi;
use std::error::Error;
use std::time::Duration;

async fn show_decive_menu(
    prompt: String,
    adapter: &Adapter,
    device_description: &mut DeviceDescription,
) {
    loop {
        let mut element_names: Vec<String> = device_description
            .status
            .to_string()
            .split('\n')
            .map(|s| s.to_string())
            .collect();

        element_names.push("-------".to_string());
        element_names.push("Back".to_string());
        element_names.push("Exit".to_string());

        match Rofi::new(&element_names)
            .prompt(prompt.as_str())
            .run_index()
        {
            Ok(index) => {
                let selected_option = element_names[index].as_str();
                if selected_option.starts_with("Connected") {
                    let _ = connect_device(device_description, adapter).await;
                } else if selected_option.starts_with("Paired") {
                    let _ = connect_device(device_description, adapter).await;
                } else if selected_option.starts_with("Trusted") {
                    let _ = connect_device(device_description, adapter).await;
                } else if selected_option.starts_with("Back") {
                    break;
                } else if selected_option.starts_with("Exit") {
                    std::process::exit(0);
                } else {
                    continue;
                }
            }
            _ => (),
        }
    }
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
        scan_device(&adapter, &mut devices, options[1].active),
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

        match Rofi::new(&element_names)
            .prompt(" Bluetooth | ")
            .run_index()
        {
            Ok(index) => {
                if index == element_names.len() - 1 {
                    println!("Exit");
                    break;
                } else if index < seperator_index {
                    let device_idx = seperator_index - 1 - index;
                    let selected_device = &mut devices[device_idx];
                    let prompt = format!(
                        "{} {} | ",
                        selected_device.icon.as_str(),
                        selected_device.name.as_str()
                    );
                    show_decive_menu(prompt, &adapter, selected_device).await;
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
                                scan_device(&adapter, &mut devices, selected_option.active),
                            )
                            .await;
                        }
                        _ => (),
                    };
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
