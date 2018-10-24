use diesel::prelude::*;
use models::User;
use pg_pool::DbConn;
use request::Admin;
use request::LoginForm;
use rocket::http::Cookie;
use rocket::http::Cookies;
use rocket::http::Status;
use rocket::request::Form;
use rocket::response::Failure;
use rocket::response::Flash;
use rocket::response::Redirect;
use rocket_contrib::Template;
use tera::Context;
use models::Article;
use request::ArticleEditForm;
use diesel;
use chrono::NaiveDateTime;
use chrono::Utc;
use request::NewPasswordForm;
use rocket::request::FlashMessage;
use models::SerializeFlashMessage;


#[get("/login")]
fn admin_login() -> Template {
    let context = Context::new();
    Template::render("admin/login", &context)
}


#[post("/login", data = "<user>")]
fn admin_authentication(user: Form<LoginForm>, conn: DbConn, mut cookies: Cookies) -> Result<Redirect, Failure> {
    use schema::{users, users::dsl::*};
    let user_form = user.get();
    let fetched = users::table.filter(username.eq(&user_form.username)).first::<User>(&*conn);
    if let Err(_) = fetched {
        return Err(Failure(Status::Unauthorized));
    }
    let user: User = fetched.unwrap();

    if !user.authenticated(user_form.password.as_str()) {
        return Err(Failure(Status::Unauthorized));
    }

    cookies.add_private(Cookie::new("LOG_SESSION", user.username));
    cookies.add_private(Cookie::new("LOG_ID", user.id.to_string()));
    cookies.add_private(Cookie::new("LOG_ADMIN", "1"));

    Ok(Redirect::to("/admin"))
}


#[get("/")]
fn admin_index(admin: Admin, conn: DbConn, flash: Option<FlashMessage>) -> Template {
    let mut context = Context::new();

    let articles = Article::load_all(true, &conn);

    println!("{:?}", flash);
    context.insert("admin", &admin);
    context.insert("articles", &articles);
    context.insert("flash", &SerializeFlashMessage::from(&flash));
    Template::render("admin/index", &context)
}


#[get("/article/new")]
fn article_creation(_admin: Admin) -> Result<Template, Failure> {
    let mut context = Context::new();

    let article = Article {
        id: -1,
        title: String::new(),
        body: String::new(),
        published: true,
        user_id: 0,
        publish_at: NaiveDateTime::from_timestamp(Utc::now().timestamp(), 0),
        url: None,
    };

    context.insert("article", &article);
    Ok(Template::render("admin/edit", context))
}


#[get("/article/<article_id>")]
fn article_edit(_admin: Admin, conn: DbConn, article_id: i32) -> Result<Template, Failure> {
    let mut context = Context::new();
    let fetched_article = Article::find(article_id, &conn);

    if let Err(_err) = fetched_article {
        return Err(Failure(Status::NotFound));
    }

    let article: Article = fetched_article.unwrap();

    context.insert("article", &article);
    Ok(Template::render("admin/edit", context))
}

#[post("/article", data = "<article>")]
fn save_article(admin: Admin, conn: DbConn, article: Form<ArticleEditForm>) -> Result<Flash<Redirect>, Failure> {
    use schema::{articles, articles::dsl::*};

    let article = Article::form_article_edit_form(article.get(), admin.id);
    let _fetched_article: QueryResult<Article> = match article.id {
        Some(article_id) => diesel::update(articles::table.find(article_id)).set(&article).get_result(&*conn),

        None => diesel::insert_into(articles::table).values(&article).get_result(&*conn),
    };

    Ok(Flash::new(Redirect::to("/admin"), "success", "created"))
}

#[post("/password", data = "<password_form>")]
fn change_password(admin: Admin, conn: DbConn, password_form: Form<NewPasswordForm>) -> Flash<Redirect> {
    use schema::{users, users::dsl::*};

    let mut admin_user: User = users::table.find(admin.id).first::<User>(&*conn).unwrap();

    admin_user.password = User::password_generate(&password_form.get().password).to_string();
    let _result: QueryResult<User> = diesel::update(users::table.find(admin_user.id)).set(&admin_user).get_result(&*conn);
    Flash::new(Redirect::moved("/admin"), "success", "password is changed successfully")
}