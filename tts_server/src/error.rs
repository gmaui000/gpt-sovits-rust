use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug, Clone, Copy)]
pub enum AppError {
    #[error("convert.yaml does not exist")]
    ConfigFileLost,
}

pub fn to_integer(data: &AppError) -> u32 {
    *data as u32
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
    //错误标准返回，business_code 为内部错误码
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}
