curl --location --request POST 'localhost:8080/registration' \
    --header 'Content-Type: application/json' \
    --data-raw '{
    "email": "test@email.com",
    "password": "cleartext"
}'