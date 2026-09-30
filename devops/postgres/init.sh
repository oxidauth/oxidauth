#!/bin/bash

set -e                              # Treat unset variables as an error

SERVICES_LIST=($SERVICES)

for service in "${SERVICES_LIST[@]}"; do
    echo "::: initializing $service :::"
    echo "::: connecting to $POSTGRES_DB as $POSTGRES_USER"

# this can't move -- formatting this messes up the script
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname postgres <<-EOSQL
    CREATE USER $service WITH ENCRYPTED PASSWORD '$service' CREATEDB;
    CREATE DATABASE $service WITH OWNER $service;
EOSQL

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$service" <<-EOSQL
    GRANT ALL PRIVILEGES ON SCHEMA public TO $service;
    GRANT ALL PRIVILEGES ON DATABASE $service TO $service;
    GRANT ALL ON SCHEMA public TO $service;
EOSQL

done