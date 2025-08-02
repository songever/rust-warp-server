curl --location --request POST 'localhost:8080/questions' \
    --header 'Content-Type: application/json' \
    --data-raw '{
    "title": "NEW ass TITLE",
    "content": "OLD CONTENT shit"
}'