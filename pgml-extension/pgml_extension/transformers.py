import logging

from typing import OrderedDict

import plpy
import transformers
import datasets
import pickle
import json
import math
import torch
import numpy
import os
import string, re
import shutil
from transformers import (
    AutoTokenizer,
    DataCollatorWithPadding,
    DefaultDataCollator,
    DataCollatorForSeq2Seq,
    AutoModelForSequenceClassification,
    AutoModelForQuestionAnswering,
    AutoModelForSeq2SeqLM,
    TrainingArguments,
    Trainer,
)
from sacrebleu.metrics import BLEU
from rouge import Rouge
from sklearn.metrics import (
    mean_squared_error,
    r2_score,
    f1_score,
    precision_score,
    recall_score,
    roc_auc_score,
    accuracy_score,
    log_loss,
)
from .utils import timer
from .sql import q, c
from .exceptions import PgMLException

from .model import Project, Snapshot, Model as BaseModel

_pipeline_cache = {}


def transform(task, args, inputs):
    cache = args.pop("cache", True)

    # construct the cache key from task
    key = task
    if type(key) == dict:
        key = tuple(sorted(key.items()))

    if cache and key in _pipeline_cache:
        pipe = _pipeline_cache.get(key)
    else:
        with timer("Initializing pipeline"):
            if type(task) == str:
                pipe = transformers.pipeline(task)
            else:
                pipe = transformers.pipeline(**task)
            if cache:
                _pipeline_cache[key] = pipe

    if pipe.task == "question-answering":
        inputs = [json.loads(input) for input in inputs]

    with timer("inference"):
        result = pipe(inputs, **args)

    return result


