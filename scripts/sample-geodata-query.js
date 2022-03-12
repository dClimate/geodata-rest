db = connect('mongodb://localhost/geomancy');
printjson(
    db.geodata.find({
    location: {
        $near: {
            $geometry: { type: "Point", coordinates: [-73.91320, 40.68405] }, $minDistance: 0, $maxDistance: 10000
            }
        }
    })
);