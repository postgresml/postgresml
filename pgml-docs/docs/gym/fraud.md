Risk mitigation
===============

Most organizations have some risks that may be minimized using machine learning, by predicting the likelihood of negative outcomes before they happen. As long as you're able to track the information leading up to the unfortunate events, there are many different machine learning algorithms that can tease out the correlations across multiple variables.

One example of risk ecommerce companies face is credit card fraud with stolen credit cards. When the owner of the card sees charges they never authorized on their monthly statement, they'll report these to the credit card company, and the charges will be reversed. The ecommerce company will lose the merchandise as well as shipping charges and labor costs. If a company receives too many chargebacks, not only will they incur expensive losses, but the credit card processors may remove them from the platorm, so it's important they have some certainty about the owner of the cards identity and legitimate interests.

In this notebook, we'll demonstrate how a simplified ecommerce application might track customer orders, and use machine learning to detect chargeback risks in real time during checkout. The most important step in building any Machine Learning model is understanding the data. Knowing it's structure, application use, and the full meaning for the business will allow us to create meaningful features and labels for our models. In this notebook, we've included a fair bit of SQL to implement logic that would normally be written at the application layer to help you build an intuition about the domain. 

Ecommerce Application Data Model
--------------------------------
We'll build out a simple eccomerce schema, and populate it with some example data. First, our store needs some products to sell. Products have a name, their price, and other metadata, like whether or not they are perishable goods.

```sql
CREATE TABLE products (
  name TEXT PRIMARY KEY,
  price MONEY,
  perishable BOOLEAN
);
```

```sql
INSERT INTO PRODUCTS (name, price, perishable) 
VALUES
  ('1oz gold bar', '$1999.99', false),
  ('a tale of 2 cities', '$19.99', false),
  ('head of lettuce', '$1.99', true);
```

Now that we're in business, our first customer has shown up, named Alice. Alice is a chef that owns a salad shop, so she is going to create an order for 1000 "head of lettuce".

```sql
CREATE TABLE orders (
  id BIGSERIAL PRIMARY KEY,
  customer_name TEXT
);

CREATE TABLE line_items (
  id BIGSERIAL PRIMARY KEY,
  order_id BIGINT,
  product_name TEXT,
  count INTEGER
);
```

```sql
INSERT INTO orders (customer_name) VALUES ('Alice');

INSERT INTO line_items (
  order_id, 
  product_name, 
  count
) VALUES (
  -- a query to find Alice's most recent order
  (SELECT max(id) FROM orders WHERE customer_name = 'Alice'),
  'head of lettuce',
  1000
);
```

:note:
These inline subselects like `SELECT max(id) FROM orders WHERE customer_name = 'Alice'` are a little weird. Typically this ID would be passed in from the application layer, instead of being retrieved during the INSERT statement itself. So anyway...

We record her payment in full via credit card.

```sql
CREATE TABLE payments (
  id BIGSERIAL PRIMARY KEY,
  order_id BIGINT,
  amount MONEY
);
```

```sql
INSERT INTO payments (order_id, amount) 
-- a query to compute the full amount of Alice's most recent order
SELECT order_id, sum(count * price) AS amount
FROM orders
JOIN line_items ON line_items.order_id = orders.id
JOIN products ON products.name = line_items.product_name
WHERE orders.id = (SELECT max(id) AS order_id FROM orders WHERE customer_name = 'Alice')
GROUP BY 1;
```

Time to celebrate! Alice has paid in full for our first order, and business is good.

:celebrate gif:

Now, along comes Bob "the bad guy" who places an order for a gold bar.

```sql
INSERT INTO orders (customer_name) VALUES ('Bob');
INSERT INTO line_items (
  order_id, 
  product_name, 
  count
) VALUES (
  (SELECT max(id) FROM orders WHERE customer_name = 'Bob'),
  '1oz gold bar',
  1
);
```

Unfortunately, Bob makes his payment with a stolen credit card, but we don't know that yet.

```sql
INSERT INTO payments (order_id, amount) 
-- a query to compute the full amount of Bob's most recent order
SELECT order_id, sum(count * price) AS amount
FROM orders
JOIN line_items ON line_items.order_id = orders.id
JOIN products ON products.name = line_items.product_name
WHERE orders.id = (SELECT max(id) AS order_id FROM orders WHERE customer_name = 'Bob')
GROUP BY 1;
```

