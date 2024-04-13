use crate::IntegrationOSError;
use actix_web::{HttpResponse, ResponseError};
use http::StatusCode;

impl ResponseError for IntegrationOSError {
    fn status_code(&self) -> StatusCode {
        self.into()
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        (&self).error_response()
    }
}

impl<'a> ResponseError for &'a IntegrationOSError {
    fn status_code(&self) -> StatusCode {
        self.to_owned().into()
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut builder = HttpResponse::build(self.status_code());

        builder.insert_header(("Content-Type", "application/json"));

        builder.json(&self.to_owned().as_application().as_json())
    }
}
