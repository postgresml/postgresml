WITH query_cte AS (
  SELECT  pgml.embed(transformer => '{{ var('model_name') }}', text => '{{var('query_string','hello world')}}', kwargs => '{{ tojson(var('model_parameters',{})) }}') AS query_embedding
), 
cte AS (  SELECT chunk_id, 1 - ({{ref("embeddings")}}.embedding <=> query_cte.query_embedding :: float8[] :: vector ) AS score 
  FROM {{ref("embeddings")}} CROSS JOIN query_cte 
) 
SELECT  '{{var('query_string','hello world')}}' query, cte.score,  chunks.chunk,  documents.metadata FROM  cte 
  INNER JOIN  {{ref("chunks")}} chunks ON chunks.id = cte.chunk_id 
  INNER JOIN {{source("test_collection_1","documents")}} documents ON documents.id = chunks.document_id 
  ORDER BY cte.score DESC  LIMIT {{var('limit',5)}}