At the end of the month, the credit card company lets us know about the chargeback from the real card owner, so we record that in our database.

```sql
CREATE TABLE chargebacks (
  id BIGSERIAL PRIMARY KEY,
  payment_id BIGINT
)
```

```sql
INSERT INTO chargebacks (payment_id) 
SELECT max(payments.id) AS payment_id
FROM payments 
JOIN orders ON payments.order_id = orders.id 
WHERE customer_name = 'Bob';
```

If you've made it this far, you've won half the machine learning battle. We have 2 training data examples that are perfect for "supervised" machine learning. The chargebacks acts as the ground truth to inform the machine learning algorithm of whether or not an order is fraudulent. The chargebacks act as "labels", a.k.a "targets" or "Y-values" for the data.

We can construct a query that provides a summary view of our orders, including the fraudulent label:

```sql
CREATE VIEW orders_summaries AS
SELECT 
  orders.id AS order_id, 
  orders.customer_name,
  payments.amount AS total, 
  ARRAY_AGG(products.name) AS product_names,
  CASE WHEN chargebacks.id IS NOT NULL 
    THEN true 
    ELSE false 
  END AS fraudulent
FROM orders
LEFT JOIN payments ON payments.order_id = orders.id
LEFT JOIN chargebacks ON chargebacks.payment_id = payments.id
LEFT JOIN line_items ON line_items.order_id = orders.id
LEFT JOIN products ON products.name = line_items.product_name
GROUP BY 1, 2, 3, 5
ORDER BY orders.id;
```  

```sql
SELECT * FROM orders_summaries;
```

It's intuitive that thieves will be more attracted to gold bars, than a head of lettuce because the resell value is better. Perishable goods are more difficult to move on the black market. A good feature for our model would be the percentage of the order that is perishable. We can construct a query to return this feature for each order, along with the chargeback label. 

```sql
CREATE VIEW fraud_samples AS
SELECT 
  SUM(CASE WHEN products.perishable THEN (count * price) ELSE '$0.0' END) / SUM(payments.amount) AS perishable_percentage, 
  CASE WHEN chargebacks.id IS NOT NULL 
    THEN true 
    ELSE false 
  END AS fraudulent
FROM orders
LEFT JOIN payments ON payments.order_id = orders.id
LEFT JOIN chargebacks ON chargebacks.payment_id = payments.id
LEFT JOIN line_items ON line_items.order_id = orders.id
LEFT JOIN products ON products.name = line_items.product_name
GROUP BY orders.id, chargebacks.id
ORDER BY orders.id;
```

```sql
SELECT * FROM fraud_samples;
```

Training a model
----------------

This is a great training set for a machine learning model. We've found a feature that perfectly correlates with the label. This will allow models to generalize the fact based on our experience. Perishable orders are less likely to result in a chargeback. Now that we have a `VIEW` of this data, we can train a "classification" model to classify the features as fraud or not.

```sql
SELECT * FROM pgml.train(
  project_name => 'Our Fraud Model', -- a friendly name we'll use to identify this machine learning project
  task => 'classification', -- we want to classify into true or false
  relation_name => 'fraud_samples', -- our view of the data
  y_column_name => 'fraudulent' -- the "labels"
);
```

Oops. We're going to get an error:

```
ERROR:  ValueError: This solver needs samples of at least 2 classes in the data, but the data contains only one class: False
```

Wait a second, we know there is both a True and a False label, because we have an example of both a fraudulent and legit order. What gives? This is a glimpse into how PostgresML works inside the black box. It splits the sample data into 2 sets. One is used for training the model as we expected, and the other is used to test the model's predictions against the remaining known labels. This way we can see how well the model generalizes. In this case, since there are only 2 data samples, 1 is used for training (the False label) and 1 is used for testing (the True label). Now we can understand there isn't enough data to actually train and test. We need to generate a couple more examples so we have enough to train and test.

```sql
INSERT INTO orders (customer_name) VALUES ('Carol');
INSERT INTO line_items (
  order_id, 
  product_name, 
  count
) VALUES (
  (SELECT max(id) FROM orders WHERE customer_name = 'Carol'),
  'a tale of 2 cities',
  10
);
```

Carol has bought a book, and now will legitimately pay in full.

