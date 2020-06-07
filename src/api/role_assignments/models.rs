use crate::models::*;

use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use futures::future::{ready, Ready};
use http::Method;

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleAssignmentV3 {
    #[serde(rename="collectionId")]
    pub collection_id: Option<String>,
    #[serde(rename="userId")]
    pub user_id: Option<String>,
    pub role: String,
}

impl StateView<RoleAssignment> for RoleAssignmentV3 {
    fn to_state(&self) -> RoleAssignment {
        RoleAssignment {
            principal_id: match &self.user_id {
                Some(id) => u128::from_str_radix(&id, 16).unwrap_or_else(|_| new_id()),
                None => new_id(),
            },
            collection_id: match &self.collection_id {
                Some(id) => u128::from_str_radix(&id, 16).unwrap_or(0),
                None => 0,
            },
            role: self.role.as_str().into(),
        }
    }

    fn from_state(state: &RoleAssignment) -> Self {
        RoleAssignmentV3 {
            user_id: Some(format!("{:0>32x}", state.principal_id)),
            collection_id: Some(format!("{:0>32x}", state.collection_id)),
            role: state.role.into(),
        }
    }
}

impl From<RoleAssignment> for RoleAssignmentV3 {
    fn from(idea: RoleAssignment) -> Self {
        Self::from_state(&idea)
    }
}

impl Into<RoleAssignment> for RoleAssignmentV3 {
    fn into(self) -> RoleAssignment {
        self.to_state()
    }
}

impl Responder for RoleAssignmentV3 {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        if req.method() == Method::POST {
            let location = req.url_for("get_role_assignment_v3", &vec![
                self.collection_id.clone().expect("a collection id"),
                self.user_id.clone().expect("a user id")
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
