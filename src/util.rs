extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;

pub fn get_name(instance: &rusoto_ec2::Instance) -> String {
    match instance.tags {
        Some(ref tags) => {
            for tag in tags {
                if let Some("Name") = tag.key.as_ref().map(String::as_str) {
                    match tag.value {
                        Some(ref value) => return value.to_string(),
                        None => return String::new(),
                    }
                }
            }
        },
        None => return String::new(),
    }

    String::new()
}

pub fn get_state(instance: &rusoto_ec2::Instance) -> String {
    match instance.state {
        Some(ref state) => {
            match state.name {
                Some(ref state_name) => state_name.to_string(),
                None => String::new(),
            }
        },
        None => String::new(),
    }
}