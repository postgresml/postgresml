# Data Pre-processing

The training function also provides the option to preprocess data with the `preprocess` param. Preprocessors can be configured on a per-column basis for the training data set. There are currently three types of preprocessing available, for both categorical and quantitative variables. Below is a brief example for training data to learn a model of whether we should carry an umbrella or not.

{% hint style="info" %}
Preprocessing steps are saved after training, and repeated identically for future calls to `pgml.predict()`.
{% endhint %}

#### `weather_data`

| month | clouds    | humidity | temp |       |
| ----- | --------- | -------- | ---- | ----- |
| 'jan' | 'cumulus' | 0.8      | 5    | true  |
| 'jan' | NULL      | 0.1      | 10   | false |
| …     | …         | …        | …    | …     |
| 'dec' | 'nimbus'  | 0.9      | -2   | false |

In this example:

* `month` is an ordinal categorical `TEXT` variable
* `clouds` is a nullable nominal categorical `INT4` variable
* `humidity` is a continuous quantitative `FLOAT4` variable
* `temp` is a discrete quantitative `INT4` variable
* `rain` is a nominal categorical `BOOL` label

There are 3 steps to preprocessing data:

* [Encoding](data-pre-processing.md#categorical-encodings) categorical values into quantitative values
* [Imputing](data-pre-processing.md#imputing-missing-values) NULL values to some quantitative value
* [Scaling](data-pre-processing.md#scaling-values) quantitative values across all variables to similar ranges

These preprocessing steps may be specified on a per-column basis to the [train()](./) function. By default, PostgresML does minimal preprocessing on training data, and will raise an error during analysis if NULL values are encountered without a preprocessor. All types other than `TEXT` are treated as quantitative variables and cast to floating point representations before passing them to the underlying algorithm implementations.

```postgresql
SELECT pgml.train(
    project_name => 'preprocessed_model', 
    task => 'classification', 
    relation_name => 'weather_data',
    target => 'rain', 
    preprocess => '{
        "month":    {"encode": {"ordinal": ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"]}}
        "clouds":   {"encode": "target", scale: "standard"}
        "humidity": {"impute": "mean", scale: "standard"}
        "temp":     {"scale": "standard"}
    }'
);
```

In some cases, it may make sense to use multiple steps for a single column. For example, the `clouds` column will be target encoded, and then scaled to the standard range to avoid dominating other variables, but there are some interactions between preprocessors to keep in mind.

* `NULL` and `NaN` are treated as additional, independent categories if seen during training, so columns that `encode` will only ever `impute` novel when novel data is encountered during training values.
* It usually makes sense to scale all variables to the same scale.
* It does not usually help to scale or preprocess the target data, as that is essentially the problem formulation and/or task selection.

{% hint style="info" %}
`TEXT` is used in this document to also refer to `VARCHAR` and `CHAR(N)` types.
{% endhint %}

## Predicting with Preprocessors

A model that has been trained with preprocessors should use a Postgres tuple for prediction, rather than a `FLOAT4[]`. Tuples may contain multiple different types (like `TEXT` and `BIGINT`), while an ARRAY may only contain a single type. You can use parenthesis around values to create a Postgres tuple.

```postgresql
SELECT pgml.predict('preprocessed_model', ('jan', 'nimbus', 0.5, 7));
```

## Categorical encodings

Encoding categorical variables is an O(N log(M)) where N is the number of rows, and M is the number of distinct categories.

| name      | description                                                                                                                                     |
| --------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `none`    | **Default** - Casts the variable to a 32-bit floating point representation compatible with numerics. This is the default for non-`TEXT` values. |
| `target`  | Encodes the variable as the mean value of the target label for all members of the category. This is the default for `TEXT` variables.           |
| `one_hot` | Encodes the variable as multiple independent boolean columns.                                                                                   |
| `ordinal` | Encodes the variable as integer values provided by their position in the input array. NULLS are always 0.                                       |

### `target` encoding

Target encoding is a relatively efficient way to represent a categorical variable. The average value of the target is computed for each category in the training data set. It is reasonable to `scale` target encoded variables using the same method as other variables.

```postgresql
preprocess => '{
    "clouds": {"encode": "target" }
}'
```

!!! note

Target encoding is currently limited to the first label column specified in a joint optimization model when there are multiple labels.

!!!

### `one_hot` encoding

One-hot encoding converts each category into an independent boolean column, where all columns are false except the one column the instance is a member of. This is generally not as efficient or as effective as target encoding because the number of additional columns for a single feature can swamp the other features, regardless of scaling in some algorithms. In addition, the columns are highly correlated which can also cause quality issues in some algorithms. PostgresML drops one column by default to break the correlation but preserves the information, which is also referred to as dummy encoding.

```
preprocess => '{
    "clouds": {"encode": "one_hot" }
}
```

!!! note

All one-hot encoded data is scaled from 0-1 by definition, and will not be further scaled, unlike the other encodings which are scaled.

!!!

### `ordinal` encoding

Some categorical variables have a natural ordering, like months of the year, or days of the week that can be effectively treated as a discrete quantitative variable. You may set the order of your categorical values, by passing an exhaustive ordered array. e.g.

```
preprocess => '{
    "month": {"encode": {"ordinal": ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"]}}
}
```

## Imputing missing values

`NULL` and `NaN` values can be replaced by several statistical measures observed in the training data.

| **name** | **description**                                                                      |
| -------- | ------------------------------------------------------------------------------------ |
| `error`  | **Default** - will abort training or inference when a `NULL` or `NAN` is encountered |
| `mean`   | the mean value of the variable in the training data set                              |
| `median` | the middle value of the variable in the sorted training data set                     |
| `mode`   | the most common value of the variable in the training data set                       |
| `min`    | the minimum value of the variable in the training data set                           |
| `max`    | the maximum value of the variable in the training data set                           |
| `zero`   | replaces all missing values with 0.0                                                 |

```postgresql
preprocess => '{
    "temp": {"impute": "mean"}
}'
```

## Scaling values

Scaling all variables to a standardized range can help make sure that no feature dominates the model, strictly because it has a naturally larger scale.

| **name**   | **description**                                                                                                      |
| ---------- | -------------------------------------------------------------------------------------------------------------------- |
| `preserve` | **Default** - Does not scale the variable at all.                                                                    |
| `standard` | Scales data to have a mean of zero, and variance of one.                                                             |
| `min_max`  | Scales data from zero to one. The minimum becomes 0.0 and maximum becomes 1.0.                                       |
| `max_abs`  | Scales data from -1.0 to +1.0. Data will not be centered around 0, unless abs(min) == abs(max).                      |
| `robust`   | Scales data as a factor of the first and third quartiles. This method may handle outliers more robustly than others. |

```postgresql
preprocess => '{
    "temp": {"scale": "standard"}
}'
```
