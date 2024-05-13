use super::*;

#[derive(Debug)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub text: String,
    pub html: Option<String>,
}

impl Message {
    pub fn builder() -> MessageBuilder {
        MessageBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct MessageBuilder {
    pub from: Option<String>,
    pub to: Option<String>,
    pub subject: Option<String>,
    pub text: Option<String>,
    pub html: Option<String>,
}

impl MessageBuilder {
    pub fn build(self) -> Result<Message, MessageBuildError> {
        let mut error = MessageBuildError::default();

        if self.from.is_none() {
            error.push("from");
        }

        if self.to.is_none() {
            error.push("to");
        }

        if self.subject.is_none() {
            error.push("subject");
        }

        if self.text.is_none() {
            error.push("text");
        }

        if !error.is_empty() {
            return Err(error);
        }

        let from = self.from.unwrap();
        let to = self.to.unwrap();
        let subject = self.subject.unwrap();
        let text = self.text.unwrap();
        let html = self.html;

        Ok(Message {
            from,
            to,
            subject,
            text,
            html,
        })
    }

    pub fn from(mut self, from: &str) -> Self {
        self.from = Some(from.to_owned());

        self
    }

    pub fn to(mut self, to: &str) -> Self {
        self.to = Some(to.to_owned());

        self
    }

    pub fn subject(mut self, subject: &str) -> Self {
        self.subject = Some(subject.to_owned());

        self
    }

    pub fn text(mut self, text: &str) -> Self {
        self.text = Some(text.to_owned());

        self
    }

    pub fn html(mut self, html: &str) -> Self {
        self.html = Some(html.to_owned());

        self
    }
}

#[derive(Debug, Default)]
pub struct MessageBuildError {
    missing_fields: Vec<&'static str>,
}

impl MessageBuildError {
    pub fn is_empty(&self) -> bool {
        self.missing_fields.is_empty()
    }

    pub fn push(&mut self, field: &'static str) {
        self.missing_fields
            .push(field)
    }
}

impl fmt::Display for MessageBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "missing fields: [ {} ]",
            self.missing_fields.join(", ")
        )
    }
}
