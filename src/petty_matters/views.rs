use crate::authn::session::User;
use crate::persistence::repository::{ListParameters, Page, PageNumber, PageSize, Repository};
use crate::petty_matters::comment::{Comment, CommentId};
use crate::petty_matters::service::PettyMattersService;
use crate::petty_matters::topic::{Topic, TopicId};
use crate::queue::base::Queue;
use crate::render_template;
use crate::templates::{filters, Nonce};
use crate::time::Seconds;
use crate::views::pagination::PageFilters;
use crate::views::templates::{show_error_page, show_not_found_page, HtmlResponse};
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;
use std::collections::HashMap;
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

async fn list_petty_matters<T, C, Q>(
    user: User,
    nonce: Nonce,
    State(service): State<Arc<PettyMattersService<T, C, Q>>>,
    pagination: Query<Pagination>,
) -> Result<HtmlResponse, StatusCode>
where
    T: Repository<TopicId, Topic> + Send + Sync,
    C: Repository<CommentId, Comment> + Send + Sync,
    Q: Queue + Send + Sync,
{
    let list_parameters = ListParameters {
        page_number: page_filters.page.unwrap_or(PageNumber(1)),
        page_size: page_filters.page_size.unwrap_or(PageSize(10)),
        filters: Some(page_filters.filters.clone()),
    };
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

async fn register_petty_matter<T, C, Q>(
    user: User,
    State(service): State<Arc<PettyMattersService<T, C, Q>>>,
    form: Form<PettyMattersRegistrationForm>,
) -> Result<Response, StatusCode>
where
    T: Repository<TopicId, Topic> + Send + Sync,
    C: Repository<CommentId, Comment> + Send + Sync,
    Q: Queue + Send + Sync,
{
    let topic = Topic::new(form.subject.clone(), form.content.clone(), user);
    match service.create_topic(topic).await {
        Ok(t) => t,
        Err(e) => return Ok(show_error_page(e).into_response()),
    }
    Ok(Redirect::to("/petty-matters").into_response())
}

async fn view_petty_matter<T, C, Q>(
    nonce: Nonce,
    Path(topic_id): Path<TopicId>,
    State(service): State<Arc<PettyMattersService<T, C, Q>>>,
) -> Result<HtmlResponse, StatusCode>
where
    T: Repository<TopicId, Topic> + Send + Sync,
    C: Repository<CommentId, Comment> + Send + Sync,
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

async fn add_comment<T, C, Q>(
    user: User,
    Path(topic_id): Path<TopicId>,
    State(service): State<Arc<PettyMattersService<T, C, Q>>>,
    form: Form<CommentForm>,
) -> Result<impl IntoResponse, StatusCode>
where
    T: Repository<TopicId, Topic> + Send + Sync,
    C: Repository<CommentId, Comment> + Send + Sync,
    Q: Queue + Send + Sync,
{
    service
        .reply_to_topic(&topic_id, form.content.clone(), user)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Redirect::to(&format!("/petty-matters/{topic_id}")))
}

pub fn topics_router<T, C, Q>(service: Arc<PettyMattersService<T, C, Q>>) -> Router
where
    T: Repository<TopicId, Topic> + Send + Sync + 'static,
    C: Repository<CommentId, Comment> + Send + Sync + 'static,
    Q: Queue + Send + Sync + 'static,
{
    Router::new()
        .route("/", get(list_petty_matters).post(register_petty_matter))
        .route("/register", get(render_registration_form))
        .route("/{topic_id}", get(view_petty_matter))
        .route("/{topic_id}/comments", post(add_comment))
        .with_state(service)
}
