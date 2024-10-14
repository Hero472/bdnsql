use actix_web::{web, HttpResponse, Responder};
use futures::StreamExt;
use mongodb::{bson::{doc, oid::ObjectId}, Client, Collection, Cursor, Database};
use serde::{Deserialize, Serialize};

use crate::comment::{Comment, CommentSend};

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
#[derive(Debug, Serialize, Deserialize)]
pub struct ClassyReceive {
    unit_id: Option<ObjectId>,
    name: String,
    description: String,
    order: usize,
    video: String,
    tutor: String,
    support_material: Vec<String>
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ClassySend {
    name: String,
    description: String,
    order: usize,
    video: String,
    tutor: String,
    support_material: Vec<String>
}

pub async fn create_class(client: web::Data<mongodb::Client>, new_classy: web::Json<ClassyReceive>) -> impl Responder {
    let db: mongodb::Database = client.database("local");
    let collection: Collection<Classy> = db.collection("classes");

    let new_class_data: Classy = Classy {
        id: None,
        unit_id: new_classy.unit_id,
        name: new_classy.name.clone(),
        description: new_classy.description.clone(),
        order: new_classy.order,
        video: new_classy.video.clone(),
        tutor: new_classy.tutor.clone(),
        support_material: new_classy.support_material.clone(),
    };

    match collection.insert_one(new_class_data).await {
        Ok(insert_result) => HttpResponse::Ok().json(insert_result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

pub async fn get_classes_by_unit(client: web::Data<Client>, unit_id: web::Path<String>) -> impl Responder {
    let db: Database = client.database("local");
    let collection: Collection<Classy> = db.collection("classes");

    let unit_id: ObjectId = match ObjectId::parse_str(&*unit_id) {
        Ok(unit_id) => unit_id,
        Err(e) => return HttpResponse::BadRequest().body(format!("Error: {}", e))
    };

    let mut cursor: Cursor<Classy> = match collection.find(doc! {"_unit_id": unit_id}).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    };

    let mut classes: Vec<ClassySend> = Vec::new();

    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(classy_doc) => {

                let classy_send: ClassySend = ClassySend {
                    name: classy_doc.name,
                    description: classy_doc.description,
                    order: classy_doc.order,
                    video: classy_doc.video,
                    tutor: classy_doc.tutor,
                    support_material: classy_doc.support_material,
                };

                classes.push(classy_send)
            },
            Err(e) => eprintln!("Error processing document {:?}", e)
        }
    }

    HttpResponse::Ok().json(classes)
}

pub async fn get_comments_class(client: web::Data<Client>, class_id: web::Path<String>) -> impl Responder {
    let db: Database = client.database("local");
    let collection: Collection<Comment> = db.collection("comments");

    let class_id: ObjectId = match ObjectId::parse_str(&class_id.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format."),
    };

    let mut cursor: mongodb::Cursor<Comment> = match collection.find(doc! {
        "_reference_id": class_id,
        "reference_type": "Class"
    }).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    };

    let mut comments: Vec<CommentSend> = Vec::new();

    while let Some(comment) = cursor.next().await {
        match comment {
            Ok(doc) => {
                let comment_send: CommentSend = CommentSend {
                    author: doc.author().clone(),
                    date: doc.date(),
                    title: doc.title().clone(),
                    detail: doc.detail().clone(),
                    likes: doc.likes(),
                    dislikes: doc.dislikes(),
                };
                comments.push(comment_send);
            },
            Err(_) => return HttpResponse::InternalServerError().body("Error retrieving comments.")
        }
    }

    HttpResponse::Ok().json(comments)
}