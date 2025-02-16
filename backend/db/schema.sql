CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE TYPE app_user_role AS ENUM ('Customer', 'Administrator');
CREATE TYPE app_order_status AS ENUM ('Unpaid', 'Confirmed', 'Fulfilled');

CREATE TABLE appuser (
    id BIGSERIAL PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    forename BYTEA NOT NULL,
    surname BYTEA NOT NULL,
    address BYTEA NOT NULL,
    role app_user_role NOT NULL
);

CREATE TABLE password (
    user_id BIGINT PRIMARY KEY,
    password TEXT NOT NULL,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES appuser(id) ON DELETE CASCADE
);
CREATE TABLE totp (
    user_id BIGINT PRIMARY KEY,
    secret BYTEA NOT NULL
);
CREATE TABLE product (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    listed BOOLEAN NOT NULL,
    price BIGINT NOT NULL CHECK (price > 0)
);
CREATE TABLE product_image (
    product_id BIGINT NOT NULL,
    path TEXT NOT NULL,
    PRIMARY KEY(product_id, path),
    CONSTRAINT fk_product FOREIGN KEY (product_id) REFERENCES product(id) ON DELETE CASCADE
);
CREATE TABLE apporder (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    order_placed TIMESTAMP NOT NULL,
    amount_charged BIGINT NOT NULL,
    status app_order_status NOT NULL,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES appuser(id) ON DELETE CASCADE
);
CREATE TABLE order_item(
    order_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    count BIGINT NOT NULL,
    PRIMARY KEY (order_id, product_id),
    CONSTRAINT fk_order FOREIGN KEY (order_id) REFERENCES apporder(id) ON DELETE CASCADE, 
    CONSTRAINT fk_product FOREIGN KEY (product_id) REFERENCES product(id) ON DELETE CASCADE
);
