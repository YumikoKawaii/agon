use crate::net::{api::Auth, api::User, ffi, Api};

const TOKEN_KEY: &str = "agon.token";

#[derive(Default)]
pub struct Context {
    pub api: Api,
    pub user: User,
}

impl Context {
    pub fn sign_in(&mut self, auth: Auth) {
        ffi::store_set(TOKEN_KEY, &auth.token);
        self.api = Api::with_token(auth.token);
        self.user = auth.user;
    }

    pub fn sign_out(&mut self) {
        ffi::store_set(TOKEN_KEY, "");
        *self = Context::default();
    }

    pub fn saved_token() -> String {
        ffi::store_get(TOKEN_KEY)
    }
}
