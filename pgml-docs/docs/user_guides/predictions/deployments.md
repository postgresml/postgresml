# Deployments

Models are automatically deployed if their key metric (__R__<sup>2</sup> for regression, __F__<sub>1</sub> for classification) is improved over the currently deployed version during training. If you want to manage deploys manually, you can always change which model is currently responsible for making predictions.


## API

```sql linenums="1" title="pgml.deploy"
pgml.deploy(
	project_name TEXT,                            -- Human-friendly project name
	strategy pgml.strategy DEFAULT 'best_score',  -- 'rollback', 'best_score', or 'most_recent'
	algorithm pgml.algorithm DEFAULT NULL         -- filter candidates to a particular algorithm, NULL = all qualify
)
```

## Strategies
There are 3 different deployment strategies available

strategy | description
--- | ---
most_recent | The most recently trained model for this project
best_score | The model that achieved the best key metric score
rollback | The model that was previously deployed for this project

The default deployment behavior allows any algorithm to qualify.

=== "SQL"

	```sql linenums="1"
	SELECT * FROM pgml.deploy('Handwritten Digit Image Classifier', 'best_score');
	```

=== "Output"

	```sql linenums="1"
                project_name            |    strategy    | algorithm
	------------------------------------+----------------+----------------
	 Handwritten Digit Image Classifier | classification | linear
	(1 row)
	```

## Specific Algorithms
Deployment candidates can be restricted to a specific algorithm by including the `algorithm` parameter.

=== "SQL"

	```sql linenums="1"
	SELECT * FROM pgml.deploy(
        project_name => 'Handwritten Digit Image Classifier', 
        strategy => 'best_score', 
        algorithm => 'svm'
    );
	```

=== "Output"

	```sql linenums="1"
                project_name            |    strategy    | algorithm
	------------------------------------+----------------+----------------
	 Handwritten Digit Image Classifier | classification | svm
	(1 row)
	```


## Rolling back to a specific algorithm
Rolling back creates a new deployment for the model that was deployed before the current one. Multiple rollbacks in a row will effectively oscillate between the two most recently deployed models, making rollbacks a relatively safe operation. 

=== "SQL"

	```sql linenums="1"
	SELECT * FROM pgml.deploy('Handwritten Digit Image Classifier', 'rollback', 'svm');
	```

=== "Output"

	```sql linenums="1"
                project_name            |    strategy    | algorithm
	------------------------------------+----------------+----------------
	 Handwritten Digit Image Classifier | classification | svm
	(1 row)
	```

## Manual Deploys

You can also manually deploy any previously trained model by inserting a new record into `pgml.deployments`. You will need to query the `pgml.projects` and `pgml.models` tables to find the desired IDs.

!!! note 
    Deployed models are cached at the session level to improve prediction times. Manual deploys created this way will not invalidate those caches, so active sessions will not use manual deploys until they reconnect. 

=== "SQL"

	```sql linenums="1"
	INSERT INTO pgml.deploys (project_id, model_id, strategy,) VALUES (1, 1, 'rollback');
	```

=== "Output"

	```sql linenums="1"
    INSERT 0 1
	```
