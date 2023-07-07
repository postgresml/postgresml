{{
    config(
        materialized='incremental',
        unique_key='id',

    )
}}

select '{{ var('splitter_name')}}' name,  '{{ tojson(var('splitter_parameters'))}}'::jsonb parameters, md5( '{{var('splitter_name')}}' || '{{tojson(var('splitter_parameters'))}}') id

