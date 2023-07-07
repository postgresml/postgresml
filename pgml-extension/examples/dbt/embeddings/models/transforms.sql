{% set table_name = var('task') + '_' + local_md5(var('model_name') + 
					tojson(var('model_parameters',{})) + 
					var('splitter_name') + tojson(var('splitter_parameters',{})))[:5]
%}
{{
    config(
        materialized='incremental',
        unique_key = ['table_name', 'task', 'splitter_id', 'model_id'],
    )
}}
WITH splitter as (
    select id from {{ ref('splitters') }} where name = '{{ var('splitter_name')}}' and parameters @> '{{ tojson(var('splitter_parameters'))}}'::jsonb and parameters <@ '{{ tojson(var('splitter_parameters'))}}'::jsonb
),
model as (
    select id from {{ ref('models') }} where name = '{{ var('model_name')}}' and parameters @> '{{ tojson(var('model_parameters',{}))}}'::jsonb and parameters <@ '{{ tojson(var('model_parameters',{}))}}'::jsonb
),
transform as (
    select '{{ref('embeddings')}}' table_name, '{{ var('task')}}' task, splitter.id splitter_id, model.id model_id from splitter, model
)
select * from transform

