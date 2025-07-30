curl -v --location POST 'http://localhost:3030/questions' \
    --header 'Authorization: v2.local.SqUMCNM4zA2xsK0kXYItL55lJDB6lEeXc2rXEl7VtckSi3PuoruRK2R2GPUN6o_Qe6UNdvSC_NEi0yJnKDE-tVaUXSE5ndcLiOclMjbL5DrHME3ql7Lj6p1StgYVtP3H7bQy7l7H8GmoGN2u973mIjTg8PyXiR9yKQAq9sLODTN6uskL45k' \
    --header 'Content-Type: application/json' \
    --data-raw '{
        "title": "How can I code better?",
        "content": "Any tips for improving coding skills?",
        "tags": []
    }'