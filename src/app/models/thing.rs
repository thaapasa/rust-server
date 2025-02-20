use crate::db::DbThing;
use crate::service::ThingData;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct ApiThingData {
    name: String,
    description: Option<String>,
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
