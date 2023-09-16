use actix_multipart::form::MultipartForm;

use infer;
//use sha2::{Digest};
use std::fs::File;

use std::io::{BufReader, Read};

use sha3::{Digest, Sha3_256};

use crate::controller::req_structs::UploadForm;
pub struct ImageService {
    image_directory: String,
}

impl ImageService {
    pub fn new(image_directory: String) -> Self {
        ImageService { image_directory }
    }

    pub async fn save_files(
        &self,
        MultipartForm(form): MultipartForm<UploadForm>,
    ) -> Option<Vec<String>> {
        let mut file_names: Vec<String> = Vec::new();
        for f in form.files {
            let path = format!("{}/{}", self.image_directory, f.file_name.unwrap());
            //log::info!("saving to {path}");
            f.file.persist(&path).unwrap();

            let kind = infer::get_from_path(&path);
            let sha3_checksum = sha256_str(&path).await;
            if sha3_checksum.is_some() && kind.is_ok() {
                let file_name = format!(
                    "{}.{}",
                    sha3_checksum.unwrap(),
                    kind.unwrap().unwrap().extension()
                );
                let checksummed_path = format!("{}/{}", &self.image_directory, &file_name);
                let fm = std::fs::rename(&path, &checksummed_path);
                if fm.is_ok() {
                    file_names.push(file_name);
                } else if std::fs::remove_file(&path).is_ok() {
                }
            }
        }
        Some(file_names)
    }
}

async fn sha256_str(path: &str) -> Option<String> {
    if let Ok(inner) = File::open(path) {
        let mut reader = BufReader::new(inner);
        let mut hasher = Sha3_256::new();
        let mut buffer = [0; 1024];
        loop {
            let count = reader.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        Some(hex::encode(hasher.finalize()))
    } else {
        None
    }
}
