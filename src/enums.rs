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
        code: &str,
    ) -> Result<Message, lettre::error::Error> {
        match self {
            &Self::Register => Message::builder()
                .from(from)
                .to(to)
                .subject("Twój kod do kantyny")
                .body(format!("Wpisz ten kod aby aktywowć konto: {}", code)),
            &Self::Delete => Message::builder()
                .from(from)
                .to(to)
                .subject("Kantyna - usuwanie konta")
                .body(format!("Wpisz ten kod aby usunąć konto: {}", code)),
        }
    }

    pub fn code_len(&self) -> usize {
        match self {
            &Self::Register => 4,
            &Self::Delete => 8,
        }
    }
}
