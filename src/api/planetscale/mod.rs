use actix_web::web::{self, ServiceConfig};

pub mod controller;

pub fn router(cfg: &mut ServiceConfig) {
    cfg.route("/users", web::get().to(controller::list_users));
    cfg.route("/users/{id}", web::get().to(controller::read_by_id));
}
