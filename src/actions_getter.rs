use std::{fs, path};

use regex::Regex;
use reqwest::header::HeaderMap;
use scraper::{ElementRef, Selector};
use serde::{Deserialize, Serialize};

use crate::service_getter::Service;

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    name: String,
    description: String,
    #[serde(rename(serialize = "documentURI"))]
    document_uri: String,
}

/// Get list of actions for service
pub async fn get_actions_for_service(
    service: &Service,
) -> Result<Vec<Action>, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "user-agent",
        "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:47.0) Gecko/20100101 Firefox/47.0"
            .parse()
            .unwrap(),
    );
    let client = reqwest::Client::new();

    println!("Fetching page with URI {} \n", service.document_uri);

    let response = client
        .get(service.document_uri.clone())
        .headers(headers)
        .send()
        .await?;

    let mut actions: Vec<Action> = Vec::new();

    let html = response;

    let html = html.text().await.unwrap();

    let document = scraper::Html::parse_document(&html);

    let prefix = format!("{}-", service.code);

    let prefix_selector = Selector::parse("#main-col-body code.code").unwrap();

    if let Some(elem) = document.select(&prefix_selector).next() {
        if let Some(service_prefix) = elem.text().next() {
            let anchor_selector_text = format!("a[id^=\"{}\"", prefix);

            let anchors_selector = Selector::parse(&anchor_selector_text).unwrap();

            let anchors = document.select(&anchors_selector);

            for a in anchors {
                let reg_text = format!(r"^{}", prefix);
                let name_re = Regex::new(&reg_text).unwrap();

                let id = a.value().attr("id").unwrap().trim();

                let action_name = name_re.replace(id, "").to_string();

                let mut is_camel_case = false;

                let first = action_name.chars().next().unwrap();

                if first == first.to_uppercase().next().unwrap() {
                    is_camel_case = true;
                }

                if !is_camel_case {
                    continue;
                }

                let name = format!("{}:{}", service_prefix, action_name);

                let node = a.parent().unwrap().parent().unwrap();
                // let el = node.value().as_element().unwrap();
                if node.value().as_element().unwrap().name() != "tr" {
                    continue;
                }

                let mut desc: String = String::new();

                for (i, c) in node
                    .children()
                    .filter(|c| c.value().is_element())
                    .enumerate()
                {
                    if i == 1 {
                        if let Some(_ref) = ElementRef::wrap(c) {
                            let texts = (_ref.text().collect::<Vec<&str>>() as Vec<&str>).join(" ");

                            desc = String::from(texts.trim());
                        }
                    }
                }

                if desc.is_empty() {
                    continue;
                }

                let mut sibs = a
                    .next_siblings()
                    .filter(|x| x.value().is_element())
                    .map(|x| ElementRef::wrap(x).unwrap().value())
                    .take(1);

                let mut href = "";

                if let Some(first_sibling) = sibs.next() {
                    href = first_sibling.attr("href").unwrap();
                } else {
                    println!("DOC URI not found for {}", name);
                }

                actions.push(Action {
                    name,
                    description: desc,
                    document_uri: String::from(href),
                });
            }
        } else {
            let message = format!("NODE HTML FOUND FOR CODE {}", service.code);
            println!("{}", message);
        }
    } else {
        let message = format!("NODE HTML FOUND FOR CODE {}", service.code);
        println!("{}", message);
        println!("Error");
    }

    Ok(actions)
}

/// Saves actions to a cache
pub fn save_actions_to_cache(
    service_name: &str,
    actions: &Vec<Action>,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(&actions)?;
    fs::write(
        path::Path::new(&format!("./data/actions/{}.json", service_name)),
        json,
    )?;

    Ok(())
}
