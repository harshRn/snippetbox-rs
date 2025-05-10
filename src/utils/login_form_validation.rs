use axum::{
    Form,
    extract::{FromRequest, Request, rejection::FormRejection},
    response::{IntoResponse, Response},
};
use validator::ValidationErrorsKind::Field;

use chrono::Datelike;
use regex::Regex;
use std::{borrow::Cow, collections::HashMap};
use validator::{Validate, ValidationError}; // not used directly anywhere but it is used in codegen of askama for the current year

use askama::Template;
use serde::Deserialize;

use crate::AppState;

use super::validation_errors::ServerError;

#[derive(Template)] // this will generate the code...
#[template(path = "pages/login.html")]
// struct HelloTemplate<'a> {
pub struct LoginTemplate {
    email: String,
    password: String,
    pub user_errors: HashMap<String, String>,
    pub flash: String,
}

impl LoginTemplate {
    pub fn new(email: String, password: String) -> Self {
        Self {
            email,
            password,
            user_errors: HashMap::new(),
            flash: "".to_string(),
        }
    }

    fn get(&self, key: &str) -> &str {
        if let Some(msg) = self.user_errors.get(key) {
            msg
        } else {
            ""
        }
    }
}

#[derive(Deserialize, Debug, Validate, Clone)]
pub struct LoginData {
    #[validate(length(min = 8, message = "This field must be at least 8 characters long"))]
    pub password: String,
    // validate , value in 1,7,365
    #[validate(custom(function = "validate_email"))]
    pub email: String,
}

fn validate_email(email: &String) -> Result<(), ValidationError> {
    let re = Regex::new("^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();
    let caps = re.captures(email);
    if let None = caps {
        return Err(ValidationError::new("email value")
            .with_message(Cow::Borrowed("This field must contain a valid email")));
    }
    Ok(())
}

impl<S> FromRequest<S> for LoginData
where
    S: Send + Sync,
    Form<LoginData>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = RejectionWithUserInput;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let form_result = Form::<LoginData>::from_request(req, state).await;
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
    value: Option<LoginData>,
}

impl IntoResponse for RejectionWithUserInput {
    fn into_response(self) -> Response {
        match self.error {
            ServerError::ValidationError(e) => {
                let value = self.value.unwrap();
                let mut login_template = LoginTemplate::new(value.email, value.password);
                // this can be deserialized with serde.
                let field_errors = e.errors();
                for error in field_errors.keys() {
                    let mut error_string = "".to_string();
                    if let Some(Field(e)) = field_errors.get(error) {
                        e.into_iter().for_each(|err| {
                            error_string += &err.message.as_ref().unwrap().to_string()
                        });
                    }
                    login_template
                        .user_errors
                        .insert("login_error".to_string(), error_string);
                }
                AppState::render(login_template.render()) // rendering should not be done in form validation
            }
            ServerError::AxumFormRejection(e) => AppState::server_error(Box::new(e)),
        }
        .into_response()
    }
}
