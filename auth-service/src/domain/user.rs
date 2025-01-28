use serde::Serialize;

use super::{Email, Password};

#[derive(Debug, Serialize, Clone)]
pub struct User {
    email: Email,
    password: Password,
    requires_2fa: bool
}

impl User {
    pub fn new (email: Email, password: Password, requires_2fa: bool) -> Self {
        User { email, password, requires_2fa }
    }

    pub fn get_email(&self) -> Email {
        self.email.clone()
    }

    pub fn get_password(&self) -> Password {
        self.password.clone()
    }

    pub fn use_requires_2fa(&self) -> bool {
        self.requires_2fa
    }
}