class Model(BaseModel):
    @property
    def algorithm(self):
        if self._algorithm is None:
            files = plpy.execute(f"SELECT * FROM pgml.files WHERE model_id = {self.id} ORDER BY part ASC")
            for file in files:
                dir = os.path.dirname(file["path"])
                if not os.path.isdir(dir):
                    os.makedirs(dir)
                if file["part"] == 0:
                    with open(file["path"], mode="wb") as handle:
                        handle.write(file["data"])
                else:
                    with open(file["path"], mode="ab") as handle:
                        handle.write(file["data"])

            if os.path.exists(self.path):
                source = self.path
            else:
                source = self.algorithm_name

            if source is None or source == "":
                pipeline = transformers.pipeline(self.task)
                self._algorithm = {
                    "tokenizer": pipeline.tokenizer,
                    "model": pipeline.model,
                }
            elif self.project.task == "summarization":
                self._algorithm = {
                    "tokenizer": AutoTokenizer.from_pretrained(source),
                    "model": AutoModelForSeq2SeqLM.from_pretrained(source),
                }
            elif self.project.task == "text-classification":
                self._algorithm = {
                    "tokenizer": AutoTokenizer.from_pretrained(source),
                    "model": AutoModelForSequenceClassification.from_pretrained(source),
                }
            elif self.project.task_type == "translation":
                task = self.project.task.split("_")
                self._algorithm = {
                    "from": task[1],
                    "to": task[3],
                    "tokenizer": AutoTokenizer.from_pretrained(source),
                    "model": AutoModelForSeq2SeqLM.from_pretrained(source),
                }
            elif self.project.task == "question-answering":
                self._algorithm = {
                    "tokenizer": AutoTokenizer.from_pretrained(source),
                    "model": AutoModelForQuestionAnswering.from_pretrained(source),
                }
            else:
                raise PgMLException(f"unhandled task type: {self.project.task}")

        return self._algorithm

    def train(self):
        dataset = self.snapshot.dataset

        self._algorithm = {"tokenizer": AutoTokenizer.from_pretrained(self.algorithm_name)}
        if self.project.task == "text-classification":
            self._algorithm["model"] = AutoModelForSequenceClassification.from_pretrained(
                self.algorithm_name, num_labels=2
            )
            tokenized_dataset = self.tokenize_text_classification(dataset)
            data_collator = DefaultDataCollator()
        elif self.project.task == "question-answering":
            self._algorithm["max_length"] = self.hyperparams.pop("max_length", 384)
            self._algorithm["stride"] = self.hyperparams.pop("stride", 128)
            self._algorithm["model"] = AutoModelForQuestionAnswering.from_pretrained(self.algorithm_name)
            tokenized_dataset = self.tokenize_question_answering(dataset)
            data_collator = DefaultDataCollator()
        elif self.project.task == "summarization":
            self._algorithm["max_summary_length"] = self.hyperparams.pop("max_summary_length", 1024)
            self._algorithm["max_input_length"] = self.hyperparams.pop("max_input_length", 128)
            self._algorithm["model"] = AutoModelForSeq2SeqLM.from_pretrained(self.algorithm_name)
            tokenized_dataset = self.tokenize_summarization(dataset)
            data_collator = DataCollatorForSeq2Seq(tokenizer=self.tokenizer, model=self.model)
        elif self.project.task.startswith("translation"):
            task = self.project.task.split("_")
            if task[0] != "translation" and task[2] != "to":
                raise PgMLException(f"unhandled translation task: {self.project.task}")
            self._algorithm["max_length"] = self.hyperparams.pop("max_length", None)
            self._algorithm["from"] = task[1]
            self._algorithm["to"] = task[3]
            self._algorithm["model"] = AutoModelForSeq2SeqLM.from_pretrained(self.algorithm_name)
            tokenized_dataset = self.tokenize_translation(dataset)
            data_collator = DataCollatorForSeq2Seq(self.tokenizer, model=self.model, return_tensors="pt")
        else:
            raise PgMLException(f"unhandled task type: {self.project.task}")

        training_args = TrainingArguments(
            output_dir=self.path,
            **self.hyperparams,
        )

        trainer = Trainer(
            model=self.model,
            args=training_args,
            train_dataset=tokenized_dataset["train"],
            eval_dataset=tokenized_dataset["test"],
            tokenizer=self.tokenizer,
            data_collator=data_collator,
        )

        trainer.train()

        self.model.eval()

        if torch.cuda.is_available():
            torch.cuda.empty_cache()

        # Test
        if self.project.task == "summarization":
            self.metrics = self.compute_metrics_summarization(dataset["test"])
        elif self.project.task == "text-classification":
            self.metrics = self.compute_metrics_text_classification(dataset["test"])
        elif self.project.task == "question-answering":
            self.metrics = self.compute_metrics_question_answering(dataset["test"])
        elif self.project.task.startswith("translation"):
            self.metrics = self.compute_metrics_translation(dataset["test"])
        else:
            raise PgMLException(f"unhandled task type: {self.project.task}")

        # Save the results
        if os.path.isdir(self.path):
            shutil.rmtree(self.path, ignore_errors=True)
        trainer.save_model()
        for filename in os.listdir(self.path):
            path = os.path.join(self.path, filename)
            part = 0
            max_size = 100_000_000
            with open(path, mode="rb") as file:
                while True:
                    data = file.read(max_size)
                    if not data:
                        break
                    plpy.execute(
                        f"""
                        INSERT into pgml.files (model_id, path, part, data) 
                        VALUES ({q(self.id)}, {q(path)}, {q(part)}, '\\x{data.hex()}')
                        """
                    )
                    part += 1
        shutil.rmtree(self.path, ignore_errors=True)

    def tokenize_summarization(self, dataset):
        feature = self.snapshot.feature_names[0]
        label = self.snapshot.y_column_name[0]

        max_input_length = self.algorithm["max_input_length"]
        max_summary_length = self.algorithm["max_summary_length"]

        def preprocess_function(examples):
            inputs = [doc for doc in examples[feature]]
            model_inputs = self.tokenizer(inputs, max_length=max_input_length, truncation=True)

            with self.tokenizer.as_target_tokenizer():
                labels = self.tokenizer(examples[label], max_length=max_summary_length, truncation=True)

            model_inputs["labels"] = labels["input_ids"]
            return model_inputs

        return dataset.map(preprocess_function, batched=True, remove_columns=dataset["train"].column_names)

    def tokenize_text_classification(self, dataset):
        # text classification only supports a single feature other than the label
        feature = self.snapshot.feature_names[0]
        tokenizer = self.tokenizer

        def preprocess_function(examples):
            return tokenizer(examples[feature], padding=True, truncation=True)

        return dataset.map(preprocess_function, batched=True)

    def tokenize_translation(self, dataset):
        max_length = self.algorithm["max_length"]

        def preprocess_function(examples):
            inputs = [ex[self.algorithm["from"]] for ex in examples[self.snapshot.y_column_name[0]]]
            targets = [ex[self.algorithm["to"]] for ex in examples[self.snapshot.y_column_name[0]]]
            model_inputs = self.tokenizer(inputs, max_length=max_length, truncation=True)

            # Set up the tokenizer for targets
            with self.tokenizer.as_target_tokenizer():
                labels = self.tokenizer(targets, max_length=max_length, truncation=True)

            model_inputs["labels"] = labels["input_ids"]
            return model_inputs

        return dataset.map(preprocess_function, batched=True, remove_columns=dataset["train"].column_names)

    def tokenize_question_answering(self, dataset):
        tokenizer = self._algorithm["tokenizer"]

        def preprocess_function(examples):
            questions = [q.strip() for q in examples["question"]]
            inputs = tokenizer(
                questions,
                examples["context"],
                max_length=self.algorithm["max_length"],
                stride=self.algorithm["stride"],
                truncation="only_second",
                return_offsets_mapping=True,
                return_overflowing_tokens=True,
                padding="max_length",
            )

            offset_mapping = inputs.pop("offset_mapping")
            sample_map = inputs.pop("overflow_to_sample_mapping")

            answers = examples[self.snapshot.y_column_name[0]]
            start_positions = []
            end_positions = []

            for i, offset in enumerate(offset_mapping):
                sample_idx = sample_map[i]
                answer = answers[sample_idx]
                # If there is no answer available, label it (0, 0)
                if len(answer["answer_start"]) == 0:
                    start_positions.append(0)
                    end_positions.append(0)
                    continue

                start_char = answer["answer_start"][0]
                end_char = answer["answer_start"][0] + len(answer["text"][0])
                sequence_ids = inputs.sequence_ids(i)

                # Find the start and end of the context
                idx = 0
                while sequence_ids[idx] != 1:
                    idx += 1
                context_start = idx
                while sequence_ids[idx] == 1:
                    idx += 1
                context_end = idx - 1

                # If the answer is not fully inside the context, label it (0, 0)
                if offset[context_start][0] > end_char or offset[context_end][1] < start_char:
                    start_positions.append(0)
                    end_positions.append(0)
                else:
                    # Otherwise it's the start and end token positions
                    idx = context_start
                    while idx <= context_end and offset[idx][0] <= start_char:
                        idx += 1
                    start_positions.append(idx - 1)

                    idx = context_end
                    while idx >= context_start and offset[idx][1] >= end_char:
                        idx -= 1
                    end_positions.append(idx + 1)

            inputs["start_positions"] = start_positions
            inputs["end_positions"] = end_positions
            return inputs

        return dataset.map(preprocess_function, batched=True, remove_columns=dataset["train"].column_names)

    def compute_metrics_summarization(self, dataset):
        feature = self.snapshot.feature_names[0]
        label = self.snapshot.y_column_name[0]

        all_preds = []
        all_labels = [d for d in dataset[label]]

        batch_size = self.hyperparams["per_device_eval_batch_size"]
        batches = int(math.ceil(len(dataset) / batch_size))

        with torch.no_grad():
            for i in range(batches):
                slice = dataset.select(range(i * batch_size, min((i + 1) * batch_size, len(dataset))))
                inputs = slice[feature]
                tokens = self.tokenizer.batch_encode_plus(
                    inputs,
                    padding=True,
                    truncation=True,
                    return_tensors="pt",
                    return_token_type_ids=False,
                ).to(self.model.device)
                predictions = self.model.generate(**tokens)
                decoded_preds = self.tokenizer.batch_decode(predictions, skip_special_tokens=True)
                all_preds.extend(decoded_preds)

        bleu = BLEU().corpus_score(all_preds, [[l] for l in all_labels])
        rouge = Rouge().get_scores(all_preds, all_labels, avg=True)
        return {
            "bleu": bleu.score,
            "rouge_ngram_f1": rouge["rouge-1"]["f"],
            "rouge_ngram_precision": rouge["rouge-1"]["p"],
            "rouge_ngram_recall": rouge["rouge-1"]["r"],
            "rouge_bigram_f1": rouge["rouge-2"]["f"],
            "rouge_bigram_precision": rouge["rouge-2"]["p"],
            "rouge_bigram_recall": rouge["rouge-2"]["r"],
        }

    def compute_metrics_text_classification(self, dataset):
        feature = label = None
        for name, type in dataset.features.items():
            if isinstance(type, datasets.features.features.ClassLabel):
                label = name
            elif isinstance(type, datasets.features.features.Value):
                feature = name
            else:
                raise PgMLException(f"Unhandled feature type: {type}")
        logits = torch.Tensor(device="cpu")

        batch_size = self.hyperparams["per_device_eval_batch_size"]
        batches = int(len(dataset) / batch_size) + 1

        with torch.no_grad():
            for i in range(batches):
                slice = dataset.select(range(i * batch_size, min((i + 1) * batch_size, len(dataset))))
                tokens = self.tokenizer(slice[feature], padding=True, truncation=True, return_tensors="pt")
                tokens.to(self.model.device)
                result = self.model(**tokens).logits.to("cpu")
                logits = torch.cat((logits, result), 0)

        metrics = {}

        y_pred = logits.argmax(-1)
        y_prob = torch.nn.functional.softmax(logits, dim=-1)
        y_test = numpy.array(dataset[label]).flatten()

        metrics["mean_squared_error"] = mean_squared_error(y_test, y_pred)
        metrics["r2"] = r2_score(y_test, y_pred)
        metrics["f1"] = f1_score(y_test, y_pred, average="weighted")
        metrics["precision"] = precision_score(y_test, y_pred, average="weighted")
        metrics["recall"] = recall_score(y_test, y_pred, average="weighted")
        metrics["accuracy"] = accuracy_score(y_test, y_pred)
        metrics["log_loss"] = log_loss(y_test, y_prob)
        roc_auc_y_prob = y_prob
        if y_prob.shape[1] == 2:  # binary classification requires only the greater label by passed to roc_auc_score
            roc_auc_y_prob = y_prob[:, 1]
        metrics["roc_auc"] = roc_auc_score(y_test, roc_auc_y_prob, average="weighted", multi_class="ovo")

        return metrics

    def compute_metrics_translation(self, dataset):
        all_preds = []
        all_labels = [d[self.algorithm["to"]] for d in dataset[self.snapshot.y_column_name[0]]]

        batch_size = self.hyperparams["per_device_eval_batch_size"]
        batches = int(len(dataset) / batch_size) + 1

        with torch.no_grad():
            for i in range(batches):
                slice = dataset.select(range(i * batch_size, min((i + 1) * batch_size, len(dataset))))
                inputs = [ex[self.algorithm["from"]] for ex in slice[self.snapshot.y_column_name[0]]]
                tokens = self.tokenizer.batch_encode_plus(
                    inputs,
                    padding=True,
                    truncation=True,
                    return_tensors="pt",
                    return_token_type_ids=False,
                ).to(self.model.device)
                predictions = self.model.generate(**tokens)
                decoded_preds = self.tokenizer.batch_decode(predictions, skip_special_tokens=True)
                all_preds.extend(decoded_preds)

        bleu = BLEU().corpus_score(all_preds, [[l] for l in all_labels])
        rouge = Rouge().get_scores(all_preds, all_labels, avg=True)
        return {
            "bleu": bleu.score,
            "rouge_ngram_f1": rouge["rouge-1"]["f"],
            "rouge_ngram_precision": rouge["rouge-1"]["p"],
            "rouge_ngram_recall": rouge["rouge-1"]["r"],
            "rouge_bigram_f1": rouge["rouge-2"]["f"],
            "rouge_bigram_precision": rouge["rouge-2"]["p"],
            "rouge_bigram_recall": rouge["rouge-2"]["r"],
        }

    def compute_metrics_question_answering(self, dataset):
        batch_size = self.hyperparams["per_device_eval_batch_size"]
        batches = int(len(dataset) / batch_size) + 1

        with torch.no_grad():
            for i in range(batches):
                slice = dataset.select(range(i * batch_size, min((i + 1) * batch_size, len(dataset))))
                tokens = self.algorithm["tokenizer"].encode_plus(
                    slice["question"], slice["context"], return_tensors="pt"
                )
                tokens.to(self.algorithm["model"].device)
                outputs = self.algorithm["model"](**tokens)
                answer_start = torch.argmax(outputs[0])
                answer_end = torch.argmax(outputs[1]) + 1
                answer = self.algorithm["tokenizer"].convert_tokens_to_string(
                    self.algorithm["tokenizer"].convert_ids_to_tokens(tokens["input_ids"][0][answer_start:answer_end])
                )

        def compute_exact_match(prediction, truth):
            return int(normalize_text(prediction) == normalize_text(truth))

        def compute_f1(prediction, truth):
            pred_tokens = normalize_text(prediction).split()
            truth_tokens = normalize_text(truth).split()

            # if either the prediction or the truth is no-answer then f1 = 1 if they agree, 0 otherwise
            if len(pred_tokens) == 0 or len(truth_tokens) == 0:
                return int(pred_tokens == truth_tokens)

            common_tokens = set(pred_tokens) & set(truth_tokens)

            # if there are no common tokens then f1 = 0
            if len(common_tokens) == 0:
                return 0

            prec = len(common_tokens) / len(pred_tokens)
            rec = len(common_tokens) / len(truth_tokens)

            return 2 * (prec * rec) / (prec + rec)

        def get_gold_answers(example):
            """helper function that retrieves all possible true answers from a squad2.0 example"""

            gold_answers = [answer["text"] for answer in example.answers if answer["text"]]

            # if gold_answers doesn't exist it's because this is a negative example -
            # the only correct answer is an empty string
            if not gold_answers:
                gold_answers = [""]

            return gold_answers

        metrics = {}
        metrics["exact_match"] = 0

        return metrics

    def predict(self, data: list):
        return [int(logit.argmax()) for logit in self.predict_logits(data)][0]

    def predict_proba(self, data: list):
        return torch.nn.functional.softmax(self.predict_logits(data), dim=-1).tolist()

    def generate(self, data: list):
        if self.project.task_type == "summarization":
            return self.generate_summarization(data)
        elif self.project.task_type == "translation":
            return self.generate_translation(data)
        raise PgMLException(f"unhandled task: {self.project.task}")

    def predict_logits(self, data: list):
        if self.project.task == "text-classification":
            return self.predict_logits_text_classification(data)
        elif self.project.task == "question-answering":
            return self.predict_logits_question_answering(data)
        raise PgMLException(f"unhandled task: {self.project.task}")

    def predict_logits_text_classification(self, data: list):
        tokens = self.tokenizer(data, padding=True, truncation=True, return_tensors="pt")
        with torch.no_grad():
            return self.model(**tokens).logits

    def predict_logits_question_answering(self, data: list):
        question = [d["question"] for d in data]
        context = [d["context"] for d in data]

        inputs = self.tokenizer.encode_plus(question, context, padding=True, truncation=True, return_tensors="pt")
        with torch.no_grad():
            outputs = self.model(**inputs)

        answer_start = torch.argmax(outputs[0])  # get the most likely beginning of answer with the argmax of the score
        answer_end = torch.argmax(outputs[1]) + 1

        answer = self.tokenizer.convert_tokens_to_string(
            self.tokenizer.convert_ids_to_tokens(inputs["input_ids"][0][answer_start:answer_end])
        )

        return answer

    def generate_summarization(self, data: list):
        all_preds = []

        batch_size = self.hyperparams["per_device_eval_batch_size"]
        batches = int(len(data) / batch_size) + 1

        with torch.no_grad():
            for i in range(batches):
                start = i * batch_size
                end = min((i + 1) * batch_size, len(data))
                tokens = self.tokenizer.batch_encode_plus(
                    data[start:end],
                    padding=True,
                    truncation=True,
                    return_tensors="pt",
                    return_token_type_ids=False,
                ).to(self.model.device)
                predictions = self.model.generate(**tokens)
                decoded_preds = self.tokenizer.batch_decode(predictions, skip_special_tokens=True)
                all_preds.extend(decoded_preds)
        return all_preds

    def generate_translation(self, data: list):
        all_preds = []

        batch_size = self.hyperparams["per_device_eval_batch_size"]
        batches = int(len(data) / batch_size) + 1

        with torch.no_grad():
            for i in range(batches):
                start = i * batch_size
                end = min((i + 1) * batch_size, len(data))
                tokens = self.tokenizer.batch_encode_plus(
                    data[start:end],
                    padding=True,
                    truncation=True,
                    return_tensors="pt",
                    return_token_type_ids=False,
                ).to(self.model.device)
                predictions = self.model.generate(**tokens)
                decoded_preds = self.tokenizer.batch_decode(predictions, skip_special_tokens=True)
                all_preds.extend(decoded_preds)
        return all_preds

    @property
    def tokenizer(self):
        return self.algorithm["tokenizer"]

    @property
    def model(self):
        return self.algorithm["model"]


