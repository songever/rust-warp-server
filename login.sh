curl --location --request POST 'http://localhost:3030/login' \
    --header 'Content-Type: application/json' \
    --data-raw '{
    "email": "test@email.com",
    "password": "cleartext"
}'
    