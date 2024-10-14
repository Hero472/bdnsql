use actix_web::web;

use crate::{class::{create_class, get_classes_by_unit, get_comments_class}, comment::create_comment, course::{create_course, get_available_courses, get_comments_course, get_course}, unit::{create_unit, get_units_by_course}, users::create_user};

// done
pub fn user_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/users")
        .route(web::post().to(create_user))
    );
}

// done
pub fn unit_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/units")
            .route(web::post().to(create_unit))
    )
    .service(
        web::resource("/units/{course_id}")
            .route(web::get().to(get_units_by_course))
    );
}

// done
pub fn courses_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/courses")
            .route(web::post().to(create_course))
            .route(web::get().to(get_available_courses))
    )
    .service(
        web::resource("/courses/{course_id}")
            .route(web::get().to(get_course))
    )
    .service(
        web::resource("/courses/comments/{course_id}")
            .route(web::get().to(get_comments_course))
    );
}

// done
pub fn comment_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/comments")
            .route(web::post().to(create_comment))
    );
}


pub fn classy_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/classes")
            .route(web::post().to(create_class)) 
    )
    .service(
        web::resource("/classes/unit/{unit_id}")
            .route(web::get().to(get_classes_by_unit))
    )
    .service(
        web::resource("/classes/comments/{class_id}")
            .route(web::get().to(get_comments_class))
    );
}