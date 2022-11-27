use chrono::Local;
use models::Category;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;

use crate::spending::SpendingDocumentBulletPoints;

#[derive(Serialize)]
pub struct Document {
    parent: Value,
    icon: Value,
    properties: Value,
    children: Vec<Value>,
}

impl Document {
    pub fn new(database_id: String) -> Document {
        Document {
            parent: json!({ "database_id": database_id }),
            properties: json!(null),
            children: Vec::new(),
            icon: json!(null),
        }
    }

    pub fn convert_to_spending<T: ToString>(
        self,
        title: String,
        category: &Category,
        spent: f64,
        remain: f64,
        entries: &Vec<T>,
    ) -> Document {
        let properties = json!({
            "Name": {
                "type": "title",
                "title": [
                    {
                        "plain_text": title,
                        "text": {
                            "content": title,
                            "link": serde_json::Value::Null},
                        "type": "text"
                    }
                ]
            },
            "Tags": {
                "multi_select": [{
                    "id": category.to_storage_id(),
                }],
                "type": "multi_select"
            },
            "Date": {"date": {"start": Local::now().to_rfc3339()}, "type": "date"},
            "Spent": {"number": -spent, "type": "number"},
            "For current period": {"number": remain, "type": "number"},
        });
        let bullets = SpendingDocumentBulletPoints::new(entries);
        Document { parent: self.parent, icon: self.icon, properties, children: bullets.bullets }
    }

    pub fn convert_to_replenishment(
        self,
        category: &Category,
        amount_to_add: f64,
    ) -> Document {
        let assembled_title = format!("{} - бот відкриває період", category.to_string());
        let properties = json!({
            "Name": {
                "type": "title",
                "title": [
                    {
                        "plain_text": assembled_title,
                        "text": {
                            "content": assembled_title,
                            "link": serde_json::Value::Null},
                        "type": "text"
                    }
                ]
            },
            "Tags": {
                "multi_select": [{
                    "id": category.to_storage_id(),
                }],
                "type": "multi_select"
            },
            "Date": {"date": {"start": Local::now().to_rfc3339()}, "type": "date"},
            "Spent": {"number": 0, "type": "number"},
            "For current period": {"number": amount_to_add, "type": "number"},
        });
        let icon = json!({"emoji": "💸"});
        Document { parent: self.parent, icon, properties, children: self.children }
    }
}
