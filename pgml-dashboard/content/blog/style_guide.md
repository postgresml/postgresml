## Docs and Blog widgets rendered

This document shows the styles available for PostgresML markdown files. These widgets can be used in Blogs and Docs. 

### Tabs 

Below is a tab widget. 

=== "Tab 1"

information in the first tab

=== "Tab 2"

information in the second tab

===

### Admonitions

!!! note

This is a Note admonition.

!!!

!!! abstract

This is an Abstract admonition.

!!!

!!! info

This is an Info admonition.

!!!

!!! tip

This is a Tip admonition.

!!!

!!! example

This is an Example admonition.

!!!

!!! question

This is a Question admonition.

!!!

!!! success

This is a Success admonition.

!!!

!!! quote

This is a Quote admonition.

!!!

!!! bug	

This is a Bug admonition.

!!!

!!! warning

This is a Warning admonition.

!!!

!!! fail

This is a Fail admonition.

!!!

!!! danger

This is a Danger admonition.

!!!

### Code 

#### Inline Code 

In a sentence you may want to add some code commands `This is some inline code`

#### Fenced Code 

Rendered output of normal markdown fenced code.

```
This is normal markdown fenced code. 
```

#### Code Block 

Below is code placed in a code block with a title and execution time. 

!!! code_block title="Code Title" time="21ms"

``` sql
SELECT pgml.train(
	'Orders Likely To Be Returned something really wide to cause some overflow for testing stuff ',-- name of your model
	'regression', -- objective (regression or classification)
	'public.orders', -- table
	'refunded', -- label (what are we predicting)
	'xgboost' -- algorithm
);

SELECT
	pgml.predict(
		'Orders Likely To Be Returned',
		ARRAY[orders.*]) AS refund_likelihood,
		orders.*
FROM orders
ORDER BY refund_likelyhood DESC
LIMIT 100;
```

!!!


#### Results 

Below is a results placed in a results block with a title. 

!!! results title="Your Results"

``` sql
SELECT pgml.train(
	'Orders Likely To Be Returned', -- name of your model
	'regression', -- objective (regression or classification)
	'public.orders', -- table
	'refunded', -- label (what are we predicting)
	'xgboost' -- algorithm
);

SELECT
	pgml.predict(
		'Orders Likely To Be Returned',
		ARRAY[orders.*]) AS refund_likelihood,
		orders.*
FROM orders
ORDER BY refund_likelyhood DESC
LIMIT 100;
```

This is a footnote about the output.

!!!

Results do not need to be code.  Below is a table in a results block with a title.  

!!! results title="My table title"

| Column            | Type    | Collation | Nullable | Default |
|-------------------|---------|-----------|----------|---------|
| marketplace       | text    |           |          |         |
| customer_id       | text    |           |          |         |
| review_id         | text    |           |          |         |
| product_id        | text    |           |          |         |
| product_parent    | text    |           |          |         |
| product_title     | text    |           |          |         |
| product_category  | text    |           |          |         |
| star_rating       | integer |           |          |         |
| helpful_votes     | integer |           |          |         |
| total_votes       | integer |           |          |         |
| vine              | bigint  |           |          |         |
| verified_purchase | bigint  |           |          |         |
| review_headline   | text    |           |          |         |
| `review_body`     | text    |           |          |         |
| `review_date`     | text    |           |          |         |

!!!


#### Suggestion 

Below is code and results placed in a generic admonition. 

!!! generic

!!! code_block title="Code Title" time="22ms"

``` sql
SELECT pgml.train(
	'Orders Likely To Be Returned', -- name of your model
	'regression', -- objective (regression or classification)
	'public.orders', -- table
	'refunded', -- label (what are we predicting)
	'xgboost' -- algorithm
);

SELECT
	pgml.predict(
		'Orders Likely To Be Returned',
		ARRAY[orders.*]) AS refund_likelihood,
		orders.*
FROM orders
ORDER BY refund_likelyhood DESC
LIMIT 100;
```

!!!

!!! results title="Result Title"

```  sql
SELECT pgml.train(
	'Orders Likely To Be Returned', -- name of your model
	'regression', -- objective (regression or classification)
	'public.orders', -- table
	'refunded', -- label (what are we predicting)
	'xgboost' -- algorithm
);

SELECT
	pgml.predict(
		'Orders Likely To Be Returned',
		ARRAY[orders.*]) AS refund_likelihood,
		orders.*
FROM orders
ORDER BY refund_likelyhood DESC
LIMIT 100;
```

!!!

!!!
