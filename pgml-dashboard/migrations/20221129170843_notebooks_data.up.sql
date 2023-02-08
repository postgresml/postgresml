
INSERT INTO pgml.notebooks VALUES (0, 'Tutorial 0: 🎉 Welcome to PostgresML!', '2022-08-19 18:47:08.93719', '2022-08-19 18:47:08.93719');
INSERT INTO pgml.notebooks VALUES (1, 'Tutorial 1: ⏱️ Real Time Fraud Detection', '2022-08-15 15:26:18.428227', '2022-08-15 15:26:18.428241');
INSERT INTO pgml.notebooks VALUES (2, 'Tutorial 2: ⚕️ Tumor Detection w/ Binary Classification', '2022-08-19 16:10:23.120983', '2022-08-19 16:10:23.120996');
INSERT INTO pgml.notebooks VALUES (3, 'Tutorial 3: ✍️ Handwritten Digit Image Classification', '2022-08-20 09:46:40.856497', '2022-08-20 09:46:40.856511');
INSERT INTO pgml.notebooks VALUES (4, 'Tutorial 4: 🍭 Diabetes Progression w/ Regression', '2022-08-19 19:18:14.608456', '2022-08-19 19:18:14.608474');
INSERT INTO pgml.notebooks VALUES (5, 'Tutorial 5: 🤗 Deep Learning w/ Transformers', '2022-08-20 09:47:47.830932', '2022-08-20 09:47:47.830946');
INSERT INTO pgml.notebooks VALUES (6, 'Tutorial 6: ↗️ Working w/ Embeddings', '2022-08-20 09:48:16.252016', '2022-08-20 09:48:16.252029');
INSERT INTO pgml.notebooks VALUES (7, 'Tutorial 7: 📒 Managing Model Deployments', '2022-08-20 09:48:40.044312', '2022-08-20 09:48:40.044325');
INSERT INTO pgml.notebooks VALUES (8, 'Tutorial 8: 💻 Working w/ the Internal Schema of PostgresML', '2022-08-20 09:49:41.363292', '2022-08-20 09:49:41.363306');
INSERT INTO pgml.notebooks VALUES (9, 'Tutorial 9: 🏁 Launch PostgresML w/ Your Production Stack', '2022-08-23 19:36:49.286982', '2022-08-23 19:36:49.286998');

SELECT pg_catalog.setval('pgml.notebooks_id_seq', (SELECT MAX(id) + 1 FROM pgml.notebooks), true);

--
-- PostgreSQL database dump
--

--
-- Data for Name: notebook_cells; Type: TABLE DATA; Schema:  Owner: lev
--

