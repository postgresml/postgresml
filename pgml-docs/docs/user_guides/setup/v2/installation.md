# Installation

The PostgresML deployment consists of two parts: the Posgres extension and the dashboard app. The extension provides all the machine learning functionality and can be used independently. The dashboard app provides notebooks for writing experiments and system overview to easier management.

## Extension

The extension can be installed from our Ubuntu `apt` repository or, if you're using a different distribution, from source.

### Install the extension

=== ":material-ubuntu: Ubuntu"

	1. Add our repository into your sources:

		```bash
		echo "deb [trusted=yes] https://apt.postgresml.org $(lsb_release -cs) main" >> /etc/apt/sources.list
		```

	2. Install the extension:

		```
		apt-get update && apt-get install -y postgresql-pgml-14
		```


=== ":material-linux: :material-microsoft: From Source (Linux & WSL)"

	1. Install the latest Rust compiler from [rust-lang.org](https://www.rust-lang.org/learn/get-started).

	2. Clone our git repository:

		```bash
		git clone https://github.com/postgresml/postgresml && \
		cd postgresml/pgml-extension/pgml_rust
		```
	3. Install a [modern version](https://apt.kitware.com/) of CMake.
	4. Install PostgreSQL development headers and other dependencies:

		```bash
		apt-get update && \
		apt-get install -y \
			postgresql-server-14-dev \
			libpython3-dev \
			libclang-dev \
			cmake \
			pkg-config \
			libssl-dev
		```
	5. Install [`pgx`](https://github.com/tcdi/pgx) and build the extension:
		```bash
		cargo install cargo-pgx && \
		cargo pgx init --pg14 /usr/bin/pg_config && \
		cargo pgx package && \
		cargo pgx install
		```

=== ":material-apple: From Source (Mac)"
	
	1. Install the latest Rust compiler from [rust-lang.org](https://www.rust-lang.org/learn/get-started).

	2. Clone our git repository:

		```bash
		git clone https://github.com/postgresml/postgresml && \
		cd postgresml/pgml-extension/pgml_rust
		```
	3. Install PostgreSQL and other dependencies:

		```bash
		brew install llvm postgresql cmake openssl pkg-config
		```
	4. Install [`pgx`](https://github.com/tcdi/pgx) and build the extension:
		```bash
		cargo install cargo-pgx && \
		cargo pgx init --pg14 /usr/bin/pg_config && \
		cargo pgx package && \
		cargo pgx install
		```


### Install into the database

Now that the extension is installed on your system, you need to add it into the PostgreSQL database where you'd like to use it:

=== "SQL"

	Connect to the database and create the extension:

	```
	CREATE EXTENSION pgml;
	```

=== "Output"

	```
	pgml=# CREATE EXTENSION pgml;
	INFO:  Python version: 3.10.4 (main, Jun 29 2022, 12:14:53) [GCC 11.2.0]
	INFO:  Scikit-learn 1.1.1, XGBoost 1.62, LightGBM 3.3.2
	CREATE EXTENSION
	```


## Dashboard

The dashboard is a Django application. Installing it requires no special dependencies or commands:


=== ":material-linux: :material-microsoft: Linux & WSL"

	Install Python if you don't have it already:

	```bash
	apt-get update && \
	apt-get install python3 python3-pip python3-virtualenv
	```

=== ":material-apple: Mac"

	Install Python if you don't have it already:

	```bash
	brew install python3
	```

1. Clone our repository:

	```bash
	git clone https://github.com/postgresml/postgresml && \
	cd postgresml/pgml-dashboard
	```

2. Setup a virtual environment (recommended but not required):

	```bash
	virtualenv venv && \
	source venv/bin/activate
	```

3. Run the dashboard:

	```bash
	python install -r requirements.txt && \
	python manage.py migrate && \
	python manage.py runserver
	```
