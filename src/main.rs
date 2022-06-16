use std::{
    fs::{self, OpenOptions},
    io::Write,
    path,
};

use tokio;

mod actions_getter;
mod service_getter;

#[tokio::main]
async fn main() {
    let services = service_getter::get_services().await;

    service_getter::save_services_to_cache(&services);

    // NOTE: Uncomment if services are already saved and need to load from cache
    // let services = service_getter::get_services_from_cache().unwrap();

    if !path::Path::new("./data").is_dir() {
        fs::create_dir("./data").unwrap();
    }

    if !path::Path::new("./data/actions").is_dir() {
        fs::create_dir("./data/actions").unwrap();
    }

    if services.len() > 0 {
        for s in services {
            let getter = actions_getter::get_actions_for_service(&s).await;
            if let Ok(actions) = getter {
                if let Ok(()) = actions_getter::save_actions_to_cache(&s.code, &actions) {
                    // println!("Actions saved for {}", &s.code);
                } else {
                    println!("Error in saving cache for {}", s.code);
                }
            } else {
                let mut file = OpenOptions::new().append(true).open("errors.txt").unwrap();

                let e = format!("Error getting actions for {}", s.name);
                let ee = getter.err().unwrap().to_string();

                file.write_all(e.as_bytes()).unwrap();
                file.write_all(ee.as_bytes()).unwrap();

                println!("{}", e);
            }
        }
    }
}
