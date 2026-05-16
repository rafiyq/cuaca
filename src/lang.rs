use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Lang {
    #[value(name = "en")]
    EN,
    #[value(name = "id")]
    ID,
}

impl Lang {
    pub fn humidity(&self) -> &'static str {
        match self {
            Self::EN => "Humidity",
            Self::ID => "Kelembaban",
        }
    }

    pub fn cloud_cover(&self) -> &'static str {
        match self {
            Self::EN => "Cloud Cover",
            Self::ID => "Tutupan Awan",
        }
    }

    pub fn precipitation(&self) -> &'static str {
        match self {
            Self::EN => "Precipitation",
            Self::ID => "Curah Hujan",
        }
    }

    pub fn wind(&self) -> &'static str {
        match self {
            Self::EN => "Wind",
            Self::ID => "Angin",
        }
    }

    pub fn visibility(&self) -> &'static str {
        match self {
            Self::EN => "Visibility",
            Self::ID => "Jarak Pandang",
        }
    }

    pub fn location(&self) -> &'static str {
        match self {
            Self::EN => "Location",
            Self::ID => "Lokasi",
        }
    }

    pub fn source(&self) -> &'static str {
        match self {
            Self::EN => "Source: BMKG (Badan Meteorologi, Klimatologi, dan Geofisika)",
            Self::ID => "Sumber: BMKG (Badan Meteorologi, Klimatologi, dan Geofisika)",
        }
    }

    pub fn today(&self) -> &'static str {
        match self {
            Self::EN => "Today",
            Self::ID => "Hari Ini",
        }
    }

    pub fn tomorrow(&self) -> &'static str {
        match self {
            Self::EN => "Tomorrow",
            Self::ID => "Besok",
        }
    }

    pub fn day_after_tomorrow(&self) -> &'static str {
        match self {
            Self::EN => "Day After Tomorrow",
            Self::ID => "Lusa",
        }
    }

    pub fn weather_desc_key(&self) -> &'static str {
        match self {
            Self::EN => "weather_desc_en",
            Self::ID => "weather_desc",
        }
    }
}
