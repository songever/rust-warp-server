curl --location --request POST 'localhost:3030/registration' \
    --header 'Content-Type: application/json' \
    --data-raw '{
    "email": "test@email.com",
    "password": "cleartext"
}'