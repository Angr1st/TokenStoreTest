use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Token {
    created: DateTime<Utc>,
    expires_in: u64,
    user_id: u32,
    value: String,
}

impl Token {
    pub fn new(user_id: u32, token_value: String) -> Self {
        Self {
            created: Utc::now(),
            expires_in: 3600,
            user_id,
            value: token_value,
        }
    }

    fn expiration_duration(&self) -> chrono::Duration {
        chrono::Duration::seconds(
            self.expires_in
                .try_into()
                .expect("3600 seconds should always work."),
        )
    }

    pub fn compute_expiry_date(&self) -> DateTime<Utc> {
        let expiration_duration = self.expiration_duration();
        self.created
            .checked_add_signed(expiration_duration)
            .expect("This should always return a valid DateTime!")
    }
}
