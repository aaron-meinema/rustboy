use serde::{Serialize, Deserialize};
use std::{fs::{self, File}, io::Write};

const PATH: &str = "settings.json";
#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub render_scale: u32
}

impl Settings {
    pub fn get_settings() -> Self {
        let file = File::open(PATH);
        let settings = match file {
            Ok(_) => Self::serialize(PATH),
            Err(_) => Self::default(),
    };

    settings
    }

    pub fn deserialize(&self) {
        let result = fs::File::create(PATH);
        match result {
            Ok(file) => self.write(file),
            Err(error) => println!("{}", error)
        }
    }

    fn write(&self, mut file: File) {
        let result = serde_json::to_string(&self);
        match result {
            Ok(text) => file.write_all(text.as_bytes()).unwrap(),
            Err(error) => println!("{}", error)
        }
    }

    fn default() -> Self {
        let default = Settings {
            render_scale: 1
        };

        default.deserialize();
        default
    }

    fn serialize(path: &str) -> Self {
        let text = fs::read_to_string(path).unwrap();
        serde_json::from_str(&text).unwrap()
    }
}

