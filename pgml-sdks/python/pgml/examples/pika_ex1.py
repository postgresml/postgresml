from pypika import Query, Table, AliasedQuery, Order, Field
from pypika.functions import Cast
from pypika.enums import SqlTypes
from pgml.queries import Embed, CosineDistance
from pypika.utils import format_quotes
from psycopg import sql



embeddings_table = Table("embeddings_d2beb7",schema="test_collection_1")
chunks_table = Table("chunks",schema="test_collection_1")
documents_table = Table("documents", schema="test_collection_1")

model = "intfloat/e5-small"
text = "hello world"
query_embed = Query().select(Embed(transformer=model, text=text))
query_cte = AliasedQuery("query_cte")
cte = AliasedQuery("cte")
table_embed = (
    Query()
    .from_(embeddings_table)
    .select(
        "chunk_id",
        CosineDistance(
            embeddings_table.embedding, Cast(query_cte.embedding, "vector")
        ).as_("score"),
    )
    .inner_join(AliasedQuery("query_cte"))
    .cross()
)

query_cte = (
    Query()
    .with_(query_embed, "query_cte")
    .with_(table_embed, "cte")
    .from_("cte")
    .select(cte.score, chunks_table.chunk, documents_table.metadata).orderby(cte.score, order=Order.desc)
    .inner_join(chunks_table)
    .on(chunks_table.id == cte.chunk_id)
    .inner_join(documents_table)
    .on(documents_table.id == chunks_table.document_id)
)

final_query = query_cte.where(documents_table.metadata.contains({"reviews" : 42})).limit(5)
print(final_query)
