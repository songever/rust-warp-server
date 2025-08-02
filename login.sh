curl --location --request POST 'http://localhost:8080/login' \
    --header 'Content-Type: application/json' \
    --data-raw '{
    "email": "test@email.com",
    "password": "cleartext"
}'
    