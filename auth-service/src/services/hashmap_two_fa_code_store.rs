use std::collections::HashMap;

use crate::domain::{
    data_store::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(&mut self, email: Email, login_attempt_id: LoginAttemptId, code: TwoFACode) -> Result<(), TwoFACodeStoreError> {
        if LoginAttemptId::parse(login_attempt_id.as_ref().to_string()).is_err() {
            return Err(TwoFACodeStoreError::UnexpectedError);
        };

        if TwoFACode::parse(code.as_ref().to_string()).is_err() {
            return Err(TwoFACodeStoreError::UnexpectedError);
        }

        self.codes.insert(email, (login_attempt_id, code));

        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        self.codes.remove(email).ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound).map(|_| ())
    }

    async fn get_code(&self, email: &Email) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        match self.codes.get(email) {
            Some(result) => Ok(result.clone()),
            None => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use super::*;

    #[tokio::test]
    async fn test_two_fa_code_store() {
        let mut two_fa_store = HashmapTwoFACodeStore::default();

        let email = Email::parse(SafeEmail().fake()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let two_fa_code = TwoFACode::default();

        let add_result = two_fa_store.add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone()).await;
        assert!(add_result.is_ok());
        assert_eq!(two_fa_store.codes.len(), 1);

        let get_result = two_fa_store.get_code(&email).await.unwrap();
        assert_eq!(login_attempt_id, get_result.0);
        assert_eq!(two_fa_code, get_result.1);

        let another_email = Email::parse(SafeEmail().fake()).unwrap();
        let get_with_wrong_email_result = two_fa_store.get_code(&another_email).await;
        assert!(get_with_wrong_email_result.is_err());

        let remove_with_wrong_email_result = two_fa_store.remove_code(&another_email).await;
        assert!(remove_with_wrong_email_result.is_err());

        let remove_result = two_fa_store.remove_code(&email).await;
        assert!(remove_result.is_ok());
        assert_eq!(two_fa_store.codes.len(), 0);
    }
}