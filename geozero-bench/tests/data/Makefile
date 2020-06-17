DBNAME = geozerobench

all: countries.shp countries-geom.shp countries.json osm-buildings-3857-ch.shp osm-buildings-3857-ch-geom.shp

countries.fgb: ../../../geozero-core/tests/data/countries.fgb
	cp $< $@

countries.gpkg: countries.fgb
	docker run --rm --user $$(id -u):$$(id -g) -v $$PWD:/data osgeo/gdal:alpine-normal-latest ogr2ogr -f GPKG /data/$@ /data/$<
	#echo "PRAGMA journal_mode = MEMORY" | sqlite3 $@

countries.shp: countries.gpkg
	ogr2ogr -f "ESRI Shapefile" -lco SPATIAL_INDEX=YES -lco ENCODING=UTF-8 $@ $<

countries-geom.shp: countries.gpkg
	ogr2ogr -f "ESRI Shapefile" -lco SPATIAL_INDEX=YES -select geom $@ $<

countries.json: countries.gpkg
	ogr2ogr -f "GeoJSON" $@ $<

osm-buildings-3857-ch.fgb:
	wget https://pkg.sourcepole.com/osm-buildings-3857-ch.fgb

osm-buildings-3857-ch.gpkg: osm-buildings-3857-ch.fgb
	docker run --rm --user $$(id -u):$$(id -g) -v $$PWD:/data osgeo/gdal:alpine-normal-latest ogr2ogr -f GPKG -nln buildings /data/$@ /data/$<

osm-buildings-3857-ch.shp: osm-buildings-3857-ch.gpkg
	ogr2ogr -f "ESRI Shapefile" -lco SPATIAL_INDEX=YES -lco ENCODING=UTF-8 $@ $<

osm-buildings-3857-ch-geom.shp: osm-buildings-3857-ch.gpkg
	ogr2ogr -f "ESRI Shapefile" -lco SPATIAL_INDEX=YES -select geom $@ $<

osm-buildings-3857-ch.json: osm-buildings-3857-ch.gpkg
	ogr2ogr -f "GeoJSON" $@ $<

createdb:
	psql postgres -c "DROP DATABASE IF EXISTS $(DBNAME)"
	psql postgres -c "CREATE DATABASE $(DBNAME)"
	psql $(DBNAME) -c "CREATE EXTENSION postgis"

countries_table: countries.gpkg
	ogr2ogr -f PostgreSQL -lco GEOMETRY_NAME=geom PG:dbname=$(DBNAME) $<

osm_buildings_table: osm-buildings-3857-ch.gpkg
	ogr2ogr -f PostgreSQL -lco GEOMETRY_NAME=geom PG:dbname=$(DBNAME) $<
