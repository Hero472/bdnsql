use futures::StreamExt;
use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;

use actix_web::{web, HttpResponse, Responder};
use mongodb::{Client, Database, Collection, bson::doc};

use crate::comment::{Comment, CommentSend};

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
    pub units: Vec<ObjectId>,
    inscribed: u64
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CourseReceive {
    name: String,
    description: String,
    rating: Option<f32>,
    image: String
}

#[derive(Debug, Serialize)]
pub struct CourseSummary {
    name: String,
    description: String,
    image: String,
    rating: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseWithComments {
    course: Course,
    comments: Vec<CommentSend>,
}

pub async fn create_course(client: web::Data<Client>, new_course: web::Json<CourseReceive>) -> impl Responder {
    let db: Database = client.database("local");
    let collection: Collection<Course> = db.collection("courses");
    
    if let Some(rating) = new_course.rating {
        if rating < 1.0 || rating > 5.0 {
            return HttpResponse::BadRequest().body("Rating must be between 1 and 5.");
        }
    }

    let new_course_data: Course = Course {
        id: None,
        name: new_course.name.clone(),
        description: new_course.description.clone(),
        rating: new_course.rating.clone(),
        image: new_course.image.clone(),
        units: Vec::new(),
        inscribed: 0,
    };
    println!("{new_course_data:?} Inserted Succesfully");
    match collection.insert_one(new_course_data).await {
        Ok(insert_result) => HttpResponse::Ok().json(insert_result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

pub async fn get_available_courses(client: web::Data<Client>) -> impl Responder {
    let db: Database = client.database("local");
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
    println!("{courses:?}");
    HttpResponse::Ok().json(courses)
}

pub async fn get_course(client: web::Data<Client>, course_id: web::Path<String>) -> impl Responder {
    let db: Database = client.database("local");
    let collection_courses: Collection<Course> = db.collection("courses");
    let collections_comments: Collection<Comment> = db.collection("comments");

    let course_id: ObjectId = match ObjectId::parse_str(&course_id.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format."),
    };

    match collection_courses.find_one(doc! { "_id": course_id }).await {
        Ok(Some(course)) => {
            let mut cursor: mongodb::Cursor<Comment> = match collections_comments.find(doc! {
                "_reference_id": course_id,
                "reference_type": "Course",
            }).sort(doc! {"likes": -1}).limit(3).await {
                Ok(cursor) => cursor,
                Err(e) => return HttpResponse::InternalServerError().body(format!("Error retrieving comments: {}", e)),
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
                    Err(_) => return HttpResponse::InternalServerError().body("Error retrieving comments."),
                }
            }
            println!("{comments:?}");
            let response: CourseWithComments = CourseWithComments {
                course,
                comments,
            };

            println!("{:?}", response);
            HttpResponse::Ok().json(response)

        },
        Ok(None) => HttpResponse::NotFound().body("Course not found."),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

pub async fn get_comments_course(client: web::Data<Client>, course_id: web::Path<String>) -> impl Responder {
    let db: Database = client.database("local");
    let collection: Collection<Comment> = db.collection("comments");

    let course_id: ObjectId = match ObjectId::parse_str(&course_id.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format."),
    };

    let mut cursor: mongodb::Cursor<Comment> = match collection.find(doc! {
        "_reference_id": course_id,
        "reference_type": "Course"
    }).sort(doc! {"likes": -1}).await {
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
            Err(_) => return HttpResponse::InternalServerError().body("Error retrieving comments."),
        }
    }

    HttpResponse::Ok().json(comments)
}