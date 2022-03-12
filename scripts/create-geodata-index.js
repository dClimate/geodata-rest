db = connect( 'mongodb://localhost/geomancy' );
db.geodata.createIndex( { location : "2dsphere" } )