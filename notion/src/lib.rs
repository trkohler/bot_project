mod spending;
pub mod document;

use models::Category;
use models::Check;
use models::RemainAmount;
use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;

use crate::document::Document;

pub const DATABASE_ID: &str = "a27c68e4c1d8481c913b2d4fabefc4c9";
const NOTION_HOST: &str = "https://api.notion.com/v1/";


#[derive(Debug, Clone)]
pub struct NotionApi {
    secret: String,
    client: Client,
}

impl NotionApi {
    pub fn from_env() -> Self {
        NotionApi {
            secret: std::env::var("NOTION_SECRET").unwrap(),
            client: Client::new(),
        }
    }
}

pub async fn make_new_notion_entry_for_check(api: NotionApi, category: &Category, check: &Check) {
    let remain = get_how_much_money_remain(&api, category.to_owned()).await;

    if let Ok(remain) = remain {
        let spending_document: Document = Document::new(DATABASE_ID.to_string()).convert_to_spending(
            category.to_string(),
            category,
            check.spent,
            remain.amount,
            &check.entries,
        );
        post_new_notion_document::<Document>(&api, spending_document).await;
    }
}

pub async fn post_new_notion_document<T: Serialize>(api: &NotionApi, document: T) {
    let endpoint = "pages";
    let assembled_url = format!("{}{}", NOTION_HOST, endpoint);
    let response = api
        .client
        .post(assembled_url)
        .header("Notion-Version", "2022-06-28")
        .bearer_auth(&api.secret)
        .json(&document)
        .send()
        .await;

    if let Ok(response) = response {
        let _json_resp = response.json::<Value>().await.unwrap();
        println!("posted");
    }
}

pub async fn get_how_much_money_remain(api: &NotionApi, category: Category) -> Result<RemainAmount, ()> {
    let endpoint = format!("databases/{}/query", DATABASE_ID);
    let kwargs = json!({
        "filter": {
            "property": "Tags",
            "multi_select": {
                "contains": category.to_string()
            }
        },
        "sorts": [{
            "property": "Date",
            "timestamp": "created_time",
            "direction": "descending"
        }]
    });
    let assembled_url = format!("{}{}", NOTION_HOST, endpoint);

    let response = api
        .client
        .post(assembled_url)
        .header("Notion-Version", "2022-06-28")
        .bearer_auth(&api.secret)
        .json(&kwargs)
        .send()
        .await;

    if let Ok(response) = response {
        let json_resp = response.json::<Value>().await.unwrap();
        let remain = json_resp.get("results").and_then(|value| {
            value.get(0).and_then(|value| {
                value.get("properties").and_then(|value| {
                    value.get("Remain").and_then(|value| {
                        value.get("formula").and_then(|value| value.get("number"))
                    })
                })
            })
        });

        if let Some(remain) = remain {
            return Ok(RemainAmount {
                category,
                amount: remain.as_f64().unwrap(),
            });
        }
    }
    Err(())
}
