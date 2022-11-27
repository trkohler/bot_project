use serde_json::json;
use serde_json::Value;

pub struct SpendingDocumentBulletPoints {
    pub bullets: Vec<Value>,
}

impl SpendingDocumentBulletPoints {
    pub fn new<T: ToString>(entries: &Vec<T>) -> Self {
        SpendingDocumentBulletPoints {
            bullets: entries
                .into_iter()
                .map(|line| {
                    json!({
                        "object": "block",
                        "type": "bulleted_list_item",
                        "bulleted_list_item":
                            {
                                "rich_text": [{
                                    "type": "text",
                                    "text": {
                                        "content": line.to_string(),
                                    }
                                }]
                            }
                    })
                })
                .collect::<Vec<Value>>(),
        }
    }
}
