# Quick Start

PostgresML is really easy to get started with. We'll use one of our example dataset to show you how to use it.

## Get data

Navigate to the IDE tab and run this query:

```sql
SELECT * FROM pgml.load_dataset('diabetes');
```

You should see something like this:

![IDE](/gym/ide.png)

We have more example Scikit datasets avaialble, e.g.:

- `iris`
- `digits`
- `wine`

To load them into PostgresML, use the same function above with the desired dataset name as parameter. They will become available in the `pgml` schema, as `pgml.iris`, `pgml.digits` and `pgml.wine` respectively.

## Browse data

The SQL editor you just used can run arbitrary queries on the PostgresML instance. For example,
if we want to see what dataset we just loaded looks like, we can run:

```sql
SELECT * FROM pgml.diabetes LIMIT 5;
```

![Data](/gym/data.png)

The diabetes dataset is a toy (small, not realistic) dataset published by Scikit Learn. It contains 10 feature columns and one target column:

| **Column** | **Description**                                                      | **Data type** |
|------------|----------------------------------------------------------------------|---------------|
| age        | The age of the patient (in years).                                   | float         |
| sex        | The sex of the patient (normalized).                                 | float         |
| bmi        | The BMI Body Mass index.                                             | float         |
| bp         | Average blood pressure of the patient.                               | float         |
| s1         | Total serum cholesterol.                                             | float         |
| s2         | Low-density lipoproteins.                                            | float         |
| s3         | High-density lipoproteins.                                           | float         |
| s4         | Total cholesterol / HDL.                                             | float         |
| s5         | Possibly log of serum triglycerides level.                           | float         |
| s6         | Blood sugar level.                                                   | float         |
| **target** | Quantitative measure of disease progression one year after baseline. | float         |


This dataset is not realistic because all data is perfectly arranged and normalized, which won't be the case with most datasets you'll run into the real world, but it's perfect for our quick tutorial.


Alright, we're ready to do some machine learning!

## First project

PostgresML organizes itself into projects. A project is just a name for model(s) trained on a particular dataset. Let's create our first project by training an XGBoost
model on our diabetes dataset.

Using the IDE, run:

```sql
SELECT * FROM pgml.train(
	'My First Project',
	task => 'regression',
	relation_name => 'pgml.diabetes',
	y_column_name => 'target',
	algorithm => 'xgboost');
```

You should see this:

![Train](/gym/train.png)

By executing `pmgl.train()` we did the following:

- created a project called "My First Project",
- snapshotted the table `pgml.diabetes` thus making the experiment reproducible (in case data changes, as it happens in the real world),
- trained an XGBoost regression model on the data contained in the `pgml.diabetes` table, using the column `target` as the label,
- deployed the model into production.

We're ready to predict novel data points!

## Inference

Let's try and predict some new values. Using the IDE, run:

```sql
SELECT pgml.predict(
	'My First Project',
	ARRAY[
		0.06, -- age
		0.05, -- sex
		0.05, -- bmi
		-0.0056, -- bp
		0.012191, -- s1
		-0.043401, -- s2
		0.034309, -- s3
		-0.031938, -- s4
		-0.061988, --s5
		-0.031988 -- s6
	]
) AS prediction;
```

You should see something like this:

![Prediction](/gym/predict.png)

Congratulations, you just did machine learning in just a few simple steps!

## Browse around

By creating our first project, we made the Dashboard a little bit more interesting. This is how the `pgml.diabetes` snapshot we just created looks like:

![Snapshot](/gym/snapshot.png)

As you can see, we automatically performed some analysis on the data. Visualizing the data is important to understand how it could potentially behave given different models, and maybe even predict how it could evolve in the future.

XGBoost is a good algorithm, but what if there are better ones? Let's try training a few more using the IDE. Run these one at a time:

```sql
-- Simple linear regression.
SELECT * FROM pgml.train(
	'My First Project',
	algorithm => 'linear');

-- The Lasso (much fancier linear regression).
SELECT * FROM pgml.train(
	'My First Project',
	algorithm => 'lasso'); 
```

If you navigate to the Models tab, you should see all three algorithms you just trained:

![Trained Algorithms](/gym/trained_models.png)

Huh, apparently XGBoost isn't as good we originally thought! In this case, a simple linear regression did significantly better than all the others. It's hard to know which algorithm will perform best given a dataset; even experienced machine learning engineers get this one wrong.

With PostgresML, you needn't worry; you can train all of them and see which one does best for your data. PostgresML will automatically use the best one for inference.

## Conclusion

Congratulations on becoming a Machine Learning engineer. If you thought ML was scary or mysterious, we hope that this small tutorial made it a little bit more approachable.

Keep exploring our other tutorials and try some things on your own. Happy machine learning!