def tune(
    project_name: str,
    task: str = None,
    relation_name: str = None,
    y_column_name: str = None,
    model_name: str = None,
    hyperparams: dict = {},
    search: str = None,
    search_params: dict = {},
    search_args: dict = {},
    test_size: float or int = 0.25,
    test_sampling: str = "random",
):
    # Project
    try:
        project = Project.find_by_name(project_name)
        if task is not None and task != project.task:
            raise PgMLException(
                f"Project `{project_name}` already exists with a different task: `{project.task}`. Create a new project instead."
            )
        task = project.task
    except PgMLException:
        project = Project.create(project_name, task)

    # Create or use an existing snapshot.
    if relation_name is None:
        snapshot = project.last_snapshot
        if snapshot is None:
            raise PgMLException(
                f"You must pass a `relation_name` and `y_column_name` to snapshot the first time you train a model."
            )
        if y_column_name is not None and y_column_name != [None] and y_column_name != snapshot.y_column_name:
            raise PgMLException(
                f"You must pass a `relation_name` to use a different `y_column_name` than previous runs. {y_column_name} vs {snapshot.y_column_name}"
            )
    else:
        snapshot = Snapshot.create(relation_name, y_column_name, test_size, test_sampling)

    # Model
    if model_name is None:
        raise PgMLException(f"You must pass a `model_name` to fine tune.")

    model = Model.create(project, snapshot, model_name, hyperparams, search, search_params, search_args)
    model.fit(snapshot)

    # Deployment
    if (
        project.deployed_model is None
        or project.deployed_model.metrics[project.key_metric_name] < model.metrics[project.key_metric_name]
    ):
        model.deploy("new_score")
        return "deployed"
    else:
        return "not deployed"
