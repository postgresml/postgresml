
# Pre-Trained Models
PostgresML integrates Transformers to bring state-of-the-art models into the data layer. There are tens of thousands of pre-trained models with pipelines to turn raw inputs into useful results. Many state of the art deep learning architectures have been published and made available for download. You will want to browse all the [models](https://huggingface.co/models) available to find the perfect solution for your dataset and task.

!!! tip

    These models are cached per connection to improve repeated calls in a single session. To free that memory, you'll need to close your connection. You may want to establish dedicated credentials and connection pools via [pgcat](https://github.com/levkk/pgcat) or [pgbouncer](https://www.pgbouncer.org/) for larger models that have billions of parameters. You may also pass `{"cache": false}` in the JSON `call` args to prevent this behavior.

## Examples
All of the tasks and models demonstrated here can be customized by passing additional arguments to the `Pipeline` initializer or call. You'll find additional links to documentation in the examples below.

The Hugging Face [`Pipeline`](https://huggingface.co/docs/transformers/main_classes/pipelines) API is exposed in Postgres via:

```sql linenums="1" title="transformer.sql"
pgml.transform(
    task TEXT OR JSONB,      -- task name or full pipeline initializer arguments
    call JSONB,              -- additional call arguments alongside the inputs
    inputs TEXT[] OR BYTEA[] -- inputs for inference
)
```

This is roughly equivalent to the following Python:

```python linenums="1" title="transformer.py"
import transformers

def transform(task, call, inputs):
    return transformers.pipeline(**task)(inputs, **call)
```

Most pipelines operate on `TEXT[]` inputs, but some require binary `BYTEA[]` data like audio classifiers; `inputs` can be `SELECT`ed from tables in the database, or they may be passed in directly with the query. The output of this call is a `JSONB` structure that is task specific. 


### Translation
An English to French translation with the default pre-trained model:

=== "SQL"

    ```sql linenums="1" 
    SELECT pgml.transform(
        'translation_en_to_fr',
        inputs => ARRAY[
            'Welcome to the future!',
            'Where have you been all this time?'
        ]
    ) AS french;
    ```

=== "Result"

    ```sql linenums="1"
                             french                                 
    ------------------------------------------------------------
    [
        {"translation_text": "Bienvenue à l'avenir!"},
        {"translation_text": "Où êtes-vous allé tout ce temps?"}
    ]
    ```

See [translation documentation](https://huggingface.co/docs/transformers/tasks/translation) for more options.

### Sentiment Analysis
Sentiment analysis is one use of `text-classification`, but there are [many others](https://huggingface.co/tasks/text-classification). This example demonstrates specifying the `model` to be used rather than the task. The [`roberta-large-mnli`](https://huggingface.co/roberta-large-mnli) model specifies the task of `sentiment-analysis` in it's default configuration.

=== "SQL"

    ```sql linenums="1" 
    SELECT pgml.transform(
        '{"model": "roberta-large-mnli"}'::JSONB,
        inputs => ARRAY[
            'I love how amazingly simple ML has become!', 
            'I hate doing mundane and thankless tasks. ☹️'
        ]
    ) AS positivity;
    ```

=== "Result"

    ```sql linenums="1"
                            positivity
    ------------------------------------------------------
    [
        {"label": "NEUTRAL", "score": 0.8143417835235596}, 
        {"label": "NEUTRAL", "score": 0.7637073993682861}
    ]
    ```

See [text classification documentation](https://huggingface.co/tasks/text-classification) for more options and potential use cases beyond sentiment analysis. You'll notice the outputs are not great in this example. RoBERTa is a breakthrough model, that demonstrated just how important each particular hyperparameter is for the task and particular dataset regardless of how large your model is. We'll show how to [fine tune](/user_guides/transformers/fine_tuning/) models on your data in the next step.

### Summarization
Sometimes we need all the nuanced detail, but sometimes it's nice to get to the point:

=== "SQL"

    ```sql linenums="1" 
    SELECT pgml.transform(
        'summarization',
        inputs => ARRAY['
            Dominic Cobb is the foremost practitioner of the artistic science 
            of extraction, inserting oneself into a subject''s dreams to 
            obtain hidden information without the subject knowing, a concept 
            taught to him by his professor father-in-law, Dr. Stephen Miles. 
            Dom''s associates are Miles'' former students, who Dom requires 
            as he has given up being the dream architect for reasons he 
            won''t disclose. Dom''s primary associate, Arthur, believes it 
            has something to do with Dom''s deceased wife, Mal, who often 
            figures prominently and violently in those dreams, or Dom''s want 
            to "go home" (get back to his own reality, which includes two 
            young children). Dom''s work is generally in corporate espionage. 
            As the subjects don''t want the information to get into the wrong 
            hands, the clients have zero tolerance for failure. Dom is also a 
            wanted man, as many of his past subjects have learned what Dom 
            has done to them. One of those subjects, Mr. Saito, offers Dom a 
            job he can''t refuse: to take the concept one step further into 
            inception, namely planting thoughts into the subject''s dreams 
            without them knowing. Inception can fundamentally alter that 
            person as a being. Saito''s target is Robert Michael Fischer, the 
            heir to an energy business empire, which has the potential to 
            rule the world if continued on the current trajectory. Beyond the 
            complex logistics of the dream architecture of the case and some 
            unknowns concerning Fischer, the biggest obstacles in success for 
            the team become worrying about one aspect of inception which Cobb 
            fails to disclose to the other team members prior to the job, and 
            Cobb''s newest associate Ariadne''s belief that Cobb''s own 
            subconscious, especially as it relates to Mal, may be taking over 
            what happens in the dreams.
        ']
    ) AS result;
    ```

=== "Result"

    ```sql linenums="1"
                                     result
    --------------------------------------------------------------------------
    [{"summary_text": "Dominic Cobb is the foremost practitioner of the 
    artistic science of extraction . his associates are former students, who 
    Dom requires as he has given up being the dream architect . he is also a 
    wanted man, as many of his past subjects have learned what Dom has done 
    to them ."}]
    ```

See [summarization documentation](https://huggingface.co/tasks/summarization) for more options.


### Question Answering
Question Answering extracts an answer from a given context:

=== "SQL"

    ```sql linenums="1" 
    SELECT pgml.transform(
        'question-answering',
        inputs => ARRAY[
            '{
                "question": "Am I dreaming?", 
                "context": "I got a good nights sleep last night 
                    and started a simple tutorial over my cup of 
                    morning coffee. The capabilities seem unreal, 
                    compared to what I came to expect from the 
                    simple SQL standard I studied so long ago. The 
                    answer is staring me in the face, and I feel 
                    the uncanny call from beyond the screen 
                    to check the results."
            }'
        ]
    ) AS answer;

    ```

=== "Result"

    ```sql linenums="1"
                            answer
    -----------------------------------------------------
    {
        "end": 36, 
        "score": 0.20027603209018707, 
        "start": 0, 
        "answer": "I got a good nights sleep last night"
    }
    ```

See [question answering documentation](https://huggingface.co/tasks/question-answering) for more options.

### Text Generation
If you need to expand on some thoughts, you can have AI complete your sentences for you:

=== "SQL"

    ```sql linenums="1" 
    SELECT pgml.transform(
        '{"task": "text-generation"}',
        '{"num_return_sequences": 2}',
        ARRAY['Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone']
    ) AS result;
    ```

=== "Result"

    ```sql linenums="1"
                                       result
    -----------------------------------------------------------------------------
    [[
        {
            "generated_text": "Three Rings for the Elven-kings under the sky,
             Seven for the Dwarf-lords in their halls of stone, and five for 
             the Elves.\nWhen, from all that's happening, he sees these things, 
             he says to himself,"
        }, 
        {
            "generated_text": "Three Rings for the Elven-kings under the sky, 
            Seven for the Dwarf-lords in their halls of stone, Eight for the
            Erogean-kings in their halls of stone -- \"and so forth;\" and 
            \"of these"
        }
    ]]
    ```

### More
There are many different [tasks](https://huggingface.co/tasks) and tens of thousands of state-of-the-art [models](https://huggingface.co/models) available for you to explore. The possibilities are expanding every day.
