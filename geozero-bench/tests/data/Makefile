DBNAME = testdb

all: countries.fgb countries.gpkg

countries.fgb: ../../../geozero-core/tests/data/countries.fgb
	cp $< $@

countries.gpkg: countries.fgb
	docker run --rm -v $$PWD:/data osgeo/gdal:alpine-normal-latest ogr2ogr -f GPKG /data/$@ /data/$<

osm-buildings-3857-ch.gpkg: osm-buildings-3857-ch.fgb
	docker run --rm -v $$PWD:/data osgeo/gdal:alpine-normal-latest ogr2ogr -f GPKG -nln buildings /data/$@ /data/$<

createdb:
	psql postgres -c "DROP DATABASE IF EXISTS $(DBNAME)"
	psql postgres -c "CREATE DATABASE $(DBNAME)"
	psql $(DBNAME) -c "CREATE EXTENSION postgis"

# countriesdb: countries.fgb
# 	docker run --rm -v $$PWD:/data osgeo/gdal:alpine-normal-latest ogr2ogr -f PostgreSQL PG:dbname=$(DBNAME) /data/$<

countries_table: countries.gpkg
	ogr2ogr -f PostgreSQL PG:dbname=$(DBNAME) $<

osm_buildings_table: osm-buildings-3857-ch.gpkg
	ogr2ogr -f PostgreSQL PG:dbname=$(DBNAME) $<
