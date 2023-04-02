use lettre::{
    message::{Mailbox, MultiPart},
    Message,
};

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
            Self::Register => Message::builder()
                .from(from)
                .to(to)
                .subject("Twój kod do kantyny")
                .multipart(MultiPart::alternative_plain_html(
                    format!(
                        "Kod na potwierdzenie aktywowania konta w kantynie: {}",
                        code
                    ),
                    Self::body_html("Twój kod do aktywacji konta", code),
                )),
            Self::Delete => Message::builder()
                .from(from)
                .to(to)
                .subject("Kantyna - usuwanie konta")
                .multipart(MultiPart::alternative_plain_html(
                    format!("Wpisz ten kod aby usunąć konto: {}", code),
                    Self::body_html("Twój kod do usunięcia konta", code),
                )),
        }
    }

    pub fn code_len(&self) -> usize {
        match self {
            Self::Register => 4,
            Self::Delete => 4,
        }
    }

    fn body_html(text: &str, code: &str) -> String {
        format!(r#"
             <table width="100%" width="0" cellspacing="0" cellpading="0" style="font-size: 300%;">
                    <tr>
                            <td align="center">{}</td>
                    </tr>
                    <tr>
                            <td align="center">
                                    <span style=" gap: 10px; font-size: 150%; background-color: #AAAAAA20; padding: 20px; border-radius: 10px;">
                                        {}
                                    </span>
                            </td>
                    </tr>
            </table>
           "#,text,
           code.chars().map(|ch| format!(r#"<b style="background-color: #AAAAAA30;border-radius: 10px; padding: 10px;">{}</b>"#,ch)).collect::<String>())
    }
}
