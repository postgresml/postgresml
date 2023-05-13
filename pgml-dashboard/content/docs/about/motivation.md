# Motivation

Deploying machine learning models into existing applications is not straight forward. It involves operating new services, which need to be written in specialized languages with libraries outside of the experience of many software engineers. Those services tend to be architected around specialized datastores and hardware that requires additional management and know how. Data access needs to be secure across production and development environments without impeding productivity. This complexity pushes risks and costs beyond acceptable trade off limits for many otherwise valuable use cases.

PostgresML makes ML simple by moving the code to your data, rather than copying the data all over the place. You train models using simple SQL commands, and you get the predictions in your apps via a mechanism you're already using: a query over a standard Postgres connection.

Our goal is that anyone with a basic understanding of SQL should be able to build, deploy and maintain machine learning models in production, while receiving the benefits of a high performance machine learning platform. Ultimately, PostgresML aims to be the easiest, safest and fastest way to gain value from machine learning.
