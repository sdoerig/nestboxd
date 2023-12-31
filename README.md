# nestbox

## Note

I'm still developing. My aim is to create a minimum viable product. This means

- Having a web based GUI (missing)
- Having a backend, which is able to
  - show data according to the scanned QR code.
  - authenticate a user
  - accept modifications of nestboxes by an authenticated and authorized user


## Idea

Some birdwatcher are also taking care of nestboxes. So these nestboxes are regularly checked during wintertime. To give you an idea, we're taking care of nearly 400 boxes. 
If a bird has been using the nestbox one can distinguish the specific bird by the nest built. Then the birdwatcher cleans the box, which means the nest is removed and the box is ready for the next breeding. Normaly there is also a list keept, which nestboxes where used by which birds and which year. So my intention is to give the nestbox cleaner a tool as today nearly everybody has a smartphone available. So the key thoughts are:

- A QR code tag on each nestbox.
- A authenticaated cleaner now can scan the QR code and perform the follwing action.
  - Set a new geolocation for the box, so it can be found later.
  - Set a detected breeding (type of bird and year)
- Somebody just walking by can also scan the QR code an gets the following informations:
  - Association responsible for the box and how to contact them.
  - History of the box, breedings in the past.
- Getting the associations taking care of those boxes "out of the dark", since they do valuable protection work and many people do not know the even exist.
- Personally I find it fun developing this kind of application beside work. Any backend like software is written in Rust - not because I'm exceptionally good at Rust, but I like it and I like learning too.

## Repo
This repo contains the backend.

## Database model

### General

Allthough mongodb is a schemaless database, the concept of the model can described easy. Note it is just a skeleton, which means I just included the attributs I really consider to tbe vital. 

### uuid

In each record one finds an attribut `uuid`. It acts as public visible primary key, since I think on should not unveil to much about the database and mongodb object ids are guessable. So the uuid is randomly generated. Any field within a collection named "uuid" is a public accessable key and can be used in URLs an so on. They all are randomly generated type 4 uuids.

### Model

Before going into details of each collection here is a quick overview ot the relations of the collections.

```

mandant 1 ----- n users
|
+----- 1 ------ n nestboxes 1 ----- n breeds
                  |
                  \ 1 ------------- n geolocation
                  
```

### mandants

A mandant is an association of birdwatchers if you want so. They are taking care of a cuple of birdboxes in a defined geografical area. One record holds now the attributes

- _id: ObjectId - not used in any context now.
- uuid: Public accessable key e.g. in URLs. Example e7620353-b6f6-47e9-b543-66af20769145
- name: Name of the association, E.g. BirdLife 
- website: Most of these associations do have a website e.g. https://www.birdwatcher.ch
- email: Obvious e.g. bird@iseeyou.ch 


### users

A user belongs to a mandant and can mutate any birdbox belonging tho this mandant. Currently a simple password based authentication is implemented. The attributs:

- _id: ObjectId - not used in any context now. 
- mandant_uuid: user belongs to this mandant. 
- username: login name of the user  
- uuid: Public accessable key 
- lastname
- firstname
- email
- password_hash: Salted SHA3 hash of the password
- salt: Type 4 uuid

### sessions

If a user successfully logged in a copy of the user record with the attribute session_key is stored in the sessions collections.

- session_key: uuid type 4
- session_created_at: timestamp used for a TTL within mongodb, set to 86400 seconds. Means the database removes the session form the collection after one day.

The session key must be uniqe. A user can only have one session at the same time. The session object will be deleted after 86400 seconds.

### nestboxes

A nestbox represents by concept a QR code referencable item. In our case it would be a wooden nestbox e.g. hanging in a tree.

- _id: ObjectId 
- public: Is the nestbox data public - true or false 
- uuid: Public accessable key
- mandant_uuid: Nestbox belongs to this mandant 
- created_at: ISODate Zulu time

### geolocations 

Geolocation indicating where the nestbox was placed over the time.

