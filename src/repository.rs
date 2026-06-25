use std::convert::Infallible;
use axum::extract::FromRequestParts;
use sqlx::PgPool;
use crate::{
    app::AppState,
    models::{Asset, UserRecord},
};
pub struct Repository {
    db: PgPool,
}
impl Repository {
    pub async fn list_assets(&self) -> sqlx::Result<Vec<Asset>> {
        sqlx::query_as!(
            Asset,
            "SELECT id, name, unit_value
             FROM assets;"
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn total_value(&self) -> sqlx::Result<f64> {
        let assets = self.list_assets().await?;
        let total: f64 = assets.iter().map(|asset| asset.unit_value).sum();
        Ok(total)
    }

    pub async fn create_asset(&self, name: String, unit_value: f64) -> sqlx::Result<Asset> {
        sqlx::query_as!(
            Asset,
            "INSERT INTO assets (name, unit_value)
             VALUES ($1, $2)
             RETURNING id, name, unit_value;",
            name,
            unit_value
        )
        .fetch_one(&self.db)
        .await
    }
    pub async fn update_asset(
        &self,
        asset_id: i64,
        name: Option<String>,
        unit_value: Option<f64>,
    ) -> sqlx::Result<Option<Asset>> {
        sqlx::query_as!(
            Asset,
            "UPDATE assets
             SET name=COALESCE($2, name),
                 unit_value=COALESCE($3, unit_value)
             WHERE id=$1
             RETURNING id, name, unit_value;",
            asset_id,
            name,
            unit_value
        )
        .fetch_optional(&self.db)
        .await
    }
    pub async fn add_user(&self, username: &str, password_hash: &str) -> sqlx::Result<UserRecord> {
        sqlx::query_as!(
            UserRecord,
            "INSERT INTO users (username, password_hash)
             VALUES ($1, $2)
             RETURNING id, username, password_hash;",
            username,
            password_hash,
        )
        .fetch_one(&self.db)
        .await
    }
    pub async fn get_user_by_name(&self, username: &str) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as!(
            UserRecord,
            "SELECT id, username, password_hash
             FROM users
             WHERE username = $1;",
            username
        )
        .fetch_optional(&self.db)
        .await
    }
}
impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;
    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            db: state.db.clone(),
        })
    }
}
#[cfg(test)]
impl From<PgPool> for Repository {
    fn from(db: PgPool) -> Self {
        Self { db }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_total_value(db: PgPool) {
        let repository: Repository = db.into();

        repository.create_asset("Bitcoin".to_string(), 350000.0).await.expect("criar bitcoin");
        repository.create_asset("Ethereum".to_string(), 18000.5).await.expect("criar ethereum");

        let total = repository.total_value().await.expect("calcular total");

        assert_eq!(total, 368000.5);
    }
}