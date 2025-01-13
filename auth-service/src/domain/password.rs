use serde::Serialize;

use super::AuthAPIError;

#[derive(Debug, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct Password(String);

impl Password {
    pub fn parse(s: String) -> Result<Self, AuthAPIError> {
        if s.len() >= 8 {
            Ok(Password(s))
        } else{
            Err(AuthAPIError::InvalidCredentials)
        }
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid_password() {
        let str_password = "12345678".to_string();
        let password = Password::parse(str_password);
        assert!(password.is_ok());
    }

    #[test]
    fn invalid_password() {
        let str_password = "1234567".to_string();
        let password = Password::parse(str_password);
        assert!(password.is_err());
    }
}