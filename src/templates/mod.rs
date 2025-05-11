use askama::Template;
use chrono::Datelike; // not used directly anywhere but it is used in codegen of askama for the current year
use sqlx::types::chrono::{DateTime, Utc};

use crate::models::snippet::Snippet; // bring trait in scope

#[derive(Template)]
#[template(path = "partials/nav.html")]
pub struct Nav {
    pub is_authenticated: bool,
}

#[derive(Template)]
#[template(path = "pages/home.html")] // using the template in this path, relative
// to the `templates` dir in the crate root
pub struct HomeTemplate {
    pub view_snippets: Vec<ViewTemplate>, // the name of the struct can be anything
    // name: &'a str, // the field name should match the variable name
    // in your template
    pub flash: String,
    pub is_authenticated: bool,
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
    flash: String,
    pub is_authenticated: bool,
}

impl ViewTemplate {
    pub fn new(
        title: String,
        id: i32,
        content: String,
        created: DateTime<Utc>,
        expires: DateTime<Utc>,
        is_authenticated: bool,
    ) -> Self {
        Self {
            title,
            id,
            content,
            created,
            expires,
            flash: "".to_string(),
            is_authenticated,
        }
    }

    pub fn set_flash(&mut self, flash: String) {
        self.flash = flash;
    }

    pub fn convert_to_view(value: Snippet, is_authenticated: bool) -> Self {
        ViewTemplate::new(
            value.title.clone(),
            value.id,
            value.content.clone(),
            value.created,
            value.expires,
            is_authenticated,
        )
    }
}
