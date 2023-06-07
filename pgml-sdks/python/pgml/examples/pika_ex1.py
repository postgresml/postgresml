from pypika import Query, Table, AliasedQuery, Order
from pgml.queries import Embed, CosineDistance

embeddings_table = Table("embeddings_table")
chunks_table = Table("chunks_table")
documents_table = Table("documents_table")

query_embed = Query().select(Embed(transformer="instructxl", text="hello"))
print(query_embed)

query_table = AliasedQuery("query_cte")
query_cte = Query().with_(query_embed, "query_cte").from_(query_table).select('*')
print(query_cte)

table_embed = (
    Query()
    .with_(
        Query()
        .from_(embeddings_table)
        .cross_join(query_table)
        .on(embeddings_table.embedding == query_table.embedding).select('*'),
        "cte",
    )
    .from_(AliasedQuery("cte"))
    .select("score")
)
print(table_embed)
