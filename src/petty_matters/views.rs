use chrono::DateTime;
use crate::petty_matters::service::TopicService;
use crate::petty_matters::topic::entity::{Topic, TopicId};
use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::get;
use axum::{Form, Router};
use std::sync::Arc;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "petty_matters/list.html")]
pub struct PettyMattersList {
    pub topics: Vec<Topic>,
}

#[derive(Template)]
#[template(path = "petty_matters/add.html")]
pub struct PettyMattersRegistration {}

#[derive(Template)]
#[template(path = "petty_matters/view.html")]
pub struct PettyMatter {
    pub topic: Topic,
}

#[derive(Deserialize)]
struct PettyMattersRegistrationForm {
    subject: String,
    content: String,
}

async fn list_petty_matters(
    State(service): State<Arc<TopicService>>,
) -> Result<impl IntoResponse, StatusCode> {
    let topics = service
        .list_topics()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let template = PettyMattersList { topics };
    let rendered = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(rendered))
}

async fn render_registration_form() -> Result<impl IntoResponse, StatusCode> {
    let template = PettyMattersRegistration {};
    let rendered = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(rendered))
}

async fn register_petty_matter(
    State(service): State<Arc<TopicService>>,
    form: Form<PettyMattersRegistrationForm>,
) -> Result<impl IntoResponse, StatusCode> {
    let topic = Topic {
        id: TopicId(Uuid::new_v4()),
        title: form.subject.clone(),
        content: form.content.clone(),
        upvotes_count: 0,
        downvotes_count: 0,
        creation_time: DateTime::default(),
        last_updated_time: None,
    };
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
    let found = service
        .get_topic(&topic_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let template = PettyMatter {
        topic: found.ok_or(StatusCode::NOT_FOUND)?,
    };
    let rendered = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(rendered))
}

pub fn topics_router(service: Arc<TopicService>) -> Router {
    Router::new()
        .route("/", get(list_petty_matters).post(register_petty_matter))
        .route("/register", get(render_registration_form))
        .route("/{topic_id}", get(view_petty_matter))
        .with_state(service)
}
