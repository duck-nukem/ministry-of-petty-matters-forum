use std::collections::HashMap;
use crate::petty_matters::service::TopicService;
use crate::petty_matters::topic::{Topic, TopicId};
use crate::time::Seconds;
use crate::view::cache_response;
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;
use std::sync::Arc;
use crate::persistence::repository::{ListParameters, Page, PageNumber, PageSize};
use crate::petty_matters::comment::Comment;

#[derive(Template)]
#[template(path = "petty_matters/list.html")]
pub struct PettyMattersList {
    pub topics: Page<Topic>,
}

#[derive(Template)]
#[template(path = "petty_matters/add.html")]
pub struct PettyMattersRegistration {}

#[derive(Template)]
#[template(path = "petty_matters/view.html")]
pub struct PettyMatter {
    pub topic: Topic,
    pub comments: Vec<Comment>
}

#[derive(Deserialize)]
struct PettyMattersRegistrationForm {
    subject: String,
    content: String,
}

#[derive(Deserialize)]
struct CommentForm {
    content: String,
}

#[derive(Clone, Deserialize)]
struct Pagination {
    page: Option<PageNumber>,
    page_size: Option<PageSize>,
}

async fn list_petty_matters(
    State(service): State<Arc<TopicService>>,
    pagination: Query<Pagination>
) -> Result<impl IntoResponse, StatusCode> {
    let list_parameters = ListParameters {
        page_number: pagination.page.clone().unwrap_or(PageNumber(1)),
        page_size: pagination.page_size.clone().unwrap_or(PageSize(10)),
        filters: None,
    };
    let topics = service
        .list_topics(list_parameters)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let template = PettyMattersList { topics }
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(template))
}

async fn render_registration_form() -> Result<impl IntoResponse, StatusCode> {
    let template = PettyMattersRegistration {}
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(template))
}

async fn register_petty_matter(
    State(service): State<Arc<TopicService>>,
    form: Form<PettyMattersRegistrationForm>,
) -> Result<impl IntoResponse, StatusCode> {
    let topic = Topic::new(form.subject.clone(), form.content.clone());
    service
        .create_topic(topic)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Redirect::to("/petty-matters"))
}

async fn view_petty_matter(
    Path(topic_id): Path<TopicId>,
    State(service): State<Arc<TopicService>>,
) -> Result<impl IntoResponse, StatusCode> {
    let topic = service
        .get_topic(&topic_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let comments = service.list_comments(&topic_id, ListParameters {
        page_size: PageSize(1000),
        page_number: PageNumber(1),
        filters:  Some(HashMap::from([
            ("topic_id".to_string(), topic_id.to_string()),
        ])),
    })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let template = PettyMatter { topic, comments: comments.items }
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(cache_response(Html(template), Some(Seconds(60))))
}

async fn add_comment(
    Path(topic_id): Path<TopicId>,
    State(service): State<Arc<TopicService>>,
    form: Form<CommentForm>,
) -> Result<impl IntoResponse, StatusCode> {
    service
        .reply_to_topic(&topic_id, form.content.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Redirect::to(&format!("/petty-matters/{topic_id}")))
}

pub fn topics_router(service: Arc<TopicService>) -> Router {
    Router::new()
        .route("/", get(list_petty_matters).post(register_petty_matter))
        .route("/register", get(render_registration_form))
        .route("/{topic_id}", get(view_petty_matter))
        .route("/{topic_id}/comments", post(add_comment))
        .with_state(service)
}
