-- load the diamonds dataset, that contains text categorical variables
SELECT pgml.load_dataset('jdxcosta/diamonds');

-- view the data
SELECT * FROM pgml."jdxcosta/diamonds" LIMIT 10;

-- drop the Unamed column, since it's not useful for training (you could create a view instead)
ALTER TABLE pgml."jdxcosta/diamonds" DROP COLUMN "Unnamed: 0";

-- train a model using preprocessors to scale the numeric variables, and target encode the categoricals
SELECT pgml.train(
       project_name => 'Diamond prices',
       task => 'regression',
       relation_name => 'pgml.jdxcosta/diamonds',
       y_column_name => 'price',
       algorithm => 'lightgbm',
       preprocess => '{
                      "carat": {"scale": "standard"},
                      "depth": {"scale": "standard"},
                      "table": {"scale": "standard"},
                      "cut": {"encode": "target", "scale": "standard"},
                      "color": {"encode": "target", "scale": "standard"},
                      "clarity": {"encode": "target", "scale": "standard"}
                  }'
);

-- run some predictions, notice we're passing a heterogeneous row (tuple) as input, rather than a homogenous ARRAY[].
SELECT price, pgml.predict('Diamond prices', (carat, cut, color, clarity, depth, "table", x, y, z)) AS prediction
FROM pgml."jdxcosta/diamonds"
LIMIT 10;

-- This is a difficult dataset for more algorithms, which makes it a good challenge for preprocessing, and additional
-- feature engineering. What's next?
