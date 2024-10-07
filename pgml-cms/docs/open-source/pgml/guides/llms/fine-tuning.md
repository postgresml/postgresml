---
description: An in-depth guide on fine-tuning LLMs
---

# LLM Fine-tuning

In this section, we will provide a step-by-step walkthrough for fine-tuning a Language Model (LLM) for differnt tasks.

## Prerequisites

1. Ensure you have the PostgresML extension installed and configured in your PostgreSQL database. You can find installation instructions for PostgresML in the official documentation.

2. Obtain a Hugging Face API token to push the fine-tuned model to the Hugging Face Model Hub. Follow the instructions on the [Hugging Face website](https://huggingface.co/settings/tokens) to get your API token.

## Text Classification 2 Classes

### 1. Loading the Dataset

To begin, create a table to store your dataset. In this example, we use the 'imdb' dataset from Hugging Face. IMDB dataset contains three splits: train (25K rows), test (25K rows) and unsupervised (50K rows). In train and test splits, negative class has label 0 and positive class label 1. All rows in unsupervised split has a label of -1.
```postgresql
SELECT pgml.load_dataset('imdb');
```

### 2. Prepare dataset for fine-tuning

We will create a view of the dataset by performing the following operations:

- Add a new text column named "class" that has positive and negative classes.
- Shuffled view of the dataset to ensure randomness in the distribution of data.
- Remove all the unsupervised splits that have label = -1.

```postgresql
CREATE VIEW pgml.imdb_shuffled_view AS
SELECT
    label,
    CASE WHEN label = 0 THEN 'negative'
         WHEN label = 1 THEN 'positive'
         ELSE 'neutral'
    END AS class,
    text
FROM pgml.imdb
WHERE label != -1
ORDER BY RANDOM();
```

### 3 Exploratory Data Analysis (EDA) on Shuffled Data

Before splitting the data into training and test sets, it's essential to perform exploratory data analysis (EDA) to understand the distribution of labels and other characteristics of the dataset. In this section, we'll use the `pgml.imdb_shuffled_view` to explore the shuffled data.

#### 3.1 Distribution of Labels

To analyze the distribution of labels in the shuffled dataset, you can use the following SQL query:

```postgresql
-- Count the occurrences of each label in the shuffled dataset
pgml=# SELECT
    class,
    COUNT(*) AS label_count
FROM pgml.imdb_shuffled_view
GROUP BY class
ORDER BY class;

  class   | label_count
----------+-------------
 negative |       25000
 positive |       25000
(2 rows)
```

This query provides insights into the distribution of labels, helping you understand the balance or imbalance of classes in your dataset.

#### 3.2 Sample Records
To get a glimpse of the data, you can retrieve a sample of records from the shuffled dataset:

```postgresql
-- Retrieve a sample of records from the shuffled dataset
pgml=# SELECT LEFT(text,100) AS text, class
FROM pgml.imdb_shuffled_view
LIMIT 5;
                                                 text                                                 |  class
------------------------------------------------------------------------------------------------------+----------
 This is a VERY entertaining movie. A few of the reviews that I have read on this forum have been wri | positive
 This is one of those movies where I wish I had just stayed in the bar.<br /><br />The film is quite  | negative
 Barbershop 2: Back in Business wasn't as good as it's original but was just as funny. The movie itse | negative
 Umberto Lenzi hits new lows with this recycled trash. Janet Agren plays a lady who is looking for he | negative
 I saw this movie last night at the Phila. Film festival. It was an interesting and funny movie that  | positive
(5 rows)

Time: 101.985 ms
```

This query allows you to inspect a few records to understand the structure and content of the shuffled data.

#### 3.3 Additional Exploratory Analysis
Feel free to explore other aspects of the data, such as the distribution of text lengths, word frequencies, or any other features relevant to your analysis. Performing EDA is crucial for gaining insights into your dataset and making informed decisions during subsequent steps of the workflow.

### 4. Splitting Data into Training and Test Sets

Create views for training and test data by splitting the shuffled dataset. In this example, 80% is allocated for training, and 20% for testing. We will use `pgml.imdb_test_view` in [section 6](#6-inference-using-fine-tuned-model) for batch predictions using the finetuned model.

```postgresql
-- Create a view for training data
CREATE VIEW pgml.imdb_train_view AS
SELECT *
FROM pgml.imdb_shuffled_view
LIMIT (SELECT COUNT(*) * 0.8 FROM pgml.imdb_shuffled_view);

-- Create a view for test data
CREATE VIEW pgml.imdb_test_view AS
SELECT *
FROM pgml.imdb_shuffled_view
OFFSET (SELECT COUNT(*) * 0.8 FROM pgml.imdb_shuffled_view);
```

### 5. Fine-Tuning the Language Model

Now, fine-tune the Language Model for text classification using the created training view. In the following sections, you will see a detailed explanation of different parameters used during fine-tuning. Fine-tuned model is pushed to your public Hugging Face Hub periodically. A new repository will be created under your username using your project name (`imdb_review_sentiment` in this case). You can also choose to push the model to a private repository by setting `hub_private_repo: true` in training arguments.

```postgresql
SELECT pgml.tune(
    'imdb_review_sentiment',
    task => 'text-classification',
    relation_name => 'pgml.imdb_train_view',
    model_name => 'distilbert-base-uncased',
    test_size => 0.2,
    test_sampling => 'last',
    hyperparams => '{
        "training_args" : {
            "learning_rate": 2e-5,
            "per_device_train_batch_size": 16,
            "per_device_eval_batch_size": 16,
            "num_train_epochs": 20,
            "weight_decay": 0.01,
            "hub_token" : "YOUR_HUB_TOKEN",
            "push_to_hub" : true
        },
        "dataset_args" : { "text_column" : "text", "class_column" : "class" }
    }'
);
```

* project_name ('imdb_review_sentiment'): The project_name parameter specifies a unique name for your fine-tuning project. It helps identify and organize different fine-tuning tasks within the PostgreSQL database. In this example, the project is named 'imdb_review_sentiment,' reflecting the sentiment analysis task on the IMDb dataset. You can check `pgml.projects` for list of projects.

* task ('text-classification'): The task parameter defines the nature of the machine learning task to be performed. In this case, it's set to 'text-classification,' indicating that the fine-tuning is geared towards training a model for text classification.

* relation_name ('pgml.imdb_train_view'): The relation_name parameter identifies the training dataset to be used for fine-tuning. It specifies the view or table containing the training data. In this example, 'pgml.imdb_train_view' is the view created from the shuffled IMDb dataset, and it serves as the source for model training.

* model_name ('distilbert-base-uncased'): The model_name parameter denotes the pre-trained language model architecture to be fine-tuned. In this case, 'distilbert-base-uncased' is selected. DistilBERT is a distilled version of BERT, and the 'uncased' variant indicates that the model does not differentiate between uppercase and lowercase letters.

* test_size (0.2): The test_size parameter determines the proportion of the dataset reserved for testing during fine-tuning. In this example, 20% of the dataset is set aside for evaluation, helping assess the model's performance on unseen data.

* test_sampling ('last'): The test_sampling parameter defines the strategy for sampling test data from the dataset. In this case, 'last' indicates that the most recent portion of the data, following the specified test size, is used for testing. Adjusting this parameter might be necessary based on your specific requirements and dataset characteristics.

#### 5.1 Dataset Arguments (dataset_args)
The dataset_args section allows you to specify critical parameters related to your dataset for language model fine-tuning.

* text_column: The name of the column containing the text data in your dataset. In this example, it's set to "text."
* class_column: The name of the column containing the class labels in your dataset. In this example, it's set to "class."

#### 5.2 Training Arguments (training_args)
Fine-tuning a language model requires careful consideration of training parameters in the training_args section. Below is a subset of training args that you can pass to fine-tuning. You can find an exhaustive list of parameters in Hugging Face documentation on [TrainingArguments](https://huggingface.co/docs/transformers/main_classes/trainer#transformers.TrainingArguments).

* learning_rate: The learning rate for the training. It controls the step size during the optimization process. Adjust based on your model's convergence behavior.
* per_device_train_batch_size: The batch size per GPU for training. This parameter controls the number of training samples utilized in one iteration. Adjust based on your available GPU memory.
* per_device_eval_batch_size: The batch size per GPU for evaluation. Similar to per_device_train_batch_size, but used during model evaluation.
* num_train_epochs: The number of training epochs. An epoch is one complete pass through the entire training dataset. Adjust based on the model's convergence and your dataset size.
* weight_decay: L2 regularization term for weight decay. It helps prevent overfitting. Adjust based on the complexity of your model.
* hub_token: Your Hugging Face API token to push the fine-tuned model to the Hugging Face Model Hub. Replace "YOUR_HUB_TOKEN" with the actual token.
* push_to_hub: A boolean flag indicating whether to push the model to the Hugging Face Model Hub after fine-tuning.

#### 5.3 Monitoring
During training, metrics like loss, gradient norm will be printed as info and also logged in pgml.logs table. Below is a snapshot of such output.

```json
INFO:  {
    "loss": 0.3453,
    "grad_norm": 5.230295181274414,
    "learning_rate": 1.9e-05,
    "epoch": 0.25,
    "step": 500,
    "max_steps": 10000,
    "timestamp": "2024-03-07 01:59:15.090612"
}
INFO:  {
    "loss": 0.2479,
    "grad_norm": 2.7754225730895996,
    "learning_rate": 1.8e-05,
    "epoch": 0.5,
    "step": 1000,
    "max_steps": 10000,
    "timestamp": "2024-03-07 02:01:12.064098"
}
INFO:  {
    "loss": 0.223,
    "learning_rate": 1.6000000000000003e-05,
    "epoch": 1.0,
    "step": 2000,
    "max_steps": 10000,
    "timestamp": "2024-03-07 02:05:08.141220"
}
```

Once the training is completed, model will be evaluated against the validation dataset. You will see the below in the client terminal. Accuracy on the evaluation dataset is 0.934 and F1-score is 0.93.

```json
INFO:  {
    "train_runtime": 2359.5335,
    "train_samples_per_second": 67.81,
    "train_steps_per_second": 4.238,
    "train_loss": 0.11267969808578492,
    "epoch": 5.0,
    "step": 10000,
    "max_steps": 10000,
    "timestamp": "2024-03-07 02:36:38.783279"
}
INFO:  {
    "eval_loss": 0.3691485524177551,
    "eval_f1": 0.9343711842996372,
    "eval_accuracy": 0.934375,
    "eval_runtime": 41.6167,
    "eval_samples_per_second": 192.23,
    "eval_steps_per_second": 12.014,
    "epoch": 5.0,
    "step": 10000,
    "max_steps": 10000,
    "timestamp": "2024-03-07 02:37:31.762917"
}
```

Once the training is completed, you can check query pgml.logs table using the model_id or by finding the latest model on the project.

```bash
pgml: SELECT logs->>'epoch' AS epoch, logs->>'step' AS step, logs->>'loss' AS loss FROM pgml.logs WHERE model_id = 993 AND jsonb_exists(logs, 'loss');
 epoch | step  |  loss
-------+-------+--------
 0.25  | 500   | 0.3453
 0.5   | 1000  | 0.2479
 0.75  | 1500  | 0.223
 1.0   | 2000  | 0.2165
 1.25  | 2500  | 0.1485
 1.5   | 3000  | 0.1563
 1.75  | 3500  | 0.1559
 2.0   | 4000  | 0.142
 2.25  | 4500  | 0.0816
 2.5   | 5000  | 0.0942
 2.75  | 5500  | 0.075
 3.0   | 6000  | 0.0883
 3.25  | 6500  | 0.0432
 3.5   | 7000  | 0.0426
 3.75  | 7500  | 0.0444
 4.0   | 8000  | 0.0504
 4.25  | 8500  | 0.0186
 4.5   | 9000  | 0.0265
 4.75  | 9500  | 0.0248
 5.0   | 10000 | 0.0284
```

During training, model is periodically uploaded to Hugging Face Hub. You will find the model at `https://huggingface.co/<username>/<project_name>`. An example model that was automatically pushed to Hugging Face Hub is [here](https://huggingface.co/santiadavani/imdb_review_sentiement).

### 6. Inference using fine-tuned model
Now, that we have fine-tuned model on Hugging Face Hub, we can use [`pgml.transform`](/docs/open-source/pgml/api/pgml.transform) to perform real-time predictions as well as batch predictions.

**Real-time predictions**

Here is an example pgml.transform call for real-time predictions on the newly minted LLM fine-tuned on IMDB review dataset.
```postgresql
 SELECT pgml.transform(
  task   => '{
    "task": "text-classification",
    "model": "santiadavani/imdb_review_sentiement"
  }'::JSONB,
  inputs => ARRAY[
    'I would not give this movie a rating, its not worthy. I watched it only because I am a Pfieffer fan. ',
    'This movie was sooooooo good! It was hilarious! There are so many jokes that you can just watch the'
  ]
);
                                               transform
--------------------------------------------------------------------------------------------------------
 [{"label": "negative", "score": 0.999561846256256}, {"label": "positive", "score": 0.986771047115326}]
(1 row)

Time: 175.264 ms
```

**Batch predictions**

```postgresql
pgml=# SELECT
    LEFT(text, 100) AS truncated_text,
    class,
    predicted_class[0]->>'label' AS predicted_class,
    (predicted_class[0]->>'score')::float AS score
FROM (
    SELECT
        LEFT(text, 100) AS text,
        class,
        pgml.transform(
            task => '{
                "task": "text-classification",
                "model": "santiadavani/imdb_review_sentiement"
            }'::JSONB,
            inputs => ARRAY[text]
        ) AS predicted_class
    FROM pgml.imdb_test_view
    LIMIT 2
) AS subquery;
                                            truncated_text                                            |  class   | predicted_class |       score
------------------------------------------------------------------------------------------------------+----------+-----------------+--------------------
 I wouldn't give this movie a rating, it's not worthy. I watched it only because I'm a Pfieffer fan.  | negative | negative        | 0.9996490478515624
 This movie was sooooooo good! It was hilarious! There are so many jokes that you can just watch the  | positive | positive        | 0.9972313046455384

 Time: 1337.290 ms (00:01.337)
 ```

## 7. Restarting Training from a Previous Trained Model

Sometimes, it's necessary to restart the training process from a previously trained model. This can be advantageous for various reasons, such as model fine-tuning, hyperparameter adjustments, or addressing interruptions in the training process. `pgml.tune` provides a seamless way to restart training while leveraging the progress made in the existing model. Below is a guide on how to restart training using a previous model as a starting point:

### Define the Previous Model

Specify the name of the existing model you want to use as a starting point. This is achieved by setting the `model_name` parameter in the `pgml.tune` function. In the example below, it is set to 'santiadavani/imdb_review_sentiement'.

```postgresql
model_name => 'santiadavani/imdb_review_sentiement',
```

### Adjust Hyperparameters
Fine-tune hyperparameters as needed for the restarted training process. This might include modifying learning rates, batch sizes, or training epochs. In the example below, hyperparameters such as learning rate, batch sizes, and epochs are adjusted.

```postgresql
"training_args": {
    "learning_rate": 2e-5,
    "per_device_train_batch_size": 16,
    "per_device_eval_batch_size": 16,
    "num_train_epochs": 1,
    "weight_decay": 0.01,
    "hub_token": "",
    "push_to_hub": true
},
```

### Ensure Consistent Dataset Configuration
Confirm that the dataset configuration remains consistent, including specifying the same text and class columns as in the previous training. This ensures compatibility between the existing model and the restarted training process.

```postgresql
"dataset_args": {
    "text_column": "text",
    "class_column": "class"
},
```

### Run the pgml.tune Function
Execute the `pgml.tune` function with the updated parameters to initiate the training restart. The function will leverage the existing model and adapt it based on the adjusted hyperparameters and dataset configuration.

```postgresql
SELECT pgml.tune(
    'imdb_review_sentiement',
    task => 'text-classification',
    relation_name => 'pgml.imdb_train_view',
    model_name => 'santiadavani/imdb_review_sentiement',
    test_size => 0.2,
    test_sampling => 'last',
    hyperparams => '{
        "training_args": {
            "learning_rate": 2e-5,
            "per_device_train_batch_size": 16,
            "per_device_eval_batch_size": 16,
            "num_train_epochs": 1,
            "weight_decay": 0.01,
            "hub_token": "YOUR_HUB_TOKEN",
            "push_to_hub": true
        },
        "dataset_args": { "text_column": "text", "class_column": "class" }
    }'
);
```

By following these steps, you can effectively restart training from a previously trained model, allowing for further refinement and adaptation of the model based on new requirements or insights. Adjust parameters as needed for your specific use case and dataset.

## 8. Hugging Face Hub vs. PostgresML as Model Repository
We utilize the Hugging Face Hub as the primary repository for fine-tuning Large Language Models (LLMs). Leveraging the HF hub offers several advantages:

* The HF repository serves as the platform for pushing incremental updates to the model during the training process. In the event of any disruptions in the database connection, you have the flexibility to resume training from where it was left off.
* If you prefer to keep the model private, you can push it to a private repository within the Hugging Face Hub. This ensures that the model is not publicly accessible by setting the parameter hub_private_repo to true.
* The pgml.transform function, designed around utilizing models from the Hugging Face Hub, can be reused without any modifications.

However, in certain scenarios, pushing the model to a central repository and pulling it for inference may not be the most suitable approach. To address this situation, we save all the model weights and additional artifacts, such as tokenizer configurations and vocabulary, in the pgml.files table at the end of the training process. It's important to note that as of the current writing, hooks to use models directly from pgml.files in the pgml.transform function have not been implemented. We welcome Pull Requests (PRs) from the community to enhance this functionality.

## Text Classification 9 Classes

### 1. Load and Shuffle the Dataset
In this section, we begin by loading the FinGPT sentiment analysis dataset using the `pgml.load_dataset` function. The dataset is then processed and organized into a shuffled view (pgml.fingpt_sentiment_shuffled_view), ensuring a randomized order of records. This step is crucial for preventing biases introduced by the original data ordering and enhancing the training process.

```postgresql
-- Load the dataset
SELECT pgml.load_dataset('FinGPT/fingpt-sentiment-train');

-- Create a shuffled view
CREATE VIEW pgml.fingpt_sentiment_shuffled_view AS
SELECT * FROM pgml."FinGPT/fingpt-sentiment-train" ORDER BY RANDOM();
```

### 2. Explore Class Distribution
Once the dataset is loaded and shuffled, we delve into understanding the distribution of sentiment classes within the data. By querying the shuffled view, we obtain valuable insights into the number of instances for each sentiment class. This exploration is essential for gaining a comprehensive understanding of the dataset and its inherent class imbalances.

```postgresql
-- Explore class distribution
SELECTpgml=# SELECT
    output,
    COUNT(*) AS class_count
FROM pgml.fingpt_sentiment_shuffled_view
GROUP BY output
ORDER BY output;

       output        | class_count
---------------------+-------------
 mildly negative     |        2108
 mildly positive     |        2548
 moderately negative |        2972
 moderately positive |        6163
 negative            |       11749
 neutral             |       29215
 positive            |       21588
 strong negative     |         218
 strong positive     |         211

```

### 3. Create Training and Test Views
To facilitate the training process, we create distinct views for training and testing purposes. The training view (pgml.fingpt_sentiment_train_view) contains 80% of the shuffled dataset, enabling the model to learn patterns and associations. Simultaneously, the test view (pgml.fingpt_sentiment_test_view) encompasses the remaining 20% of the data, providing a reliable evaluation set to assess the model's performance.

```postgresql
-- Create a view for training data (e.g., 80% of the shuffled records)
CREATE VIEW pgml.fingpt_sentiment_train_view AS
SELECT *
FROM pgml.fingpt_sentiment_shuffled_view
LIMIT (SELECT COUNT(*) * 0.8 FROM pgml.fingpt_sentiment_shuffled_view);

-- Create a view for test data (remaining 20% of the shuffled records)
CREATE VIEW pgml.fingpt_sentiment_test_view AS
SELECT *
FROM pgml.fingpt_sentiment_shuffled_view
OFFSET (SELECT COUNT(*) * 0.8 FROM pgml.fingpt_sentiment_shuffled_view);

```

### 4. Fine-Tune the Model for 9 Classes
In the final section, we kick off the fine-tuning process using the `pgml.tune` function. The model will be internally configured for sentiment analysis with 9 classes. The training is executed on the 80% of the train view and evaluated on the remaining 20% of the train view. The test view is reserved for evaluating the model's accuracy after training is completed. Please note that the option `hub_private_repo: true` is used to push the model to a private Hugging Face repository.

```postgresql
-- Fine-tune the model for 9 classes without HUB token
SELECT pgml.tune(
    'fingpt_sentiement',
    task => 'text-classification',
    relation_name => 'pgml.fingpt_sentiment_train_view',
    model_name => 'distilbert-base-uncased',
    test_size => 0.2,
    test_sampling => 'last',
    hyperparams => '{
        "training_args": {
            "learning_rate": 2e-5,
            "per_device_train_batch_size": 16,
            "per_device_eval_batch_size": 16,
            "num_train_epochs": 5,
            "weight_decay": 0.01,
            "hub_token" : "YOUR_HUB_TOKEN",
            "push_to_hub": true,
            "hub_private_repo": true
        },
        "dataset_args": { "text_column": "input", "class_column": "output" }
    }'
);

```

## Conversation

In this section, we will discuss conversational task using state-of-the-art NLP techniques. Conversational AI has garnered immense interest and significance in recent years due to its wide range of applications, from virtual assistants to customer service chatbots and beyond.

### Understanding the Conversation Task

At the core of conversational AI lies the conversation task, a fundamental NLP problem that involves processing and generating human-like text-based interactions. Let's break down this task into its key components:

- **Input:** The input to the conversation task typically consists of a sequence of conversational turns, often represented as text. These turns can encompass a dialogue between two or more speakers, capturing the flow of communication over time.

- **Model:** Central to the conversation task is the NLP model, which is trained to understand the nuances of human conversation and generate appropriate responses. These models leverage sophisticated transformer based architectures like Llama2, Mistral, GPT etc., empowered by large-scale datasets and advanced training techniques.

- **Output:** The ultimate output of the conversation task is the model's response to the input conversation. This response aims to be contextually relevant, coherent, and engaging, reflecting a natural human-like interaction.

### Versatility of the Conversation Task

What makes the conversation task truly remarkable is its remarkable versatility. Beyond its traditional application in dialogue systems, the conversation task can be adapted to solve several NLP problems by tweaking the input representation or task formulation.

- **Text Classification:** By providing individual utterances with corresponding labels, the conversation task can be repurposed for tasks such as sentiment analysis, intent detection, or topic classification.

    **Input:**
    - System: Chatbot: "Hello! How can I assist you today?"
    - User: "I'm having trouble connecting to the internet."

    **Model Output (Text Classification):**
    - Predicted Label: Technical Support
    - Confidence Score: 0.85

- **Token Classification:** Annotating the conversation with labels for specific tokens or phrases enables applications like named entity recognition within conversational text.

    **Input:**
    - System: Chatbot: "Please describe the issue you're facing in detail."
    - User: "I can't access any websites, and the Wi-Fi indicator on my router is blinking."

    **Model Output (Token Classification):**
    - User's Description: "I can't access any websites, and the Wi-Fi indicator on my router is blinking."
    - Token Labels:
    - "access" - Action
    - "websites" - Entity (Location)
    - "Wi-Fi" - Entity (Technology)
    - "indicator" - Entity (Device Component)
    - "blinking" - State

- **Question Answering:** Transforming conversational exchanges into a question-answering format enables extracting relevant information and providing concise answers, akin to human comprehension and response.

    **Input:**
    - System: Chatbot: "How can I help you today?"
    - User: "What are the symptoms of COVID-19?"

    **Model Output (Question Answering):**
    - Answer: "Common symptoms of COVID-19 include fever, cough, fatigue, shortness of breath, loss of taste or smell, and body aches."

### Fine-tuning Llama2-7b model using LoRA
In this section, we will explore how to fine-tune the Llama2-7b-chat large language model for the financial sentiment data discussed in the previous [section](#text-classification-9-classes) utilizing the pgml.tune function and employing the LoRA approach.  LoRA is a technique that enables efficient fine-tuning of large language models by only updating a small subset of the model's weights during fine-tuning, while keeping the majority of the weights frozen. This approach can significantly reduce the computational requirements and memory footprint compared to traditional full model fine-tuning.

```postgresql
SELECT pgml.tune(
    'fingpt-llama2-7b-chat',
    task => 'conversation',
    relation_name => 'pgml.fingpt_sentiment_train_view',
    model_name => 'meta-llama/Llama-2-7b-chat-hf',
    test_size => 0.8,
    test_sampling => 'last',
    hyperparams => '{
        "training_args" : {
            "learning_rate": 2e-5,
            "per_device_train_batch_size": 4,
            "per_device_eval_batch_size": 4,
            "num_train_epochs": 1,
            "weight_decay": 0.01,
            "hub_token" : "HF_TOKEN",
            "push_to_hub" : true,
            "optim" : "adamw_bnb_8bit",
            "gradient_accumulation_steps" : 4,
            "gradient_checkpointing" : true
        },
        "dataset_args" : { "system_column" : "instruction", "user_column" : "input", "assistant_column" : "output" },
        "lora_config" : {"r": 2, "lora_alpha" : 4, "lora_dropout" : 0.05, "bias": "none", "task_type": "CAUSAL_LM"},
        "load_in_8bit" : false,
        "token" : "HF_TOKEN"
    }'
);
```
Let's break down each argument and its significance:

1. **Model Name (`model_name`):**
   - This argument specifies the name or identifier of the base model that will be fine-tuned. In the context of the provided query, it refers to the pre-trained model "meta-llama/Llama-2-7b-chat-hf."

2. **Task (`task`):**
   - Indicates the specific task for which the model is being fine-tuned. In this case, it's set to "conversation," signifying that the model will be adapted to process conversational data.

3. **Relation Name (`relation_name`):**
   - Refers to the name of the dataset or database relation containing the training data used for fine-tuning. In the provided query, it's set to "pgml.fingpt_sentiment_train_view."

4. **Test Size (`test_size`):**
   - Specifies the proportion of the dataset reserved for testing, expressed as a fraction. In the example, it's set to 0.8, indicating that 80% of the data will be used for training, and the remaining 20% will be held out for testing.

5. **Test Sampling (`test_sampling`):**
   - Determines the strategy for sampling the test data. In the provided query, it's set to "last," indicating that the last portion of the dataset will be used for testing.

6. **Hyperparameters (`hyperparams`):**
   - This argument encapsulates a JSON object containing various hyperparameters essential for the fine-tuning process. Let's break down its subcomponents:
     - **Training Args (`training_args`):** Specifies parameters related to the training process, including learning rate, batch size, number of epochs, weight decay, optimizer settings, and other training configurations.
     - **Dataset Args (`dataset_args`):** Provides arguments related to dataset processing, such as column names for system responses, user inputs, and assistant outputs.
     - **LORA Config (`lora_config`):** Defines settings for the LORA (Learned Optimizer and Rate Adaptation) algorithm, including parameters like the attention radius (`r`), LORA alpha (`lora_alpha`), dropout rate (`lora_dropout`), bias, and task type.
     - **Load in 8-bit (`load_in_8bit`):** Determines whether to load data in 8-bit format, which can be beneficial for memory and performance optimization.
     - **Token (`token`):** Specifies the Hugging Face token required for accessing private repositories and pushing the fine-tuned model to the Hugging Face Hub.

7. **Hub Private Repo (`hub_private_repo`):**
   - This optional parameter indicates whether the fine-tuned model should be pushed to a private repository on the Hugging Face Hub. In the provided query, it's set to `true`, signifying that the model will be stored in a private repository.

### Training Args:

Expanding on the `training_args` within the `hyperparams` argument provides insight into the specific parameters governing the training process of the model. Here's a breakdown of the individual training arguments and their significance:

- **Learning Rate (`learning_rate`):**
   - Determines the step size at which the model parameters are updated during training. A higher learning rate may lead to faster convergence but risks overshooting optimal solutions, while a lower learning rate may ensure more stable training but may take longer to converge.

- **Per-device Train Batch Size (`per_device_train_batch_size`):**
   - Specifies the number of training samples processed in each batch per device during training. Adjusting this parameter can impact memory usage and training speed, with larger batch sizes potentially accelerating training but requiring more memory.

- **Per-device Eval Batch Size (`per_device_eval_batch_size`):**
   - Similar to `per_device_train_batch_size`, this parameter determines the batch size used for evaluation (validation) during training. It allows for efficient evaluation of the model's performance on validation data.

- **Number of Train Epochs (`num_train_epochs`):**
   - Defines the number of times the entire training dataset is passed through the model during training. Increasing the number of epochs can improve model performance up to a certain point, after which it may lead to overfitting.

- **Weight Decay (`weight_decay`):**
   - Introduces regularization by penalizing large weights in the model, thereby preventing overfitting. It helps to control the complexity of the model and improve generalization to unseen data.

- **Hub Token (`hub_token`):**
   - A token required for authentication when pushing the fine-tuned model to the Hugging Face Hub or accessing private repositories. It ensures secure communication with the Hub platform.

- **Push to Hub (`push_to_hub`):**
   - A boolean flag indicating whether the fine-tuned model should be uploaded to the Hugging Face Hub after training. Setting this parameter to `true` facilitates sharing and deployment of the model for wider usage.

- **Optimizer (`optim`):**
   - Specifies the optimization algorithm used during training. In the provided query, it's set to "adamw_bnb_8bit," indicating the use of the AdamW optimizer with gradient clipping and 8-bit quantization.

- **Gradient Accumulation Steps (`gradient_accumulation_steps`):**
   - Controls the accumulation of gradients over multiple batches before updating the model's parameters. It can help mitigate memory constraints and stabilize training, especially with large batch sizes.

- **Gradient Checkpointing (`gradient_checkpointing`):**
    - Enables gradient checkpointing, a memory-saving technique that trades off compute for memory during backpropagation. It allows training of larger models or with larger batch sizes without running out of memory.

Each of these training arguments plays a crucial role in shaping the training process, ensuring efficient convergence, regularization, and optimization of the model for the specific task at hand. Adjusting these parameters appropriately is essential for achieving optimal model performance.

### LORA Args:

Expanding on the `lora_config` within the `hyperparams` argument provides clarity on its role in configuring the LORA (Learned Optimizer and Rate Adaptation) algorithm:

- **Attention Radius (`r`):**
   - Specifies the radius of the attention window for the LORA algorithm. It determines the range of tokens considered for calculating attention weights, allowing the model to focus on relevant information while processing conversational data.

- **LORA Alpha (`lora_alpha`):**
   - Controls the strength of the learned regularization term in the LORA algorithm. A higher alpha value encourages sparsity in attention distributions, promoting selective attention and enhancing interpretability.

- **LORA Dropout (`lora_dropout`):**
   - Defines the dropout rate applied to the LORA attention scores during training. Dropout introduces noise to prevent overfitting and improve generalization by randomly zeroing out a fraction of attention weights.

- **Bias (`bias`):**
   - Determines whether bias terms are included in the LORA attention calculation. Bias terms can introduce additional flexibility to the attention mechanism, enabling the model to learn more complex relationships between tokens.

- **Task Type (`task_type`):**
   - Specifies the type of task for which the LORA algorithm is applied. In this context, it's set to "CAUSAL_LM" for causal language modeling, indicating that the model predicts the next token based on the previous tokens in the sequence.

Configuring these LORA arguments appropriately ensures that the attention mechanism of the model is optimized for processing conversational data, allowing it to capture relevant information and generate coherent responses effectively.

### Dataset Args:

Expanding on the `dataset_args` within the `hyperparams` argument provides insight into its role in processing the dataset:

- **System Column (`system_column`):**
   - Specifies the name or identifier of the column containing system responses (e.g., prompts or instructions) within the dataset. This column is crucial for distinguishing between different types of conversational turns and facilitating model training.

- **User Column (`user_column`):**
   - Indicates the column containing user inputs or queries within the dataset. These inputs form the basis for the model's understanding of user intentions, sentiments, or requests during training and inference.

- **Assistant Column (`assistant_column`):**
   - Refers to the column containing assistant outputs or responses generated by the model during training. These outputs serve as targets for the model to learn from and are compared against the actual responses during evaluation to assess model performance.

Configuring these dataset arguments ensures that the model is trained on the appropriate input-output pairs, enabling it to learn from the conversational data and generate contextually relevant responses.

Once the fine-tuning is completed, you will see the model in your Hugging Face repository (example: https://huggingface.co/santiadavani/fingpt-llama2-7b-chat). Since we are using LoRA to fine tune the model we only save the adapter weights (~2MB) instead of all the 7B weights (14GB) in Llama2-7b model.

## Inference
For inference, we will be utilizing the [OpenSourceAI](https://postgresml.org/docs/open-source/korvus/guides/opensourceai) class from the [pgml SDK](https://postgresml.org/docs/open-source/korvus/). Here's an example code snippet:

```python
import pgml

database_url = "DATABASE_URL"

client = pgml.OpenSourceAI(database_url)

results = client.chat_completions_create(
    {
        "model" : "santiadavani/fingpt-llama2-7b-chat",
        "token" : "TOKEN",
        "load_in_8bit": "true",
        "temperature" : 0.1,
        "repetition_penalty" : 1.5,
    },
    [
        {
            "role" : "system",
            "content" : "What is the sentiment of this news? Please choose an answer from {strong negative/moderately negative/mildly negative/neutral/mildly positive/moderately positive/strong positive}.",
        },
        {
            "role": "user",
            "content": "Starbucks says the workers violated safety policies while workers said they'd never heard of the policy before and are alleging retaliation.",
        },
    ]
)

print(results)
```

In this code snippet, we first import the pgml module and create an instance of the OpenSourceAI class, providing the necessary database URL. We then call the chat_completions_create method, specifying the model we want to use (in this case, "santiadavani/fingpt-llama2-7b-chat"), along with other parameters such as the token, whether to load the model in 8-bit precision, the temperature for sampling, and the repetition penalty.

The chat_completions_create method takes two arguments: a dictionary containing the model configuration and a list of dictionaries representing the chat conversation. In this example, the conversation consists of a system prompt asking for the sentiment of a given news snippet, and a user message containing the news text.

The results are:

```json
{
    "choices": [
        {
            "index": 0,
            "message": {
                "content": " Moderately negative ",
                "role": "assistant"
            }
        }
    ],
    "created": 1711144872,
    "id": "b663f701-db97-491f-b186-cae1086f7b79",
    "model": "santiadavani/fingpt-llama2-7b-chat",
    "object": "chat.completion",
    "system_fingerprint": "e36f4fa5-3d0b-e354-ea4f-950cd1d10787",
    "usage": {
        "completion_tokens": 0,
        "prompt_tokens": 0,
        "total_tokens": 0
    }
}
```

This dictionary contains the response from the language model, `santiadavani/fingpt-llama2-7b-chat`, for the given news text.

The key information in the response is:

1. `choices`: A list containing the model's response. In this case, there is only one choice.
2. `message.content`: The actual response from the model, which is " Moderately negative".
3. `model`: The name of the model used, "santiadavani/fingpt-llama2-7b-chat".
4. `created`: A timestamp indicating when the response was generated.
5. `id`: A unique identifier for this response.
6. `object`: Indicates that this is a "chat.completion" object.
7. `usage`: Information about the token usage for this response, although all values are 0 in this case.

So, the language model has analyzed the news text **_Starbucks says the workers violated safety policies while workers said they'd never heard of the policy before and are alleging retaliation._** and determined that the sentiment expressed in this text is **_Moderately negative_**
