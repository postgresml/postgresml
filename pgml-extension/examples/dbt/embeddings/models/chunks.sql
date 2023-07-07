{{
    config(
        materialized='incremental',
        unique_key=['document_id', 'splitter_id', 'chunk']
    )
}}
WITH document as (
	select id, text
	from {{ source('test_collection_1', 'documents') }}
),
splitter as (
	select id, name, parameters
	from {{ ref('splitters') }}
	where name = '{{ var('splitter_name')}}' and parameters @> '{{ tojson(var('splitter_parameters'))}}'::jsonb and parameters <@ '{{ tojson(var('splitter_parameters'))}}'::jsonb
),
chunks as (
    select document.id document_id, splitter.id splitter_id, chunk_index, 
    chunk, md5(document.id::text || splitter.id || chunk_index::text) id, CURRENT_TIMESTAMP created_at
    from document, splitter, pgml.chunk(splitter.name, document.text, splitter.parameters)
)
select * from chunks

{% if is_incremental() %}
    where document_id not in (select document_id from {{ this }})
{% endif %}