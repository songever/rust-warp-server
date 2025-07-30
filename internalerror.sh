curl -X GET http://localhost:3030/external_api?url=http://127.0.0.1:9999/not_exist

curl -X GET http://localhost:3030/external_api?url=https://httpbin.org/status/404

curl -X GET http://localhost:3030/external_api?url=https://httpbin.org/status/500