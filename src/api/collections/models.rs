use crate::models::*;
use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use futures::future::{ready, Ready};
use http::Method;

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionV3 {
    pub id: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub name: String,
}

impl StateView<Collection> for CollectionV3 {
    fn to_state(&self) -> Collection {
        Collection {
            principal_id: match &self.user_id {
                Some(id) => u128::from_str_radix(&id, 16).unwrap_or_else(|_| new_id()),
                None => new_id(),
            },
            id: match &self.id {
                Some(id) => u128::from_str_radix(&id, 16).unwrap_or(0),
                None => 0,
            },
            name: self.name.clone(),
        }
    }

    fn from_state(state: &Collection) -> Self {
        CollectionV3 {
            id: Some(format!("{:0>32x}", state.id)),
            user_id: Some(format!("{:0>32x}", state.principal_id)),
            name: state.name.clone(),
        }
    }
}

impl From<Collection> for CollectionV3 {
    fn from(idea: Collection) -> Self {
        Self::from_state(&idea)
    }
}

impl Into<Collection> for CollectionV3 {
    fn into(self) -> Collection {
        self.to_state()
    }
}

impl Responder for CollectionV3 {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        if req.method() == Method::POST {
            let location = req.url_for("get_collection_v3", &vec![
                self.id.clone().expect("a collection id")
            ]);

            ready(Ok(HttpResponse::Created()
                .content_type("application/json")
                .header(
                    "Location",
                    location.expect("a location string").into_string(),
                )
                .json(&self)))
        } else {
            ready(Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(&self)))
        }
    }
}