INSERT INTO pgml.notebook_cells VALUES (1, 0, 1, 1, 1, '## Welcome!

You''re set up and running on PostgresML! This is an end-to-end system for training and deploying real time machine learning models. It handles data versioning, model training and validation, and safe production release. This dashboard web app will give you an overview of what''s happening in the system and also helps build and deploy projects. You can use notebooks like this one to interact with your database in real time and organize your SQL while documenting your code.


### Notebooks

These notebooks are similar to Jupyter Notebooks, which you might be familiar with already. On the bottom of the page, you will find a text editor which is used to create new cells. Each cell can contain either Markdown which is just text really, and SQL which will be executed directly by your Postgres database server.

Each cell has a little menu in the top right corner, allowing you to (re)run it (if it''s SQL), edit it, and delete it.


Let me give you an example. The next cell (cell #2) will be a SQL cell which will execute a simple query. Go ahead and click the next "Play" button now.', '<article class="markdown-body"><h2>Welcome!</h2>
<p>You''re set up and running on PostgresML! This is an end-to-end system for training and deploying real time machine learning models. It handles data versioning, model training and validation, and safe production release. This dashboard web app will give you an overview of what''s happening in the system and also helps build and deploy projects. You can use notebooks like this one to interact with your database in real time and organize your SQL while documenting your code.</p>
<h3>Notebooks</h3>
<p>These notebooks are similar to Jupyter Notebooks, which you might be familiar with already. On the bottom of the page, you will find a text editor which is used to create new cells. Each cell can contain either Markdown which is just text really, and SQL which will be executed directly by your Postgres database server.</p>
<p>Each cell has a little menu in the top right corner, allowing you to (re)run it (if it''s SQL), edit it, and delete it.</p>
<p>Let me give you an example. The next cell (cell #2) will be a SQL cell which will execute a simple query. Go ahead and click the next "Play" button now.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (2, 0, 3, 2, 1, 'SELECT random();', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (4, 0, 1, 4, 1, 'We just asked Postgres to return a random number. Pretty simple query, but it demonstrates the notebook functionality pretty well. You can see that the result of `random()` is a float between 0 and 1. On the bottom right corner, you can see that it took `0:00:00.000654` or 0 hours, 0 minutes, 0 seconds and only 654ns, or 0.6ms. This run time is good to keep an eye on. It will help build an intuition for how fast Postgres really is, and how certain operations scale as the data grows.

Try rerunning the cell again by clicking the "Play" button in the top right corner. You''ll see that the random number will change. Rerunning is a real time operation and Postgres will give you a different random number every time (otherwise it wouldn''t be random).

#### Editing a cell
You can edit a cell at any time, including SQL cells which will then run the new query immediately.

#### Deleting a cell
Deleting a cell is pretty easy: just click on the delete button in the top right corner. You can undo the delete if you so desire; we wouldn''t want you to lose your work because of an accidental click.

#### Shortcuts
The text editor supports the following helpful shortcuts:


|  Shortcut |             Description               
|-----------| --------------------------------------
| `Cmd-/` or `Ctrl-/` | Comment out SQL code.      |
| `Cmd-Enter` or `Ctrl-Enter` | Save/create a cell.|
| `Shift-Enter` | Run the currently selected cell. |

By the way, this was a Markdown table, you can make those here as well.
      
### Thank you
Thank you for trying out PostgresML! We hope you enjoy your time here and have fun learning about machine learning, in the comfort of your favorite database.

You may want to check out the rest of [the tutorials](../) or dive straight in with a notebook to test [Tutorial 1: ⏱️ Real Time Fraud Detection](../1/)', '<article class="markdown-body"><p>We just asked Postgres to return a random number. Pretty simple query, but it demonstrates the notebook functionality pretty well. You can see that the result of <code>random()</code> is a float between 0 and 1. On the bottom right corner, you can see that it took <code>0:00:00.000654</code> or 0 hours, 0 minutes, 0 seconds and only 654ns, or 0.6ms. This run time is good to keep an eye on. It will help build an intuition for how fast Postgres really is, and how certain operations scale as the data grows.</p>
<p>Try rerunning the cell again by clicking the "Play" button in the top right corner. You''ll see that the random number will change. Rerunning is a real time operation and Postgres will give you a different random number every time (otherwise it wouldn''t be random).</p>
<h4>Editing a cell</h4>
<p>You can edit a cell at any time, including SQL cells which will then run the new query immediately.</p>
<h4>Deleting a cell</h4>
<p>Deleting a cell is pretty easy: just click on the delete button in the top right corner. You can undo the delete if you so desire; we wouldn''t want you to lose your work because of an accidental click.</p>
<h4>Shortcuts</h4>
<p>The text editor supports the following helpful shortcuts:</p>
<table>
<thead>
<tr>
<th>Shortcut</th>
<th>Description</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>Cmd-/</code> or <code>Ctrl-/</code></td>
<td>Comment out SQL code.</td>
</tr>
<tr>
<td><code>Cmd-Enter</code> or <code>Ctrl-Enter</code></td>
<td>Save/create a cell.</td>
</tr>
<tr>
<td><code>Shift-Enter</code></td>
<td>Run the currently selected cell.</td>
</tr>
</tbody>
</table>
<p>By the way, this was a Markdown table, you can make those here as well.</p>
<h3>Thank you</h3>
<p>Thank you for trying out PostgresML! We hope you enjoy your time here and have fun learning about machine learning, in the comfort of your favorite database.</p>
<p>You may want to check out the rest of <a href="../">the tutorials</a> or dive straight in with a notebook to test <a href="../1/">Tutorial 1: ⏱️ Real Time Fraud Detection</a></p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (5, 0, 3, 5, 1, 'SELECT ''Have a nice day!'' AS greeting;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (6, 1, 1, 1, 1, 'Introduction
------------

Most organizations have some risks that may be minimized using machine learning, by predicting the likelihood of negative outcomes before they happen. As long as you''re able to track the information leading up to the unfortunate events, there are many different machine learning algorithms that can tease out the correlations across multiple variables.

One example of risk ecommerce companies face is credit card fraud with stolen credit cards. When the owner of the card sees charges they never authorized on their monthly statement, they''ll report these to the credit card company, and the charges will be reversed. The ecommerce company will lose the merchandise as well as shipping charges and labor costs. If a company receives too many chargebacks, not only will they incur expensive losses, but the credit card processors may remove them from the platform, so it''s important they have some certainty about the owner of the cards identity and legitimate interests.

In this notebook, we''ll demonstrate how a simplified ecommerce application might track customer orders, and use machine learning to detect chargeback risks in real time during checkout. The most important step in building any Machine Learning model is understanding the data. Knowing its structure, application use, and the full meaning for the business will allow us to create meaningful features and labels for our models. In this notebook, we''ve included a fair bit of SQL to implement logic that would normally be written at the application layer to help you build an intuition about the domain.

**Contents**

- Part 1: Ecommerce Application Data Model
- Part 2: Structuring the Training Data
- Part 3: Training a Model
- Part 4: Adding More Features
- Part 5: Upgrading the Machine Learning Algorithm

Part 1: Ecommerce Application Data Model
--------------------------------
We''ll build out a simple ecommerce schema, and populate it with some example data. First, our store needs some products to sell. Products have a name, their price, and other metadata, like whether or not they are perishable goods.', '<article class="markdown-body"><h2>Introduction</h2>
<p>Most organizations have some risks that may be minimized using machine learning, by predicting the likelihood of negative outcomes before they happen. As long as you''re able to track the information leading up to the unfortunate events, there are many different machine learning algorithms that can tease out the correlations across multiple variables.</p>
<p>One example of risk ecommerce companies face is credit card fraud with stolen credit cards. When the owner of the card sees charges they never authorized on their monthly statement, they''ll report these to the credit card company, and the charges will be reversed. The ecommerce company will lose the merchandise as well as shipping charges and labor costs. If a company receives too many chargebacks, not only will they incur expensive losses, but the credit card processors may remove them from the platform, so it''s important they have some certainty about the owner of the cards identity and legitimate interests.</p>
<p>In this notebook, we''ll demonstrate how a simplified ecommerce application might track customer orders, and use machine learning to detect chargeback risks in real time during checkout. The most important step in building any Machine Learning model is understanding the data. Knowing its structure, application use, and the full meaning for the business will allow us to create meaningful features and labels for our models. In this notebook, we''ve included a fair bit of SQL to implement logic that would normally be written at the application layer to help you build an intuition about the domain.</p>
<p><strong>Contents</strong></p>
<ul>
<li>Part 1: Ecommerce Application Data Model</li>
<li>Part 2: Structuring the Training Data</li>
<li>Part 3: Training a Model</li>
<li>Part 4: Adding More Features</li>
<li>Part 5: Upgrading the Machine Learning Algorithm</li>
</ul>
<h2>Part 1: Ecommerce Application Data Model</h2>
<p>We''ll build out a simple ecommerce schema, and populate it with some example data. First, our store needs some products to sell. Products have a name, their price, and other metadata, like whether or not they are perishable goods.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (7, 1, 3, 2, 1, 'CREATE TABLE products (
  emoji TEXT PRIMARY KEY,
  name TEXT,
  price MONEY,
  perishable BOOLEAN
);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (8, 1, 3, 3, 1, 'INSERT INTO PRODUCTS (emoji, name, price, perishable) 
VALUES
  (''💰'', ''1oz gold bar'', ''$1999.99'', false),
  (''📕'', ''a tale of 2 cities'', ''$19.99'', false),
  (''🥬'', ''head of lettuce'', ''$1.99'', true)
RETURNING *;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (9, 1, 1, 4, 1, 'Now that we''re in business, our first customer has shown up, named Alice. She is a chef that owns a salad shop, so she is going to create an order for 1,000 🥬 `head of lettuce`.

Our ecommerce site will record `orders` and their `line_items` in our database with the following schema.', '<article class="markdown-body"><p>Now that we''re in business, our first customer has shown up, named Alice. She is a chef that owns a salad shop, so she is going to create an order for 1,000 🥬 <code>head of lettuce</code>.</p>
<p>Our ecommerce site will record <code>orders</code> and their <code>line_items</code> in our database with the following schema.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (10, 1, 3, 5, 1, 'CREATE TABLE orders (
  id BIGSERIAL PRIMARY KEY,
  customer_name TEXT
);

CREATE TABLE line_items (
  id BIGSERIAL PRIMARY KEY,
  order_id BIGINT,
  product_emoji TEXT,
  count INTEGER
);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (11, 1, 1, 6, 1, 'Now that we have created the schema, we can record Alice''s order', '<article class="markdown-body"><p>Now that we have created the schema, we can record Alice''s order</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (12, 1, 3, 7, 1, 'INSERT INTO orders (customer_name) VALUES (''Alice'');

INSERT INTO line_items (order_id, product_emoji, count) 
VALUES (
  -- a query to find Alice''s most recent order
  (SELECT max(id) FROM orders WHERE customer_name = ''Alice''),
  ''🥬'',
  1000
)
RETURNING *;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (13, 1, 1, 8, 1, '🔎 That inline subquery in #7 is a little weird.

```sql
-- a query to find Alice''s most recent order
(SELECT max(id) FROM orders WHERE customer_name = ''Alice'')
```

Typically this ID would be passed in from the application layer, instead of being retrieved during the INSERT statement itself. But anyway... 

Next, we''ll record her payment in full via credit card in our `payments` table.', '<article class="markdown-body"><p>🔎 That inline subquery in #7 is a little weird.</p>
<pre><code class="language-sql">-- a query to find Alice''s most recent order
(SELECT max(id) FROM orders WHERE customer_name = ''Alice'')
</code></pre>
<p>Typically this ID would be passed in from the application layer, instead of being retrieved during the INSERT statement itself. But anyway... </p>
<p>Next, we''ll record her payment in full via credit card in our <code>payments</code> table.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (14, 1, 3, 9, 1, 'CREATE TABLE payments (
  id BIGSERIAL PRIMARY KEY,
  order_id BIGINT,
  amount MONEY
);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (15, 1, 1, 10, 1, 'We''ll be doing a little bit of heavy lifting in the next query to calculate her payment total on the fly.', '<article class="markdown-body"><p>We''ll be doing a little bit of heavy lifting in the next query to calculate her payment total on the fly.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (100, 8, 1, 4, 1, '## Projects

Projects are an artifact of calls to `pgml.train`.', '<article class="markdown-body"><h2>Projects</h2>
<p>Projects are an artifact of calls to <code>pgml.train</code>.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (16, 1, 3, 11, 1, 'INSERT INTO payments (order_id, amount) 
-- a query to compute the full amount of Alice''s most recent order
SELECT order_id, SUM(count * price) AS amount
FROM orders
JOIN line_items ON line_items.order_id = orders.id
JOIN products ON products.emoji = line_items.product_emoji
WHERE orders.id = (SELECT max(id) AS order_id FROM orders WHERE customer_name = ''Alice'')
GROUP BY 1
RETURNING *;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (17, 1, 1, 12, 1, '🎉 Time to celebrate! Alice has paid in full for our first order, and business is good.


Now, along comes Bob "the bad guy" who places an order for a 💰 1oz gold bar.', '<article class="markdown-body"><p>🎉 Time to celebrate! Alice has paid in full for our first order, and business is good.</p>
<p>Now, along comes Bob "the bad guy" who places an order for a 💰 1oz gold bar.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (18, 1, 3, 13, 1, 'INSERT INTO orders (customer_name) VALUES (''Bob'');
INSERT INTO line_items (order_id, product_emoji, count) VALUES (
  (SELECT max(id) FROM orders WHERE customer_name = ''Bob''),
  ''💰'',
  1
)
RETURNING *;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (19, 1, 1, 14, 1, 'Unfortunately, Bob makes his payment with a stolen credit card, but we don''t know that yet.', '<article class="markdown-body"><p>Unfortunately, Bob makes his payment with a stolen credit card, but we don''t know that yet.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (20, 1, 3, 15, 1, 'INSERT INTO payments (order_id, amount) 
-- a query to compute the full amount of Bob''s most recent order
SELECT order_id, SUM(count * price) AS amount
FROM orders
JOIN line_items ON line_items.order_id = orders.id
JOIN products ON products.emoji = line_items.product_emoji
WHERE orders.id = (SELECT max(id) AS order_id FROM orders WHERE customer_name = ''Bob'')
GROUP BY 1
RETURNING *;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (21, 1, 1, 16, 1, 'At the end of the month, the credit card company lets us know about the chargeback from the real card owner.  We''ll need to create another table to keep track of this.', '<article class="markdown-body"><p>At the end of the month, the credit card company lets us know about the chargeback from the real card owner.  We''ll need to create another table to keep track of this.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (22, 1, 3, 17, 1, 'CREATE TABLE chargebacks (
  id BIGSERIAL PRIMARY KEY,
  payment_id BIGINT
)', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (23, 1, 1, 18, 1, 'And now we can record the example of fraud', '<article class="markdown-body"><p>And now we can record the example of fraud</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (24, 1, 3, 19, 1, 'INSERT INTO chargebacks (payment_id) 
SELECT max(payments.id) AS payment_id
FROM payments 
JOIN orders ON payments.order_id = orders.id 
WHERE customer_name = ''Bob''
RETURNING *;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (25, 1, 1, 20, 1, '🏁 Congratulations! 🏁 
----------------
If you''ve made it this far, you''ve won half the machine learning battle. We have created 2 training data examples that are perfect for "supervised" machine learning. The chargebacks act as the ground truth to inform the machine learning algorithm of whether or not an order is fraudulent. These records are what we refer to as "labels", a.k.a "targets" or "Y-values" for the data.

Part 2: Structuring the Training Data
--------------------------
We can construct a query that provides a summary view of our orders, including the fraudulent label.', '<article class="markdown-body"><h2>🏁 Congratulations! 🏁</h2>
<p>If you''ve made it this far, you''ve won half the machine learning battle. We have created 2 training data examples that are perfect for "supervised" machine learning. The chargebacks act as the ground truth to inform the machine learning algorithm of whether or not an order is fraudulent. These records are what we refer to as "labels", a.k.a "targets" or "Y-values" for the data.</p>
<h2>Part 2: Structuring the Training Data</h2>
<p>We can construct a query that provides a summary view of our orders, including the fraudulent label.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (26, 1, 3, 21, 1, 'CREATE VIEW orders_summaries AS
SELECT 
  orders.id AS order_id, 
  orders.customer_name,
  payments.amount AS total, 
  ARRAY_AGG(products.emoji) AS product_emojis,
  CASE WHEN chargebacks.id IS NOT NULL 
    THEN true 
    ELSE false 
  END AS fraudulent
FROM orders
LEFT JOIN payments ON payments.order_id = orders.id
LEFT JOIN chargebacks ON chargebacks.payment_id = payments.id
LEFT JOIN line_items ON line_items.order_id = orders.id
LEFT JOIN products ON products.emoji = line_items.product_emoji
GROUP BY 1, 2, 3, 5
ORDER BY orders.id;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (27, 1, 1, 22, 1, 'Now, let''s have a look at the summary', '<article class="markdown-body"><p>Now, let''s have a look at the summary</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (28, 1, 3, 23, 1, 'SELECT * FROM orders_summaries;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (29, 1, 1, 24, 1, 'It''s intuitive that thieves will be more attracted to gold bars than a head of lettuce because the resell value is better. Perishable goods are more difficult to move on the black market. A good piece of information for our model would be the percentage of the order that is perishable. We call this a "feature" of the data model. We can construct a query to return this feature for each order, along with the chargeback label.', '<article class="markdown-body"><p>It''s intuitive that thieves will be more attracted to gold bars, than a head of lettuce because the resell value is better. Perishable goods are more difficult to move on the black market. A good piece of information for our model would be the percentage of the order that is perishable. We call this a "feature" of the data model. We can construct a query to return this feature for each order, along with the chargeback label.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (30, 1, 3, 25, 1, 'CREATE VIEW fraud_samples AS
SELECT 
  SUM(CASE WHEN products.perishable THEN (count * price) ELSE ''$0.0'' END) / SUM(payments.amount) AS perishable_percentage, 
  CASE WHEN chargebacks.id IS NOT NULL 
    THEN true 
    ELSE false 
  END AS fraudulent
FROM orders
LEFT JOIN payments ON payments.order_id = orders.id
LEFT JOIN chargebacks ON chargebacks.payment_id = payments.id
LEFT JOIN line_items ON line_items.order_id = orders.id
LEFT JOIN products ON products.emoji = line_items.product_emoji
GROUP BY orders.id, chargebacks.id
ORDER BY orders.id;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (31, 1, 3, 26, 1, 'SELECT * FROM fraud_samples;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (48, 1, 1, 43, 1, 'Uh oh, the model was trained on a perfectly small dataset. It learned that unless the order is perishable goods, it''s going to predict fraud 100% of the time, but our test data shows that''s not 100% true. Let''s generate some samples to further explore our model.', '<article class="markdown-body"><p>Uh oh, the model was trained on a perfectly small dataset. It learned that unless the order is perishable goods, it''s going to predict fraud 100% of the time, but our test data shows that''s not 100% true. Let''s generate some samples to further explore our model.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (49, 1, 3, 44, 1, 'WITH exploration_samples AS (
  SELECT generate_series(0, 1, 0.1) AS perishable_percentage
)
SELECT 
  perishable_percentage, 
  pgml.predict(''Our Fraud Classification'', ARRAY[perishable_percentage::real]) AS predict_fraud 
FROM exploration_samples;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (101, 8, 3, 5, 1, 'SELECT id, name, task::TEXT FROM pgml.projects LIMIT 10;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (158, 6, 3, 23, 1, 'SELECT pgml.distance_l2(ARRAY[1.0::real, 2.0, 3.0], ARRAY[4.0, 5.0, 6.0]);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (32, 1, 1, 27, 1, 'Training a model
----------------

This is a great training set for a machine learning model. We''ve found a feature `perishable_percentage` that perfectly correlates with the label `fraudulent`. Perishable orders are less likely to result in a chargeback. A good model will be able to generalize from the example data we have to new examples that we may never have seen before, like an order that is only 33% perishable goods. Now that we have a `VIEW` of this data, we can train a "classification" model to classify the features as `fraudulent` or not.', '<article class="markdown-body"><h2>Training a model</h2>
<p>This is a great training set for a machine learning model. We''ve found a feature <code>perishable_percentage</code> that perfectly correlates with the label <code>fraudulent</code>. Perishable orders are less likely to result in a chargeback. A good model will be able to generalize from the example data we have to new examples that we may never have seen before, like an order that is only 33% perishable goods. Now that we have a <code>VIEW</code> of this data, we can train a "classification" model to classify the features as <code>fraudulent</code> or not.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (33, 1, 3, 28, 1, 'SELECT * FROM pgml.train(
  project_name => ''Our Fraud Model'', -- a friendly name we''ll use to identify this machine learning project
  task => ''classification'', -- we want to classify into true or false
  relation_name => ''fraud_samples'', -- our view of the data
  y_column_name => ''fraudulent'', -- the "labels"
  test_sampling => ''last'', -- the part of the data to use for testing our model
  test_size => 0.5 -- use half the data for tests
);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (34, 1, 1, 29, 1, 'Oops. We''re going to get an error:

```
ERROR:  ValueError: This solver needs samples of at least 2 classes in the data, but the data contains only one class: False
```

Wait a second, we know there is both a True and a False label, because we have an example of both a fraudulent and legit order. What gives? This is a glimpse into how PostgresML works inside the black box. It splits the sample data into 2 sets. One is used for training the model as we expected, and the other is used to test the model''s predictions against the remaining known labels. This way we can see how well the model generalizes. In this case, since there are only 2 data samples, 1 is used for training (the False label) and 1 is used for testing (the True label). Now we can understand there isn''t enough data to actually train and test. We need to generate a couple more examples so we have enough to train and test.', '<article class="markdown-body"><p>Oops. We''re going to get an error:</p>
<pre><code>ERROR:  ValueError: This solver needs samples of at least 2 classes in the data, but the data contains only one class: False
</code></pre>
<p>Wait a second, we know there is both a True and a False label, because we have an example of both a fraudulent and legit order. What gives? This is a glimpse into how PostgresML works inside the black box. It splits the sample data into 2 sets. One is used for training the model as we expected, and the other is used to test the model''s predictions against the remaining known labels. This way we can see how well the model generalizes. In this case, since there are only 2 data samples, 1 is used for training (the False label) and 1 is used for testing (the True label). Now we can understand there isn''t enough data to actually train and test. We need to generate a couple more examples so we have enough to train and test.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (35, 1, 3, 30, 1, 'INSERT INTO orders (customer_name) VALUES (''Carol'');
INSERT INTO line_items (
  order_id, 
  product_emoji, 
  count
) VALUES (
  (SELECT max(id) FROM orders WHERE customer_name = ''Carol''),
  ''📕'',
  10
)
RETURNING *;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (36, 1, 1, 31, 1, 'Carol has bought a book, and now will legitimately pay in full.', '<article class="markdown-body"><p>Carol has bought a book, and now will legitimately pay in full.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (37, 1, 3, 32, 1, 'INSERT INTO payments (order_id, amount) 
-- a query to compute the full amount of Carol''s most recent order
SELECT order_id, SUM(count * price) AS amount
FROM orders
JOIN line_items ON line_items.order_id = orders.id
JOIN products ON products.emoji = line_items.product_emoji
WHERE orders.id = (SELECT max(id) AS order_id FROM orders WHERE customer_name = ''Carol'')
GROUP BY 1
RETURNING *;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (38, 1, 1, 33, 1, 'And now Dan (another fraudster) shows up to steal more books:', '<article class="markdown-body"><p>And now Dan (another fraudster) shows up to steal more books:</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (39, 1, 3, 34, 1, 'INSERT INTO orders (customer_name) VALUES (''Dan'');
INSERT INTO line_items (
  order_id, 
  product_emoji, 
  count
) VALUES (
  (SELECT max(id) FROM orders WHERE customer_name = ''Dan''),
  ''📕'',
  50
)
RETURNING *;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (40, 1, 1, 35, 1, 'Here comes the fraudulent payment.', '<article class="markdown-body"><p>Here comes the fraudulent payment.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (41, 1, 3, 36, 1, 'INSERT INTO payments (order_id, amount) 
-- a query to compute the full amount of Dan''s most recent order
SELECT order_id, SUM(count * price) AS amount
FROM orders
JOIN line_items ON line_items.order_id = orders.id
JOIN products ON products.emoji = line_items.product_emoji
WHERE orders.id = (SELECT max(id) AS order_id FROM orders WHERE customer_name = ''Dan'')
GROUP BY 1
RETURNING *;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (42, 1, 1, 37, 1, 'And when the credit card company let''s us know about the issue, we''ll record it.', '<article class="markdown-body"><p>And when the credit card company let''s us know about the issue, we''ll record it.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (43, 1, 3, 38, 1, 'INSERT INTO chargebacks (payment_id) 
SELECT max(payments.id) AS payment_id
FROM payments 
JOIN orders ON payments.order_id = orders.id 
WHERE customer_name = ''Dan''
RETURNING *;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (44, 1, 1, 39, 1, 'And now we can try to train the model again.', '<article class="markdown-body"><p>And now we can try to train the model again.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (45, 1, 3, 40, 1, 'SELECT * FROM pgml.train(
  project_name => ''Our Fraud Classification'', -- a friendly name we''ll use to identify this machine learning project
  task => ''classification'', -- we want to classify into true or false
  relation_name => ''fraud_samples'', -- our view of the data
  y_column_name => ''fraudulent'', -- the "labels"
  test_sampling => ''last'',
  test_size => 0.5 -- use half the data for testing rather than the default test size of 25%
);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (46, 1, 1, 41, 1, '🏁 Success! 🏁
--------------

We can demonstrate basic usage of the model with another SQL call', '<article class="markdown-body"><h2>🏁 Success! 🏁</h2>
<p>We can demonstrate basic usage of the model with another SQL call</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (47, 1, 3, 42, 1, 'SELECT 
  perishable_percentage, 
  fraudulent, 
  pgml.predict(''Our Fraud Classification'', ARRAY[perishable_percentage::real]) AS predict_fraud 
FROM fraud_samples;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (130, 4, 3, 13, 1, 'SELECT * FROM pgml.train(
    ''Diabetes Progression'', 
    algorithm => ''xgboost'', 
    search => ''grid'', 
    search_params => ''{
        "max_depth": [1, 2], 
        "n_estimators": [20, 40],
        "learning_rate": [0.1, 0.2]
    }''
);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (159, 6, 3, 24, 1, 'SELECT pgml.dot_product(ARRAY[1.0::real, 2.0, 3.0], ARRAY[4.0, 5.0, 6.0]);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (50, 1, 1, 45, 1, 'The default model is a linear regression, so it has learned from the training half of the data that high amounts of perishable goods make for safe orders.

Part 4: Adding more features
----------------------------
We need to add some more features to create a better model. Instead of just using the perishable percentage, we can use dollar values as our features, since we know criminals want to steal large amounts more than small amounts.', '<article class="markdown-body"><p>The default model is a linear regression, so it has learned from the training half of the data that high amounts of perishable goods make for safe orders.</p>
<h2>Part 4: Adding more features</h2>
<p>We need to add some more features to create a better model. Instead of just using the perishable percentage, we can use dollar values as our features, since we know criminals want to steal large amounts more than small amounts.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (51, 1, 3, 46, 1, 'DROP VIEW fraud_samples;
CREATE VIEW fraud_samples AS
SELECT 
  SUM(CASE WHEN products.perishable THEN (count * price)::NUMERIC::FLOAT ELSE 0.0 END) AS perishable_amount, 
  SUM(CASE WHEN NOT products.perishable THEN (count * price)::NUMERIC::FLOAT ELSE 0.0 END) AS non_perishable_amount, 
  CASE WHEN chargebacks.id IS NOT NULL 
    THEN true 
    ELSE false 
  END AS fraudulent
FROM orders
LEFT JOIN payments ON payments.order_id = orders.id
LEFT JOIN chargebacks ON chargebacks.payment_id = payments.id
LEFT JOIN line_items ON line_items.order_id = orders.id
LEFT JOIN products ON products.emoji = line_items.product_emoji
GROUP BY orders.id, chargebacks.id
ORDER BY orders.id;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (52, 1, 1, 47, 1, 'And now we retrain a new version of the model, by calling train with the same parameters again.', '<article class="markdown-body"><p>And now we retrain a new version of the model, by calling train with the same parameters again.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (53, 1, 3, 48, 1, 'SELECT * FROM pgml.train(
  project_name => ''Our Fraud Classification'', -- a friendly name we''ll use to identify this machine learning project
  task => ''classification'', -- we want to classify into true or false
  relation_name => ''fraud_samples'', -- our view of the data
  y_column_name => ''fraudulent'', -- the "labels"
  test_sampling => ''last'',
  test_size => 0.5 -- use half the data for testing rather than the default test size of 25%
);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (54, 1, 1, 49, 1, 'And then we can deploy this most recent version', '<article class="markdown-body"><p>And then we can deploy this most recent version</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (55, 1, 3, 50, 1, 'SELECT * FROM pgml.deploy(''Our Fraud Classification'', ''most_recent'');', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (56, 1, 1, 51, 1, 'And view the input/outputs of this model based on our data:', '<article class="markdown-body"><p>And view the input/outputs of this model based on our data:</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (57, 1, 3, 52, 1, 'SELECT 
  perishable_amount, 
  non_perishable_amount, 
  fraudulent, 
  pgml.predict(
    ''Our Fraud Classification'', 
    ARRAY[perishable_amount::real, non_perishable_amount::real]
  ) AS predict_fraud 
FROM fraud_samples;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (58, 1, 1, 53, 1, 'This is the basic development cycle for a model. 
  
1. Add new features.
2. Retrain the new model.
3. Analyze performance.

Even with a toy schema like this, it''s possible to create many different features over the data. Examples of other statistical features we could add:

- how many orders the customer has previously made without chargebacks,
- what has their total spend been so far,
- how old is this account,
- what is their average order size,
- how frequently do they typically order,
- do they typically buy perishable or non perishable goods.

We can create additional VIEWs, subqueries or [Common Table Expressions](https://www.postgresql.org/docs/current/queries-with.html) to standardize these features across models or reports.

Subqueries may be preferable to Common Table Expressions for generating complex features, because CTEs create an optimization gate that prevents the query planner from pushing predicates down, which will hurt performance if you intend to reuse this VIEW during inference for a single row.

If you''re querying a particular view frequently that is expensive to produce, you may consider using a `CREATE MATERIALIZED VIEW`, to cache the results.


```sql
-- A subquery
LEFT JOIN (
  SELECT DISTINCT orders.customer_name, COUNT(*) AS previous_orders FROM ORDERS
) AS customer_stats
  ON customer_stats.customer_name = orders.customer_name
```

```sql
-- A view
CREATE VIEW customer_stats AS 
SELECT DISTINCT orders.customer_name, COUNT(*) AS previous_orders FROM ORDERS;

...
LEFT JOIN customer_stats ON customer_stats.customer_name = orders.customer_name
```

```sql
-- A Common Table Expression
WITH customer_stats AS ( 
  SELECT DISTINCT orders.customer_name, COUNT(*) AS previous_orders FROM ORDERS;
)

...
LEFT JOIN customer_stats ON customer_stats.customer_name = orders.customer_name
```', '<article class="markdown-body"><p>This is the basic development cycle for a model. </p>
<ol>
<li>Add new features.</li>
<li>Retrain the new model.</li>
<li>Analyze performance.</li>
</ol>
<p>Even with a toy schema like this, it''s possible to create many different features over the data. Examples of other statistical features we could add:</p>
<ul>
<li>how many orders the customer has previously made without chargebacks,</li>
<li>what has their total spend been so far,</li>
<li>how old is this account,</li>
<li>what is their average order size,</li>
<li>how frequently do they typically order,</li>
<li>do they typically buy perishable or non perishable goods.</li>
</ul>
<p>We can create additional VIEWs, subqueries or <a href="https://www.postgresql.org/docs/current/queries-with.html">Common Table Expressions</a> to standardize these features across models or reports.</p>
<p>Subqueries may be preferable to Common Table Expressions for generating complex features, because CTEs create an optimization gate that prevents the query planner from pushing predicates down, which will hurt performance if you intend to reuse this VIEW during inference for a single row.</p>
<p>If you''re querying a particular view frequently that is expensive to produce, you may consider using a <code>CREATE MATERIALIZED VIEW</code>, to cache the results.</p>
<pre><code class="language-sql">-- A subquery
LEFT JOIN (
  SELECT DISTINCT orders.customer_name, COUNT(*) AS previous_orders FROM ORDERS
) AS customer_stats
  ON customer_stats.customer_name = orders.customer_name
</code></pre>
<pre><code class="language-sql">-- A view
CREATE VIEW customer_stats AS 
SELECT DISTINCT orders.customer_name, COUNT(*) AS previous_orders FROM ORDERS;

...
LEFT JOIN customer_stats ON customer_stats.customer_name = orders.customer_name
</code></pre>
<pre><code class="language-sql">-- A Common Table Expression
WITH customer_stats AS ( 
  SELECT DISTINCT orders.customer_name, COUNT(*) AS previous_orders FROM ORDERS;
)

...
LEFT JOIN customer_stats ON customer_stats.customer_name = orders.customer_name
</code></pre></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (59, 1, 1, 54, 1, 'Part 5: Upgrading the Machine Learning Algorithm
------------------------------------------

When you''re out of ideas for features that might help the model distinguish orders that are likely to result in chargebacks, you may want to start testing different algorithms to see how the performance changes. PostgresML makes algorithm selection as easy as passing an additional parameter to `pgml.train`. You may want to test them all just to see, but `xgboost` typically gives excellent performance in terms of both accuracy and latency.', '<article class="markdown-body"><h2>Part 5: Upgrading the Machine Learning Algorithm</h2>
<p>When you''re out of ideas for features that might help the model distinguish orders that are likely to result in chargebacks, you may want to start testing different algorithms to see how the performance changes. PostgresML makes algorithm selection as easy as passing an additional parameter to <code>pgml.train</code>. You may want to test them all just to see, but <code>xgboost</code> typically gives excellent performance in terms of both accuracy and latency.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (60, 1, 3, 55, 1, 'SELECT * FROM pgml.train(
  project_name => ''Our Fraud Classification'', -- a friendly name we''ll use to identify this machine learning project
  task => ''classification'', -- we want to classify into true or false
  relation_name => ''fraud_samples'', -- our view of the data
  y_column_name => ''fraudulent'', -- the "labels"
  algorithm => ''xgboost'', -- tree based models like xgboost are often the best performers for tabular data at scale
  test_size => 0.5, -- use half the data for testing rather than the default test size of 25%
  test_sampling => ''last''
);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (61, 1, 1, 56, 1, 'So far we''ve been training a classifier that gives us a binary 0 or 1 output to indicate fraud or not. If we''d like to refine our application response to the models predictions in a more nuanced way, say high/medium/low risk instead of binary, we can use "regression" instead of "classification" to predict a likelihood between 0 and 1, instead of binary.', '<article class="markdown-body"><p>So far we''ve been training a classifier that gives us a binary 0 or 1 output to indicate fraud or not. If we''d like to refine our application response to the models predictions in a more nuanced way, say high/medium/low risk instead of binary, we can use "regression" instead of "classification" to predict a likelihood between 0 and 1, instead of binary.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (62, 1, 3, 57, 1, 'SELECT * FROM pgml.train(
  project_name => ''Our Fraud Regression'', -- a friendly name we''ll use to identify this machine learning project
  task => ''regression'', -- predict the likelihood
  relation_name => ''fraud_samples'', -- our view of the data
  y_column_name => ''fraudulent'', -- the "labels"
  algorithm => ''linear'', 
  test_size => 0.5, -- use half the data for testing rather than the default test size of 25%
  test_sampling => ''last''
);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (152, 6, 1, 17, 1, '### Normalization', '<article class="markdown-body"><h3>Normalization</h3></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (153, 6, 3, 18, 1, 'SELECT pgml.normalize_l1(ARRAY[1.0::real, 2.0, 3.0]);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (63, 1, 1, 58, 1, 'At this point, the primary limitation of our model is the amount of data, the number of examples we have to train it on. Luckily, as time marches on, and data accumulates in the database, we can simply retrain this model with additional calls to `pgml.train` and watch it adjust as new information becomes available.', '<article class="markdown-body"><p>At this point, the primary limitation of our model is the amount of data, the number of examples we have to train it on. Luckily, as time marches on, and data accumulates in the database, we can simply retrain this model with additional calls to <code>pgml.train</code> and watch it adjust as new information becomes available.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (64, 1, 3, 59, 1, '-- If you''d like to start this tutorial over, you can clear out the tables we created.
-- use Ctrl-/ to comment/uncomment blocks in this editor.
DROP TABLE IF EXISTS products CASCADE; 
DROP TABLE IF EXISTS orders CASCADE; 
DROP TABLE IF EXISTS line_items CASCADE; 
DROP TABLE IF EXISTS chargebacks CASCADE; 
DROP TABLE IF EXISTS payments CASCADE;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (65, 2, 1, 1, 1, 'Binary classification means categorizing data into 2 categories. Usually these are categories like:

- `True` or `False`
- `0` or `1`
- `hot_dog` or `not_hot_dog`

These categories divide a population into things we care about, and things we can ignore. Binary classification is a common task for machine learning models. It can be applied across a broad set of scenarios, once you understand the way to structure your problem as a set of example data with labeled outcomes.

In this tutorial, we''ll train models using various "supervised learning" algorithms to classify medical samples as benign or malignant. Supervised learning techniques require us to label the sample data for the algorithm to learn how the inputs correlate with the labels. After the algorithm has been trained on the labeled data set we created, we can present it with new unlabeled data to classify based on the most likely outcome.

As we saw in [Tutorial 1: Real Time Fraud Model](../1/) understanding the structure of the data and the labels is a complex and critical step for real world machine learning projects. In this example we''ll focus more on the different algorithms, and use an academic benchmark dataset that already includes binary labels from UCI ML Breast Cancer Wisconsin. Features were computed from a digitized image of a fine needle aspirate (FNA) of a breast mass. They describe characteristics of the cell nuclei present in the image. The labels are either True for a malignant sample of False for a benign sample.

You can load this dataset into your Postgres database with the following SQL.', '<article class="markdown-body"><p>Binary classification means categorizing data into 2 categories. Usually these are categories like:</p>
<ul>
<li><code>True</code> or <code>False</code></li>
<li><code>0</code> or <code>1</code></li>
<li><code>hot_dog</code> or <code>not_hot_dog</code></li>
</ul>
<p>These categories divide a population into things we care about, and things we can ignore. Binary classification is a common task for machine learning models. It can be applied across a broad set of scenarios, once you understand the way to structure your problem as a set of example data with labeled outcomes.</p>
<p>In this tutorial, we''ll train models using various "supervised learning" algorithms to classify medical samples as benign or malignant. Supervised learning techniques require us to label the sample data for the algorithm to learn how the inputs correlate with the labels. After the algorithm has been trained on the labeled data set we created, we can present it with new unlabeled data to classify based on the most likely outcome.</p>
<p>As we saw in <a href="../1/">Tutorial 1: Real Time Fraud Model</a> understanding the structure of the data and the labels is a complex and critical step for real world machine learning projects. In this example we''ll focus more on the different algorithms, and use an academic benchmark dataset that already includes binary labels from UCI ML Breast Cancer Wisconsin. Features were computed from a digitized image of a fine needle aspirate (FNA) of a breast mass. They describe characteristics of the cell nuclei present in the image. The labels are either True for a malignant sample of False for a benign sample.</p>
<p>You can load this dataset into your Postgres database with the following SQL.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (66, 2, 3, 2, 1, 'SELECT * FROM pgml.load_dataset(''breast_cancer'');', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (67, 2, 1, 3, 1, 'This function has created a new table in your database named `pgml.breast_cancer`. Let''s look at a random sample of the data with some more SQL.', '<article class="markdown-body"><p>This function has created a new table in your database named <code>pgml.breast_cancer</code>. Let''s look at a random sample of the data with some more SQL.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (68, 2, 3, 4, 1, 'SELECT * 
FROM pgml.breast_cancer 
ORDER BY random()
LIMIT 10;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (69, 2, 1, 5, 1, 'That''s a lot of numeric feature data describing various attributes of the cells, but if you scroll all the way to the right above, after running the query, you''ll see that each sample set of feature data is labeled `malignant` [`True` or `False`]. It would be extremely difficult for a human to study all these numbers, and see how they correlate with malignant or not, and then be able to make a prediction for new samples, but mathematicians have been working on algorithms to do exactly this using computers which happen to be exceptionally good at this by now. This is statistical machine learning.

PostgresML makes it easy to use this data to create a model. It only takes a single function call with a few parameters.', '<article class="markdown-body"><p>That''s a lot of numeric feature data describing various attributes of the cells, but if you scroll all the way to the right above, after running the query, you''ll see that each sample set of feature data is labeled <code>malignant</code> [<code>True</code> or <code>False</code>]. It would be extremely difficult for a human to study all these numbers, and see how they correlate with malignant or not, and then be able to make a prediction for new samples, but mathematicians have been working on algorithms to do exactly this using computers which happen to be exceptionally good at this by now. This is statistical machine learning.</p>
<p>PostgresML makes it easy to use this data to create a model. It only takes a single function call with a few parameters.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (70, 2, 3, 6, 1, 'SELECT * FROM pgml.train(
  project_name => ''Breast Cancer Detection'', 
  task => ''classification'', 
  relation_name => ''pgml.breast_cancer'', 
  y_column_name => ''malignant''
);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (71, 2, 1, 7, 1, '🏁 Congratulations 🏁
---------------------

You''ve just created a machine learning model, tested it''s accuracy, and deployed it to production. PostgresML orchestrated a bunch of the traditional ML drudgery in that couple of seconds to make it as simple as possible for you to get value. We''ll organize our work on this task under the project name "Breast Cancer Detection", which you can now see it in your [list of projects](../../projects/). You can see that the first model uses the default linear algorithm, and that it achieves an [F1 score](https://en.wikipedia.org/wiki/F-score) in the mid 90''s, which is pretty good. A score of 1.0 is perfect, and 0.5 would be as good as random guessing. The better the F1 score, the better the algorithm can perform on this dataset. 

We can now use this model to make some predictions in real time, using the training data as input to the `pgml.predict` function.', '<article class="markdown-body"><h2>🏁 Congratulations 🏁</h2>
<p>You''ve just created a machine learning model, tested it''s accuracy, and deployed it to production. PostgresML orchestrated a bunch of the traditional ML drudgery in that couple of seconds to make it as simple as possible for you to get value. We''ll organize our work on this task under the project name "Breast Cancer Detection", which you can now see it in your <a href="../../projects/">list of projects</a>. You can see that the first model uses the default linear algorithm, and that it achieves an <a href="https://en.wikipedia.org/wiki/F-score">F1 score</a> in the mid 90''s, which is pretty good. A score of 1.0 is perfect, and 0.5 would be as good as random guessing. The better the F1 score, the better the algorithm can perform on this dataset. </p>
<p>We can now use this model to make some predictions in real time, using the training data as input to the <code>pgml.predict</code> function.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (72, 2, 3, 8, 1, 'SELECT malignant, pgml.predict(
    ''Breast Cancer Detection'', 
    ARRAY[
        "mean radius", 
        "mean texture", 
        "mean perimeter", 
        "mean area",
        "mean smoothness",
        "mean compactness",
        "mean concavity",
        "mean concave points",
        "mean symmetry",
        "mean fractal dimension",
        "radius error",
        "texture error",
        "perimeter error",
        "area error",
        "smoothness error",
        "compactness error",
        "concavity error",
        "concave points error",
        "symmetry error",
        "fractal dimension error",
        "worst radius",
        "worst texture",
        "worst perimeter",
        "worst area",
        "worst smoothness",
        "worst compactness",
        "worst concavity",
        "worst concave points",
        "worst symmetry",
        "worst fractal dimension"
    ]
) AS prediction
FROM pgml.breast_cancer
ORDER BY random()
LIMIT 10;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (154, 6, 3, 19, 1, 'SELECT pgml.normalize_l2(ARRAY[1.0::real, 2.0, 3.0]);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (73, 2, 1, 9, 1, 'You can see the model is pretty good at predicting `0` for non malignant samples, and `1` for malignant samples. This isn''t a great test though, because we''re using the same data we trained with. We could have just looked up the data in the database table if this is all we wanted to do. The point of training a machine learning model is to generalize these statistics to data we''ve never seen before. What do you think this model would predict if all the input features happened to be 0 or 1? How does that compare to what it''s seen before? 

It''s easy to test the model and see by providing new sample data in real time. There are lots of ways we could feed new data to a model in Postgres. We could write new samples to a table just like our training data, or we could pass parameters directly into a query without recording anything in the database at all. Postgres gives us a lot of ways to get data in and out at run time. We''ll demonstrate with a `VALUES` example for batch prediction.', '<article class="markdown-body"><p>You can see the model is pretty good at predicting <code>0</code> for non malignant samples, and <code>1</code> for malignant samples. This isn''t a great test though, because we''re using the same data we trained with. We could have just looked up the data in the database table if this is all we wanted to do. The point of training a machine learning model is to generalize these statistics to data we''ve never seen before. What do you think this model would predict if all the input features happened to be 0 or 1? How does that compare to what it''s seen before? </p>
<p>It''s easy to test the model and see by providing new sample data in real time. There are lots of ways we could feed new data to a model in Postgres. We could write new samples to a table just like our training data, or we could pass parameters directly into a query without recording anything in the database at all. Postgres gives us a lot of ways to get data in and out at run time. We''ll demonstrate with a <code>VALUES</code> example for batch prediction.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (74, 2, 3, 10, 1, 'SELECT sample_name, pgml.predict(
    ''Breast Cancer Detection'', 
    ARRAY[
        "mean radius", 
        "mean texture", 
        "mean perimeter", 
        "mean area",
        "mean smoothness",
        "mean compactness",
        "mean concavity",
        "mean concave points",
        "mean symmetry",
        "mean fractal dimension",
        "radius error",
        "texture error",
        "perimeter error",
        "area error",
        "smoothness error",
        "compactness error",
        "concavity error",
        "concave points error",
        "symmetry error",
        "fractal dimension error",
        "worst radius",
        "worst texture",
        "worst perimeter",
        "worst area",
        "worst smoothness",
        "worst compactness",
        "worst concavity",
        "worst concave points",
        "worst symmetry",
        "worst fractal dimension"
    ]
) AS prediction
FROM (
  VALUES 
    (''all_zeroes'',0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0),
    (''all_ones'',  1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1)
) 
  AS t (
    "sample_name",
    "mean radius", 
    "mean texture", 
    "mean perimeter", 
    "mean area",
    "mean smoothness",
    "mean compactness",
    "mean concavity",
    "mean concave points",
    "mean symmetry",
    "mean fractal dimension",
    "radius error",
    "texture error",
    "perimeter error",
    "area error",
    "smoothness error",
    "compactness error",
    "concavity error",
    "concave points error",
    "symmetry error",
    "fractal dimension error",
    "worst radius",
    "worst texture",
    "worst perimeter",
    "worst area",
    "worst smoothness",
    "worst compactness",
    "worst concavity",
    "worst concave points",
    "worst symmetry",
    "worst fractal dimension"
  );', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (75, 2, 1, 11, 1, 'Even though the inputs are not data we''ve ever seen before, the model is telling us both of these new samples are likely to be benign based on their statistical correlations to the training samples we had labeled. As we collect new data samples, we could potentially use this model for multiple purposes, like screening the samples before doing further more expensive or invasive analysis.

To demonstrate a more concise call that omits all the feature names (careful to get the order right):', '<article class="markdown-body"><p>Even though the inputs are not data we''ve ever seen before, the model is telling us both of these new samples are likely to be benign based on their statistical correlations to the training samples we had labeled. As we collect new data samples, we could potentially use this model for multiple purposes, like screening the samples before doing further more expensive or invasive analysis.</p>
<p>To demonstrate a more concise call that omits all the feature names (careful to get the order right):</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (76, 2, 3, 12, 1, 'SELECT pgml.predict(
    ''Breast Cancer Detection'', 
    ARRAY[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,100000]
)', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (77, 2, 1, 13, 1, 'Ah hah! We put a really big number into the last feature (worst fractal dimension), and got the model to give us a `True` prediction, indicating that large values there correlate with a malignant sample all else being equal using our default linear algorithm. There are lots of ways we can probe the model with test data, but before we spend too much time on this one, it might be informative to try other algorithms.

PostgresML makes it easy to reuse your training data with many of the best algorithms available. Why not try them all?', '<article class="markdown-body"><p>Ah hah! We put a really big number into the last feature (worst fractal dimension), and got the model to give us a <code>True</code> prediction, indicating that large values there correlate with a malignant sample all else being equal using our default linear algorithm. There are lots of ways we can probe the model with test data, but before we spend too much time on this one, it might be informative to try other algorithms.</p>
<p>PostgresML makes it easy to reuse your training data with many of the best algorithms available. Why not try them all?</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (78, 2, 3, 14, 1, '--
-- After a project has been trained, omitted parameters will be reused from previous training runs
-- In these examples we''ll reuse the training data snapshots from the initial call.
--

-- Linear Models
SELECT * FROM pgml.train(''Breast Cancer Detection'', algorithm => ''ridge'');
SELECT * FROM pgml.train(''Breast Cancer Detection'', algorithm => ''stochastic_gradient_descent'');
SELECT * FROM pgml.train(''Breast Cancer Detection'', algorithm => ''perceptron'');
SELECT * FROM pgml.train(''Breast Cancer Detection'', algorithm => ''passive_aggressive'');

-- Support Vector Machines
SELECT * FROM pgml.train(''Breast Cancer Detection'', algorithm => ''svm'');
SELECT * FROM pgml.train(''Breast Cancer Detection'', algorithm => ''nu_svm'');
SELECT * FROM pgml.train(''Breast Cancer Detection'', algorithm => ''linear_svm'');

-- Ensembles
SELECT * FROM pgml.train(''Breast Cancer Detection'', algorithm => ''ada_boost'');
SELECT * FROM pgml.train(''Breast Cancer Detection'', algorithm => ''bagging'');
SELECT * FROM pgml.train(''Breast Cancer Detection'', algorithm => ''extra_trees'', hyperparams => ''{"n_estimators": 10}'');
SELECT * FROM pgml.train(''Breast Cancer Detection'', algorithm => ''gradient_boosting_trees'', hyperparams => ''{"n_estimators": 10}'');
SELECT * FROM pgml.train(''Breast Cancer Detection'', algorithm => ''random_forest'', hyperparams => ''{"n_estimators": 10}'');

-- Gradient Boosting
SELECT * FROM pgml.train(''Breast Cancer Detection'', algorithm => ''xgboost'', hyperparams => ''{"n_estimators": 10}'');
SELECT * FROM pgml.train(''Breast Cancer Detection'', algorithm => ''xgboost_random_forest'', hyperparams => ''{"n_estimators": 10}'');
SELECT * FROM pgml.train(''Breast Cancer Detection'', algorithm => ''lightgbm'', hyperparams => ''{"n_estimators": 1}'');', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (79, 2, 1, 15, 1, 'Turns out, computers are pretty fast these days, even with state of the art algorithms running on a free tier computation resources. 😊 

You can pop over to the [projects](../../projects/) tab for a visualization of the performance of all these algorithms on this dataset, or you can check out the artifacts directly in the database.', '<article class="markdown-body"><p>Turns out, computers are pretty fast these days, even with state of the art algorithms running on a free tier computation resources. 😊 </p>
<p>You can pop over to the <a href="../../projects/">projects</a> tab for a visualization of the performance of all these algorithms on this dataset, or you can check out the artifacts directly in the database.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (80, 2, 3, 16, 1, 'SELECT 
  projects.name,
  models.algorithm,
  round((models.metrics->>''f1'')::numeric, 4) AS f1_score,
  round((models.metrics->>''precision'')::numeric, 4) AS precision,
  round((models.metrics->>''recall'')::numeric, 4) AS recall
FROM pgml.models
JOIN pgml.projects on projects.id = models.project_id
  AND projects.name = ''Breast Cancer Detection''
ORDER BY models.metrics->>''f1'' DESC LIMIT 5;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (81, 2, 1, 17, 1, 'Tree based algorithms like `random_forest`, `xgboost` and `lightgbm` do well on tabular datasets and frequently lead the pack with A+ level performance as measured by the `f1_score`. They are generally sensitive to small changes in the inputs, but also robust to outliers. They are also relatively fast algorithms that can perform predictions in sub millisecond times, meaning most of the cost of inference is in fetching the data they require as inputs. When your inputs are already in the database with the model, that time is as fast as possible!', '<article class="markdown-body"><p>Tree based algorithms like <code>random_forest</code>, <code>xgboost</code> and <code>lightgbm</code> do well on tabular datasets and frequently lead the pack with A+ level performance as measured by the <code>f1_score</code>. They are generally sensitive to small changes in the inputs, but also robust to outliers. They are also relatively fast algorithms that can perform predictions in sub millisecond times, meaning most of the cost of inference is in fetching the data they require as inputs. When your inputs are already in the database with the model, that time is as fast as possible!</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (82, 4, 1, 1, 1, 'So far we''ve focussed on Classification tasks which divide the world into discrete groups. Sometimes we need to take a more nuanced view when issues are not black and white. Sometimes there are no hard boundaries between options, or sometimes one sort of classification error might be much more painful than another. There are many algorithms that can produce a raw score rather than a discrete class for us. These are "Regression" tasks instead of "Classification".

For this example, we''ll look at several medical indicators that correlate with the progression of diabetes one year later. Let''s load up the data and take a look', '<article class="markdown-body"><p>So far we''ve focussed on Classification tasks which divide the world into discrete groups. Sometimes we need to take a more nuanced view when issues are not black and white. Sometimes there are no hard boundaries between options, or sometimes one sort of classification error might be much more painful than another. There are many algorithms that can produce a raw score rather than a discrete class for us. These are "Regression" tasks instead of "Classification".</p>
<p>For this example, we''ll look at several medical indicators that correlate with the progression of diabetes one year later. Let''s load up the data and take a look</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (83, 4, 3, 2, 1, 'SELECT * FROM pgml.load_dataset(''diabetes'');', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (84, 3, 1, 1, 1, 'Image classification is a great application of machine learning. In this tutorial we''ll examine a classic version of this problem, recognizing hand written digits to automatically parse zip codes out of addresses. For machine learning purposes, we decompose images into their uncompressed pixel values as 2D arrays for gray scale images, or 3D arrays for color images. 

Convolutional Neural Nets and other forms of deep learning, leverage the 2D and 3D adjacency of the pixels to get breakthrough state of the art results on difficult image classification tasks over thousands of categories, and also for image labeling. Postgres has native support for multidimensional `ARRAY` data types, that PostgresML can treat accordingly.

Let''s load the dataset to start:', '<article class="markdown-body"><p>Image classification is a great application of machine learning. In this tutorial we''ll examine a classic version of this problem, recognizing hand written digits to automatically parse zip codes out of addresses. For machine learning purposes, we decompose images into their uncompressed pixel values as 2D arrays for gray scale images, or 3D arrays for color images. </p>
<p>Convolutional Neural Nets and other forms of deep learning, leverage the 2D and 3D adjacency of the pixels to get breakthrough state of the art results on difficult image classification tasks over thousands of categories, and also for image labeling. Postgres has native support for multidimensional <code>ARRAY</code> data types, that PostgresML can treat accordingly.</p>
<p>Let''s load the dataset to start:</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (85, 5, 1, 1, 1, 'PostgresML integrates [🤗 Hugging Face Transformers](https://huggingface.co/transformers) to bring state-of-the-art models into the data layer. There are tens of thousands of pre-trained models with pipelines to turn raw inputs into useful results. Many state of the art deep learning architectures have been published and made available for download. You will want to browse all the [models](https://huggingface.co/models) available to find the perfect solution for your [dataset](https://huggingface.co/dataset) and [task](https://huggingface.co/tasks).

We''ll demonstrate some of the tasks that are immediately available to users of your database upon installation.

### ⚠️ Warning ⚠️
These examples take a fair bit of compute. The deep learning models themselves can be several gigabytes, so they may run out of memory if you are accessing this notebook on the free tier of our cloud service. If you''re not using a GPU to accelerate inference, you can expect them to take 10-20 seconds to execute.

### Examples
All of the tasks and models demonstrated here can be customized by passing additional arguments to the `Pipeline` initializer or call. You''ll find additional links to documentation in the examples below. 

The Hugging Face [`Pipeline`](https://huggingface.co/docs/transformers/main_classes/pipelines) API is exposed in Postgres via:

```
pgml.transform(
    task TEXT OR JSONB,      -- task name or full pipeline initializer arguments
    call JSONB,              -- additional call arguments alongside the inputs
    inputs TEXT[] OR BYTEA[] -- inputs for inference
)
```

This is roughly equivalent to the following Python:

```
import transformers

def transform(task, call, inputs):
    return transformers.pipeline(**task)(inputs, **call)
```', '<article class="markdown-body"><p>PostgresML integrates <a href="https://huggingface.co/transformers">🤗 Hugging Face Transformers</a> to bring state-of-the-art models into the data layer. There are tens of thousands of pre-trained models with pipelines to turn raw inputs into useful results. Many state of the art deep learning architectures have been published and made available for download. You will want to browse all the <a href="https://huggingface.co/models">models</a> available to find the perfect solution for your <a href="https://huggingface.co/dataset">dataset</a> and <a href="https://huggingface.co/tasks">task</a>.</p>
<p>We''ll demonstrate some of the tasks that are immediately available to users of your database upon installation.</p>
<h3>⚠️ Warning ⚠️</h3>
<p>These examples take a fair bit of compute. The deep learning models themselves can be several gigabytes, so they may run out of memory if you are accessing this notebook on the free tier of our cloud service. If you''re not using a GPU to accelerate inference, you can expect them to take 10-20 seconds to execute.</p>
<h3>Examples</h3>
<p>All of the tasks and models demonstrated here can be customized by passing additional arguments to the <code>Pipeline</code> initializer or call. You''ll find additional links to documentation in the examples below. </p>
<p>The Hugging Face <a href="https://huggingface.co/docs/transformers/main_classes/pipelines"><code>Pipeline</code></a> API is exposed in Postgres via:</p>
<pre><code>pgml.transform(
    task TEXT OR JSONB,      -- task name or full pipeline initializer arguments
    call JSONB,              -- additional call arguments alongside the inputs
    inputs TEXT[] OR BYTEA[] -- inputs for inference
)
</code></pre>
<p>This is roughly equivalent to the following Python:</p>
<pre><code>import transformers

def transform(task, call, inputs):
    return transformers.pipeline(**task)(inputs, **call)
</code></pre></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (93, 5, 1, 4, 1, '### Sentiment Analysis
Sentiment analysis is one use of `text-classification`, but there are [many others](https://huggingface.co/tasks/text-classification). This model returns both a label classification `["POSITIVE", "NEUTRAL", "NEGATIVE"]`, as well as the score where 0.0 is perfectly negative, and 1.0 is perfectly positive. This example demonstrates specifying the `model` to be used rather than the task. The [`roberta-large-mnli`](https://huggingface.co/roberta-large-mnli) model specifies the task of `sentiment-analysis` in it''s default configuration, so we may omit it from the parameters. Because this is a batch call with 2 inputs, we''ll get 2 outputs in the JSONB.

See [text classification documentation](https://huggingface.co/tasks/text-classification) for more options and potential use cases beyond sentiment analysis. You''ll notice the outputs are not great in this example. RoBERTa is a breakthrough model that demonstrated just how important each particular hyperparameter is for the task and particular dataset regardless of how large your model is. We''ll show how to [fine tune](/user_guides/transformers/fine_tuning/) models on your data in the next step.', '<article class="markdown-body"><h3>Sentiment Analysis</h3>
<p>Sentiment analysis is one use of <code>text-classification</code>, but there are <a href="https://huggingface.co/tasks/text-classification">many others</a>. This model returns both a label classification <code>["POSITIVE", "NEUTRAL", "NEGATIVE"]</code>, as well as the score where 0.0 is perfectly negative, and 1.0 is perfectly positive. This example demonstrates specifying the <code>model</code> to be used rather than the task. The <a href="https://huggingface.co/roberta-large-mnli"><code>roberta-large-mnli</code></a> model specifies the task of <code>sentiment-analysis</code> in it''s default configuration, so we may omit it from the parameters. Because this is a batch call with 2 inputs, we''ll get 2 outputs in the JSONB.</p>
<p>See <a href="https://huggingface.co/tasks/text-classification">text classification documentation</a> for more options and potential use cases beyond sentiment analysis. You''ll notice the outputs are not great in this example. RoBERTa is a breakthrough model that demonstrated just how important each particular hyperparameter is for the task and particular dataset regardless of how large your model is. We''ll show how to <a href="/user_guides/transformers/fine_tuning/">fine tune</a> models on your data in the next step.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (86, 6, 1, 1, 1, 'PostgresML adds [native vector operations](https://github.com/postgresml/postgresml/tree/master/pgml-extension/sql/install/vectors.sql) that can be used in SQL queries. Vector operations are particularly useful for dealing with embeddings that have been generated from other machine learning algorithms and can provide functions like nearest neighbor calculations using the distance functions.

Emeddings can be a relatively efficient mechanism to leverage the power of deep learning, without the runtime inference costs. These functions are relatively fast and the more expensive distance functions can compute ~100k per second for a memory resident dataset on modern hardware.

The PostgreSQL planner will also [automatically parallelize](https://www.postgresql.org/docs/current/parallel-query.html) evaluation on larger datasets, as configured to take advantage of multiple CPU cores when available.', '<article class="markdown-body"><p>PostgresML adds <a href="https://github.com/postgresml/postgresml/tree/master/pgml-extension/sql/install/vectors.sql">native vector operations</a> that can be used in SQL queries. Vector operations are particularly useful for dealing with embeddings that have been generated from other machine learning algorithms and can provide functions like nearest neighbor calculations using the distance functions.</p>
<p>Emeddings can be a relatively efficient mechanism to leverage the power of deep learning, without the runtime inference costs. These functions are relatively fast and the more expensive distance functions can compute ~100k per second for a memory resident dataset on modern hardware.</p>
<p>The PostgreSQL planner will also <a href="https://www.postgresql.org/docs/current/parallel-query.html">automatically parallelize</a> evaluation on larger datasets, as configured to take advantage of multiple CPU cores when available.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (87, 7, 1, 1, 1, 'Models are automatically deployed if their key metric (R2 for regression, F1 for classification) is improved over the currently deployed version during training. If you want to manage deploys manually, you can always change which model is currently responsible for making predictions.

```
pgml.deploy(
    project_name TEXT,                  -- Human-friendly project name
    strategy TEXT DEFAULT ''best_score'', -- ''rollback'', ''best_score'', or ''most_recent''
    algorithm TEXT DEFAULT NULL    -- filter candidates to a particular algorithm, NULL = all qualify
)
```

The default behavior allows any algorithm to qualify, but deployment candidates can be further restricted to a specific algorithm by passing the `algorithm`.

## Strategies
There are 3 different deployment strategies available

strategy | description
--- | ---
most_recent | The most recently trained model for this project
best_score | The model that achieved the best key metric score
rollback | The model that was previously deployed for this project', '<article class="markdown-body"><p>Models are automatically deployed if their key metric (R2 for regression, F1 for classification) is improved over the currently deployed version during training. If you want to manage deploys manually, you can always change which model is currently responsible for making predictions.</p>
<pre><code>pgml.deploy(
    project_name TEXT,                  -- Human-friendly project name
    strategy TEXT DEFAULT ''best_score'', -- ''rollback'', ''best_score'', or ''most_recent''
    algorithm TEXT DEFAULT NULL    -- filter candidates to a particular algorithm, NULL = all qualify
)
</code></pre>
<p>The default behavior allows any algorithm to qualify, but deployment candidates can be further restricted to a specific algorithm by passing the <code>algorithm</code>.</p>
<h2>Strategies</h2>
<p>There are 3 different deployment strategies available</p>
<table>
<thead>
<tr>
<th>strategy</th>
<th>description</th>
</tr>
</thead>
<tbody>
<tr>
<td>most_recent</td>
<td>The most recently trained model for this project</td>
</tr>
<tr>
<td>best_score</td>
<td>The model that achieved the best key metric score</td>
</tr>
<tr>
<td>rollback</td>
<td>The model that was previously deployed for this project</td>
</tr>
</tbody>
</table></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (88, 7, 1, 2, 1, '', '<article class="markdown-body"></article>', NULL, '2022-08-22 15:09:15.475779');
INSERT INTO pgml.notebook_cells VALUES (89, 8, 1, 1, 1, 'PostgresML stores all artifacts from training in the database under the `pgml` schema. You can manually inspect these tables to further understand the inner workings, or to generate additional reporting and analytics across your models.', '<article class="markdown-body"><p>PostgresML stores all artifacts from training in the database under the <code>pgml</code> schema. You can manually inspect these tables to further understand the inner workings, or to generate additional reporting and analytics across your models.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (90, 3, 3, 2, 1, 'SELECT * FROM pgml.load_dataset(''digits'');', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (91, 5, 1, 2, 1, '### Translation
There are thousands of different pre-trained translation models between language pairs. They generally take a single input string in the "from" language, and translate it into the "to" language as a result of the call. PostgresML transformations provide a batch interface where you can pass an array of `TEXT` to process in a single call for efficiency. Not all language pairs have a default task name like this example of English to French. In those cases, you''ll need to specify [the desired model](https://huggingface.co/models?pipeline_tag=translation) by name. Because this is a batch call with 2 inputs, we''ll get 2 outputs in the JSONB.

See [translation documentation](https://huggingface.co/docs/transformers/tasks/translation) for more options.

For a translation from English to French with the default pre-trained model:', '<article class="markdown-body"><h3>Translation</h3>
<p>There are thousands of different pre-trained translation models between language pairs. They generally take a single input string in the "from" language, and translate it into the "to" language as a result of the call. PostgresML transformations provide a batch interface where you can pass an array of <code>TEXT</code> to process in a single call for efficiency. Not all language pairs have a default task name like this example of English to French. In those cases, you''ll need to specify <a href="https://huggingface.co/models?pipeline_tag=translation">the desired model</a> by name. Because this is a batch call with 2 inputs, we''ll get 2 outputs in the JSONB.</p>
<p>See <a href="https://huggingface.co/docs/transformers/tasks/translation">translation documentation</a> for more options.</p>
<p>For a translation from English to French with the default pre-trained model:</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (92, 5, 3, 3, 1, 'SELECT pgml.transform(
        ''translation_en_to_fr'',
        inputs => ARRAY[
            ''Welcome to the future!'',
            ''Where have you been all this time?''
        ]
    ) AS french;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (94, 5, 3, 5, 1, 'SELECT pgml.transform(
        ''{"model": "roberta-large-mnli"}''::JSONB,
        inputs => ARRAY[
            ''I love how amazingly simple ML has become!'', 
            ''I hate doing mundane and thankless tasks. ☹️''
        ]
    ) AS positivity;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (95, 6, 1, 2, 1, '### Elementwise arithmetic w/ constants', '<article class="markdown-body"><h3>Elementwise arithmetic w/ constants</h3></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (96, 6, 3, 3, 1, '', NULL, NULL, '2022-08-22 15:14:31.875531');
INSERT INTO pgml.notebook_cells VALUES (97, 7, 3, 2, 1, '-- deploy the "best" model for prediction use
SELECT * FROM pgml.deploy(''Handwritten Digits'', ''best_score'');', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (98, 8, 1, 2, 1, '## Models

Models are an artifact of calls to `pgml.train`.', '<article class="markdown-body"><h2>Models</h2>
<p>Models are an artifact of calls to <code>pgml.train</code>.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (99, 8, 3, 3, 1, 'SELECT id, algorithm::TEXT, runtime::TEXT FROM pgml.models LIMIT 10;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (155, 6, 3, 20, 1, 'SELECT pgml.normalize_max(ARRAY[1.0::real, 2.0, 3.0]);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (102, 8, 1, 6, 1, '## Snapshots

Snapshots are an artifact of calls to `pgml.train` that include a specific `relation_name` parameter. A full copy of all data in the relation at training time will be saved in a new table named `pgml.snapshot_{id}`. You can retrieve the original training data set by inspecting tables like `pgml.snapshot_1`.', '<article class="markdown-body"><h2>Snapshots</h2>
<p>Snapshots are an artifact of calls to <code>pgml.train</code> that include a specific <code>relation_name</code> parameter. A full copy of all data in the relation at training time will be saved in a new table named <code>pgml.snapshot_{id}</code>. You can retrieve the original training data set by inspecting tables like <code>pgml.snapshot_1</code>.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (103, 3, 1, 3, 1, 'We can view a sample of the data with a simple `SELECT`', '<article class="markdown-body"><p>We can view a sample of the data with a simple <code>SELECT</code></p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (104, 3, 3, 4, 1, 'SELECT target, array_to_json(image) FROM pgml.digits LIMIT 10;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (105, 3, 1, 5, 1, 'The images are 8x8 gray scale arrays with gray values from 0 (white) to 16 (black) pixels. These images have been fairly heavily processed to center and crop each one, and the represented digit is labeled in the `target` column. By now you should start to have an idea what comes next in this tutorial. We''ve got data, so we train a model with a simple call to PostgresML.', '<article class="markdown-body"><p>The images are 8x8 gray scale arrays with gray values from 0 (white) to 16 (black) pixels. These images have been fairly heavily processed to center and crop each one, and the represented digit is labeled in the <code>target</code> column. By now you should start to have an idea what comes next in this tutorial. We''ve got data, so we train a model with a simple call to PostgresML.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (106, 3, 3, 6, 1, 'SELECT * FROM pgml.train(
  project_name => ''Handwritten Digits'', 
  task => ''classification'', 
  relation_name => ''pgml.digits'', 
  y_column_name => ''target''
);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (107, 3, 1, 7, 1, 'We can view some of the predictions of the model on the training data.', '<article class="markdown-body"><p>We can view some of the predictions of the model on the training data.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (108, 3, 3, 8, 1, 'SELECT target, pgml.predict(''Handwritten Digits'', image) AS prediction
FROM pgml.digits 
LIMIT 10;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (109, 3, 1, 9, 1, 'Hah! Even the default linear classification algorithm performs extremely well on such carefully engineered, but real world data. It''s a  demonstration of how effective feature engineering and clean data can be even with relatively simple algorithms. Let''s take a look at that models metrics.', '<article class="markdown-body"><p>Hah! Even the default linear classification algorithm performs extremely well on such carefully engineered, but real world data. It''s a  demonstration of how effective feature engineering and clean data can be even with relatively simple algorithms. Let''s take a look at that models metrics.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (110, 3, 3, 10, 1, 'SELECT 
  projects.name,
  models.algorithm,
  round((models.metrics->>''f1'')::numeric, 4) AS f1_score,
  round((models.metrics->>''precision'')::numeric, 4) AS precision,
  round((models.metrics->>''recall'')::numeric, 4) AS recall
FROM pgml.models
JOIN pgml.projects on projects.id = models.project_id
  AND projects.name = ''Handwritten Digits''
ORDER BY models.created_at DESC LIMIT 5;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (111, 3, 1, 11, 1, 'An F1 score in the mid nineties are grade A results, although there is room for improvement. We need to keep in mind the cost/benefit tradeoffs in the real world. If this algorithm is wrong about a digit 1 out of 20 times, it''ll give us the wrong ZIP code on every 3rd piece of mail. It might be a lot more expensive to re-route 1/3rd of all mail to fix these mistakes than it is to hire human''s read and input every zip code manually, so even though the results are pretty good, they are not good enough to create real value.

Luckily, we have the benefit of the last 40 years of some very smart people developing a bunch of different algorithms for learning that all have different tradeoffs strengths and weaknesses. You could go spend a few years getting a degree trying to understand how they all work, or we can just try them all since computers are cheaper and more plentiful than engineers.', '<article class="markdown-body"><p>An F1 score in the mid nineties are grade A results, although there is room for improvement. We need to keep in mind the cost/benefit tradeoffs in the real world. If this algorithm is wrong about a digit 1 out of 20 times, it''ll give us the wrong ZIP code on every 3rd piece of mail. It might be a lot more expensive to re-route 1/3rd of all mail to fix these mistakes than it is to hire human''s read and input every zip code manually, so even though the results are pretty good, they are not good enough to create real value.</p>
<p>Luckily, we have the benefit of the last 40 years of some very smart people developing a bunch of different algorithms for learning that all have different tradeoffs strengths and weaknesses. You could go spend a few years getting a degree trying to understand how they all work, or we can just try them all since computers are cheaper and more plentiful than engineers.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (112, 3, 3, 12, 1, '--
-- After a project has been trained, omitted parameters will be reused from previous training runs
-- In these examples we''ll reuse the training data snapshots from the initial call.
--

-- linear models
SELECT * FROM pgml.train(''Handwritten Digits'', algorithm => ''ridge'');
SELECT * FROM pgml.train(''Handwritten Digits'', algorithm => ''stochastic_gradient_descent'');
SELECT * FROM pgml.train(''Handwritten Digits'', algorithm => ''perceptron'');
SELECT * FROM pgml.train(''Handwritten Digits'', algorithm => ''passive_aggressive'');

-- support vector machines
SELECT * FROM pgml.train(''Handwritten Digits'', algorithm => ''svm'');
SELECT * FROM pgml.train(''Handwritten Digits'', algorithm => ''nu_svm'');
SELECT * FROM pgml.train(''Handwritten Digits'', algorithm => ''linear_svm'');

-- ensembles
SELECT * FROM pgml.train(''Handwritten Digits'', algorithm => ''ada_boost'');
SELECT * FROM pgml.train(''Handwritten Digits'', algorithm => ''bagging'');
SELECT * FROM pgml.train(''Handwritten Digits'', algorithm => ''extra_trees'', hyperparams => ''{"n_estimators": 10}'');
SELECT * FROM pgml.train(''Handwritten Digits'', algorithm => ''gradient_boosting_trees'', hyperparams => ''{"n_estimators": 10}'');
SELECT * FROM pgml.train(''Handwritten Digits'', algorithm => ''random_forest'', hyperparams => ''{"n_estimators": 10}'');

-- gradient boosting
SELECT * FROM pgml.train(''Handwritten Digits'', algorithm => ''xgboost'', hyperparams => ''{"n_estimators": 10}'');
SELECT * FROM pgml.train(''Handwritten Digits'', algorithm => ''xgboost_random_forest'', hyperparams => ''{"n_estimators": 10}'');
SELECT * FROM pgml.train(''Handwritten Digits'', algorithm => ''lightgbm'', hyperparams => ''{"n_estimators": 1}'');', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (113, 3, 1, 13, 1, 'In less than 10 seconds, we''ve thrown a barrage of algorithms at the problem and measured how they perform. Now let''s take a look at the best one''s metrics.', '<article class="markdown-body"><p>In less than 10 seconds, we''ve thrown a barrage of algorithms at the problem and measured how they perform. Now let''s take a look at the best one''s metrics.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (156, 6, 1, 21, 1, '### Comparisons', '<article class="markdown-body"><h3>Comparisons</h3></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (114, 3, 3, 14, 1, 'SELECT 
  projects.name,
  models.algorithm,
  round((models.metrics->>''f1'')::numeric, 4) AS f1_score,
  round((models.metrics->>''precision'')::numeric, 4) AS precision,
  round((models.metrics->>''recall'')::numeric, 4) AS recall
FROM pgml.models
JOIN pgml.projects on projects.id = models.project_id
  AND projects.name = ''Handwritten Digits''
ORDER BY models.metrics->>''f1'' DESC LIMIT 5;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (115, 3, 1, 15, 1, '`svm` stands for Support Vector Machines. They do well on this particular problem, and can reach A+ F1 scores. Back in our real world performance evaluation where they are only wrong 1 out of 100 digits, or 1/14 zip codes, instead of our original 1/3rd wrong baseline model. In the real world this means that about 7% of our mail would end up getting auto-routed to the wrong zip code. Is that good enough to start automating? Let''s ask the Postmaster general... If he says not quite, there is one more thing to try before we break out deep learning. 

Many algorithm''s have a few options we can tweak. These options are called hyperparameters. You can find the available ones for SVMs in the [docs](https://scikit-learn.org/stable/modules/generated/sklearn.svm.SVC.html). Then we can automatically search all the combinations of the hyperparams to see how to tweak the knobs. We don''t actually have to have that degree just yet...', '<article class="markdown-body"><p><code>svm</code> stands for Support Vector Machines. They do well on this particular problem, and can reach A+ F1 scores. Back in our real world performance evaluation where they are only wrong 1 out of 100 digits, or 1/14 zip codes, instead of our original 1/3rd wrong baseline model. In the real world this means that about 7% of our mail would end up getting auto-routed to the wrong zip code. Is that good enough to start automating? Let''s ask the Postmaster general... If he says not quite, there is one more thing to try before we break out deep learning. </p>
<p>Many algorithm''s have a few options we can tweak. These options are called hyperparameters. You can find the available ones for SVMs in the <a href="https://scikit-learn.org/stable/modules/generated/sklearn.svm.SVC.html">docs</a>. Then we can automatically search all the combinations of the hyperparams to see how to tweak the knobs. We don''t actually have to have that degree just yet...</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (116, 3, 3, 16, 1, 'SELECT * FROM pgml.train(
    ''Handwritten Digits'', 
    algorithm => ''svm'', 
    hyperparams => ''{"random_state": 0}'',
    search => ''grid'', 
    search_params => ''{
        "kernel": ["linear", "poly", "sigmoid"], 
        "shrinking": [true, false]
    }''
);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (117, 3, 1, 17, 1, 'And then we can peak at the metrics directly with a bit more SQL.', '<article class="markdown-body"><p>And then we can peak at the metrics directly with a bit more SQL.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (118, 3, 3, 18, 1, 'SELECT metrics
FROM pgml.models
ORDER BY created_at DESC
LIMIT 1;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (119, 3, 1, 19, 1, 'It''s a bit tough to parse the results of the search in pure SQL, so you can hop over to the [Projects](../../projects/) list to see a visualization.', '<article class="markdown-body"><p>It''s a bit tough to parse the results of the search in pure SQL, so you can hop over to the <a href="../../projects/">Projects</a> list to see a visualization.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (120, 4, 3, 3, 1, 'SELECT * 
FROM pgml.diabetes 
LIMIT 10;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (121, 4, 1, 4, 1, 'In this case, the `target` is a number that represents the severity of the disease progression one year later, with larger values indicating worse outcomes. Building a Regression model uses the same PostgresML API as Classification, just with a different task. You''re going to start breezing through these tutorials faster and faster.', '<article class="markdown-body"><p>In this case, the <code>target</code> is a number that represents the severity of the disease progression one year later, with larger values indicating worse outcomes. Building a Regression model uses the same PostgresML API as Classification, just with a different task. You''re going to start breezing through these tutorials faster and faster.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (122, 4, 3, 5, 1, 'SELECT * FROM pgml.train(
  project_name => ''Diabetes Progression'', 
  task => ''regression'', 
  relation_name => ''pgml.diabetes'', 
  y_column_name => ''target''
);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (123, 4, 1, 6, 1, 'With our baseline model automatically deployed, we can sample some of the predictions', '<article class="markdown-body"><p>With our baseline model automatically deployed, we can sample some of the predictions</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (124, 4, 3, 7, 1, 'SELECT target, pgml.predict(''Diabetes Progression'', ARRAY[age, sex, bmi, bp, s1, s2, s3, s4, s5, s6]) AS prediction
FROM pgml.diabetes 
LIMIT 10;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (125, 4, 1, 8, 1, 'To get an objective measure of just how far off every single prediction is from the target, we can look at the key metrics recorded during training.', '<article class="markdown-body"><p>To get an objective measure of just how far off every single prediction is from the target, we can look at the key metrics recorded during training.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (126, 4, 3, 9, 1, 'SELECT 
  projects.name,
  models.algorithm,
  round((models.metrics->>''r2'')::numeric, 4) AS r2_score
FROM pgml.models
JOIN pgml.projects on projects.id = models.project_id
  AND projects.name = ''Diabetes Progression''
ORDER BY models.created_at DESC LIMIT 5;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (127, 4, 1, 10, 1, 'I like to look at the R2 score, since it is fixed between 0 and 1 it can help us compare the performance of different algorithms on our data. Let''s throw our bag of tricks at the problem and see what sticks.', '<article class="markdown-body"><p>I like to look at the R2 score, since it is fixed between 0 and 1 it can help us compare the performance of different algorithms on our data. Let''s throw our bag of tricks at the problem and see what sticks.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (128, 4, 3, 11, 1, '-- linear models
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''ridge'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''lasso'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''elastic_net'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''least_angle'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''lasso_least_angle'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''orthogonal_matching_pursuit'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''bayesian_ridge'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''automatic_relevance_determination'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''stochastic_gradient_descent'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''passive_aggressive'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''ransac'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''theil_sen'', hyperparams => ''{"max_iter": 10, "max_subpopulation": 100}'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''huber'');

-- support vector machines
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''svm'', hyperparams => ''{"max_iter": 100}'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''nu_svm'', hyperparams => ''{"max_iter": 10}'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''linear_svm'', hyperparams => ''{"max_iter": 100}'');

-- ensembles
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''ada_boost'', hyperparams => ''{"n_estimators": 5}'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''bagging'', hyperparams => ''{"n_estimators": 5}'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''extra_trees'', hyperparams => ''{"n_estimators": 5}'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''gradient_boosting_trees'', hyperparams => ''{"n_estimators": 5}'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''random_forest'', hyperparams => ''{"n_estimators": 5}'');

-- gradient boosting
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''xgboost'', hyperparams => ''{"n_estimators": 10}'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''xgboost_random_forest'', hyperparams => ''{"n_estimators": 10}'');
SELECT * FROM pgml.train(''Diabetes Progression'', algorithm => ''lightgbm'', hyperparams => ''{"n_estimators": 1}'');', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (129, 4, 1, 12, 1, 'It''s that easy, and that fast, to test all the algorithm''s in our toolkit to see what fares the best, and the best one has automatically been deployed. Once we''ve honed in on a few good candidate algorithms, we can check the docs for their hyperparams, and then do another brute force search across all combinations to find the best set.', '<article class="markdown-body"><p>It''s that easy, and that fast, to test all the algorithm''s in our toolkit to see what fares the best, and the best one has automatically been deployed. Once we''ve honed in on a few good candidate algorithms, we can check the docs for their hyperparams, and then do another brute force search across all combinations to find the best set.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (157, 6, 3, 22, 1, 'SELECT pgml.distance_l1(ARRAY[1.0::real, 2.0, 3.0], ARRAY[4.0, 5.0, 6.0]);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (131, 5, 1, 6, 1, '### Summarization
Sometimes we need all the nuanced detail, but sometimes it''s nice to get to the point. Summarization can reduce a very long and complex document to a few sentences. One studied application is reducing legal bills passed by Congress into a plain english summary. Hollywood may also need some intelligence to reduce a full synopsis down to a pithy blurb for movies like Inception.

See [summarization documentation](https://huggingface.co/tasks/summarization) for more options.', '<article class="markdown-body"><h3>Summarization</h3>
<p>Sometimes we need all the nuanced detail, but sometimes it''s nice to get to the point. Summarization can reduce a very long and complex document to a few sentences. One studied application is reducing legal bills passed by Congress into a plain english summary. Hollywood may also need some intelligence to reduce a full synopsis down to a pithy blurb for movies like Inception.</p>
<p>See <a href="https://huggingface.co/tasks/summarization">summarization documentation</a> for more options.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (132, 5, 3, 7, 1, 'SELECT pgml.transform(
        ''summarization'',
        inputs => ARRAY[''
            Dominic Cobb is the foremost practitioner of the artistic science 
            of extraction, inserting oneself into a subject''''s dreams to 
            obtain hidden information without the subject knowing, a concept 
            taught to him by his professor father-in-law, Dr. Stephen Miles. 
            Dom''''s associates are Miles'''' former students, who Dom requires 
            as he has given up being the dream architect for reasons he 
            won''''t disclose. Dom''''s primary associate, Arthur, believes it 
            has something to do with Dom''''s deceased wife, Mal, who often 
            figures prominently and violently in those dreams, or Dom''''s want 
            to "go home" (get back to his own reality, which includes two 
            young children). Dom''''s work is generally in corporate espionage. 
            As the subjects don''''t want the information to get into the wrong 
            hands, the clients have zero tolerance for failure. Dom is also a 
            wanted man, as many of his past subjects have learned what Dom 
            has done to them. One of those subjects, Mr. Saito, offers Dom a 
            job he can''''t refuse: to take the concept one step further into 
            inception, namely planting thoughts into the subject''''s dreams 
            without them knowing. Inception can fundamentally alter that 
            person as a being. Saito''''s target is Robert Michael Fischer, the 
            heir to an energy business empire, which has the potential to 
            rule the world if continued on the current trajectory. Beyond the 
            complex logistics of the dream architecture of the case and some 
            unknowns concerning Fischer, the biggest obstacles in success for 
            the team become worrying about one aspect of inception which Cobb 
            fails to disclose to the other team members prior to the job, and 
            Cobb''''s newest associate Ariadne''''s belief that Cobb''''s own 
            subconscious, especially as it relates to Mal, may be taking over 
            what happens in the dreams.
        '']
    ) AS result;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (133, 5, 1, 8, 1, '### Question Answering
Question Answering extracts an answer from a given context. Recent progress has enabled models to also specify if the answer is present in the context at all. If you were trying to build a general question answering system, you could first turn the question into a keyword search against Wikipedia articles, and then use a model to retrieve the correct answer from the top hit. Another application would provide automated support from a knowledge base, based on the customers question.

See [question answering documentation](https://huggingface.co/tasks/question-answering) for more options.', '<article class="markdown-body"><h3>Question Answering</h3>
<p>Question Answering extracts an answer from a given context. Recent progress has enabled models to also specify if the answer is present in the context at all. If you were trying to build a general question answering system, you could first turn the question into a keyword search against Wikipedia articles, and then use a model to retrieve the correct answer from the top hit. Another application would provide automated support from a knowledge base, based on the customers question.</p>
<p>See <a href="https://huggingface.co/tasks/question-answering">question answering documentation</a> for more options.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (134, 5, 3, 9, 1, 'SELECT pgml.transform(
        ''question-answering'',
        inputs => ARRAY[
            ''{
                "question": "Am I dreaming?",
                "context": "I got a good nights sleep last night and started a simple tutorial over my cup of morning coffee. The capabilities seem unreal, compared to what I came to expect from the simple SQL standard I studied so long ago. The answer is staring me in the face, and I feel the uncanny call from beyond the screen to check the results."
            }''
        ]
    ) AS answer;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (135, 5, 1, 10, 1, '### Text Generation
If you need to expand on some thoughts, you can have AI complete your sentences for you:', '<article class="markdown-body"><h3>Text Generation</h3>
<p>If you need to expand on some thoughts, you can have AI complete your sentences for you:</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (136, 5, 3, 11, 1, 'SELECT pgml.transform(
        ''text-generation'',
        ''{"num_return_sequences": 2}'',
        ARRAY[''Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'']
    ) AS result;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (137, 5, 1, 12, 1, '### More
There are many different [tasks](https://huggingface.co/tasks) and tens of thousands of state-of-the-art [models](https://huggingface.co/models) available for you to explore. The possibilities are expanding every day. There can be amazing performance improvements in domain specific versions of these general tasks by fine tuning published models on your dataset. See the next section for [fine tuning](/user_guides/transformers/fine_tuning/) demonstrations.', '<article class="markdown-body"><h3>More</h3>
<p>There are many different <a href="https://huggingface.co/tasks">tasks</a> and tens of thousands of state-of-the-art <a href="https://huggingface.co/models">models</a> available for you to explore. The possibilities are expanding every day. There can be amazing performance improvements in domain specific versions of these general tasks by fine tuning published models on your dataset. See the next section for <a href="/user_guides/transformers/fine_tuning/">fine tuning</a> demonstrations.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (138, 6, 3, 3, 1, 'SELECT pgml.add(ARRAY[1.0::real, 2.0, 3.0], 3);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (139, 6, 3, 4, 1, 'SELECT pgml.subtract(ARRAY[1.0::real, 2.0, 3.0], 3);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (140, 6, 3, 5, 1, 'SELECT pgml.multiply(ARRAY[1.0::real, 2.0, 3.0], 3);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (141, 6, 3, 6, 1, 'SELECT pgml.divide(ARRAY[1.0::real, 2.0, 3.0], 100);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (142, 6, 1, 7, 1, '### Pairwise arithmetic', '<article class="markdown-body"><h3>Pairwise arithmetic</h3></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (143, 6, 3, 8, 1, 'SELECT pgml.add(ARRAY[1.0::real, 2.0, 3.0], ARRAY[4.0, 5.0, 6.0]);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (144, 6, 3, 9, 1, 'SELECT pgml.subtract(ARRAY[1.0::real, 2.0, 3.0], ARRAY[4.0, 5.0, 6.0]);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (145, 6, 3, 10, 1, 'SELECT pgml.multiply(ARRAY[1.0::real, 2.0, 3.0], ARRAY[4.0, 5.0, 6.0]);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (146, 6, 3, 11, 1, 'SELECT pgml.divide(ARRAY[1.0::real, 2.0, 3.0], ARRAY[4.0, 5.0, 6.0]);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (147, 6, 1, 12, 1, '### Norms', '<article class="markdown-body"><h3>Norms</h3></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (148, 6, 3, 13, 1, 'SELECT pgml.norm_l0(ARRAY[1.0::real, 2.0, 3.0]);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (149, 6, 3, 14, 1, 'SELECT pgml.norm_l1(ARRAY[1.0::real, 2.0, 3.0]);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (150, 6, 3, 15, 1, 'SELECT pgml.norm_l2(ARRAY[1.0::real, 2.0, 3.0]);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (151, 6, 3, 16, 1, 'SELECT pgml.norm_max(ARRAY[1.0::real, 2.0, 3.0]);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (160, 6, 3, 25, 1, 'SELECT pgml.cosine_similarity(ARRAY[1.0::real, 2.0, 3.0], ARRAY[1.0::real, 2.0, 3.0]);', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (161, 6, 1, 26, 1, '### Generating Random Embeddings

We can populate a table of embeddings with 10,000 rows that have a 128 dimension embedding to demonstrate some vector functionality like nearest neighbor search.', '<article class="markdown-body"><h3>Generating Random Embeddings</h3>
<p>We can populate a table of embeddings with 10,000 rows that have a 128 dimension embedding to demonstrate some vector functionality like nearest neighbor search.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (162, 6, 3, 27, 1, 'CREATE TABLE embeddings AS
SELECT id, ARRAY_AGG(rand) AS vector
FROM (
  SELECT row_number() over () % 10000 + 1 AS id, random()::REAL AS rand
  FROM generate_series(1, 1280000) AS t
) series
GROUP BY id
ORDER BY id;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (163, 6, 3, 28, 1, '-- Nearest neighbors to e1 using cosine similarity
SELECT 
    e1.id, 
    e2.id,
    pgml.cosine_similarity(e1.vector, e2.vector) AS distance
FROM embeddings e1
JOIN embeddings e2 ON 1=1
WHERE e1.id = 1
ORDER BY distance DESC
LIMIT 10;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (164, 7, 1, 3, 1, '## Rolling back to a specific algorithm
Rolling back creates a new deployment for the model that was deployed before the current one. Multiple rollbacks in a row will effectively oscillate between the two most recently deployed models, making rollbacks a relatively safe operation.', '<article class="markdown-body"><h2>Rolling back to a specific algorithm</h2>
<p>Rolling back creates a new deployment for the model that was deployed before the current one. Multiple rollbacks in a row will effectively oscillate between the two most recently deployed models, making rollbacks a relatively safe operation.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (165, 7, 3, 4, 1, 'SELECT * FROM pgml.deploy(''Handwritten Digits'', ''rollback'', ''svm'');', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (166, 8, 3, 7, 1, 'SELECT id, relation_name, test_sampling::TEXT FROM pgml.snapshots LIMIT 10;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (167, 8, 3, 8, 1, 'SELECT * FROM pgml.snapshot_1 LIMIT 10;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (168, 8, 1, 9, 1, '## Deployments

Deployments happen automatically if a new project has a better key metric after training, or when triggered manually. You can view all deployments.', '<article class="markdown-body"><h2>Deployments</h2>
<p>Deployments happen automatically if a new project has a better key metric after training, or when triggered manually. You can view all deployments.</p></article>', NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (169, 8, 3, 10, 1, 'SELECT id, model_id, strategy::TEXT FROM pgml.deployments LIMIT 10;', NULL, NULL, NULL);
INSERT INTO pgml.notebook_cells VALUES (170, 9, 1, 1, 1, '## Native Installation

A PostgresML deployment consists of two different runtimes. The foundational runtime is a Python extension for Postgres ([pgml-extension](https://github.com/postgresml/postgresml/tree/master/pgml-extension/)) that facilitates the machine learning lifecycle inside the database. Additionally, we provide a dashboard ([pgml-dashboard](https://github.com/postgresml/postgresml/tree/master/pgml-dashboard/)) that can connect to your Postgres server and provide additional management functionality. It will also provide visibility into the models you build and data they use.

Check out our documentation for [installation instructions](https://postgresml.org/user_guides/setup/native_installation/) in your datacenter.

We''d also love to hear your feedback. 

- Email us at team@postgresml.org
- Start a [discussion on github](https://github.com/postgresml/postgresml/discussions)', '<article class="markdown-body"><h2>Native Installation</h2>
<p>A PostgresML deployment consists of two different runtimes. The foundational runtime is a Python extension for Postgres (<a href="https://github.com/postgresml/postgresml/tree/master/pgml-extension/">pgml-extension</a>) that facilitates the machine learning lifecycle inside the database. Additionally, we provide a dashboard (<a href="https://github.com/postgresml/postgresml/tree/master/pgml-dashboard/">pgml-dashboard</a>) that can connect to your Postgres server and provide additional management functionality. It will also provide visibility into the models you build and data they use.</p>
<p>Check out our documentation for <a href="https://postgresml.org/user_guides/setup/native_installation/">installation instructions</a> in your datacenter.</p>
<p>We''d also love to hear your feedback. </p>
<ul>
<li>Email us at team@postgresml.org</li>
<li>Start a <a href="https://github.com/postgresml/postgresml/discussions">discussion on github</a></li>
</ul></article>', NULL, NULL);


--
-- Name: pgml.notebook_cells_id_seq; Type: SEQUENCE SET; Schema:  Owner: lev
--

SELECT pg_catalog.setval('pgml.notebook_cells_id_seq', (SELECT MAX(id) + 1 FROM pgml.notebook_cells), true);


--
-- PostgreSQL database dump complete
--

