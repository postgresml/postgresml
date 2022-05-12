# Deployments

Deployments are an artifact of calls to `pgml.deploy`. See [deployments](../guides/deployments/) for ways to create new deployments.

![Deployment](../images/deployment.png)

## Schema

```sql linenums="1"
pgml.deployments(
	id BIGSERIAL PRIMARY KEY,
	project_id BIGINT NOT NULL,
	model_id BIGINT NOT NULL,
	strategy TEXT NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	CONSTRAINT project_id_fk FOREIGN KEY(project_id) REFERENCES pgml.projects(id),
	CONSTRAINT model_id_fk FOREIGN KEY(model_id) REFERENCES pgml.models(id)
);
```
