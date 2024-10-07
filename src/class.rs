use actix_web::{web, HttpResponse, Responder};
use futures::StreamExt;
use mongodb::{bson::{doc, oid::ObjectId}, Client, Collection, Cursor, Database};
use serde::{Deserialize, Serialize};

use crate::comment::Comment;



#[derive(Debug, Serialize, Deserialize)]
pub struct Classy {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    #[serde(rename = "_unit_id", skip_serializing_if = "Option::is_none")]
    unit_id: Option<ObjectId>,
    name: String,
    description: String,
    order: usize,
    video: String,
    tutor: String,
    support_material: Vec<String>
}

pub async fn create_class(client: web::Data<mongodb::Client>, new_user: web::Json<Classy>) -> impl Responder {
    let db: mongodb::Database = client.database("test");
    let collection: Collection<Classy> = db.collection("classes");

    let new_class_data: Classy = new_user.into_inner();

    match collection.insert_one(new_class_data).await {
        Ok(insert_result) => HttpResponse::Ok().json(insert_result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

pub async fn get_classes_by_unit(client: web::Data<Client>, unit_id: web::Path<String>) -> impl Responder {
    let db: Database = client.database("test");
    let collection: Collection<Classy> = db.collection("classes");

    let unit_id: ObjectId = match ObjectId::parse_str(&*unit_id) {
        Ok(unit_id) => unit_id,
        Err(e) => return HttpResponse::BadRequest().body(format!("Error: {}", e))
    };

    let mut cursor: Cursor<Classy> = match collection.find(doc! {"unit_id": unit_id}).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    };

    let mut classes: Vec<Classy> = Vec::new();

    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(classy_doc) => classes.push(classy_doc),
            Err(e) => eprintln!("Error processing document {:?}", e)
        }
    }

    HttpResponse::Ok().json(classes)
}

pub async fn get_comments(client: web::Data<Client>, class_id: web::Path<String>) -> impl Responder {
    let db: Database = client.database("test");
    let collection: Collection<Comment> = db.collection("comments");

    let class_id: ObjectId = match ObjectId::parse_str(&class_id.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format."),
    };

    let mut cursor: mongodb::Cursor<Comment> = match collection.find(doc! {
        "reference_id": class_id,
        "reference_type": "class"
    }).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    };

    let mut comments: Vec<Comment> = Vec::new();

    while let Some(comment) = cursor.next().await {
        match comment {
            Ok(doc) => comments.push(doc),
            Err(_) => return HttpResponse::InternalServerError().body("Error retrieving comments.")
        }
    }

    HttpResponse::Ok().json(comments)
}