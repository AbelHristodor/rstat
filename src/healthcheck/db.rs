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
    
    pub async fn get_by_service_id(
        pool: &sqlx::PgPool,
        service_id: uuid::Uuid,
    ) -> Result<Vec<HealthCheckResult>, sqlx::Error> {
        let results = sqlx::query!(
            "SELECT id, success, code, response_time, service_id, message FROM healthcheck_results WHERE service_id = $1",
            service_id
        ).fetch_all(pool).await?;

        Ok(results.into_iter().map(|r| HealthCheckResult {
            id: r.id,
            success: r.success,
            code: r.code.unwrap_or_default().parse::<u64>().unwrap_or_default(),
            response_time: r.response_time.unwrap_or_default().try_into().unwrap_or_default(),
            message: r.message.unwrap_or_default(),
        }).collect())
    }
}
