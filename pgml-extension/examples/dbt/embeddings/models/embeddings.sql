{% set alias_name = var('task') + '_' + local_md5(var('model_name') + 
					tojson(var('model_parameters',{})) + 
					var('splitter_name') + tojson(var('splitter_parameters',{})))[:5]
%}
{{
    config(
        materialized='incremental',
		unique_key=['document_id', 'splitter_id', 'chunk_index'],
		alias = alias_name
    )
}}
WITH chunk as (
	select document_id, splitter_id, chunk_index, chunk, id
	from {{ ref('chunks') }}
),
model as (
	select id, name, parameters
	from {{  ref('models') }}
	where name = '{{ var('model_name')}}' 
	and parameters @> '{{ tojson(var('model_parameters',{}))}}'::jsonb 
	and parameters <@ '{{ tojson(var('model_parameters',{}))}}'::jsonb
),
embeddings as (
    select chunk.id chunk_id, chunk.document_id document_id, chunk.splitter_id splitter_id, 
		   chunk.chunk_index chunk_index, cast(embed as vector) embedding 
	from chunk, model, pgml.embed(model.name, chunk.chunk, model.parameters)
)
select * from embeddings

{% if is_incremental() %}
    where chunk_id not in (select chunk_id from {{ this }})
{% endif %}
