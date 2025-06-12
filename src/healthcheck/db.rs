pub mod result {
    use crate::healthcheck::HealthCheckResult;

    pub async fn create(
        pool: &sqlx::PgPool,
        result: HealthCheckResult,
        service_id: uuid::Uuid,
    ) -> Result<uuid::Uuid, sqlx::Error> {
        let id = uuid::Uuid::new_v4();
        let created= sqlx::query!(
            "INSERT INTO healthcheck_results (id, success, code, response_time, service_id, message) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
            id,
            result.success,
            result.code as i64,
            result.response_time as i64,
            service_id,
            result.message
        ).fetch_one(pool).await?;

        Ok(created.id)
    }
    
}
