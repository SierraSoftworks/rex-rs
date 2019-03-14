use rocket_contrib::json::Json;

#[derive(Serialize, Deserialize)]
pub struct Error {
    pub code: u16,
    pub error: String,
    pub description: String,
}

#[catch(404)]
pub fn error_404(_req: &rocket::Request) -> Json<Error> {
    Json(Error{
        code: 404,
        error: "Not Found".into(),
        description: "The resource you requested could not be found, please check your request and try again.".into(),
    })
}

#[catch(422)]
pub fn error_422(_req: &rocket::Request) -> Json<Error> {
    Json(Error{
        code: 422,
        error: "Unprocessable Entity".into(),
        description: "The request you submitted could not be processed according to the required schema. Please check your request and try again.".into(),
    })
}