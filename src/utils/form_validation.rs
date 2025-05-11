use askama::Template;
use axum::{
    Form,
    extract::{FromRequest, Request, rejection::FormRejection},
    response::{IntoResponse, Response},
};
use chrono::Datelike; // not used directly anywhere but it is used in codegen of askama for the current year
use serde::Deserialize;
use std::{borrow::Cow, collections::HashMap};
use validator::ValidationErrorsKind::Field;
use validator::{Validate, ValidationError};

use crate::AppState;

use super::validation_errors::ServerError;

#[derive(Template, Deserialize, Debug)]
#[template(path = "pages/create.html")]
pub struct CreateTemplate {
    pub user_errors: HashMap<String, String>,
    pub title: String,
    pub content: String,
    pub expires: u16,
    pub is_authenticated: bool,
}

impl CreateTemplate {
    fn get(&self, key: &str) -> &str {
        if let Some(msg) = self.user_errors.get(key) {
            msg
        } else {
            ""
        }
    }
}

#[derive(Deserialize, Debug, Validate, Clone)]
pub struct SnippetData {
    #[validate(length(
        min = 1,
        max = 100,
        message = "This field cannot be empty and cannot have more than 100 characters including whitespaces"
    ))]
    pub title: String,
    #[validate(length(min = 1, message = "This field cannot be empty"))]
    pub content: String,
    // validate , value in 1,7,365
    #[validate(custom(function = "validate_expires"))]
    pub expires: u16,
}

fn validate_expires(expires: u16) -> Result<(), ValidationError> {
    if expires != 1 && expires != 7 && expires != 365 {
        return Err(ValidationError::new("expiration duration value")
            .with_message(Cow::Borrowed("This field must equal 1, 7 or 365")));
    }
    Ok(())
}

impl<S> FromRequest<S> for SnippetData
where
    S: Send + Sync,
    Form<SnippetData>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = RejectionWithUserInput;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let form_result = Form::<SnippetData>::from_request(req, state).await;
        match form_result {
            Ok(Form(value)) => {
                if let Err(e) = value.validate() {
                    Err(RejectionWithUserInput {
                        error: ServerError::ValidationError(e),
                        value: Some(value),
                    })
                } else {
                    Ok(value)
                }
            }
            Err(form_rejection) => Err(RejectionWithUserInput {
                error: ServerError::AxumFormRejection(form_rejection),
                value: None,
            }),
        }
    }
}

pub struct RejectionWithUserInput {
    // FIX - ServerError::ValidationError should always be paired with a non-None value for the 'value' field of this struct
    error: ServerError,
    value: Option<SnippetData>,
}

impl IntoResponse for RejectionWithUserInput {
    fn into_response(self) -> Response {
        match self.error {
            ServerError::ValidationError(e) => {
                let value = self.value.unwrap();
                let mut create_template = CreateTemplate {
                    user_errors: HashMap::new(),
                    title: value.title,
                    content: value.content,
                    expires: value.expires,
                    // this is super shady
                    is_authenticated: true,
                };
                // this can be deserialized with serde... figure it out
                // this is going to be the worst part of this code base until and unless I learn serde
                // learn serde and implement a Deserializer for format! -ed error.
                let field_errors = e.errors();
                for error in field_errors.keys() {
                    let mut error_string = "".to_string();
                    if let Some(Field(e)) = field_errors.get(error) {
                        e.into_iter().for_each(|err| {
                            error_string += &err.message.as_ref().unwrap().to_string()
                        });
                    }
                    create_template
                        .user_errors
                        .insert(error.to_string(), error_string);
                }

                AppState::render(create_template.render())
            }
            // WHAT THE FUCK IS AXUMFORMREJECTION - VALIDATOR NEEDS SOME MORE RESEARCH
            ServerError::AxumFormRejection(e) => AppState::server_error(Box::new(e)),
        }
        .into_response()
    }
}
