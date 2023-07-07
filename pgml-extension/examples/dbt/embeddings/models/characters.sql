
/*
    Welcome to your first dbt model!
    Did you know that you can also configure models directly within SQL files?
    This will override configurations stated in dbt_project.yml

    Try changing "table" to "view" below
*/

{{ config(materialized='view') }}


select id document_id, char_length(text) character_count from {{ source('test_collection_1', 'documents') }}

/*
    Uncomment the line below to remove records with null `id` values
*/

-- where id is not null
