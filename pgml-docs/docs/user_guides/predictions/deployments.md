# Deployments

Models are automatically deployed if their key metric (__R__<sup>2</sup> for regression, __F__<sub>1</sub> for classification) is improved over the currently deployed version during training. If you want to manage deploys manually, you can always change which model is currently responsible for making predictions.


## API

```sql linenums="1" title="pgml.deploy"
pgml.deploy(
	project_name TEXT,                  -- Human-friendly project name
	strategy TEXT DEFAULT 'best_score', -- 'rollback', 'best_score', or 'most_recent'
	algorithm TEXT DEFAULT NULL    -- filter candidates to a particular algorithm, NULL = all qualify
)
```

The default behavior allows any algorithm to qualify, but deployment candidates can be further restricted to a specific algorithm by passing the `algorithm`.

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


## Strategies
There are 3 different deployment strategies available

strategy | description
--- | ---
most_recent | The most recently trained model for this project
best_score | The model that achieved the best key metric score
rollback | The model that was previously deployed for this project


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
	 Handwritten Digit Image Classifier | classification | linear
	(1 row)
	```

## Manual Deploys

You can also manually deploy any previously trained model by inserting a new record into `pgml.deployments`. You will need to query the `pgml.projects` and `pgml.models` tables to find the desired IDs.

!!! note 
    Deployed models are cached at the session level to improve prediction times. Manual deploys created this way will not invalidate the cache, and active sessions will not see manual deploys until they reconnect. 

=== "SQL"

	```sql linenums="1"
	INSERT INTO pgml.deploys (project_id, model_id, strategy,) VALUES (1, 1, 'rollback');
	```

=== "Output"

	```sql linenums="1"
    INSERT 0 1
	```
