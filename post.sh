curl -v --location POST 'http://localhost:8080/questions' \
    --header 'Authorization: v2.local.pC8UatuQ1vN9AhzU0TWTS0yL1nmd_SNKIcA5mtRfvY48mHMkhMrBNRfpVCmmlmSsQtKqqeh6tpAwS3ZWa7UtFKwVU-5jwIzBNtorqERr3mesVqU7ZthklbmiemB_VIuPLIULRfiKwxIiXoP4iqRFPEQVjz0JToPre84BEMiwW9L4z6dt79Q' \
    --header 'Content-Type: application/json' \
    --data-raw '{
        "title": "How can I code better?",
        "content": "Any tips for improving coding skills?",
        "tags": []
    }'