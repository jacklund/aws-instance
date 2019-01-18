use dirs;
use regex::Regex;
use rusoto_core::Region;
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path;
use std::str::FromStr;

type ConfigMap = BTreeMap<String, Profile>;

lazy_static! {
    static ref SECTION_REGEX: Regex = Regex::new(r"^\[([^\]]*)\].*$").unwrap();
    static ref VALUE_REGEX: Regex = Regex::new(r"^\s*([a-zA-Z]*)\s*=\s*(\S*).*$").unwrap();
}

#[derive(Clone, Debug)]
pub struct Config {
    map: ConfigMap,
    profile_name: Option<String>,
    current_profile: Profile,
}

impl Default for Config {
    fn default() -> Self {
        let mut map = ConfigMap::new();
        map.insert("default".to_string(), Profile::default());

        Config {
            map,
            profile_name: None,
            current_profile: Profile::default(),
        }
    }
}

impl Config {
    fn get() -> Self {
        let mut config = Config::default();
        config.parse_config_file(&Config::get_config_file_path());

        config
    }

    fn add_profile(&mut self, name: &str, profile: Profile) {
        self.map.insert(name.to_string(), profile);
    }

    fn get_profile(&self, name: &str) -> Option<&Profile> {
        let profile_name = if name == "default" {
            name.to_string()
        } else {
            format!("profile {}", name)
        };
        self.map.get(&profile_name)
    }

    fn parse_config_file(&mut self, path: &path::PathBuf) {
        if path.exists() {
            let file = File::open(path.clone())
                .unwrap_or_else(|_| panic!("Error opening config file {:?}", path));
            for line_or_error in BufReader::new(file).lines() {
                match line_or_error {
                    Err(error) => panic!("Error reading config file: {}", error),
                    Ok(line) => {
                        self.parse_line(&line);
                    }
                };
            }

            if self.profile_name.is_some() {
                let name = self.clone().profile_name.unwrap();
                let profile = self.current_profile.clone();
                self.add_profile(&name, profile);
            }
        }
    }

    fn parse_line(&mut self, line: &str) {
        if SECTION_REGEX.is_match(&line) {
            self.set_section_name(
                SECTION_REGEX
                    .captures(&line)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str(),
            );
        } else if VALUE_REGEX.is_match(&line) {
            let captures = VALUE_REGEX.captures(&line).unwrap();
            self.set_value(
                captures.get(1).unwrap().as_str(),
                captures.get(2).unwrap().as_str(),
            );
        }
    }

    fn set_section_name(&mut self, section_name: &str) {
        if self.profile_name.is_some() {
            let config = self.clone();
            let profile_name = config.profile_name.unwrap().clone();
            let profile = config.current_profile.clone();
            self.add_profile(&profile_name, profile);
            self.current_profile = Profile::default();
        }
        self.profile_name = Some(section_name.to_string());
    }

    fn set_value(&mut self, key: &str, value: &str) {
        if self.profile_name.is_none() {
            println!("Got entry in config outside profile");
        } else if key == "region" {
            let region_name = value;
            self.current_profile.region =
                Region::from_str(region_name).expect("Error parsing AWS config file");
        }
    }

    fn get_config_file_path() -> path::PathBuf {
        match env::var_os("AWS_CONFIG_FILE") {
            Some(value) => path::PathBuf::from(value),
            None => Self::get_default_config_file_path(),
        }
    }

    fn get_default_config_file_path() -> path::PathBuf {
        let mut config_file_path = dirs::home_dir().expect("Unable to locate home directory");
        config_file_path.push(".aws");
        config_file_path.push("config");

        config_file_path
    }
}

#[derive(Clone, Debug)]
pub struct Profile {
    pub region: Region,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            region: Region::default(),
        }
    }
}

impl Profile {
    pub fn get(name: &str) -> Option<Profile> {
        let config = Config::get();

        config.get_profile(name).cloned()
    }
}
