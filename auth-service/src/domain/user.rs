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
        return self.email.clone();
    }

    pub fn get_password(&self) -> Password {
        return self.password.clone();
    } 
}