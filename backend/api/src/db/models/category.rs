use sqlx::{query, query_as, Error, PgPool};

pub struct CategoryInsert {
    pub name: String,
    pub description: String,
}

pub struct Category {
    id: i64,
    pub name: String,
    pub description: String,
}

impl CategoryInsert {
    pub async fn store(&self, db_client: &PgPool) -> Result<Category, Error> {
        query_as!(
            Category,
            "INSERT INTO category (name, description) VALUES ($1, $2) RETURNING *",
            self.name,
            self.description
        )
        .fetch_one(db_client)
        .await
    }
}

impl Category {
    pub async fn select_one(id: i64, db_client: &PgPool) -> Result<Option<Self>, Error> {
        query_as!(Self, "SELECT * FROM category WHERE id = $1", id)
            .fetch_optional(db_client)
            .await
    }
    pub async fn select_all(db_client: &PgPool) -> Result<Vec<Self>, Error> {
        query_as!(Self, "SELECT * FROM category")
            .fetch_all(db_client)
            .await
    }
    pub fn id(&self) -> i64 {
        self.id
    }
    pub async fn update(&self, db_client: &PgPool) -> Result<(), Error> {
        query!(
            "UPDATE category SET name = $1, description = $2 WHERE id = $3",
            self.name,
            self.description,
            self.id
        )
        .execute(db_client)
        .await
        .map(|_| ())
    }
    pub async fn delete(self, db_client: &PgPool) -> Result<(), Error> {
        query!("DELETE FROM category WHERE id = $1", self.id)
            .execute(db_client)
            .await
            .map(|_| ())
    }
}
