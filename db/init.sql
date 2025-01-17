CREATE TABLE users (
    id SERIAL PRIMARY KEY,              -- Unique identifier
    first_name VARCHAR(50) NOT NULL,    -- First name
    last_name VARCHAR(50) NOT NULL,     -- Last name
    email VARCHAR(255) UNIQUE NOT NULL, -- Email address
    latitude REAL,                      -- Latitude with precision up to 6 decimal places
    longitude REAL                     -- Longitude with precision up to 6 decimal places
);
