use actix_web::{web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use mongodb::{bson::oid::ObjectId, Collection, Database};
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    author: String,
    date: DateTime<Utc>,
    title: String,
    detail: String,
    likes: usize,
    dislikes: usize,
    #[serde(rename = "_reference_id", skip_serializing_if = "Option::is_none")]
    reference_id: Option<ObjectId>,
    reference_type: String
}

pub async fn create_comment(client: web::Data<mongodb::Client>, new_comment: web::Json<Comment>) -> impl Responder {
    let db: Database = client.database("test");
    let collection: Collection<Comment> = db.collection("comments");
    
    let mut new_comment_data: Comment = new_comment.into_inner();
    new_comment_data.date = Utc::now();

    match collection.insert_one(new_comment_data).await {
        Ok(insert_result) => HttpResponse::Ok().json(insert_result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e))
    }
}
