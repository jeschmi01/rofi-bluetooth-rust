use bluer::Address;

pub struct DeviceDescription {
    pub icon: String,
    pub name: String,
    pub addr: Address,
    pub status: DeviceStatus,
}

pub struct DeviceStatus {
    pub connected: bool,
    pub paired: bool,
    pub trusted: bool,
}

impl DeviceStatus {
    pub fn toogle_connect(&mut self) {
        self.connected = !self.connected;
    }
    pub fn toogle_pair(&mut self) {
        self.paired = !self.paired;
    }
    pub fn toogle_trust(&mut self) {
        self.trusted = !self.trusted;
    }
}

pub struct BltSetting {
    pub name: String,
    pub active: bool,
}

impl BltSetting {
    pub fn toggle(&mut self) {
        self.active = !self.active;
    }
}

impl ToString for DeviceDescription {
    fn to_string(&self) -> String {
        let addr_string = self.addr.to_string();
        format!("{} {} | {}", self.icon, self.name, addr_string)
    }
}

impl ToString for DeviceStatus {
    fn to_string(&self) -> String {
        format!(
            "Connected: {}\n\
            Paired: {}\n\
            Trusted: {}",
            self.connected, self.paired, self.trusted
        )
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

pub fn get_icon(icon_name: &str) -> &'static str {
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
