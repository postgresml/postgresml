{{
    config(
        materialized='incremental',
        unique_key='id',

    )
}}

select '{{ var('model_name')}}' name,  '{{ tojson(var('model_parameters',{}))}}'::jsonb parameters, md5( '{{var('model_name')}}' || '{{tojson(var('model_parameters',{}))}}') id