- _id: ObjectId
- uuid: Public key of the geolocation 
- nestbox_uuid: nestbox the location is attached to
- from_date: Geolocation valid from, timestamp Zulu time
- until_date: Geolocation valid until, timestamp Zulu time
- position: Geospatial type point 

### breeds

Holds track of all the breeds.

- _id: ObjectId
- uuid: Public key 
- nestbox_uuid: Breed discovered in this nestbox
- user_uuid: Breed discoverd by this user 
- discovery_date: Breed discovered at timestamp Zulu time
- bird_uuid: Estimated bird according to the nest found in the box

### birds

This collection stores all the birds of one mandant. Each mandant must create its own birds. The reasons for this redundancy are

- different location different birds
- different clima, different birds
- different altitude, different birds
- different language, different birds

The attributes are

- _id: ObjectId
- uuid: Public key of the bird
- bird: Name of the bird
- mandant_uuid: Mandant by which the bird was created

## Indices

The database needs to perform the indices below.

```
db.mandants.createIndex({"uuid": 1}, {"unique": true})
db.nestboxes.createIndex({"uuid": 1}, {"unique": true})
db.breeds.createIndex({"uuid": 1}, {"unique": true})
db.breeds.createIndex({"nestbox_uuid": 1})
db.users.createIndex({"uuid": 1}, {"unique": true})
db.users.createIndex({"username": 1}, {"unique": true})
db.geolocations.createIndex({"uuid": 1}, {"unique": true})
db.birds.createIndex({"uuid": 1}, {"unique": true})
db.birds.createIndex({"mandant_uuid": 1})
db.sessions.createIndex({"session_key": 2}, {"unique": true})
db.sessions.createIndex({"session_created_at": 1}, { expireAfterSeconds: 86400 })
db.geolocations.createIndex({"nestbox_uuid": 1})
db.nestboxes.createIndex({"mandant_uuid":1})
```

## Backend

