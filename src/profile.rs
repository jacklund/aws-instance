use dirs;
use regex::Regex;
use rusoto_core::Region;
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Profile {
    pub region: Region,
}

type Config = BTreeMap<String, Profile>;

lazy_static! {
    static ref SECTION_REGEX: Regex = Regex::new(r"^\[([^\]]*)\].*$").unwrap();
    static ref VALUE_REGEX: Regex = Regex::new(r"^\s*([a-zA-Z]*)\s*=\s*(\S*).*$").unwrap();
}

impl Profile {
    pub fn get(name: &str) -> Option<Self> {
        let config = Self::parse_config_file();
        println!("config: {:?}", config);

        config.get(name).map(|value| (*value).clone())
    }

    fn parse_config_file() -> Config {
        let file_path = Self::get_config_file_path();
        let mut config: Config = Config::new();
        let mut current_profile: Profile = Profile::default();
        if !file_path.exists() {
            config.insert("default".to_string(), current_profile);
            return config;
        }
        println!("Opening {:?}", file_path);
        let file = File::open(file_path.clone())
            .unwrap_or_else(|_| panic!("Error opening config file {:?}", file_path));
        let mut profile_name: Option<String> = None;
        for line_or_error in BufReader::new(file).lines() {
            match line_or_error {
                Err(error) => panic!("Error reading config file: {}", error),
                Ok(line) => {
                    println!("Reading line '{}'", line);
                    if SECTION_REGEX.is_match(&line) {
                        if profile_name.is_some() {
                            config.insert(profile_name.unwrap(), current_profile);
                            current_profile = Profile::default();
                        }
                        profile_name = Some(
                            SECTION_REGEX
                                .captures(&line)
                                .unwrap()
                                .get(1)
                                .unwrap()
                                .as_str()
                                .to_string(),
                        );
                    } else if VALUE_REGEX.is_match(&line) {
                        let captures = VALUE_REGEX.captures(&line).unwrap();
                        if profile_name.is_none() {
                            println!("Got entry in config outside profile");
                        } else {
                            let key = captures.get(1).unwrap().as_str();
                            if key == "region" {
                                let region_name = captures.get(2).unwrap().as_str();
                                println!("Got region {}", region_name);
                                current_profile.region = Region::from_str(region_name)
                                    .expect("Error parsing AWS config file");
                            }
                        }
                    }
                }
            };
        }

        if profile_name.is_some() {
            config.insert(profile_name.unwrap(), current_profile);
        }
        config
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

impl Default for Profile {
    fn default() -> Self {
        Profile {
            region: Region::default(),
        }
    }
}
