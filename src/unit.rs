use std::env::current_dir;

use actix_web::{web, HttpResponse, Responder};
use futures::{io::Cursor, StreamExt};
use mongodb::{bson::{doc, oid::ObjectId}, Collection};
use serde::{Deserialize, Serialize};

use crate::{class::Classy, course};



#[derive(Debug, Serialize, Deserialize)]
pub struct Unit {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    #[serde(rename = "_course_id", skip_serializing_if = "Option::is_none")]
    course_id: Option<ObjectId>,
    name: String,
    order: usize
}

pub async fn create_unit(client: web::Data<mongodb::Client>, new_unit: web::Json<Unit>) -> impl Responder {
    let db: mongodb::Database = client.database("test");
    let collection: Collection<Unit> = db.collection("units");

    let new_unit_data: Unit = new_unit.into_inner();

    match collection.insert_one(new_unit_data).await {
        Ok(insert_result) => HttpResponse::Ok().json(insert_result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

pub async fn get_units_by_course(client: web::Data<mongodb::Client>, course_id: web::Path<String>) -> impl Responder {
    let db: mongodb::Database = client.database("test");
    let collection: Collection<Unit> = db.collection("units");

    let course_id: ObjectId = match ObjectId::parse_str(&*course_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid Course ID format")
    };

    let mut cursor: mongodb::Cursor<Unit> = match collection.find(doc! {"couse_id": course_id}).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    };

    let mut units: Vec<Unit> = Vec::new();

    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(unit_doc) => units.push(unit_doc),
            Err(e) => eprintln!("Error processing document: {:?}", e),
        }
    }

    HttpResponse::Ok().json(units)
}