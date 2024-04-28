# Why PostgresML?

PostgresML offers a unique and modern architecture which replaces service-based machine learning applications with a single database. The benefits of this approach are measurable in performance, ease of use, and data integrity.

## Service-based architecture

Most applications today are built using services. In the extreme case, microservices with singular purpose are employed to achieve additional separation of concerns.

For an application to use a machine learning model, it is typical to build and maintain separate services and data synchronization pipelines, to allow machine learning engineers that work in Python to build and deploy their models separately and independently from application engineering.

<figure>
	<img src="/docs/.gitbook/assets/performance_1.png" alt="Before PostgresML" width="80%">
	<figcaption class="mt-4"><i>Service-based machine learning architecture</i></figcaption>
</figure>

### Impact

Building on top of service-based architecture has major performance disadvantages. Any task that may fall outside the domain of the engineering team that built the service, like machine learning, will require additional communication between teams, and additional services to be built and maintained.

Communication between services is done with protocols like gRPC or HTTP, which being stateless, require additional context fetched from a database or a cache. Since communication happens over the network, serialization and deserialization of data is required, costing additional time and resources.

The diagram above illustrates the work required to service a single user request. With below-linear scaling characteristics and increasing brittleness, this architecture eventually breaks down and costs teams time, and the organization resources.


## PostgresML architecture

PostgresML simplifies things. By moving machine learning models to the database, we eliminate the need for an additional feature store, data synchronization and inference services, and the need for RPC calls requiring (de)serialization and network latency & reliability costs.

<figure>
	<img src="/docs/.gitbook/assets/performance_2.png" alt="After PostgresML" width="80%">
	<figcaption class="mt-4"><i>PostgresML architecture</i></figcaption>
</figure>
