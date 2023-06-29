CREATE TABLE authorities (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    name VARCHAR(32) UNIQUE NOT NULL,
    client_key UUID UNIQUE DEFAULT uuid_generate_v4(),
    status VARCHAR(32) NOT NULL,
    strategy VARCHAR(32) NOT NULL UNIQUE,
    settings JSONB DEFAULT '{}'::jsonb,
    params JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
