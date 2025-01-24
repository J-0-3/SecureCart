CREATE TABLE appuser (
    id BIGSERIAL PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    forename TEXT NOT NULL,
    surname TEXT NOT NULL,
    age SMALLINT NOT NULL
);

CREATE TABLE password (
    user_id BIGINT PRIMARY KEY,
    password TEXT NOT NULL,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES appuser(id)
);
CREATE TABLE totp (
    user_id BIGINT PRIMARY KEY,
    secret BYTEA NOT NULL
);
CREATE TABLE product (
    id BIGSERIAL PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT NOT NULL,
    stock BIGINT NOT NULL CHECK (stock >= 0),
    price BIGINT NOT NULL CHECK (price > 0)
);
CREATE TABLE category (
    id BIGSERIAL PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT NOT NULL
);
CREATE TABLE category_membership (
    product_id BIGINT, 
    category_id BIGINT,
    PRIMARY KEY(product_id, category_id),
    CONSTRAINT fk_product FOREIGN KEY (product_id) REFERENCES product(id),
    CONSTRAINT fk_category FOREIGN KEY (category_id) REFERENCES category(id)
);
CREATE TABLE offer (
    id BIGSERIAL PRIMARY KEY,
    percentage SMALLINT NOT NULL,
    label TEXT
);
CREATE TABLE offer_membership (
    product_id BIGINT,
    offer_id BIGINT,
    PRIMARY KEY (product_id, offer_id)
);
CREATE TABLE apporder (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    order_placed TIMESTAMP NOT NULL,
    amount_charged BIGINT NOT NULL,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES appuser(id)
);
CREATE TABLE order_item(
    order_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    PRIMARY KEY (order_id, product_id),
    CONSTRAINT fk_order FOREIGN KEY (order_id) REFERENCES apporder(id),
    CONSTRAINT fk_product FOREIGN KEY (product_id) REFERENCES product(id)
);
