use serde::Serialize;
use validator::validate_email;

#[derive(Debug, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    pub fn parse(s: String) -> Result<Self, String> {
        if validate_email(&s) {
            Ok(Email(s))
        } else {
            Err(format!("{} is not a valid email.", s))
        }
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}


#[cfg(test)]
mod tests {
    use fake::{faker::internet::en::SafeEmail, Fake};
    use super::*;

    #[test]
    fn valid_email() {
        let email_str = SafeEmail().fake(); 
        let email = Email::parse(email_str);
        assert!(email.is_ok());
    }

    #[test]
    fn invalid_email_missing_symbol() {
        let email_str = "invalidemail.com".to_string();
        let email = Email::parse(email_str);
        assert!(email.is_err());
    }

    #[test]
    fn invalid_email_is_empty() {
        let email_str = "".to_string();
        let email = Email::parse(email_str);
        assert!(email.is_err());
    }
}