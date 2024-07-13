# Quick Start with Docker

To try PostgresML on your system for the first time, [Docker](https://docs.docker.com/engine/install/) is a great tool to get you started quicky. We've prepared a Docker image that comes with the latest version of PostgresML and all of its dependencies. If you have Nvidia GPUs on your machine, you'll also be able to use GPU acceleration.

!!! tip

If you're looking to get started with PostgresML as quickly as possible, [sign up](https://postgresml.org/signup) for our free serverless [cloud](https://postgresml.org/signup). You'll get a database in seconds, and will be able to use all the latest Hugging Face models on modern GPUs.

!!!

## Get Started

{% tabs %}
{% tab title="macOS" %}
```bash
docker run \
    -it \
    -v postgresml_data:/var/lib/postgresql \
    -p 5433:5432 \
    -p 8000:8000 \
    ghcr.io/postgresml/postgresml:2.7.13 \
    sudo -u postgresml psql -d postgresml
```
{% endtab %}

{% tab title="Linux with GPUs" %}
Make sure you have Cuda, the Cuda container toolkit, and matching graphics drivers installed. You can install everything from [Nvidia](https://developer.nvidia.com/cuda-downloads).

On Ubuntu, you can install everything with:

```bash
sudo apt install -y \
    cuda \
    cuda-container-toolkit
```

To run the container with GPU capabilities:

```bash
docker run \
    -it \
    -v postgresml_data:/var/lib/postgresql \
    --gpus all \
    -p 5433:5432 \
    -p 8000:8000 \
    ghcr.io/postgresml/postgresml:2.7.3 \
    sudo -u postgresml psql -d postgresml
```

If your machine doesn't have a GPU, just omit the `--gpus all` option, and the container will start and use the CPU instead.
{% endtab %}

{% tab title="Windows" %}
Install [WSL](https://learn.microsoft.com/en-us/windows/wsl/install) and [Docker Desktop](https://www.docker.com/products/docker-desktop/). You can then use **Linux with GPUs** instructions. GPU support is included, make sure to [enable CUDA](https://learn.microsoft.com/en-us/windows/ai/directml/gpu-cuda-in-wsl).

Once the container is running, setting up PostgresML is as simple as creating the extension and running a few queries to make sure everything is working correctly.
{% endtab %}
{% endtabs %}

!!! generic

!!! code\_block time="41.520ms"

```postgresql
CREATE EXTENSION IF NOT EXISTS pgml;
SELECT pgml.version();
```

!!!

!!! results

```
postgresml=# CREATE EXTENSION IF NOT EXISTS pgml;
INFO:  Python version: 3.10.6 (main, May 29 2023, 11:10:38) [GCC 11.3.0]
INFO:  Scikit-learn 1.2.2, XGBoost 1.7.5, LightGBM 3.3.5, NumPy 1.25.1
CREATE EXTENSION
Time: 41.520 ms

postgresml=# SELECT pgml.version();
 version 
---------
 2.9.2
(1 row)
```

!!!

!!!

You can continue using the command line, or connect to the container using any of the commonly used PostgreSQL tools like `psql`, pgAdmin, DBeaver, and others:

```bash
psql -h 127.0.0.1 -p 5433 -U postgresml
```

## Workflows

PostgresML allows you to generate embeddings with open source models from Hugging Face, easily prompt LLMs with tasks like translation and text generation, and train classical machine learning models on tabular data.

### Embeddings

To generate an embedding, all you have to do is use the `pgml.embed(model_name, text)` function with any open source model available on Hugging Face.

!!! example

!!! code\_block time="51.907ms"

```postgresql
SELECT pgml.embed(
    'Alibaba-NLP/gte-base-en-v1.5',
    'passage: PostgresML is so easy!'
);
```

!!!

!!! results

```
postgres=# SELECT pgml.embed(
    'Alibaba-NLP/gte-base-en-v1.5',
    'passage: PostgresML is so easy!'
);

{0.02997742,-0.083322115,-0.074212186,0.016167048,0.09899471,-0.08137268,-0.030717574,0.03474584,-0.078880586,0.053087912,-0.027900297,-0.06316991,
 0.04218509,-0.05953648,0.028624319,-0.047688972,0.055339724,0.06451558,-0.022694778,0.029539965,-0.03861752,-0.03565117,0.06457901,0.016581751,
0.030634841,-0.026699776,-0.03840521,0.10052487,0.04131341,-0.036192447,0.036209006,-0.044945586,-0.053815156,0.060391728,-0.042378396,
 -0.008441956,-0.07911099,0.021774381,0.034313954,0.011788908,-0.08744744,-0.011105505,0.04577902,0.0045646844,-0.026846683,-0.03492123,0.068385094,
-0.057966642,-0.04777695,0.11460253,0.010138827,-0.0023120022,0.052329376,0.039127126,-0.100108854,-0.03925074,-0.0064703166,-0.078960024,-0.046833295,
0.04841002,0.029004619,-0.06588247,-0.012441916,0.001127402,-0.064730585,0.05566701,-0.08166461,0.08834854,-0.030919826,0.017261868,-0.031665307,
0.039764903,-0.0747297,-0.079097,-0.063424855,0.057243366,-0.025710078,0.033673875,0.050384883,-0.06700917,-0.020863676,0.001511638,-0.012377004,
-0.01928165,-0.0053149736,0.07477675,0.03526208,-0.033746846,-0.034142617,0.048519857,0.03142429,-0.009989936,-0.018366965,0.098441005,-0.060974542,
0.066505,-0.013180869,-0.067969725,0.06731659,-0.008099243,-0.010721313,0.06885249,-0.047483806,0.004565877,-0.03747329,-0.048288923,-0.021769432,
0.033546787,0.008165753,-0.0018901207,-0.05621888,0.025734955,-0.07408746,-0.053908117,-0.021819277,0.045596648,0.0586417,0.0057576317,-0.05601786,
-0.03452876,-0.049566686,-0.055589233,0.0056059696,0.034660816,0.018012922,-0.06444576,0.036400944,-0.064374834,-0.019948835,-0.09571418,0.09412033,-0.07085108,0.039256454,-0.030016104,-0.07527431,-0.019969895,-0.09996753,0.008969355,0.016372273,0.021206321,0.0041883467,0.032393526,0.04027315,-0.03194125,-0.03397957,-0.035261292,0.061776843,0.019698814,-0.01767779,0.018515844,-0.03544395,-0.08169962,-0.02272048,-0.0830616,-0.049991447,-0.04813149,-0.06792019,0.031181566,-0.04156394,-0.058702122,-0.060489867,0.0020844154,0.18472219,0.05215536,-0.038624488,-0.0029086764,0.08512023,0.08431501,-0.03901469,-0.05836445,0.118146114,-0.053862963,0.014351494,0.0151984785,0.06532256,-0.056947585,0.057420347,0.05119938,0.001644649,0.05911524,0.012656099,-0.00918104,-0.009667282,-0.037909098,0.028913427,-0.056370094,-0.06015602,-0.06306665,-0.030340875,-0.14780329,0.0502743,-0.039765555,0.00015358179,0.018831518,0.04897686,0.014638214,-0.08677867,-0.11336724,-0.03236903,-0.065230116,-0.018204475,0.022788873,0.026926292,-0.036414392,-0.053245157,-0.022078559,-0.01690316,-0.042608887,-0.000196666,-0.0018297597,-0.06743311,0.046494357,-0.013597083,-0.06582122,-0.065659754,-0.01980711,0.07082651,-0.020514658,-0.05147128,-0.012459332,0.07485931,0.037384395,-0.03292486,0.03519196,0.014782926,-0.011726298,0.016492695,-0.0141114695,0.08926231,-0.08323172,0.06442687,0.03452826,-0.015580203,0.009428933,0.06759306,0.024144053,0.055612188,-0.015218529,-0.027584016,0.1005267,-0.054801818,-0.008317948,-0.000781896,-0.0055441647,0.018137401,0.04845575,0.022881811,-0.0090647405,0.00068219384,-0.050285354,-0.05689162,0.015139549,0.03553917,-0.09011886,0.010577362,0.053231273,0.022833975,-3.470906e-05,-0.0027906548,-0.03973121,0.007263015,0.00042456342,0.07092535,-0.043497834,-0.0015815622,-0.03489149,0.050679605,0.03153052,0.037204932,-0.13364139,-0.011497628,-0.043809805,0.045094978,-0.037943177,0.0021411474,0.044974167,-0.05388966,0.03780391,0.033220228,-0.027566046,-0.043608706,0.021699436,-0.011780484,0.04654962,-0.04134961,0.00018980364,-0.0846228,-0.0055453447,0.057337128,0.08390022,-0.019327229,0.10235083,0.048388377,0.042193796,0.025521005,0.013201268,-0.0634062,-0.08712715,0.059367906,-0.007045281,0.0041695046,-0.08747506,-0.015170839,-0.07994115,0.06913491,0.06286314,0.030512255,0.0141608,0.046193067,0.0026272296,0.057590637,-0.06136263,0.069828056,-0.038925823,-0.076347575,0.08457048,0.076567,-0.06237806,0.06076619,0.05488552,-0.06070616,0.10767283,0.008605431,0.045823734,-0.0055780583,0.043272685,-0.05226901,0.035603754,0.04357865,-0.061862156,0.06919797,-0.00086810143,-0.006476894,-0.043467253,0.017243104,-0.08460669,0.07001912,0.025264058,0.048577853,-0.07994533,-0.06760861,-0.034988943,-0.024210323,-0.02578568,0.03488276,-0.0064449264,0.0345789,-0.0155197615,0.02356351,0.049044855,0.0497944,0.053986903,0.03198324,0.05944599,-0.027359396,-0.026340311,0.048312716,-0.023747599,0.041861262,0.017830249,0.0051145423,0.018402847,0.027941752,0.06337417,0.0026447168,-0.057954717,-0.037295196,0.03976777,0.057269543,0.09760822,-0.060166832,-0.039156828,0.05768707,0.020471212,0.013265894,-0.050758235,-0.020386606,0.08815887,-0.05172276,-0.040749934,0.01554588,-0.017021973,0.034403082,0.12543736}
```

!!!

!!!

### Training an XGBoost model

#### Importing a dataset

PostgresML comes with a few built-in datasets. You can also import your own CSV files or data from other sources like BigQuery, S3, and other databases or files. For our example, let's import the `digits` dataset from Scikit:

!!! generic

!!! code\_block time="47.532ms"

```postgresql
SELECT * FROM pgml.load_dataset('digits');
```

!!!

!!! results

```
postgres=# SELECT * FROM pgml.load_dataset('digits');
 table_name  | rows 
-------------+------
 pgml.digits | 1797
(1 row)
```

!!!

!!!

#### Training a model

The heart of PostgresML is its `pgml.train()` function. Using only that function, you can load the data from any table or view in the database, train any number of ML models on it, and deploy the best model to production.

!!! generic

!!! code\_block time="222.206ms"

```postgresql
SELECT * FROM pgml.train(
    project_name => 'My First PostgresML Project',
    task => 'classification',
    relation_name => 'pgml.digits',
    y_column_name => 'target',
    algorithm => 'xgboost',
    hyperparams => '{
        "n_estimators": 25
    }'
);
```

!!!

!!! results

```
postgres=# SELECT * FROM pgml.train(
    project_name => 'My First PostgresML Project',
    task => 'classification',
    relation_name => 'pgml.digits',
    y_column_name => 'target',
    algorithm => 'xgboost',
    hyperparams => '{
        "n_estimators": 25
    }'
);

[...]

INFO:  Metrics: {
    "f1": 0.88244045,
    "precision": 0.8835865,
    "recall": 0.88687027,
    "accuracy": 0.8841871,
    "mcc": 0.87189955,
    "fit_time": 0.7631203,
    "score_time": 0.007338208
}
INFO:  Deploying model id: 1
           project           |      task      | algorithm | deployed 
-----------------------------+----------------+-----------+----------
 My First PostgresML Project | classification | xgboost   | t
(1 row)
```

!!!

!!!

#### Making predictions

After training a model, you can use it to make predictions. PostgresML provides a `pgml.predict(project_name, features)` function which makes real time predictions using the best deployed model for the given project:

!!! generic

!!! code\_block time="8.676ms"

```postgresql
SELECT 
    target,
    pgml.predict('My First PostgresML Project', image) AS prediction
FROM pgml.digits
LIMIT 5;
```

!!!

!!! results

```
 target | prediction 
--------+------------
      0 |          0
      1 |          1
      2 |          2
      3 |          3
      4 |          4
```

!!!

!!!

#### Automation of common ML tasks

The following common machine learning tasks are performed automatically by PostgresML:

1. Snapshot the data so the experiment is reproducible
2. Split the dataset into train and test sets
3. Train and validate the model
4. Save it into the model store (a Postgres table)
5. Load it and cache it during inference

Check out our Training and Predictions documentation for more details. Some more advanced topics like hyperparameter search and GPU acceleration are available as well.

## Dashboard

The Dashboard app is running on [localhost:8000](http://localhost:8000/). You can use it to write experiments in Jupyter-style notebooks, manage projects, and visualize datasets used by PostgresML.
