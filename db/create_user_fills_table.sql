CREATE TABLE user_fills (
    id SERIAL PRIMARY KEY,
    user_address VARCHAR(100) NOT NULL,
    closed_pnl DOUBLE PRECISION NOT NULL,
    coin VARCHAR(50) NOT NULL,
    crossed BOOLEAN NOT NULL,
    dir VARCHAR(20) NOT NULL,
    hash VARCHAR(100) NOT NULL,
    order_id BIGINT NOT NULL,
    price DOUBLE PRECISION NOT NULL,
    side VARCHAR(10) NOT NULL,
    start_position DOUBLE PRECISION NOT NULL,
    size DOUBLE PRECISION NOT NULL,
    timestamp BIGINT NOT NULL,
    fee DOUBLE PRECISION NOT NULL
);