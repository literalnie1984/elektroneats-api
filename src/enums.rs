use lettre::{message::Mailbox, Message};

pub enum VerificationType {
    Register,
    Delete,
}

impl VerificationType {
    pub fn email_msg(
        &self,
        to: Mailbox,
        from: Mailbox,
        activation_link: &str,
    ) -> Result<Message, lettre::error::Error> {
        match self {
            &Self::Register => Message::builder()
                .from(from)
                .to(to)
                .subject("Twój kod do kantyny")
                .body(format!(
                    "http://127.0.0.1:4765/api/user/activate/{}",
                    activation_link
                )),
            &Self::Delete => Message::builder()
                .from(from)
                .to(to)
                .subject("Kantyna - usuwanie konta")
                .body(format!(
                    "http://127.0.0.1:4765/api/user/delete/{}",
                    activation_link
                )),
        }
    }
}
