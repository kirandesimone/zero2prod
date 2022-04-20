use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

// subscribe handler
#[tracing::instrument(
    name="Adding a new subscriber",
    skip(form, connection_pool),
    fields(
        subscriber_name=%form.name,
        subscriber_email=%form.email
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    connection_pool: web::Data<PgPool>,
) -> impl Responder {
    match insert_subscriber(&form, &connection_pool).await {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError(),
    }
    .await
}

#[tracing::instrument(name = "Saving a new Subscriber", skip(form, connection_pool))]
pub async fn insert_subscriber(
    form: &FormData,
    connection_pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        insert into subscriptions (id, name, email, subscribed_at)
        values ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.name,
        form.email,
        Utc::now()
    )
    .execute(connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