### Framework
The backend is a restful server written based on [actix-web](https://actix.rs/).

### Start 

The daeamon needs a config file, if started without one gets the message below.

```
Usage: target/debug/nestboxd -c CONFIG_FILE

Options:
    -c, --config CONFIG_FILE
                        Path to configuration file

```

The config file is a YAML file with the content

```
mongodb:
  uri: mongodb://localhost:27017
  database: nestbox
httpserver:
  ip: 127.0.0.1
  port: "8080"
images:
  directory: /some/where
```

### Logging

There is at the moment a standard logging to STDOUT.

```
[2021-06-03T19:25:32Z INFO  actix_server::builder] Starting 4 workers
[2021-06-03T19:25:32Z INFO  actix_server::builder] Starting "actix-web-service-127.0.0.1:8080" service on 127.0.0.1:8080
[2021-06-03T19:29:25Z INFO  actix_web::middleware::logger] 127.0.0.1:47618 "GET /nestboxes/9915a1ef-edaa-4268-b86c-7e43fe0bbd6b/breeds?page_limit=2&page_number=1 HTTP/1.1" 200 539 "-" "curl/7.68.0" 0.024426
[2021-06-03T19:29:39Z INFO  actix_web::middleware::logger] 127.0.0.1:47624 "GET /nestboxes/9915a1ef-edaa-4268-b86c-7e43fe0bbd6b/breeds?page_limit=2&page_number=1 HTTP/1.1" 200 641 "-" "curl/7.68.0" 0.021756
```


### post /login

Allows login. At the moment, database_bouncycastle hashes clear text passwords. This will be turned into hashes, which means then the client transmits only the hash.
If a user loges in twice, the old session is destroyed. If an authenticated user fails to login the current session is deleted too - which actually means the user has been logged out.

#### Request

```
curl \
  --header "Content-Type: application/json" \
  --request POST \
  --data '{"username":"fg_199","password":"secretbird"}' \
  http://127.0.0.1:8080/login
```
#### Response

```
{"username":"fg_199","success":true,"session":"28704470-0908-408e-938f-64dd2b7578b9"}
```


### get /birds

#### Request

A valid session must be provided. If so the users gets a pageable view of the birds beloging to the mandant the user session belongs to.

Note also the birds provided are the ones selectable when reporting a breed.

```
curl -H "Authorization: Basic 2c91ebd1-800e-4573-8f2b-6ac91c2a407a" http://127.0.0.1:8080/birds?page_limit=2\&page_number=1
```

#### Response

```
{
   "documents": [
      {
         "uuid": "aee03da8-e297-46da-aac2-51f6604558dc",
         "bird": "bird_0"
      },
      {
         "uuid":"31a1e34e-7ae3-4b59-9197-c0ad9468fa20",
         "bird":"bird_1"
      }
   ],
   "counted_documents":150,
   "pages":75,
   "page_number":1,
   "page_limit":2
}
```

If no or an invalid session is provided the response will be 

```
{"error":2,"error_message":"UNAUTHORIZED"}
```



### get /nestboxes/{uuid}/breeds

#### Request

```
curl \
  -H "Authorization: Basic 28704470-0908-408e-938f-64dd2b7578b9" \
  -H "Content-Type: application/json" \
  http://127.0.0.1:8080/nestboxes/9915a1ef-edaa-4268-b86c-7e43fe0bbd6b/breeds?page_limit=2\&page_number=1

```

#### Response

```
{
   "documents":[
      {
         "uuid":"0b5cec76-02ac-4c6e-933e-62ebfae3e337",
         "nestbox_uuid":"6f25fd00-011a-462f-aa3d-6959e6809017",
         "discovery_date":"2021-06-01 18:36:38.989 UTC",
         "user_uuid":"071f3391-2c8f-4807-89d8-4b2870228730",
         "bird_uuid":"ebe661d6-77ba-4bd1-bae3-9e4e7eb880a6",
         "bird":"bird_17"
      },
      {
         "uuid":"320d1b78-3e30-4741-aff2-ce8180dd09fb",
         "nestbox_uuid":"6f25fd00-011a-462f-aa3d-6959e6809017",
         "discovery_date":"2021-06-01 18:36:38.989 UTC",
         "user_uuid":"071f3391-2c8f-4807-89d8-4b2870228730",
         "bird_uuid":"afc7ffcf-92aa-4e2a-9c41-92eb300d0281",
         "bird":"bird_76"
      },
      {
         "uuid":"aae7236e-74fb-40cb-8502-111ac4f2d984",
         "nestbox_uuid":"6f25fd00-011a-462f-aa3d-6959e6809017",
         "discovery_date":"2021-06-01 18:36:38.989 UTC",
         "user_uuid":"071f3391-2c8f-4807-89d8-4b2870228730",
         "bird_uuid":"60e4a52c-cd81-44fc-90bc-b2b578caae08",
         "bird":"bird_108"
      }
   ],
   "counted_documents":3,
   "pages":1,
   "page_number":1,
   "page_limit":10
}
```
If the user is not authenticated the response is without user_uuid.

```
{
   "documents":[
      {
         "uuid":"0b5cec76-02ac-4c6e-933e-62ebfae3e337",
         "nestbox_uuid":"6f25fd00-011a-462f-aa3d-6959e6809017",
         "discovery_date":"2021-06-01 18:36:38.989 UTC",
         "bird_uuid":"ebe661d6-77ba-4bd1-bae3-9e4e7eb880a6",
         "bird":"bird_17"
      },
      {
         "uuid":"320d1b78-3e30-4741-aff2-ce8180dd09fb",
         "nestbox_uuid":"6f25fd00-011a-462f-aa3d-6959e6809017",
         "discovery_date":"2021-06-01 18:36:38.989 UTC",
         "bird_uuid":"afc7ffcf-92aa-4e2a-9c41-92eb300d0281",
         "bird":"bird_76"
      },
      {
         "uuid":"aae7236e-74fb-40cb-8502-111ac4f2d984",
         "nestbox_uuid":"6f25fd00-011a-462f-aa3d-6959e6809017",
         "discovery_date":"2021-06-01 18:36:38.989 UTC",
         "bird_uuid":"60e4a52c-cd81-44fc-90bc-b2b578caae08",
         "bird":"bird_108"
      }
   ],
   "counted_documents":3,
   "pages":1,
   "page_number":1,
   "page_limit":10
}
```


### post /nestboxes/{uuid}/breeds

#### Request

```
curl \
  -H "Authorization: Basic b955d5ab-531d-45a5-b610-5b456fa509d9" \
  --H "Content-Type: application/json" \
  --request POST \
  --data '{"bird_uuid": "a4152a25-b734-4748-8a43-2401ed387c65", "bird":"a"}' \
  http://127.0.0.1:8080/nestboxes/9973e59f-771d-452f-9a1b-8b4a6d5c4f95/breeds

```


#### Response

```
{"inserted_id":{"$oid":"60bfcc160014769d00e0b88a"}}
```

### post /nestboxes/{uuid}/geolocations

#### Request

Adds a new geolocation to a nestbox. This is the case if it had been moved e.g. tree has been cut or broken down.

```
curl   \
  -H "Authorization: Basic 8f42f009-dda8-4448-a2db-f9abb8326b06" \
  -H "Content-Type: application/json"   \
  --request POST   \
  --data '{ "long":-11.6453, "lat": -47.2345}'   \
  http://127.0.0.1:8080/nestboxes/787c9399-b10a-44f7-bcc5-251e4414cbb0/geolocations

```

#### Response

In case everything went well

```
{"inserted_id":"ObjectId(\"60d5a4ed0062a0f1006a1967\")"}
```

If the user was not properly authenticated

```
{"error":2,"error_message":"UNAUTHORIZED"}
```
or if the nestbox belongs to another mandant then the user session

```
{"error":1,"error_message":"NESTBOX_OF_OTHER_MANDANT"}
```

### get /nestboxes/{uuid}

#### Request
Fetches a nestbox.
```
curl  http://127.0.0.1:8080/nestboxes/1bec20fc-5416-4941-b7e4-e15aa26a5c7a
```

#### Response

```
{
  "uuid": "5d04fab4-6bdd-4103-b57f-9e231e777790",
  "created_at": "2021-06-01 18:36:38.443 +00:00:00",
  "images": [
    "935206e4bf27eccfa4562f32043ef9435c0a037dfd08811817c187ed618412c1.jpg"
  ],
  "mandant_uuid": "a9ed6720-5e01-417a-9278-9417df4d8aa0",
  "mandant_name": "BirdLife 100",
  "mandant_website": "https://www.birdwatcher.ch"
}
``` 

### post /nestboxes/{uuid}/images

#### Request
Allows to post an image of the nestbox.
```
curl -v   -H "Authorization: Basic 9e693578-500c-4505-990a-5db978e557c2"     -F file=@20211024_170535.jpg  http://127.0.0.1:8080/nestboxes/5d04fab4-6bdd-4103-b57f-9e231e777790/images
```

#### Response

If authenticated and authorized a SHA3 hash of the file is returned.

```
< HTTP/1.1 201 Created
< content-length: 86
< content-type: application/json
< date: Thu, 09 Dec 2021 18:03:50 GMT
< 

{"file_name":["c9aff3597f2fbc4dd5a22c9c0764c2324c5dd68776a367ac150e6a40bfed6526.jpg"]}
```

If not authenticated or authorized 

```
< HTTP/1.1 401 Unauthorized
< content-length: 42
< content-type: application/json
< date: Thu, 09 Dec 2021 18:06:40 GMT
* HTTP error before end of send, stop sending
< 
* Closing connection 0
{"error":2,"error_message":"UNAUTHORIZED"}
```




