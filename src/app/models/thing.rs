use crate::db::DbThing;
use crate::service::ThingData;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct ApiThingData {
    name: String,
    description: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ApiThing {
    id: String,
    name: String,
    description: Option<String>,
    created_at: String,
}

impl From<DbThing> for ApiThing {
    fn from(x: DbThing) -> Self {
        Self {
            id: x.id.to_string(),
            name: x.name,
            description: x.description,
            created_at: x.created_at.to_string(),
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
