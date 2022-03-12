### dev run
* cargo install cargo-watch
* cargo watch -x run -w src

### demo run
cargo run

## rest api usage:
### create admin account
curl -X POST --header "Content-Type: application/json" -d '{"name":"admin","email":"admin@test.com","password":"test", "roles": ["user", "admin" ]}' http://localhost:8080/9f7b1ef8134d9c462d39e24212368aa8d5341c6e/accounts

### create user account
curl -X POST --header "Content-Type: application/json" -d '{"name":"user","email":"user@test.com","password":"test", "roles": ["user"]}' http://localhost:8080/9f7b1ef8134d9c462d39e24212368aa8d5341c6e/accounts

### create validator account
curl -X POST --header "Content-Type: application/json" -d '{"name":"validator","email":"validator@test.com","password":"test", "roles": ["validator"]}' http://localhost:8080/9f7b1ef8134d9c462d39e24212368aa8d5341c6e/accounts

### authenticate
curl -X POST --header "Content-Type: application/json" -d '{"email":"admin@test.com","password":"test"}' http://localhost:8080/accounts/authenticate

### authenticate admin and set env vars
eval "$(jq -M -r '@sh "ACCESS_TOKEN=\(.access_token) ACCOUNT_ID=\(.account.id)"' <<< "$(curl -H 'Content-Type: application/json' -X POST -d '{"email":"admin@test.com","password":"test"}' http://localhost:8080/accounts/authenticate)")"
echo $ACCESS_TOKEN
echo $ACCOUNT_ID

### authenticate user and set env vars
eval "$(jq -M -r '@sh "ACCESS_TOKEN=\(.access_token) ACCOUNT_ID=\(.account.id)"' <<< "$(curl -H 'Content-Type: application/json' -X POST -d '{"email":"user@test.com","password":"test"}' http://localhost:8080/accounts/authenticate)")"
echo $ACCESS_TOKEN
echo $ACCOUNT_ID

### authenticate validator and set env vars
eval "$(jq -M -r '@sh "ACCESS_TOKEN=\(.access_token) ACCOUNT_ID=\(.account.id)"' <<< "$(curl -H 'Content-Type: application/json' -X POST -d '{"email":"validator@test.com","password":"test"}' http://localhost:8080/accounts/authenticate)")"
echo $ACCESS_TOKEN
echo $ACCOUNT_ID


### get without token (invalid)
curl -s \
     -w '\n' \
     -H 'Content-Type: application/json' \
     http://localhost:8080/6b0866/geodata
{"code":40003,"message":"Invalid authentication credentials"}

### get with token (valid for user role)
curl -s \
     -w '\n' \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $ACCESS_TOKEN" \
     http://localhost:8080/6b0866/geodata

### post, create geodata as admin
curl -s \
     -w '\n' \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $ACCESS_TOKEN" \
     http://localhost:8080/6a2dda/geodata \
     -d '{"account":"$ACCOUNT_ID",

    "location": {
        "type": "GeometryCollection",
        "geometries": [
            {
                "type": "Point",
                "coordinates": [
                    -73.91320,
                    40.68405  
                ]
            }
        ]
    },
    "geotype": "Wind",
    "value": 11.1,
    "source": "Google Earth Engine",
    "quality": 5
    }'

### get near (valid for user role)
curl -s \
     -w '\n' \
     -G \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $ACCESS_TOKEN" \
     -d 'lon=-73.91320' \
     -d 'lat=40.68405' \
     -d 'min=0' \
     -d 'max=10000' \
     http://localhost:8080/6b0866/geodata/near

### validation (valid for validator role)
curl -s \
     -w '\n' \
     -G \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $ACCESS_TOKEN" \
     http://localhost:8080/5be0da/validation

## mongosh
`use geomancy`

`db.geodata.createIndex( { location : "2dsphere" } )`

### returns x of 10
db.geodata.find({
   location: {
      $near: {
         $geometry: { type: "Point", coordinates: [ -73.91320, 40.68405] }, 
         $minDistance: 0, $maxDistance: 10000 }
    } 
}).itcount()

### returns x of 10
db.geodata.find({
   location: {
      $near: {
         $geometry: { type: "Point", coordinates: [
                    -73.91320,
                    40.68405
                ]
          }, $minDistance: 0, $maxDistance: 100000
      }
    }
})

db.geodata.find({
  location: {
    $geoWithin: {
      $centerSphere: [
        [-73.91320, 40.68405],
        100
      ]
    }
  }
})

### returns 1
db.geodata.find({ location: { $nearSphere: { $geometry: { type: "Point", coordinates: [-73.91320, 40.68405] }, $maxDistance: 5000 } } })

