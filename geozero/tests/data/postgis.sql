-- Setup and run PostGIS tests:
--
-- createdb postgistest
-- psql postgistest -f tests/data/postgis.sql
--
-- DATABASE_URL="postgresql://$USER@localhost/postgistest?sslmode=disable"
-- cargo test --all-features -- --ignored postgis --test-threads 1

CREATE EXTENSION postgis;

CREATE TABLE point2d (
    fid SERIAL,
    intfield integer,
    strfield character varying,
    realfield double precision,
    datetimefield timestamp with time zone,
    datefield date,
    binaryfield bytea,
    geom geometry(Point)
);
