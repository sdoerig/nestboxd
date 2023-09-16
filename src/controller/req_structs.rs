use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use serde::{Deserialize, Serialize};

use super::validator::{is_uuid, Validator};

#[derive(Deserialize)]
pub struct NestboxReq {
    pub uuid: String,
}

impl Validator for NestboxReq {
    fn is_valid(&self) -> bool {
        is_uuid(&self.uuid)
    }
}

#[derive(Deserialize, Serialize)]
pub struct BirdReq {
    pub bird_uuid: String,
    pub bird: String,
}

#[derive(Deserialize, Serialize)]
pub struct GeolocationReq {
    pub long: f32,
    pub lat: f32,
}

#[derive(Deserialize, Serialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(rename = "file")]
    pub files: Vec<TempFile>,
}
