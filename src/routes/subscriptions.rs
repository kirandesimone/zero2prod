use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

// subscribe handler
pub async fn subscribe(
    form: web::Form<FormData>,
    connection_pool: web::Data<PgPool>,
) -> impl Responder {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );
    let _request_span_guard = request_span.enter();
    let query_span = tracing::info_span!("Adding new subscriber");
    match sqlx::query!(
        r#"
        insert into subscriptions (id, name, email, subscribed_at)
        values ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.name,
        form.email,
        Utc::now()
    )
    .execute(connection_pool.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request id:{} - A new subscriber was just added to the database",
                request_id
            );
            HttpResponse::Ok()
        }
        Err(e) => {
            tracing::error!("request id:{} - Failed to execute query: {}", request_id, e);
            HttpResponse::InternalServerError()
        }
    }
}
