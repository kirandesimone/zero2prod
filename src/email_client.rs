use crate::domain::SubscriberEmail;
use reqwest::header::{
    HeaderMap, HeaderValue, InvalidHeaderValue, AUTHORIZATION, CONTENT_TYPE,
};
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct EmailContent<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    text_body: &'a str,
}

pub struct EmailClient {
    base_url: String,
    sg_key: Secret<String>,
    sender_email: SubscriberEmail,
    http_client: Client,
}

impl EmailClient {
    pub fn new(
        sg_key: Secret<String>,
        sender_email: SubscriberEmail,
        base_url: String,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        Self {
            base_url,
            sg_key,
            sender_email,
            http_client,
        }
    }

    pub fn get_headers(&self) -> Result<HeaderMap, InvalidHeaderValue> {
        let mut headers = HeaderMap::with_capacity(2);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", &self.sg_key.expose_secret()))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json")?);
        Ok(headers)
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        text_body: &str,
    ) -> Result<(), reqwest::Error> {
        let headers = self.get_headers().expect("couldn't retrieve headers");
        let email_content = EmailContent {
            from: self.sender_email.as_ref(),
            to: recipient.as_ref(),
            subject,
            text_body,
        };

        let _response = self
            .http_client
            .post(&self.base_url)
            .headers(headers)
            .json(&email_content)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

///////////////////////////
//// UNIT TESTS //////////
/////////////////////////

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use claim::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use reqwest::header::HeaderMap;
    use secrecy::Secret;
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    const SG_KEY: &str = "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";

    // Create a customer matcher for matching the JSON Body
    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                dbg!(&body);
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    // helpers
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            Secret::new(SG_KEY.into()),
            email(),
            base_url,
            std::time::Duration::from_millis(200),
        )
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_sendgrid() {
        // Arrange
        let mock_server = MockServer::start().await;
        let base_url = mock_server.uri();
        let email_client = email_client(base_url);

        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let _ = email_client
            .send_email(email(), &subject(), &content())
            .await;

        // Assert
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        let mock_server = MockServer::start().await;
        let base_url = mock_server.uri();
        let email_client = email_client(base_url);

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content())
            .await;

        assert_ok!(outcome)
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // server set-up
        let mock_server = MockServer::start().await;
        let base_url = mock_server.uri();
        let email_client = email_client(base_url);

        // server expectations
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;
        // Act
        let outcome = email_client
            .send_email(email(), &subject(), &content())
            .await;
        // Assertion
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_takes_too_long_to_respond() {
        // server set-up
        let mock_server = MockServer::start().await;
        let base_url = mock_server.uri();
        let email_client = email_client(base_url);

        // setting a 3 minute delay
        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content())
            .await;

        assert_err!(outcome);
    }
}
