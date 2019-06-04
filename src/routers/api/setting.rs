use crate::{
    data::RubbleData,
    models::{
        setting::{Setting, UpdateSetting},
        user::User,
        CRUD,
    },
    routers::RubbleResponder,
};
use actix_web::{delete, get, post, put, web, Responder};

#[get("")]
pub fn get_settings(user: User, data: web::Data<RubbleData>) -> impl Responder {
    RubbleResponder::json(Setting::load(&data.postgres()))
}

#[put("/{key}")]
pub fn update_setting_by_key(
    user: User,
    key: web::Path<String>,
    value: web::Json<UpdateSetting>,
    data: web::Data<RubbleData>,
) -> impl Responder {
    let string = (*key).clone();
    Setting::update(&data.postgres(), string, &value)
        .map(RubbleResponder::json)
        .map_err(|_| RubbleResponder::bad_request("error on updating setting"))
}
