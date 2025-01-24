CREATE SCHEMA appuser;
CREATE SCHEMA auth;
CREATE SCHEMA product;
CREATE SCHEMA order;

CREATE TABLE appuser.data (
    id BIGSERIAL PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    forename TEXT NOT NULL,
    surname TEXT NOT NULL,
    age SMALLINT NOT NULL
);

CREATE TABLE auth.password (
    user_id BIGINT PRIMARY KEY,
    password TEXT NOT NULL,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES appuser.data(id)
);
CREATE TABLE auth.totp (
    user_id BIGINT PRIMARY KEY,
    secret BYTEA NOT NULL
);
CREATE TABLE product.product (
    id BIGSERIAL PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT NOT NULL,
    stock BIGINT NOT NULL CHECK (stock >= 0);
    price BIGINT NOT NULL CHECK (price > 0)
);
CREATE TABLE product.category (
    id BIGSERIAL PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT NOT NULL
);
CREATE TABLE product.category_membership (
    product_id BIGINT, 
    category_id BIGINT,
    PRIMARY KEY(product_id, category_id),
    CONSTRAINT fk_product FOREIGN KEY (product_id) REFERENCES product.product(id),
    CONSTRAINT fk_category FOREIGN KEY (category_id) REFERENCES product.category(id)
);
CREATE TABLE product.offer (
    id BIGSERIAL PRIMARY KEY,
    percentage SMALLINT NOT NULL,
    label TEXT
);
CREATE TABLE product.offer_membership (
    product_id BIGINT,
    offer_id BIGINT,
    PRIMARY KEY (product_id, offer_id)
);
CREATE TABLE order.order (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    order_placed DATETIME NOT NULL,
    amount_charged BIGINT NOT NULL,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES appuser.data(id)
);
CREATE TABLE order.order_item(
    order_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    PRIMARY KEY (order_id, product_id),
    CONSTRAINT fk_order FOREIGN KEY (order_id) REFERENCES order.order(id),
    CONSTRAINT fk_product FOREIGN KEY (product_id) REFERENCES product.product(id)
);
