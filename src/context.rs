use crate::net::{api::Auth, api::User, Api};

#[derive(Default)]
pub struct Context {
    pub api: Api,
    pub user: User,
}

impl Context {
    pub fn sign_in(&mut self, auth: Auth) {
        self.api = Api::with_token(auth.token);
        self.user = auth.user;
    }

    pub fn sign_out(&mut self) {
        *self = Context::default();
    }
}
