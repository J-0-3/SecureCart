use sqlx::{query, query_as, Error, PgPool};

pub struct ProductInsert {
    pub name: String,
    pub description: String,
    stock: i64,
    price: i64,
}
pub struct Product {
    id: i64,
    pub name: String,
    pub description: String,
    stock: i64,
    price: i64,
}

impl ProductInsert {
    pub fn new(name: &str, description: &str, stock: u32, price: u32) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            stock: stock as i64,
            price: price as i64,
        }
    }
    pub const fn stock(&self) -> u32 {
        self.stock as u32
    }
    pub const fn price(&self) -> u32 {
        self.price as u32
    }
    pub async fn store(self, db_client: &PgPool) -> Result<Product, Error> {
        query_as!(
            Product, 
            "INSERT INTO product (name, description, stock, price) VALUES ($1, $2, $3, $4) RETURNING *",
            self.name, self.description, self.stock, self.price
        ).fetch_one(db_client).await
    }
}

impl Product {
    pub async fn select_one(id: i64, db_client: &PgPool) -> Result<Option<Self>, Error> {
        query_as!(Self, "SELECT * FROM product WHERE id = $1", id)
            .fetch_optional(db_client)
            .await
    }
    pub async fn select_all(db_client: &PgPool) -> Result<Vec<Self>, Error> {
        query_as!(Self, "SELECT * FROM product").fetch_all(db_client).await
    }
    pub fn set_stock(&mut self, stock: u32) {
        self.stock = stock as i64
    }
    pub fn set_price(&mut self, price: u32) {
        self.price = price as i64
    }
    pub const fn stock(&self) -> u32 {
        self.stock as u32
    }
    pub const fn price(&self) -> u32 {
        self.price as u32
    }
    pub const fn id(&self) -> i64 {
        self.id
    }
    pub async fn update(&self, db_client: &PgPool) -> Result<(), Error> {
        query!(
            "UPDATE product SET name = $1, description = $2, stock = $3, price = $4 WHERE id = $5", 
            self.name, self.description, self.stock, self.price, self.id
        ).execute(db_client).await.map(|_| ())
    }
    pub async fn delete(self, db_client: &PgPool) -> Result<(), Error> {
        query!("DELETE FROM product WHERE id = $1", self.id).execute(db_client).await.map(|_| ())
    }
}
