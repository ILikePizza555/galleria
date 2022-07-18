use std::sync::Arc;

use maud::html;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, prelude::Uuid, JsonValue};
use serenity::http::StatusCode;
use sql_entities::{gallery, gallery_post};
use warp::Filter;
use tracing::{debug, warn, error};

#[derive(Debug)]
struct DbError(sea_orm::DbErr);
impl warp::reject::Reject for DbError {}

pub fn galleria_service(db: Arc<DatabaseConnection>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    frontend(db)
        .or(warp::path("static").and(warp::fs::dir("static")))
}

fn frontend(db: Arc<DatabaseConnection>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("gallery" / Uuid)
        .and(warp::any().map(move || db.clone()))
        .and_then(load_posts_into_json)
        .map(render_gallery_posts)
}

async fn load_posts_into_json(gallery_id: Uuid, db: Arc<DatabaseConnection>) -> Result<Vec<JsonValue>, warp::Rejection> {
    if let Err(why) = gallery::Entity::find_by_id(gallery_id).one(db.as_ref()).await {
        return match why {
            sea_orm::DbErr::RecordNotFound(_) => Err(warp::reject()),
            _ => Err(warp::reject::custom(DbError(why)))
        };
    }
    
    gallery_post::Entity::find()
        .filter(gallery_post::Column::Gallery.eq(gallery_id))
        .into_json()
        .all(db.as_ref())
        .await
        .map_err(|err| warp::reject::custom(DbError(err)))
        .map(|posts| {
            debug!("Loaded {} posts from gallery {}", posts.len(), gallery_id);
            posts
        })
}

fn render_gallery_posts(posts: Vec<JsonValue>) -> impl warp::Reply {
    let markup = html! {
        (maud::DOCTYPE)
        html {
            head {
                meta name="viewport" content="initial-scale=1";
                link rel="stylesheet" href="/static/galleria.css";
            }
            body {
                header {
                    h1 { "G-alpha-ria" }
                }
                main #app-container { }
                script { (maud::PreEscaped(format!("var page_data = {}; console.log(page_data);", serde_json::to_string(&posts).unwrap_or("Error".to_string())))) }
                script type="module" src="/static/index.mjs";
                
            }
        }
    };
    Ok(warp::reply::with_status(warp::reply::html(markup.into_string()), StatusCode::OK))
}

/*
@if posts.is_empty() {
                    "Looks like this gallery has no entries"
                } @else {
                    #gallery role = "list" {
                        @for post in posts {
                            @if let Some(url) = post.media_url {
                            img.gallery-item
                                rel="noreferrer"
                                role = "listitem"
                                src = (url)
                                width = (post.media_width.unwrap_or_default())
                                height = (post.media_height.unwrap_or_default())
                                loading="lazy";
                            } @else {
                                #error { "There was an error loading this image." }
                            }
                        }
                    }
                } */