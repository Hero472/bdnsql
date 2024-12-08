use actix_web::{web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use mongodb::{bson::{oid::ObjectId, Bson}, Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TypeComment {
    Class,
    Course
}

impl From<TypeComment> for Bson {
    fn from(item: TypeComment) -> Self {
        match item {
            TypeComment::Class => Bson::String("Class".to_string()),
            TypeComment::Course => Bson::String("Course".to_string()),
        }
    }
}

impl std::fmt::Display for TypeComment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TypeComment::Class => write!(f, "Class"),
            TypeComment::Course => write!(f, "Course"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<ObjectId>,
    pub(crate) author: String,
    pub(crate) date: DateTime<Utc>,
    pub(crate) title: String,
    pub(crate) detail: String,
    pub(crate) likes: usize,
    pub(crate) dislikes: usize,
    #[serde(rename = "_reference_id", skip_serializing_if = "Option::is_none")]
    pub(crate) reference_id: Option<ObjectId>,
    pub(crate) reference_type: TypeComment
}

impl Comment {
    pub fn author(&self) -> &String {
        &self.author
    }

    pub fn date(&self) -> DateTime<Utc> {
        self.date
    }

    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn detail(&self) -> &String {
        &self.detail
    }

    pub fn likes(&self) -> usize {
        self.likes
    }

    pub fn dislikes(&self) -> usize {
        self.dislikes
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentReceive {
    pub(crate) author: String,
    pub(crate) title: String,
    pub(crate) detail: String,
    pub(crate) reference_id: Option<ObjectId>,
    pub(crate) reference_type: TypeComment
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentSend {
    pub(crate) author: String,
    pub(crate) date: DateTime<Utc>,
    pub(crate) title: String,
    pub(crate) detail: String,
    pub(crate) likes: usize,
    pub(crate) dislikes: usize,
}

pub async fn create_comment(client: web::Data<mongodb::Client>, new_comment: web::Json<CommentReceive>) -> impl Responder {
    let db: Database = client.database("local");
    let collection: Collection<Comment> = db.collection("comments");
    
    let new_comment_data: Comment = Comment {
        id: None,
        author: new_comment.author.clone(),
        date: Utc::now(),
        title: new_comment.title.clone(),
        detail: new_comment.detail.clone(),
        likes: 0,
        dislikes: 0,
        reference_id: new_comment.reference_id.clone(),
        reference_type: new_comment.reference_type.clone(),
    };
    match collection.insert_one(new_comment_data).await {
        Ok(insert_result) => HttpResponse::Ok().json(insert_result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e))
    }
}