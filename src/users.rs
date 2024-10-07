use actix_web::{web, HttpResponse, Responder};
use mongodb::{bson::oid::ObjectId, Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct SeenCourse {
    name: String,
    unit: usize,
    class: usize
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    seen_unit_class: Vec<SeenCourse>
}

pub async fn create_user(client: web::Data<mongodb::Client>, new_user: web::Json<User>) -> impl Responder {
    let db: Database = client.database("test");
    let collection: Collection<User> = db.collection("users");
    
    let new_user_data: User = new_user.into_inner();

    match collection.insert_one(new_user_data).await {
        Ok(insert_result) => HttpResponse::Ok().json(insert_result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e))
    }
}

