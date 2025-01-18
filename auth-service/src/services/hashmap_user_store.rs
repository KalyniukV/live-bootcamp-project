use std::collections::HashMap;

use crate::domain::{Email, Password, User, UserStore, UserStoreError};

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<Email, User>
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {       
        if self.users.contains_key(&user.get_email()) {
            return Err(UserStoreError::UserAlreadyExists);
        }

        self.users.insert(user.get_email(), user);

        Ok(())
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {   
        match self.users.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound)
        }
    }

    async fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            Some(user) => {
                if user.get_password().eq(password) {
                    Ok(())
                } else {
                    Err(UserStoreError::InvalidCredentials)
                }
            }
            None => Err(UserStoreError::UserNotFound),
        }
    }
}


#[cfg(test)]
mod tests {
    use fake::{faker::internet::en::SafeEmail, Fake};
    use super::*;

    #[tokio::test]
    async fn test_add_user() {   
        let email = Email::parse(SafeEmail().fake()).unwrap();
        let password = Password::parse("12345678".to_string()).unwrap();

        let user = User::new(email.clone(), password.clone(), true);

        let mut hashmap_user_store = HashmapUserStore::default();
        let result = hashmap_user_store.add_user(user).await;

        assert!(result.is_ok());
        assert_eq!(1, hashmap_user_store.users.len());

        let same_user = User::new(email, password, true);

        let result = hashmap_user_store.add_user(same_user);
        assert_eq!(UserStoreError::UserAlreadyExists, result.await.unwrap_err());
    }

    #[tokio::test]
    async fn test_get_user() {
        let email = Email::parse(SafeEmail().fake()).unwrap();
        let password = Password::parse("12345678".to_string()).unwrap();

        let user = User::new(email.clone(), password.clone(), true);

        let mut hashmap_user_store = HashmapUserStore::default();
        hashmap_user_store.add_user(user).await;

        let user1 = hashmap_user_store.get_user(&email).await;
        assert!(user1.is_ok());
        assert_eq!(&email, &user1.unwrap().get_email());

        let another_email = Email::parse(SafeEmail().fake()).unwrap();
        let user2 = hashmap_user_store.get_user(&another_email).await;
        assert!(user2.is_err());
        assert_eq!(UserStoreError::UserNotFound, user2.unwrap_err());
    }

    #[tokio::test]
    async fn test_validate_user() {
        let email = Email::parse(SafeEmail().fake()).unwrap();
        let password = Password::parse("12345678".to_string()).unwrap();

        let user = User::new(email.clone(), password.clone(), true);
        
        let mut hashmap_user_store = HashmapUserStore::default();
        hashmap_user_store.add_user(user).await;

        assert_eq!(1, hashmap_user_store.users.len());

        let result = hashmap_user_store.validate_user(&email, &password).await;
        assert!(result.is_ok());
        
        let wrong_password = Password::parse("87654321".to_string()).unwrap();
        let result = hashmap_user_store.validate_user(&email, &wrong_password).await;
        assert!(result.is_err());
        assert_eq!(UserStoreError::InvalidCredentials, result.unwrap_err());

        let wrong_email = Email::parse(SafeEmail().fake()).unwrap();
        let result = hashmap_user_store.validate_user(&wrong_email, &password).await;
        assert!(result.is_err());
        assert_eq!(UserStoreError::UserNotFound, result.unwrap_err());
    }
}