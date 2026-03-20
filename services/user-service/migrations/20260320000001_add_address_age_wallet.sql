-- Migration: add address, age, wallet_address to users table

ALTER TABLE users ADD COLUMN address         VARCHAR(500);
ALTER TABLE users ADD COLUMN age             SMALLINT;
ALTER TABLE users ADD COLUMN wallet_address  VARCHAR(255);
