use askama::Template;
use sqlx::types::chrono::{DateTime, Utc};

use crate::models::snippet::Snippet; // bring trait in scope

pub struct TemplateData {
    snippet: ViewTemplate,
}

#[derive(Template)]
#[template(path = "pages/home.html")] // using the template in this path, relative
// to the `templates` dir in the crate root
pub struct HomeTemplate {
    pub view_snippets: Vec<ViewTemplate>, // the name of the struct can be anything
                                          // name: &'a str, // the field name should match the variable name
                                          // in your template
}

#[derive(Template)] // this will generate the code...
#[template(path = "pages/view.html")]
// struct HelloTemplate<'a> {
pub struct ViewTemplate {
    title: String,
    id: i32,
    content: String,
    created: DateTime<Utc>,
    expires: DateTime<Utc>,
}

impl From<Snippet> for ViewTemplate {
    fn from(value: Snippet) -> Self {
        ViewTemplate::new(
            value.title.clone(),
            value.id,
            value.content.clone(),
            value.created,
            value.expires,
        )
    }
}

impl ViewTemplate {
    pub fn new(
        title: String,
        id: i32,
        content: String,
        created: DateTime<Utc>,
        expires: DateTime<Utc>,
    ) -> Self {
        Self {
            title,
            id,
            content,
            created,
            expires,
        }
    }
}
