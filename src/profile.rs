use dirs;
use regex::Regex;
use rusoto_core::Region;
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;

use crate::{AwsInstanceError, Result};

type ConfigMap = BTreeMap<String, Profile>;

lazy_static! {
    static ref SECTION_REGEX: Regex = Regex::new(r"^\[([^\]]*)\].*$").unwrap();
    static ref AWS_CONFIG_SECTION_REGEX: Regex = Regex::new(r"^\[(?:profile )?(.*)\].*$").unwrap();
    static ref VALUE_REGEX: Regex = Regex::new(r"^\s*(\S*)\s*=\s*(\S*).*$").unwrap();
}

#[derive(Clone, Debug)]
pub struct Profile {
    pub region: Region,
    pub keypair: Option<String>,
    pub ssh_key: Option<String>,
    pub default_instance_type: Option<String>,
    pub security_groups: Option<Vec<String>>,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            region: Region::default(),
            keypair: None,
            ssh_key: None,
            default_instance_type: None,
            security_groups: None,
        }
    }
}

impl Profile {
    fn add_value(&mut self, name: &str, value: &str) {
        match name {
            "region" => {
                self.region = Region::from_str(value).expect("Error parsing AWS config file");
            }
            "keypair" => self.keypair = Some(value.to_string()),
            "key" => self.ssh_key = Some(value.into()),
            "instance-type" => self.default_instance_type = Some(value.to_string()),
            "security-groups" => {
                self.security_groups = Some(
                    value
                        .split(", ")
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>(),
                )
            }
            _ => (),
        }
    }
}

pub fn get_profile(profile_name: &str, config_file: &ConfigFileReader) -> Result<Profile> {
    match config_file.get_profile(profile_name) {
        Some(profile) => Ok(profile.clone()),
        None => Err(AwsInstanceError::ProfileNotFoundError {
            profile_name: profile_name.into(),
        }),
    }
}

pub fn get_aws_config_file_path() -> PathBuf {
    match env::var_os("AWS_CONFIG_FILE") {
        Some(value) => PathBuf::from(value),
        None => {
            let mut config_path = dirs::home_dir().expect("Home directory not found");
            config_path.push(".aws");
            config_path.push("config");

            config_path
        }
    }
}

pub fn get_our_config_file_path() -> PathBuf {
    match env::var_os("AWS_INSTANCE_CONFIG_FILE") {
        Some(value) => PathBuf::from(value),
        None => {
            let mut config_path = dirs::home_dir().expect("Home directory not found");
            config_path.push(".aws-instance");
            config_path.push("config");

            config_path
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConfigFileReader {
    config_map: ConfigMap,
    profile_name: Option<String>,
    current_profile: Profile,
}

impl ConfigFileReader {
    pub fn new() -> Self {
        let mut reader = ConfigFileReader {
            config_map: ConfigMap::default(),
            profile_name: None,
            current_profile: Profile::default(),
        };

        reader.parse(&get_aws_config_file_path(), &AWS_CONFIG_SECTION_REGEX);
        reader.parse(&get_our_config_file_path(), &SECTION_REGEX);

        reader
    }

    fn parse(&mut self, file_path: &PathBuf, section_regex: &Regex) {
        if file_path.exists() {
            debug!("Parsing {:?}", file_path);
            let file = File::open(file_path.clone())
                .unwrap_or_else(|_| panic!("Error opening config file {:?}", file_path));
            for line_or_error in BufReader::new(file).lines() {
                match line_or_error {
                    Err(error) => panic!("Error reading config file: {}", error),
                    Ok(line) => {
                        self.parse_line(&line, section_regex);
                    }
                };
            }

            if self.profile_name.is_some() {
                let name = self.clone().profile_name.unwrap();
                let profile = self.current_profile.clone();
                self.add_profile(name, profile);
            }
        }
    }

    fn parse_line(&mut self, line: &str, section_regex: &Regex) {
        debug!("Parsing line '{}'", line);
        if section_regex.is_match(&line) {
            let section_name = section_regex
                .captures(&line)
                .unwrap()
                .get(1)
                .expect("No section name found")
                .as_str();
            debug!("Parsed section name '{}'", section_name);
            self.set_section_name(section_name);
        } else if VALUE_REGEX.is_match(&line) {
            let captures = VALUE_REGEX.captures(&line).unwrap();
            let key = captures.get(1).unwrap().as_str();
            let value = captures.get(2).unwrap().as_str();
            debug!("Parsed key = {}, value = {}", key, value);
            self.set_value(key, value);
        }
    }

    fn set_section_name(&mut self, section_name: &str) {
        if self.profile_name.is_some() {
            let config = self.clone();
            let profile_name = config.profile_name.unwrap().clone();
            let profile = config.current_profile.clone();
            self.add_profile(profile_name, profile);
            self.current_profile = Profile::default();
        }
        debug!("Setting profile name to {}", section_name);
        self.profile_name = Some(section_name.to_string());
    }

    fn set_value(&mut self, key: &str, value: &str) {
        self.current_profile.add_value(key, value);
    }

    fn add_profile(&mut self, name: String, profile: Profile) {
        debug!("Adding profile named {}: {:?}", name, profile);
        self.config_map.insert(name, profile);
    }

    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.config_map.get(name)
    }
}
