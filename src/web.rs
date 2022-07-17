use std::{sync::Arc, convert::Infallible};

use maud::html;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, prelude::Uuid};
use serenity::http::StatusCode;
use sql_entities::gallery_post;
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
        .and_then(render_gallery_posts)
}

async fn render_gallery_posts(gallery_id: Uuid, db: Arc<DatabaseConnection>) -> Result<impl warp::Reply, warp::Rejection> {
    let posts_result = gallery_post::Entity::find()
        .filter(gallery_post::Column::Gallery.eq(gallery_id))
        .all(db.as_ref())
        .await;
    
    match posts_result {
        Err(why) => {
            error!("Could not load posts from db: {:?}", why);
            Err(warp::reject::custom(DbError(why)))
        }
        Ok(posts) => if posts.len() == 0 {
            warn!("Loaded zero posts from gallery id {}", gallery_id);
            Err(warp::reject())
        } else {
            debug!("Loaded {} posts from galler {}", posts.len(), gallery_id);
            let markup = html! {
                (maud::DOCTYPE)
                html {
                    head {
                        meta name="viewport" content="initial-scale=1";
                        link rel="stylesheet" href="/static/galleria.css";
                    }
                    body {
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
                    }
                }
            };
            Ok(warp::reply::with_status(warp::reply::html(markup.into_string()), StatusCode::OK))
        }
    }
}