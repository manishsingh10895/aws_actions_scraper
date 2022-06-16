use regex::Regex;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub code: String,
    #[serde(rename(
        serialize = "documentURI",
        deserialize = "document_uri",
    ))]
    #[serde(alias = "documentURI", alias = "document_uri")]
    pub document_uri: String,
}

pub const BASE_PATH: &str = "https://docs.aws.amazon.com/IAM/latest/UserGuide";

const DOC_URI: &str= "https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_actions-resources-contextkeys.html";

pub const SERVICE_DATA_PATH: &str = "./data/services.json";

/// Gets a list of `Service`s from cache if already saved
#[allow(dead_code)]
pub fn get_services_from_cache() -> Option<Vec<Service>> {
    if !Path::new(SERVICE_DATA_PATH).exists() {
        return None;
    }

    let text = fs::read_to_string(SERVICE_DATA_PATH).unwrap();

    let res: Result<Vec<Service>, _> = serde_json::from_str(&text);

    if let Ok(services) = res {
        return Some(services);
    } else {
        println!("Invalid JSON file");
        return Some(vec![]);
    }
}

///Gets all services from `DOC_URI` page
#[allow(dead_code)]
pub async fn get_services() -> Vec<Service> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "user-agent",
        "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:47.0) Gecko/20100101 Firefox/47.0"
            .parse()
            .unwrap(),
    );
    let client = reqwest::Client::new();
    let response = client.get(DOC_URI).headers(headers).send().await;
    let mut services: Vec<Service> = Vec::new();

    if let Ok(html) = response {
        let html = html.text().await.unwrap();

        let document = scraper::Html::parse_document(&html);

        let anchor_selector = scraper::Selector::parse(".highlights ul li a").unwrap();

        let anchors = document.select(&anchor_selector).map(|x| x);

        for a in anchors {
            let text = a.text().next().unwrap();

            let re = Regex::new(r"/^(AWS|Amazon)\s*/").unwrap();

            re.replace_all(&text, "");

            let name = text;

            let href = a.value().attr("href").unwrap();
            println!("HREF {}", href);
            let re = Regex::new(r"^./list_(.*?)\.html$").unwrap();
            println!("RE {}", re.as_str());
            let mut code: Option<&str> = None;

            if let Some(captures) = re.captures(&href) {
                println!("CAPTURES {:?}", captures);
                let _code = captures.get(1);

                if let Some(m) = _code {
                    code = Some(m.as_str())
                }
            } else {
                println!("REGEX didn't match a thing");
            }

            if code.is_none() {
                continue;
            }

            let doc_uri = format!("{}/{}", BASE_PATH, href.trim());

            services.push(Service {
                name: String::from(name),
                code: String::from(code.unwrap()),
                document_uri: doc_uri,
            });
        }
    } else {
        println!("Error in fetching document");
        eprintln!("{:?}", response.err());
        panic!();
    }
    println!("Fetched services, {}", services.len());
    return services;
}

///Writes services to cache
#[allow(dead_code)]
pub fn save_services_to_cache(services: &Vec<Service>) {
    if let Ok(service_json) = serde_json::to_string(&services) {
        fs::write(SERVICE_DATA_PATH, service_json).unwrap();
    } else {
        eprintln!("Error in making services");
    }
}
