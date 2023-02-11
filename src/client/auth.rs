use serde::Deserialize;
use std::collections::HashMap;
use crate::user::{UserTypes, User};
use super::Client;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
struct AuthenticationError;

impl fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Authentication Failed")
    }
}

impl Error for AuthenticationError {
    fn description(&self) -> &str {
        "Authentication Failed!"
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SuccessAuthResponse {
    pub token: String
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FailureAuthResponse {
    pub code: String,
    pub message: String,
    pub data: HashMap<String, String>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", untagged)]
pub enum AuthResponse {
    SuccessAuthResponse(SuccessAuthResponse),
    FailureAuthResponse(FailureAuthResponse)
}

impl Client {
    pub async fn auth_via_email<'a>(
        &mut self,
        email: String, password: String,
        usertype: UserTypes
    ) -> Result<(), Box<dyn Error + Send>>
    {
        let mut credentials: HashMap<String, String> = HashMap::new();
        credentials.insert(String::from("identity"), email);
        credentials.insert(String::from("password"), password);

        match usertype {
            UserTypes::User => self.authenticate_user(&credentials).await,
            UserTypes::Admin => self.authenticate_admin(&credentials).await,
        }
    }

    async fn authenticate_user(&mut self, credentials: &HashMap<String, String>) -> Result<(), Box<dyn Error + Send>> {
        let auth_response = self.post(String::from("collections/users/auth-with-password"), &credentials).await;
        let parsed_resp   = match auth_response {
            Ok(response) => {
                match response.json::<AuthResponse>().await {
                    Ok(resp) => Ok(resp),
                    Err(err) => Err(Box::new(err) as Box<dyn Error + Send>)
                }
            },
            Err(err) => Err(err)
        };

        match parsed_resp {
            Ok(body) => {
                match body {
                    AuthResponse::SuccessAuthResponse(response) =>  {
                        self.user = Some(
                            User {
                                usertype: UserTypes::User,
                                token: response.token
                            }
                        );

                        Ok(())
                    },
                    AuthResponse::FailureAuthResponse(_response) => {
                        Err(Box::new(AuthenticationError) as Box<dyn Error + Send>)
                    }
                }
            },
            Err(err) => Err(err)
        }

    }

    async fn authenticate_admin(&mut self, credentials: &HashMap<String, String>) -> Result<(), Box<dyn Error + Send>> {
        let auth_response = self.post(String::from("admins/auth-with-password"), &credentials).await;
        let parsed_resp   = match auth_response {
            Ok(response) => {
                match response.json::<AuthResponse>().await {
                    Ok(resp) => Ok(resp),
                    Err(err) => Err(Box::new(err) as Box<dyn Error + Send>)
                }
            },
            Err(err) => Err(err)
        };

        match parsed_resp {
            Ok(body) => {
                match body {
                    AuthResponse::SuccessAuthResponse(response) =>  {
                        self.user = Some(
                            User {
                                usertype: UserTypes::Admin,
                                token: response.token
                            }
                        );

                        Ok(())
                    },
                    AuthResponse::FailureAuthResponse(_response) => {
                        Err(Box::new(AuthenticationError))
                    }
                }
            },
            Err(err) => Err(err)


        }
    }
}
