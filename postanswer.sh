curl --location --request POST 'localhost:3030/questions' \
    --header 'Content-Type: application/x-www-form-urlencoded' \
    --data-urlencode 'id=1' \
    --data-urlencode 'title=First question' \
    --data-urlencode 'content=This is the question I had.'