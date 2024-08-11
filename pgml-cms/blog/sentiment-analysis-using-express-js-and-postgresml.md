---
description: >-
  An example application for an easy and scalable way to get started with
  machine learning in Express
---

# Sentiment Analysis using Express JS and PostgresML

<div align="left">

<figure><img src=".gitbook/assets/daniel.jpg" alt="Author" width="125"><figcaption><p>Daniel Illenberger</p></figcaption></figure>

</div>

Daniel Illenberger

March 26, 2024

Traditional MLOps requires continuously moving data between models and storage. Both small and large projects suffer with such an implementation on the metrics of time, cost, and complexity. PostgresML simplifies and streamlines MLOps by performing machine learning directly where your data resides.

Express is a mature JS backend framework touted as being fast and flexible. It is a popular choice for JS developers wanting to quickly develop an API or full fledge website. Since it is in the JS ecosystem, there's an endless number of open source projects you can use to add functionality.

### Application Overview

Sentiment analysis is a valuable tool for understanding the emotional polarity of text. You can determine if the text is positive, negative, or neutral. Common use cases include understanding product reviews, survey questions, and social media posts.

In this application, we'll be applying sentiment analysis to note taking. Note taking and journaling can be an excellent practice for work efficiency and self improvement. However, if you are like me, it quickly becomes impossible to find and make use of anything I've written down. Notes that are useful must be easy to navigate. With this motivation, let's create a demo that can record notes throughout the day. Each day will have a summary and sentiment score. That way, if I'm looking for that time a few weeks ago when we were frustrated with our old MLOps platform — it will be easy to find.&#x20;

We will perform all the Machine Learning heavy lifting with the pgml extension function `pgml.transform()`. This brings Hugging Face Transformers into our data layer.

### Follow Along

You can see the full code on [GitHub](https://github.com/postgresml/example-expressjs). Follow the Readme to get the application up and running on your local machine.

### The Code

This app is composed of three main parts, reading and writing to a database, performing sentiment analysis on entries, and creating a summary.

We are going to use [postgresql-client](https://www.npmjs.com/package/postgresql-client) to connect to our DB.&#x20;

When the application builds we ensure we have two tables, one for notes and one for the the daily summary and sentiment score.

```javascript
const notes = await connection.execute(`
  CREATE TABLE IF NOT EXISTS notes ( 
    id BIGSERIAL PRIMARY KEY, 
    note VARCHAR, 
    score FLOAT, 
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );`
)

const day = await connection.execute(`
  CREATE TABLE IF NOT EXISTS days ( 
    id BIGSERIAL PRIMARY KEY, 
    summary VARCHAR, 
    score FLOAT, 
    created_at DATE NOT NULL UNIQUE DEFAULT DATE(NOW())
  );`
) 
```

We also have three endpoints to hit:

* `app.get(“/", async (req, res, next)` which returns all the notes for that day and the daily summary.&#x20;
* `app.post(“/add", async (req, res, next)` which accepts a new note entry and performs a sentiment analysis. We simplify the score by converting it to 1, 0, -1 for positive, neutral, negative and save it in our notes table.

```postgresql
WITH note AS (
  SELECT pgml.transform(
    inputs => ARRAY['${req.body.note}'],
    task => '{"task": "text-classification", "model": "finiteautomata/bertweet-base-sentiment-analysis"}'::JSONB
  ) AS market_sentiment
), 

score AS (
  SELECT 
    CASE 
      WHEN (SELECT market_sentiment FROM note)[0]::JSONB ->> 'label' = 'POS' THEN 1
      WHEN (SELECT market_sentiment FROM note)[0]::JSONB ->> 'label' = 'NEG' THEN -1
      ELSE 0
    END AS score
)

INSERT INTO notes (note, score) VALUES ('${req.body.note}', (SELECT score FROM score));

```

* `app.get(“/analyze”, async (req, res, next)` which takes the daily entries, produces a summary and total sentiment score, and places that into our days table.

```postgresql
WITH day AS (
  SELECT 
    note,
    score
  FROM notes 
  WHERE DATE(created_at) = DATE(NOW())),

  sum AS (
    SELECT pgml.transform(
      task => '{"task": "summarization", "model": "sshleifer/distilbart-cnn-12-6"}'::JSONB,
      inputs => array[(SELECT STRING_AGG(note, '\n') FROM day)],
      args => '{"min_length" : 20, "max_length" : 70}'::JSONB
    ) AS summary
  )

  INSERT INTO days (summary, score) 
  VALUES ((SELECT summary FROM sum)[0]::JSONB ->> 'summary_text', (SELECT SUM(score) FROM day))
  On Conflict (created_at) DO UPDATE SET summary=EXCLUDED.summary, score=EXCLUDED.score 
  RETURNING score;
```

and this is all that is required!

### Test Run

Let's imagine a day in the life of a boy destined to save the galaxy. Throughout his day he records the following notes:

```
Woke to routine chores. Bought droids, found Leia's message. She pleads for help from Obi-Wan Kenobi. Intrigued, but uncertain.
```

```
Frantically searched for R2-D2, encountered Sand People. Saved by Obi-Wan. His presence is a glimmer of hope in this desolate place.
```

```
Returned home to find it destroyed by stormtroopers. Aunt and uncle gone. Rage and despair fill me. Empire's cruelty knows no bounds.
```

```
Left Tatooine with Obi-Wan, droids. Met Han Solo and Chewbacca in Mos Eisley. Sense of purpose grows despite uncertainty. Galaxy awaits.
```

```
On our way to Alderaan. With any luck we will find the princes soon.
```

When we analyze this info we get a score of 2 and our summary is:

```
Returned home to find it destroyed by stormtroopers . Bought droids, found Leia's message . Met Han Solo and Chewbacca in Mos Eisley . Sense of purpose grows despite uncertainty .
```

not bad for less than an hour of coding.

### Final Thoughts

This app is far from complete but does show an easy and scalable way to get started with ML in Express. From here I encourage you to head over to our [docs](https://postgresml.org/docs/) and see what other features could be added.

If SQL is not your thing, no worries. Check out or [JS SDK](https://postgresml.org/docs/open-source/korvus) to streamline all our best practices with simple JavaScript.&#x20;

We love hearing from you — please reach out to us on [Discord ](https://discord.gg/DmyJP3qJ7U)or simply [Contact Us](https://postgresml.org/contact) here if you have any questions or feedback.&#x20;
