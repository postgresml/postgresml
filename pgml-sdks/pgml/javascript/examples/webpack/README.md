# Webpack Demo

The JavaScript SDK utilizes native node modules as our SDK is written in Rust. To get it working with webpack, we need a loader that is designed to work with native node modules. In this case, we have opted to use the [node-loader](https://github.com/webpack-contrib/node-loader) module. See [webpack.config.js](./webpack.config.js) for how we configured it.

## Prerequisites

Before running, first install dependencies and set the DATABASE_URL environment variable:
```
npm i
export DATABASE_URL={YOUR DATABASE URL}
```

Optionally, configure a .env file containing a DATABASE_URL variable.

## Running the Example

The example utilizes the Builtins class to perform text classification. After following the [Prerequisites](#/Prerequisites) run the following code:
```
npm run build
node dist/index.js "I love PostgresML"
```
