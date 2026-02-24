use iced::widget::svg::Handle;

pub(crate) struct IconSet {
    pub(crate) play: Handle,
    pub(crate) pause: Handle,
    pub(crate) reset: Handle,
    pub(crate) shuffle: Handle,
    volume0: Handle,
    volume1: Handle,
    volume2: Handle,
    volume3: Handle,
}

impl IconSet {
    pub(crate) fn new() -> Self {
        Self {
            play: Handle::from_memory(include_bytes!("../../assets/icons/play.svg")),
            pause: Handle::from_memory(include_bytes!("../../assets/icons/pause.svg")),
            reset: Handle::from_memory(include_bytes!("../../assets/icons/reset.svg")),
            shuffle: Handle::from_memory(include_bytes!("../../assets/icons/shuffle.svg")),
            volume0: Handle::from_memory(include_bytes!("../../assets/icons/volume0.svg")),
            volume1: Handle::from_memory(include_bytes!("../../assets/icons/volume1.svg")),
            volume2: Handle::from_memory(include_bytes!("../../assets/icons/volume2.svg")),
            volume3: Handle::from_memory(include_bytes!("../../assets/icons/volume3.svg")),
        }
    }

    pub(crate) fn volume_icon(&self, volume_percent: f32) -> Handle {
        if volume_percent <= 0.0 {
            self.volume0.clone()
        } else if volume_percent < 30.0 {
            self.volume1.clone()
        } else if volume_percent < 80.0 {
            self.volume2.clone()
        } else {
            self.volume3.clone()
        }
    }
}
