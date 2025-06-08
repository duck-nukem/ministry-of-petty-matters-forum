use crate::authn::session::User;
use crate::persistence::repository::{ListParameters, Page, PageNumber, PageSize};
use crate::petty_matters::comment::Comment;
use crate::petty_matters::service::PettyMattersService;
use crate::petty_matters::topic::{Topic, TopicId};
use crate::queue::base::Queue;
use crate::render_template;
use crate::templates::{Nonce, filters};
use crate::time::Seconds;
use crate::views::pagination::PageFilters;
use crate::views::templates::{HtmlResponse, show_error_page, show_not_found_page};
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::Arc;

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

async fn list_petty_matters<Q>(
    user: User,
    nonce: Nonce,
    State(service): State<Arc<PettyMattersService<Q>>>,
    page_filters: Query<PageFilters>,
) -> Result<HtmlResponse, StatusCode>
where
    Q: Queue + Send + Sync,
{
    let list_parameters = ListParameters::from_query_params(&page_filters);
    let topics = match service.list_topics(list_parameters).await {
        Ok(topics) => topics,
        Err(e) => return show_error_page(e),
    };
    let template = render_template!(PettyMattersList {
        user,
        nonce,
        topics
    });
    Ok(HtmlResponse::from_string(template))
}

async fn render_registration_form(nonce: Nonce, user: User) -> Result<HtmlResponse, StatusCode> {
    let template = render_template!(PettyMattersRegistration { user, nonce });
    Ok(HtmlResponse::from_string(template))
}

async fn register_petty_matter<Q>(
    user: User,
    State(service): State<Arc<PettyMattersService<Q>>>,
    form: Form<PettyMattersRegistrationForm>,
) -> Result<Response, StatusCode>
where
    Q: Queue + Send + Sync,
{
    let topic = Topic::new(form.subject.clone(), form.content.clone(), user);
    match service.create_topic(topic).await {
        Ok(t) => t,
        Err(e) => return Ok(show_error_page(e).into_response()),
    }
    Ok(Redirect::to("/petty-matters").into_response())
}

async fn view_petty_matter<Q>(
    nonce: Nonce,
    Path(topic_id): Path<TopicId>,
    State(service): State<Arc<PettyMattersService<Q>>>,
) -> Result<HtmlResponse, StatusCode>
where
    Q: Queue + Send + Sync,
{
    let topic = match service.get_topic(&topic_id).await {
        Ok(Some(t)) => t,
        Ok(None) => return show_not_found_page(),
        Err(e) => return show_error_page(e),
    };
    let comment_filters = ListParameters {
        page_size: PageSize(1000),
        page_number: PageNumber(1),
        order_by: None,
        ordering: None,
        filters: Some(BTreeMap::from([(
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

async fn add_comment<Q>(
    user: User,
    Path(topic_id): Path<TopicId>,
    State(service): State<Arc<PettyMattersService<Q>>>,
    form: Form<CommentForm>,
) -> Result<impl IntoResponse, StatusCode>
where
    Q: Queue + Send + Sync,
{
    service
        .reply_to_topic(&topic_id, form.content.clone(), user)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Redirect::to(&format!("/petty-matters/{topic_id}")))
}

pub fn petty_matters_router<Q>(service: Arc<PettyMattersService<Q>>) -> Router
where
    Q: Queue + Send + Sync + 'static,
{
    Router::new()
        .route("/", get(list_petty_matters).post(register_petty_matter))
        .route("/register", get(render_registration_form))
        .route("/{topic_id}", get(view_petty_matter))
        .route("/{topic_id}/comments", post(add_comment))
        .with_state(service)
}
