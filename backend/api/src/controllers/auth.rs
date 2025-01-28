use serde::Serialize;

#[derive(Serialize)]
pub enum PrimaryAuthenticationMethod {
    Password { password: String },
}
pub fn list_supported_authentication_methods() -> Vec<PrimaryAuthenticationMethod> {
    vec![PrimaryAuthenticationMethod::Password {
        password: "string".to_owned(),
    }]
}
