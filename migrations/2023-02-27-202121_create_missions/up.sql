-- Your SQL goes here
CREATE TABLE Missions (
    mission_id SERIAL PRIMARY KEY,
    mission_name VARCHAR NOT NULL,
    location VARCHAR NOT NULL,
    tags TEXT [] NOT NULL
);