use actix_web::web::{self, ServiceConfig};

pub mod controller;

pub fn router(cfg: &mut ServiceConfig) {
    cfg.route("/{_id}", web::get().to(controller::read_user));
    cfg.route("", web::post().to(controller::create_user));
}
