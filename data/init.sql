CREATE DATABASE testbench;

\connect testbench;

CREATE TABLE users(email VARCHAR(255) NOT NULL PRIMARY KEY,
    first VARCHAR(255),
    last VARCHAR(255),
    county VARCHAR(255),
    city VARCHAR(255),
    age int
);