### dev run
```sh
cargo install cargo-watch
# creates test roles and accounts on localhost/test
# tests models and routes
cargo test
# launch application using test db on localhost
RUN_MODE=test cargo watch -x run -w src
```

## rest api usage:
```sh
# Terminal 1: authenticate admin and set env vars
eval "$(jq -M -r '@sh "ACCESS_TOKEN=\(.access_token) ACCOUNT_ID=\(.account.id)"' <<< "$(curl -H 'Content-Type: application/json' -X POST -d '{"email":"admin@test.com","password":"test"}' http://localhost:8080/accounts/authenticate)")"
echo $ACCESS_TOKEN
echo $ACCOUNT_ID
```
```sh
# Terminal 2: authenticate user and set env vars
eval "$(jq -M -r '@sh "ACCESS_TOKEN=\(.access_token) ACCOUNT_ID=\(.account.id)"' <<< "$(curl -H 'Content-Type: application/json' -X POST -d '{"email":"user@test.com","password":"test"}' http://localhost:8080/accounts/authenticate)")"
echo $ACCESS_TOKEN
echo $ACCOUNT_ID
```
```sh
# Terminal 3: authenticate validator and set env vars
eval "$(jq -M -r '@sh "ACCESS_TOKEN=\(.access_token) ACCOUNT_ID=\(.account.id)"' <<< "$(curl -H 'Content-Type: application/json' -X POST -d '{"email":"validator@test.com","password":"test"}' http://localhost:8080/accounts/authenticate)")"
echo $ACCESS_TOKEN
echo $ACCOUNT_ID
```
```sh
# Terminal 1: post, create geodata as admin
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
```
```sh
# get without token (invalid)
curl -s \
     -w '\n' \
     -H 'Content-Type: application/json' \
     http://localhost:8080/6b0866/geodata
# {"code":40003,"message":"Invalid authentication credentials"}
```
```sh
# Terminal2: get with token (valid for user role)
curl -s \
     -w '\n' \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $ACCESS_TOKEN" \
     http://localhost:8080/6b0866/geodata
```
```sh
# Terminal2: get near with token (valid for user role)
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
```
```sh
# Terminal 3: validation with token (valid for validator role)
curl -s \
     -w '\n' \
     -G \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $ACCESS_TOKEN" \
     http://localhost:8080/5be0da/validation
```