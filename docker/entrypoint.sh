#!/bin/bash
# TitanClaw Docker Entrypoint Script
#
# This script runs on container startup to:
# 1. Wait for database to be ready
# 2. Run database migrations
# 3. Start the TitanClaw application

set -e

echo "Starting TitanClaw..."

# Wait for PostgreSQL to be ready
if [ -n "$DATABASE_URL" ]; then
    echo "Waiting for PostgreSQL..."
    
    # Extract host from DATABASE_URL
    DB_HOST=$(echo "$DATABASE_URL" | sed -E 's|.*@([^/]+)/.*|\1|' | sed -E 's|:.*||')
    DB_PORT=$(echo "$DATABASE_URL" | sed -E 's|.*:([0-9]+)/.*|\1|' | head -1)
    
    # Default PostgreSQL port
    DB_PORT=${DB_PORT:-5432}
    
    # Wait for database
    for i in {1..30}; do
        if pg_isready -h "$DB_HOST" -p "$DB_PORT" -U titanclaw > /dev/null 2>&1; then
            echo "PostgreSQL is ready!"
            break
        fi
        echo "Waiting for PostgreSQL... ($i/30)"
        sleep 2
    done
fi

# Run migrations if DATABASE_URL is set
if [ -n "$DATABASE_URL" ]; then
    echo "Running database migrations..."
    # Note: The application handles migrations on first run
    # This is a placeholder for custom migration scripts
fi

# Create data directory if it doesn't exist
mkdir -p /data /config
chmod 755 /data /config || true

echo "Starting TitanClaw application..."
exec "$@"
