use futures::StreamExt;
use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;

use actix_web::{web, HttpResponse, Responder};
use mongodb::{Client, Database, Collection, bson::doc};

use crate::{comment::Comment, unit::Unit};

#[derive(Debug, Serialize, Deserialize)]
pub struct Course {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    description: String,
    #[serde(rename = "rating")]
    #[serde(skip_serializing_if = "Option::is_none")]
    rating: Option<f32>,
    image: String,
    units: Vec<Unit>,
    inscribed: u64
}

#[derive(Serialize)]
pub struct CourseSummary {
    name: String,
    description: String,
    image: String,
    rating: Option<f32>,
}

pub async fn create_course(client: web::Data<Client>, new_course: web::Json<Course>) -> impl Responder {
    let db: Database = client.database("test");
    let collection: Collection<Course> = db.collection("courses");
    
    let new_course_data: Course = new_course.into_inner();
    if let Some(rating) = new_course_data.rating {
        if rating < 1.0 || rating > 5.0 {
            return HttpResponse::BadRequest().body("Rating must be between 1 and 5.");
        }
    }

    match collection.insert_one(new_course_data).await {
        Ok(insert_result) => HttpResponse::Ok().json(insert_result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

pub async fn get_available_courses(client: web::Data<Client>) -> impl Responder {
    let db: Database = client.database("test");
    let collection: Collection<Course> = db.collection("courses");
    
    let mut cursor: mongodb::Cursor<Course> = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    };

    let mut courses: Vec<CourseSummary> = Vec::new();

    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(course_doc) => {
                let summary: CourseSummary = CourseSummary {
                    name: course_doc.name.clone(),
                    description: course_doc.description.clone(),
                    image: course_doc.image.clone(),
                    rating: course_doc.rating,
                };
                courses.push(summary);
            },
            Err(e) => eprintln!("Error processing document: {:?}", e),
        };
    }
    HttpResponse::Ok().json(courses)
}

pub async fn get_course(client: web::Data<Client>, course_id: web::Path<String>) -> impl Responder {
    let db: Database = client.database("test");
    let collection_courses: Collection<Course> = db.collection("courses");
    let collections_comments: Collection<Comment> = db.collection("comments");

    let course_id: ObjectId = match ObjectId::parse_str(&course_id.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format."),
    };

    match collection_courses.find_one(doc! { "_id": course_id }).await {
        Ok(Some(course)) => {
            let mut cursor: mongodb::Cursor<Comment> = match collections_comments.find(doc! {
                "reference_id": course_id,
                "reference_type": "course"
            }).sort(doc! {"likes": -1}).limit(3).await {
                Ok(cursor) => cursor,
                Err(e) => return HttpResponse::InternalServerError().body(format!("Error retrieving comments: {}", e)),
            };

            let mut comments: Vec<Comment> = Vec::new();

            while let Some(comment) = cursor.next().await {
                match comment {
                    Ok(doc) => comments.push(doc),
                    Err(_) => return HttpResponse::InternalServerError().body("Error retrieving comments.")
                }
            }

            HttpResponse::Ok().json(comments)

        },
        Ok(None) => HttpResponse::NotFound().body("Course not found."),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

pub async fn get_comments(client: web::Data<Client>, course_id: web::Path<String>) -> impl Responder {
    let db: Database = client.database("test");
    let collection: Collection<Comment> = db.collection("comments");

    let course_id: ObjectId = match ObjectId::parse_str(&course_id.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format."),
    };

    let mut cursor: mongodb::Cursor<Comment> = match collection.find(doc! {
        "reference_id": course_id,
        "reference_type": "course"
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