```sql
INSERT INTO payments (order_id, amount) 
-- a query to compute the full amount of Bob's most recent order
SELECT order_id, sum(count * price) AS amount
FROM orders
JOIN line_items ON line_items.order_id = orders.id
JOIN products ON products.name = line_items.product_name
WHERE orders.id = (SELECT max(id) AS order_id FROM orders WHERE customer_name = 'Carol')
GROUP BY 1;
```

And now Dan (another fraudster) shows up to steal more books:

```sql
INSERT INTO orders (customer_name) VALUES ('Dan');
INSERT INTO line_items (
  order_id, 
  product_name, 
  count
) VALUES (
  (SELECT max(id) FROM orders WHERE customer_name = 'Dan'),
  'a tale of 2 cities',
  50
);
```

```sql
INSERT INTO payments (order_id, amount) 
-- a query to compute the full amount of Bob's most recent order
SELECT order_id, sum(count * price) AS amount
FROM orders
JOIN line_items ON line_items.order_id = orders.id
JOIN products ON products.name = line_items.product_name
WHERE orders.id = (SELECT max(id) AS order_id FROM orders WHERE customer_name = 'Dan')
GROUP BY 1;
```

```sql
INSERT INTO chargebacks (payment_id) 
SELECT max(payments.id) AS payment_id
FROM payments 
JOIN orders ON payments.order_id = orders.id 
WHERE customer_name = 'Dan';
```

And now we can try to train the model again.

```sql
SELECT * FROM pgml.train(
  project_name => 'Our Fraud Classification', -- a friendly name we'll use to identify this machine learning project
  task => 'classification', -- we want to classify into true or false
  relation_name => 'fraud_samples', -- our view of the data
  y_column_name => 'fraudulent', -- the "labels"
  test_size => 0.5 -- use half the data for testing rather than the default test size of 25%
);
```

Success!

```sql
  project_name   |      task      | algorithm_name |  status
-----------------+----------------+----------------+----------
 Our Fraud Model | classification | linear         | deployed
(1 row)
```

We can demonstrate basic usage of the model
```sql
SELECT perishable_percentage, fraudulent, pgml.predict('Our Fraud Classification', ARRAY[perishable_percentage]) AS predict_fraud FROM fraud_samples;
```

```sql
 perishable_percentage | fraudulent | predict_fraud
-----------------------+------------+---------------
                     1 | f          |             0
                     0 | t          |             1
                     0 | f          |             1
                     0 | t          |             1
(4 rows)
```

Uh oh, the model was trained on a perfectly small dataset. It learned that unless the order is perishable goods, it's going to predict fraud 100% of the time, but our test data shows that's not 100% true. Let's generate some samples to further explore our model.

```sql
WITH exploration_samples AS (
  SELECT generate_series(0, 1, 0.1) AS perishable_percentage
)
SELECT perishable_percentage, pgml.predict('Our Fraud Classification', ARRAY[perishable_percentage]) AS predict_fraud FROM exploration_samples;
```

```sql
 perishable_percentage | predict_fraud
-----------------------+---------------
                     0 |             1
                   0.1 |             1
                   0.2 |             1
                   0.3 |             1
                   0.4 |             1
                   0.5 |             0
                   0.6 |             0
                   0.7 |             0
                   0.8 |             0
                   0.9 |             0
                   1.0 |             0
(11 rows)
```

The default model is a linear regression, so it has learned from the training half of the data that high amounts of perishible goods make for safe orders.

Adding more features
--------------------
We need to add some more features to create a better model. Instead of just using the perishable percentage, we can use dollar values as our features, since we know criminals want to steal large amounts more than small amounts.

```sql
DROP VIEW fraud_samples;
CREATE VIEW fraud_samples AS
SELECT 
  SUM(CASE WHEN products.perishable THEN (count * price)::NUMERIC ELSE 0.0 END) AS perishable_amount, 
  SUM(CASE WHEN NOT products.perishable THEN (count * price)::NUMERIC ELSE 0.0 END) AS non_perishable_amount, 
  CASE WHEN chargebacks.id IS NOT NULL 
    THEN true 
    ELSE false 
  END AS fraudulent
FROM orders
LEFT JOIN payments ON payments.order_id = orders.id
LEFT JOIN chargebacks ON chargebacks.payment_id = payments.id
LEFT JOIN line_items ON line_items.order_id = orders.id
LEFT JOIN products ON products.name = line_items.product_name
GROUP BY orders.id, chargebacks.id
ORDER BY orders.id;
```

And now we retrain a new version of the model, by calling train with the same parameters again.

