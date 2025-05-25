use crate::templates::{filters, Nonce};
use crate::persistence::repository::{ListParameters, Page, PageNumber, PageSize};
use crate::petty_matters::comment::Comment;
use crate::petty_matters::service::TopicService;
use crate::petty_matters::topic::{Topic, TopicId};
use crate::time::Seconds;
use crate::view::{show_error_page, show_not_found_page, HtmlResponse};
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use crate::authn::session::User;
use crate::render_template;

#[derive(Template)]
#[template(path = "petty_matters/list.html")]
pub struct PettyMattersList {
    user: User,
    nonce: Nonce,
    pub topics: Page<Topic>,
}

#[derive(Template)]
#[template(path = "petty_matters/add.html")]
pub struct PettyMattersRegistration {
    user: User,
    nonce: Nonce,
}

#[derive(Template)]
#[template(path = "petty_matters/view.html")]
pub struct PettyMatter {
    nonce: Nonce,
    pub topic: Topic,
    pub comments: Vec<Comment>,
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
    user: User,
    nonce: Nonce,
    State(service): State<Arc<TopicService>>,
    pagination: Query<Pagination>,
) -> Result<HtmlResponse, StatusCode> {
    let list_parameters = ListParameters {
        page_number: pagination.page.unwrap_or(PageNumber(1)),
        page_size: pagination.page_size.unwrap_or(PageSize(10)),
        filters: None,
    };
    let topics = match service.list_topics(list_parameters).await {
        Ok(topics) => topics,
        Err(e) => return show_error_page(e),
    };
    let template = render_template!(PettyMattersList { nonce, user, topics });
    Ok(HtmlResponse::from_string(template))
}

async fn render_registration_form(nonce: Nonce, user: User) -> Result<HtmlResponse, StatusCode> {
    let template = render_template!(PettyMattersRegistration { nonce, user });
    Ok(HtmlResponse::from_string(template))
}

async fn register_petty_matter(
    user: User,
    State(service): State<Arc<TopicService>>,
    form: Form<PettyMattersRegistrationForm>,
) -> Result<Response, StatusCode> {
    let topic = Topic::new(form.subject.clone(), form.content.clone(), user);
    match service.create_topic(topic).await {
        Ok(t) => t,
        Err(e) => return Ok(show_error_page(e).into_response()),
    }
    Ok(Redirect::to("/petty-matters").into_response())
}

async fn view_petty_matter(
    nonce: Nonce,
    Path(topic_id): Path<TopicId>,
    State(service): State<Arc<TopicService>>,
) -> Result<HtmlResponse, StatusCode> {
    let topic = match service.get_topic(&topic_id).await {
        Ok(Some(t)) => t,
        Ok(None) => return show_not_found_page(),
        Err(e) => return show_error_page(e),
    };
    let comment_filters = ListParameters {
        page_size: PageSize(1000),
        page_number: PageNumber(1),
        filters: Some(HashMap::from([(
            "topic_id".to_string(),
            topic_id.to_string(),
        )])),
    };
    let comments = match service.list_comments(&topic_id, comment_filters).await {
        Ok(c) => c,
        Err(e) => return show_error_page(e),
    };
    let template = render_template!(PettyMatter {
        nonce,
        topic,
        comments: comments.items,
    });

    Ok(HtmlResponse::cached(template, Seconds(60)))
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
