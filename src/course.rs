use std::collections::HashMap;

use futures::StreamExt;
use neo4rs::{query, Graph};
use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;

use actix_web::{web, HttpResponse, Responder};
use mongodb::{Client, Database, Collection, bson::doc};

use crate::{class::Classy, comment::{Comment, CommentSend}, unit::{Unit, UnitFullCourse}};

#[derive(Debug, Serialize, Deserialize)]
pub struct Course {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    description: String,
    #[serde(rename = "rating")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<f32>,
    pub total_rates: usize,
    image: String,
    image_banner: String,
    pub units: Vec<ObjectId>,
    inscribed: u64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseReceive {
    name: String,
    description: String,
    rating: Option<f32>,
    image: String,
    image_banner: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FullCourse {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) image: String,
    pub(crate) image_banner: String,
    pub(crate) units: Vec<UnitFullCourse>,
}

#[derive(Debug, Serialize)]
pub struct CourseSummary {
    name: String,
    description: String,
    image_banner: String,
    rating: Option<f32>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseWithComments {
    course: Course,
    comments: Vec<CommentSend>,
}

pub async fn create_course(
    client: web::Data<Client>,
    neo4j_client: web::Data<Graph>,
    new_course: web::Json<CourseReceive>
) -> impl Responder {
    let db: Database = client.database("local");
    let collection: Collection<Course> = db.collection("courses");
    
    // Validate rating if provided
    if let Some(rating) = new_course.rating {
        if rating < 1.0 || rating > 5.0 {
            return HttpResponse::BadRequest().body("Rating must be between 1 and 5.");
        }
    }

    // Create a new course object
    let new_course_data: Course = Course {
        id: None,  // MongoDB will generate the ID
        name: new_course.name.clone(),
        description: new_course.description.clone(),
        rating: new_course.rating.clone(),
        total_rates: 0,
        image: new_course.image.clone(),
        image_banner: new_course.image_banner.clone(),
        units: Vec::new(),
        inscribed: 0,
    };

    // Insert the course into MongoDB
    match collection.insert_one(new_course_data).await {
        Ok(insert_result) => {
            // Retrieve the inserted course's ID (MongoDB generates this)
            let course_id = insert_result.inserted_id.to_string();  // Convert to String
            
            // Prepare the parameters for Neo4j query
            let params = HashMap::from([
                ("course_id", course_id.clone()),
                ("name", new_course.name.clone()),
                ("description", new_course.description.clone()),
                ("rating", new_course.rating.unwrap_or_default().to_string()),
                ("image", new_course.image.clone()),
                ("image_banner", new_course.image_banner.clone())
            ]);
            
            // Insert the course into Neo4j
            let graph = neo4j_client.get_ref();
            match graph
                .execute(
                    query(
                        "CREATE (c:Course {
                            id: $course_id, 
                            name: $name, 
                            description: $description, 
                            rating: $rating, 
                            image: $image, 
                            image_banner: $image_banner
                        })",
                    )
                    .params(params),
                )
                .await
            {
                Ok(_) => HttpResponse::Ok().json(insert_result.inserted_id),
                Err(err) => {
                    eprintln!("Neo4j error: {}", err);
                    HttpResponse::InternalServerError().body("Failed to save course in Neo4j")
                }
            }
        },
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

pub async fn create_complete_course(client: web::Data<Client>, full_course: web::Json<FullCourse>) -> impl Responder {
    let db: Database = client.database("local");
    let collection_course: Collection<Course> = db.collection("courses");
    let collection_unit: Collection<Unit> = db.collection("units");
    let collection_classy: Collection<Classy> = db.collection("classes");

    let new_course_data: Course = Course {
        id: None,
        name: full_course.name.clone(),
        description: full_course.description.clone(),
        image: full_course.image.clone(),
        image_banner: full_course.image_banner.clone(),
        rating: None,
        total_rates: 0,
        units: Vec::new(),
        inscribed: 0,
    };

    match collection_course.insert_one(new_course_data).await {
        Ok(insert_result) => {
            
            let course_id: ObjectId = insert_result.inserted_id.as_object_id().unwrap();

            for unit_receive in &full_course.units {
                let new_unit: Unit = Unit {
                    id: None,
                    course_id: Some(course_id),
                    name: unit_receive.name.clone(),
                    order: unit_receive.order,
                };

                let unit_id: ObjectId = match collection_unit.insert_one(new_unit).await {
                    Ok(insert_result) => insert_result.inserted_id.as_object_id().unwrap(),
                    Err(error) => return HttpResponse::InternalServerError().body(format!("Error inserting unit: {}", error)),
                };

                let _ = match collection_course.update_one(doc! { "_id": course_id }, doc! {"$push": { "units": unit_id }}).await {
                    Ok(_) => {},
                    Err(error) => return HttpResponse::InternalServerError().body(format!("Error inserting updating course unit array: {}", error)),
                };

                for classy_receive in &unit_receive.classes {
                    let new_class: Classy = Classy {
                        id: None,
                        unit_id: Some(unit_id),
                        name: classy_receive.name.clone(),
                        description: classy_receive.description.clone(),
                        order: classy_receive.order,
                        video: classy_receive.video.clone(),
                        tutor: classy_receive.tutor.clone(),
                        support_material: classy_receive.support_material.clone(),
                    };

                    if let Err(error) = collection_classy.insert_one(new_class).await {
                        return HttpResponse::InternalServerError().body(format!("Error inserting class: {}", error));
                    }
                }
            }

            HttpResponse::Created().body("Course created successfully")
        },
        Err(error) => HttpResponse::InternalServerError().body(format!("Error inserting course: {}", error)),
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
                    image_banner: course_doc.image_banner.clone(),
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

            let response: CourseWithComments = CourseWithComments {
                course,
                comments,
            };

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