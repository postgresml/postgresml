<style>
.md-content video, .md-content img {
  max-width: 90%;
  margin: 2em 5%;
}
</style>

# Quick Start

PostgresML is easy to get started with. If you haven't already, sign up for our [Gym](<%- crate::utils::config::signup_url() %>) to get a free hosted PostgresML instance you can use to follow this tutorial. You can also run one yourself by following the instructions in our Github repo.

<p align="center" markdown>
  [Try PostgresML](<%- crate::utils::config::signup_url() %>){ .md-button .md-button--primary .md-button }
</p>

<video autoplay loop muted>
   <source src="https://static.postgresml.org/postgresml-org-static/gym_demo.webm" type="video/webm">
   <source src="https://static.postgresml.org/postgresml-org-static/gym_demo.mp4" type="video/mp4">
  <img src="/images/console.png" alt="PostgresML in practice" loading="lazy">
</video>

Once you have your PostgresML instance running, we'll be ready to get started.

## Get data

The first part of machine learning is getting your data in a format you can use. That's usually the hardest part, but thankfully we have a few example datasets we can use. To load one of them, navigate to the IDE tab and run this query:

```postgresql
SELECT * FROM pgml.load_dataset('diabetes');
```

You should see something like this:

![IDE](/dashboard/static/images/gym/ide.png)

We have more example [Scikit datasets](https://scikit-learn.org/stable/datasets/toy_dataset.html) available:

- `iris` (classification),
- `digits` (classification),
- `wine` (regression),

To load them into PostgresML, use the same function above with the desired dataset name as parameter. They will become available in the `pgml` schema as `pgml.iris`, `pgml.digits` and `pgml.wine` respectively.

## Browse data

The SQL editor you just used can run arbitrary queries on the PostgresML instance. For example,
if we want to see what dataset we just loaded looks like, we can run:

```postgresql
SELECT * FROM pgml.diabetes LIMIT 5;
```

![Data](/dashboard/static/images/gym/data.png)

The `diabetes` dataset is a toy (small, not realistic) dataset published by Scikit Learn. It contains ten feature columns and one label column:

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

This dataset is not realistic because all data is perfectly arranged and normalized, which won't be the case with most real world datasets you'll run into, but it's perfect for our quick tutorial.

Alright, we're ready to do some machine learning!

## First project

PostgresML organizes itself into projects. A project is just a name for model(s) trained on a particular dataset. Let's create our first project by training an XGBoost regression model on our diabetes dataset.

Using the IDE, run:

```postgresql
SELECT * FROM pgml.train(
	'My First Project',
	task => 'regression',
	relation_name => 'pgml.diabetes',
	y_column_name => 'target',
	algorithm => 'xgboost');
```

You should see this:

![Train](/dashboard/static/images/gym/train.png)

By executing `pmgl.train()` we did the following:

- created a project called "My First Project",
- snapshotted the table `pgml.diabetes` thus making the experiment reproducible (in case data changes, as it happens in the real world),
- trained an XGBoost regression model on the data contained in the `pgml.diabetes` table using the column `target` as the label,
- deployed the model into production.

We're ready to predict novel data points!

## Inference

Inference is the act of predicting labels that we haven't necessarily used in training. That's the whole point of machine learning really: predict something we haven't seen before.

Let's try and predict some new values. Using the IDE, run:

```postgresql
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

![Prediction](/dashboard/static/images/gym/predict.png)

The `prediction` column represents the possible value of the `target` column given the new features we just passed into the `pgml.predict()` function. You can just as easily predict multiple points and compare them to the actual labels in the dataset:

```postgresql
SELECT
	pgml.predict('My First Project 2', ARRAY[
		age, sex, bmi, bp, s1, s3, s3, s4, s5, s6
	]),
    target
FROM pgml.diabetes LIMIT 10;
```

Sometimes the model will be pretty close, but sometimes it will be way off. That's why we'll be training several of them and comparing them next.

## Browse around

By creating our first project, we made the Dashboard a little bit more interesting. This is how the `pgml.diabetes` snapshot we just created looks like:

![Snapshot](/dashboard/static/images/gym/snapshot.png)

As you can see, we automatically performed some analysis on the data. Visualizing the data is important to understand how it could potentially behave given different models, and maybe even predict how it could evolve in the future.

XGBoost is a good algorithm, but what if there are better ones? Let's try training a few more using the IDE. Run these one at a time:

```postgresql
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

![Trained Algorithms](/dashboard/static/images/gym/trained_models.png)

Huh, apparently XGBoost isn't as good we originally thought! In this case, a simple linear regression did significantly better than all the others. It's hard to know which algorithm will perform best given a dataset; even experienced machine learning engineers get this one wrong.

With PostgresML, you needn't worry: you can train all of them and see which one does best for your data. PostgresML will automatically use the best one for inference.

## Conclusion

Congratulations on becoming a Machine Learning engineer. If you thought ML was scary or mysterious, we hope that this small tutorial made it a little bit more approachable.

This is the first of many tutorials we'll publish, so stay tuned. Happy machine learning!
