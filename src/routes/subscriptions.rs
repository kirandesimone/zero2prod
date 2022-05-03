use crate::domain::new_subscriber::NewSubscriber;
use crate::domain::subscriber_name::SubscriberName;
use crate::domain::SubscriberEmail;
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
    let new_subscriber = match form.0.try_into() {
        Ok(new_sub) => new_sub,
        Err(_) => return HttpResponse::BadRequest().finish()
    };
    match insert_subscriber(&new_subscriber, &connection_pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving a new Subscriber",
    skip(new_subscriber, connection_pool)
)]
pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    connection_pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        insert into subscriptions (id, name, email, subscribed_at)
        values ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.name.as_ref(),
        new_subscriber.email.as_ref(),
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
