use sqlx::{query_as, query, Error, PgPool};
use time::PrimitiveDateTime;

pub struct AppOrderInsert {
    pub user_id: i64,
    pub order_placed: PrimitiveDateTime,
    pub amount_charged: i64,
}

pub struct AppOrder {
    id: i64,
    pub user_id: i64,
    pub order_placed: PrimitiveDateTime,
    pub amount_charged: i64,
}

impl AppOrderInsert {
    pub async fn store(self, db_client: &PgPool) -> Result<AppOrder, Error> {
        query_as!(
            AppOrder, 
            "INSERT INTO apporder (user_id, order_placed, amount_charged) VALUES ($1, $2, $3) RETURNING *", 
            &self.user_id, &self.order_placed, &self.amount_charged
        ).fetch_one(db_client).await
    }
}

impl AppOrder {
    pub fn id(&self) -> i64 {
        self.id
    }
    pub async fn select_one(id: i64, db_client: &PgPool) -> Result<Option<Self>, Error> {
        query_as!(Self, "SELECT * FROM apporder WHERE id = $1", &id)
            .fetch_optional(db_client)
            .await
    }
    pub async fn select_all(db_client: &PgPool) -> Result<Vec<Self>, Error> {
        query_as!(Self, "SELECT * FROM apporder")
            .fetch_all(db_client)
            .await
    }
    pub async fn update(&self, db_client: &PgPool) -> Result<(), Error> {
        query!(
            "UPDATE apporder SET user_id=$1, order_placed=$2, amount_charged=$3 WHERE id=$4", 
            self.user_id, self.order_placed, self.amount_charged, self.id
        ).execute(db_client).await?;
        Ok(())
    }
    pub async fn delete(self, db_client: &PgPool) -> Result<(), Error> {
        query!("DELETE FROM apporder WHERE id = $1", self.id).execute(db_client).await?;
        Ok(())
    }
}

