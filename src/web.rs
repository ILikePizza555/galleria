use std::{sync::Arc, convert::Infallible};

use maud::html;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use serenity::http::StatusCode;
use sql_entities::gallery_posts;
use warp::Filter;
use tracing::{debug, warn, error};

pub fn galleria_service(db: Arc<DatabaseConnection>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("gallery" / i32)
        .and(warp::any().map(move || db.clone()))
        .and_then(render_gallery_posts)
        .or(
            warp::path("static").and(warp::fs::dir("static"))
        )
}

async fn render_gallery_posts(gallery_id: i32, db: Arc<DatabaseConnection>) -> Result<impl warp::Reply, Infallible> {
    let posts_result = gallery_posts::Entity::find()
        .filter(gallery_posts::Column::Gallery.eq(gallery_id))
        .all(db.as_ref())
        .await;
    
    match posts_result {
        Err(why) => {
            error!("Could not load posts from db: {:?}", why);
            Ok(warp::reply::with_status(warp::reply::html("500 Internal Server Error".to_string()), StatusCode::INTERNAL_SERVER_ERROR))
        }
        Ok(posts) => if posts.len() == 0 {
            warn!("Loaded zero posts from gallery id {}", gallery_id);
            Ok(warp::reply::with_status(warp::reply::html("404 Not Found".to_string()), StatusCode::NOT_FOUND))
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
                                img.gallery-item role = "listitem" src = (post.link) loading="lazy";
                            }
                        }
                    }
                }
            };
            Ok(warp::reply::with_status(warp::reply::html(markup.into_string()), StatusCode::OK))
        }
    }
}