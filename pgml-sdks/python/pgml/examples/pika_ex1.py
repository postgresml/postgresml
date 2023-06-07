from pypika import Query, Table, AliasedQuery, Order, Field
from pypika.functions import Cast
from pgml.queries import Embed, CosineDistance

embeddings_table = Table("test_collection_1.embeddings_d2beb7")
chunks_table = Table("test_collection_1.chunks")
documents_table = Table("test_collection_1.documents")

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
    .on(Field(1) == Field(1))
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
print(query_cte.get_sql().replace('"', ""))

