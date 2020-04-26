use actix_web::{get, post, web::{self, Data, Form, Path}, App, HttpServer, Responder, HttpResponse, HttpRequest};
use crate::model::{Config, NoteDetail};
use std::sync::{Arc, Mutex};
use actix_identity::{Identity, CookieIdentityPolicy, IdentityService};
use rand::Rng;
use std::time::Duration;
use tera::{Tera, Context};
use std::collections::HashMap;


pub mod gist;
pub mod model;
pub mod dto;


#[get("/")]
async fn index(id: Identity, tera: Data<Arc<Tera>>, config: Data<Arc<Config>>) -> impl Responder {
    if id.identity().is_none() {
        return HttpResponse::Found().header("location", "/login").finish();
    }

    let mut context = Context::new();
    context.insert("links", &config.links);
    let string = tera.render("homepage.html", &context).unwrap();
    HttpResponse::Ok().body(string)
}

#[get("/notes/{name}")]
async fn note_detail(
    id: Identity,
    name: Path<(String, )>,
    tera: Data<Arc<Tera>>,
    config: Data<Arc<Config>>,
    cache: Data<Arc<Mutex<HashMap<String, NoteDetail>>>>,
) -> impl Responder {
    if id.identity().is_none() {
        return HttpResponse::Found().header("location", "/login").finish();
    }
    let name = name.into_inner().0;

    let filter = config.links.iter().find(|link| link.name.eq(&name));
    if let Some(notelink) = filter {
        let mut guard = cache.lock().unwrap();

        if !guard.contains_key(&name) {
            let result = gist::get_gist_file_content(&name).await;
            let a = match result {
                Ok(files) => {
                    files.into_iter().find(|item| item.name.eq("content.md"))
                }
                Err(_) => {
                    return HttpResponse::NotFound().finish();
                }
            };
            a.map(|file| NoteDetail {
                title: notelink.title.clone(),
                content: file.text,
                create_at: notelink.create_at,
            }).and_then(|detail| {
                guard.insert(name.clone(), detail)
            });
        };
        let option = guard.get(&name);
        if let Some(notedetail) = option {
            let mut context = Context::new();
            context.insert("note", &notedetail);
            let string = tera.render("note.html", &context).unwrap();
            return HttpResponse::Ok().body(string);
        }
    }

    HttpResponse::NotFound().finish()
}


#[get("/login")]
async fn login_page(id: Identity, req: HttpRequest, tera: Data<Arc<Tera>>) -> impl Responder {
    if id.identity().is_some() {
        HttpResponse::Found()
            .header(
                "location"
                , req
                    .headers()
                    .get("to")
                    .map(|header| header.to_str().unwrap_or("/"))
                    .unwrap_or("/"),
            )
            .finish()
    } else {
        let result = tera.render("login.html", &Context::new()).unwrap();
        HttpResponse::Ok()
            .body(result)
    }
}

#[post("/login")]
async fn login_request(id: Identity, req: HttpRequest, login_request: Form<dto::login::LoginRequest>, data: Data<Arc<Config>>) -> impl Responder {
    if id.identity().is_some() {
        HttpResponse::Found()
            .header(
                "location"
                , req
                    .headers()
                    .get("to")
                    .map(|header| header.to_str().unwrap_or("/"))
                    .unwrap_or("/"),
            )
            .finish()
    } else {
        if login_request.password.eq(&data.password) {
            id.remember("user".to_string());
            HttpResponse::Found().header("location", "/").finish()
        } else {
            HttpResponse::Found().header("location", "/login").finish()
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let tera = Arc::new(match Tera::new("template/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    });
    let result = gist::get_secret_gist_list().await.expect("cannot get list");
    let option = result.into_iter().find(|item| item.description.eq("darling-data-config"));
    if let Some(config) = option {
        let config_name = config.name;
        let vec = gist::get_gist_file_content(config_name).await.expect("cannot get config");
        let file = vec.into_iter().find(|item| item.name.eq("config.toml")).expect("cannot config data");
        let config1 = Arc::new(Config::from_raw(file.text));


        let cache: Arc<Mutex<HashMap<String, NoteDetail>>> = Arc::new(Mutex::new(HashMap::new()));

        // let private_key = rand::thread_rng().gen::<[u8; 32]>();
        let private_key = [0; 32];
        HttpServer::new(move || App::new()
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&private_key)
                    .name("auth-example")
                    .max_age(60 * 60 * 24 * 30)
                    .secure(false),
            ))
            .data(config1.clone())
            .data(tera.clone())
            .data(cache.clone())
            .service(index)
            .service(login_page)
            .service(login_request)
            .service(note_detail)
        )

            .bind("127.0.0.1:8080")?
            .run()
            .await
    } else {
        Ok(())
    }
}