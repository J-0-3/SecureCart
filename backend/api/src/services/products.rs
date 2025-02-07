use crate::db::{
    self,
    models::product::{Product, ProductInsert},
};

pub struct ProductSearchParameters {
    name: Option<String>,
    price_min: Option<u32>,
    price_max: Option<u32>,
}

pub async fn search_products(
    db_conn: &db::ConnectionPool,
    params: &ProductSearchParameters,
) -> Result<Vec<Product>, db::errors::DatabaseError> {
    let mut result = Vec::new();
    // We clone search_name here to avoid cloning the entire struct.
    let search_name = params.name.clone();
    let search_price_min = params.price_min;
    let search_price_max = params.price_max;
    for product in Product::select_all(db_conn).await? {
        let name_match = search_name
            .as_ref()
            .is_none_or(|name| product.name.starts_with(name));
        let price_min_match = search_price_min
            .as_ref()
            .is_none_or(|price| product.price() >= *price);
        let price_max_match = search_price_max
            .as_ref()
            .is_none_or(|price| product.price() <= *price);
        if name_match && price_min_match && price_max_match {
            result.push(product);
        }
    }
    Ok(result)
}

pub async fn retrieve_product(
    id: u32,
    db_conn: &db::ConnectionPool,
) -> Result<Option<Product>, db::errors::DatabaseError> {
    Product::select_one(id, db_conn).await
}

pub struct ProductUpdate {
    name: Option<String>,
    price: Option<u32>,
    listed: Option<bool>,
}

pub async fn update_product(
    id: u32,
    product_info: ProductUpdate,
    db_conn: &db::ConnectionPool,
) -> Result<(), errors::ProductUpdateError> {
    let mut product = Product::select_one(id, db_conn)
        .await?
        .ok_or(errors::ProductUpdateError::NonExistent)?;
    if let Some(name) = product_info.name {
        product.set_name(&name);
    }
    if let Some(price) = product_info.price {
        product.set_price(price);
    }
    //product_info.listed.map(|listed| product.set_listed(listed));
    Ok(product.update(db_conn).await?)
}

pub async fn create_product(
    data: ProductInsert,
    db_conn: &db::ConnectionPool,
) -> Result<Product, db::errors::DatabaseError> {
    data.store(db_conn).await
}

pub async fn delete_product(
    id: u32,
    db_conn: &db::ConnectionPool,
) -> Result<(), errors::ProductDeleteError> {
    let product = Product::select_one(id, db_conn)
        .await?
        .ok_or(errors::ProductDeleteError::NonExistent)?;
    Ok(product.delete(db_conn).await?)
}

pub mod errors {
    use crate::db::errors::DatabaseError;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum ProductUpdateError {
        #[error(transparent)]
        DatabaseError(#[from] DatabaseError),
        #[error("The product being updated does not exist.")]
        NonExistent,
    }
    #[derive(Error, Debug)]
    pub enum ProductDeleteError {
        #[error(transparent)]
        DatabaseError(#[from] DatabaseError),
        #[error("The product being deleted does not exist.")]
        NonExistent,
    }
}
