DO $$
DECLARE
  curr_id integer := 0;
  batch_size integer:= 2;
  total_records integer:= 10000;
  curr_val text[]; -- Use "text[]" instead of "varchar[]"
  embed_result json; -- Store the result of the pgml.embed function
BEGIN
  LOOP
    --BEGIN RAISE NOTICE 'updating % to %', curr_id, curr_id + batch_size; END;
    SELECT ARRAY(SELECT chunk::text
    FROM squad_collection_benchmark.chunks
    WHERE id BETWEEN curr_id + 1 AND curr_id + batch_size)
    INTO curr_val;

    -- Use the correct syntax to call pgml.embed and store the result
    PERFORM embed FROM pgml.embed('Alibaba-NLP/gte-base-en-v1.5', curr_val);

    curr_id := curr_id + batch_size;
    EXIT WHEN curr_id >= total_records;
  END LOOP;
  
    SELECT ARRAY(SELECT chunk::text
    FROM squad_collection_benchmark.chunks
    WHERE id BETWEEN curr_id-batch_size AND total_records)
    INTO curr_val;

    -- Use the correct syntax to call pgml.embed and store the result
    PERFORM embed FROM pgml.embed('Alibaba-NLP/gte-base-en-v1.5', curr_val);

END;
$$;
