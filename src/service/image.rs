use actix_multipart::form::MultipartForm;

use infer;
use mongodb::bson::{doc, DateTime, Document};
use mongodb::{error::Error, Collection, Database};
use uuid::Uuid;
//use sha2::{Digest};
use std::fs::File;

use std::io::{BufReader, Read};

use super::res_structs::{ImageResponse, MapDocument};
use crate::controller::req_structs::UploadForm;
use crate::controller::utilities::SessionObject;
use sha3::{Digest, Sha3_256};

use super::bird::BIRDS;
use super::breed::BREEDS;
use super::nestbox::NESTBOX;
use super::user::USERS;

const IMAGES: &str = "images";

pub enum CollectionsWithImages {
    Birds,
    Breeds,
    Mandants,
    Nestboxes,
    Users,
}

// Having the structure:
// _id:             ObjectId

// target_uuid:     uuid
// collection:      str
// user_uuid:       uuid
// mandant_uuid:    uuid
// image_name:      str
// image_sha3_name: str
// created_date:    datetime

pub struct ImageService {
    image_directory: String,
    collection: Collection<Document>,
}

impl ImageService {
    pub fn new(image_directory: String, db: &Database) -> Self {
        ImageService {
            image_directory,
            collection: db.collection(IMAGES),
        }
    }

    pub async fn save_files(
        &self,
        MultipartForm(form): MultipartForm<UploadForm>,
        session: SessionObject,
        target_uuid: &str,
        target_collection: CollectionsWithImages,
    ) -> Option<ImageResponse> {
        let mut file_names: Vec<String> = Vec::new();
        let collection = match target_collection {
            CollectionsWithImages::Birds => BIRDS,
            CollectionsWithImages::Breeds => BREEDS,
            CollectionsWithImages::Mandants => "mandants",
            CollectionsWithImages::Nestboxes => NESTBOX,
            CollectionsWithImages::Users => USERS,
        };
        for f in form.files {
            let file_name_original = f.file_name.unwrap();
            let path = format!("{}/{}", self.image_directory, &file_name_original);
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
                if let Some(value) = self
                    .find_one_image(target_uuid, collection, &file_name)
                    .await
                {
                    return Some(value);
                } else {
                    let checksummed_path = format!("{}/{}", &self.image_directory, &file_name);
                    let fm = std::fs::rename(&path, &checksummed_path);
                    if fm.is_ok() {
                        file_names.push(file_name.clone());
                        let image = doc! {
                                            "uuid": Uuid::new_v4().to_string(),
                                            "target_uuid": &target_uuid,
                                            "target_collection": &collection,
                                            "user_uuid": session.get_user_uuid(),
                                            "mandant_uuid": session.get_mandant_uuid(),
                                            "image_name": &file_name_original,
                                            "image_sha3_name": file_name,
                                            "creation_date": DateTime::now(),
                        };
                        match self.collection.insert_one(&image, None).await {
                            Ok(_o) => "",
                            Err(_e) => "",
                        };
                    } else if std::fs::remove_file(&path).is_ok() {
                    }
                }
                return self
                    .find_one_image(target_uuid, collection, &file_name)
                    .await;
            }
        }
        None
        Some(())
    }

    async fn find_one_image(
        &self,
        target_uuid: &str,
        collection: &str,
        file_name: &String,
    ) -> Option<ImageResponse> {
        match self
            .collection
            .find_one(
                doc! { "target_uuid": &target_uuid,
                "target_collection": &collection,
                "image_sha3_name": &file_name},
                None,
            )
            .await
        {
            Ok(doc) => match doc {
                Some(doc) => return Some(ImageResponse::map_doc(&doc)),
                None => return None,
            },
            Err(_) => return None,
        };
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
