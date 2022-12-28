use actix_web::web::{scope, ServiceConfig};

pub mod dev;

pub fn routes(cfg: &mut ServiceConfig) {
    cfg.service(scope("/developer").configure(dev::router));
}
