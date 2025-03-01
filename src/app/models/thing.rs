use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::DbThing;
use crate::service::ThingData;

#[derive(Deserialize, Debug, Clone)]
pub struct ApiThingData {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ApiThing {
    id: Uuid,
    name: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<DbThing> for ApiThing {
    fn from(x: DbThing) -> Self {
        Self {
            id: x.id,
            name: x.name,
            description: x.description,
            created_at: x.created_at,
        }
    }
}

impl From<ApiThingData> for ThingData {
    fn from(x: ApiThingData) -> Self {
        Self {
            name: x.name,
            description: x.description,
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_json::assert_json;
    use chrono::Utc;
    use serde_json::Value;
    use uuid::uuid;

    use crate::app::models::ApiThing;

    #[test]
    fn test_thing_serialization() {
        let thing = ApiThing {
            id: uuid!("019524da-be46-7553-94c9-490815a51432"),
            name: "Thingy".to_string(),
            description: None,
            created_at: Utc::now(),
        };
        let serialized = serde_json::to_string(&thing).unwrap();
        let json: Value = serde_json::from_str(&serialized).unwrap();
        assert_json!(json, {
            "id": "019524da-be46-7553-94c9-490815a51432",
            "name": "Thingy",
            "description": null,
        })
    }
}