```sql
SELECT * FROM pgml.train(
  project_name => 'Our Fraud Classification', -- a friendly name we'll use to identify this machine learning project
  task => 'classification', -- we want to classify into true or false
  relation_name => 'fraud_samples', -- our view of the data
  y_column_name => 'fraudulent', -- the "labels"
  test_size => 0.5 -- use half the data for testing rather than the default test size of 25%
);
```

And then we can deploy this most recent version:

```sql
SELECT * FROM pgml.deploy('Our Fraud Classification', 'most_recent');
```

And view the input/outputs of this model based on our data:

```sql
SELECT perishable_amount, non_perishable_amount, fraudulent, pgml.predict('Our Fraud Classification', ARRAY[perishable_amount, non_perishable_amount]) AS predict_fraud FROM fraud_samples;
```

This is the basic development cycle for a model. 
  
  1) Add new features
  2) Retrain the new model
  3) Analyze performance

Even with a toy schema like this, it's possible to create many different features over the data. Examples of other statistcal features we could add:

- How many orders the customer has previously made without chargebacks
- What has their total spend been so far
- How old is this account
- What is their average order size
- How frequently do they typically order
- Do they typically by perishible or non perishable goods

We can create additional `VIEW`s, Sub `SELECT`s or [Common Table Expressions](https://www.postgresql.org/docs/current/queries-with.html) to standardize these features across models or reports.

:note:
Sub SELECTs may be preferable to Common Table Expressions for generating complex features, because CTEs create an optimization gate that prevents the query planner from pushing predicates down, which will hurt performance if you intend to reuse this VIEW during inference for a single row.

:tip:
If you're querying a particular view frequently that is expensive to produce, you may consider using a `MATERILIZE VIEW`, to cache the results.

=== SUB SELECT

```sql
LEFT JOIN (
  SELECT DISTINCT orders.customer_name, COUNT(*) AS previous_orders FROM ORDERS;
) AS customer_stats
  ON customer_stats.customer_name = orders.customer_name
```

=== VIEW

```sql
CREATE VIEW customer_stats AS 
SELECT DISTINCT orders.customer_name, COUNT(*) AS previous_orders FROM ORDERS;

...
LEFT JOIN customer_stats ON customer_stats.customer_name = orders.customer_name
```

=== CTE

```sql
WITH customer_stats AS ( 
  SELECT DISTINCT orders.customer_name, COUNT(*) AS previous_orders FROM ORDERS;
)

...
LEFT JOIN customer_stats ON customer_stats.customer_name = orders.customer_name
```

When you're out of ideas for features that might help the model distinguish orders that are likely to result in chargebacks, you may want to start testing different algorithms to see how the performance changes. PostgresML makes algorithm selection as easy as passing an additional parameter to `pgml.train`. You may want to test them all just to see, but `xgboost` typically gives excellent performance in terms of both accuracy and latency.

```sql
SELECT * FROM pgml.train(
  project_name => 'Our Fraud Classification', -- a friendly name we'll use to identify this machine learning project
  task => 'classification', -- we want to classify into true or false
  relation_name => 'fraud_samples', -- our view of the data
  y_column_name => 'fraudulent', -- the "labels"
  algorithm_name => 'xgboost', -- tree based models like xgboost are often the best performers for tabular data at scale
  test_size => 0.5 -- use half the data for testing rather than the default test size of 25%
);
```

Using Regression instead of Classificaiton
------------------------------------------

So far we've been training a classifier that gives us a binary 0 or 1 output to indicate likely fraud or not. If we'd like to refine our application response to the models predictions in a more nuanced way, say high/medium/low risk instead of binary, we can use "regression" instead of "classification" to predict a likelihood between 0 and 1, instead of binary.

```sql
SELECT * FROM pgml.train(
  project_name => 'Our Fraud Regression', -- a friendly name we'll use to identify this machine learning project
  task => 'regression', -- predict the likelihood
  relation_name => 'fraud_samples', -- our view of the data
  y_column_name => 'fraudulent', -- the "labels"
  algorithm_name => 'xgboost', 
  test_size => 0.5 -- use half the data for testing rather than the default test size of 25%
);
```

At this point, the primary limitation of our model is the amount of data, the number of examples we have to train it on. Luckily, as time marches on, and data accumulates in the database, we can simply retrain this model with additional calls to `pgml.train` and watch it adjust as new information becomes available.