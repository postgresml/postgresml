import os
import json
import math
import shutil
import time


import datasets
from rouge import Rouge
from sacrebleu.metrics import BLEU
from sentence_transformers import SentenceTransformer
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
import torch
from tqdm import tqdm
import transformers
from transformers import (
    AutoModelForCausalLM,
    AutoModelForQuestionAnswering,
    AutoModelForSeq2SeqLM,
    AutoModelForSequenceClassification,
    AutoTokenizer,
    DataCollatorForLanguageModeling,
    DataCollatorForSeq2Seq,
    DataCollatorWithPadding,
    DefaultDataCollator,
    GenerationConfig,
    TrainingArguments,
    Trainer,
)

__cache_transformer_by_model_id = {}
__cache_sentence_transformer_by_project_name = {}

def transform(task, args, inputs):
    task = json.loads(task)
    args = json.loads(args)
    inputs = json.loads(inputs)

    pipe = transformers.pipeline(**task)

    if pipe.task == "question-answering":
        inputs = [json.loads(input) for input in inputs]

    return json.dumps(pipe(inputs, **args))

def embed(project, text, kwargs: "{}"):
    kwargs = json.loads(kwargs)
    if project not in __cache_sentence_transformer_by_project_name:
        __cache_sentence_transformer_by_project_name[project] = SentenceTransformer(project)
    model = __cache_sentence_transformer_by_project_name[project]
    return model.encode(text, **kwargs)

def load_dataset(name, subset, limit: None, kwargs: "{}"):
    kwargs = json.loads(kwargs)

    if limit:
        dataset = datasets.load_dataset(name, subset, split=f"train[:{limit}]", **kwargs)
    else:
        dataset = datasets.load_dataset(name, subset, **kwargs)

    data = None
    types = None
    if isinstance(dataset, datasets.Dataset):
        data = dataset.to_dict()
        types = {name: feature.dtype for name, feature in dataset.features.items()}
    elif isinstance(dataset, datasets.DatasetDict):
        data = {}
        # Merge train/test splits, we'll re-split back in PostgresML.
        for name, split in dataset.items():
            types = {name: feature.dtype for name, feature in split.features.items()}
            for field, values in split.to_dict().items():
                if field in data:
                    data[field] += values
                else:
                    data[field] = values
    else:
        raise PgMLException(f"Unhandled dataset type: {type(dataset)}")

    return json.dumps({"data": data, "types": types})

def tokenize_text_classification(tokenizer, max_length, x, y):
    encoding = tokenizer(x, padding=True, truncation=True)
    encoding["label"] = y
    return datasets.Dataset.from_dict(encoding.data)

def tokenize_translation(tokenizer, max_length, x, y):
    encoding = tokenizer(x, max_length=max_length, truncation=True, text_target=y)
    return datasets.Dataset.from_dict(encoding.data)

def tokenize_summarization(tokenizer, max_length, x, y):
    encoding = tokenizer(x, max_length=max_length, truncation=True, text_target=y)
    return datasets.Dataset.from_dict(encoding.data)

def tokenize_text_generation(tokenizer, max_length, y):
    encoding = tokenizer(y, max_length=max_length)
    return datasets.Dataset.from_dict(encoding.data)

def tokenize_question_answering(tokenizer, max_length, x, y):
    pass

def compute_metrics_summarization(model, tokenizer, hyperparams, x, y):
    all_preds = []
    all_labels = y

    batch_size = hyperparams["per_device_eval_batch_size"]
    batches = int(math.ceil(len(y) / batch_size))
    with torch.no_grad():
        for i in range(batches):
            inputs = x[i * batch_size : min((i + 1) * batch_size, len(x))]
            tokens = tokenizer.batch_encode_plus(
                inputs,
                padding=True,
                truncation=True,
                return_tensors="pt",
                return_token_type_ids=False,
            ).to(model.device)
            predictions = model.generate(**tokens)
            decoded_preds = tokenizer.batch_decode(predictions, skip_special_tokens=True)
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
    batches = int(math.ceil(len(dataset) / batch_size))

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

def compute_metrics_translation(model, tokenizer, hyperparams, x, y):
    all_preds = []
    all_labels = y

    batch_size = hyperparams["per_device_eval_batch_size"]
    batches = int(math.ceil(len(y) / batch_size))
    with torch.no_grad():
        for i in range(batches):
            inputs = x[i * batch_size : min((i + 1) * batch_size, len(x))]
            tokens = tokenizer.batch_encode_plus(
                inputs,
                padding=True,
                truncation=True,
                return_tensors="pt",
                return_token_type_ids=False,
            ).to(model.device)
            predictions = model.generate(**tokens)
            decoded_preds = tokenizer.batch_decode(predictions, skip_special_tokens=True)
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

def compute_metrics_question_answering(model, tokenizer, hyperparams, x, y):
    batch_size = self.hyperparams["per_device_eval_batch_size"]
    batches = int(math.ceil(len(dataset) / batch_size))

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

def compute_metrics_text_generation(model, tokenizer, hyperparams, y):
    full_text = ""
    for entry in y:
        if entry:
            full_text += "\n\n" + entry

    encodings = tokenizer(full_text, return_tensors="pt")

    # TODO make these more configurable
    stride = 512
    config = model.config.to_dict()
    max_length = config.get("n_positions", 1024)

    stride = min(stride, max_length)
    seq_len = encodings.input_ids.size(1)

    nlls = []
    prev_end_loc = 0
    for begin_loc in tqdm(range(0, seq_len, stride)):
        end_loc = min(begin_loc + max_length, seq_len)
        trg_len = end_loc - prev_end_loc  # may be different from stride on last loop
        input_ids = encodings.input_ids[:, begin_loc:end_loc].to(model.device)
        target_ids = input_ids.clone()
        target_ids[:, :-trg_len] = -100

        with torch.no_grad():
            outputs = model(input_ids, labels=target_ids)

            # loss is calculated using CrossEntropyLoss which averages over input tokens.
            # Multiply it with trg_len to get the summation instead of average.
            # We will take average over all the tokens to get the true average
            # in the last step of this example.
            neg_log_likelihood = outputs.loss * trg_len

        nlls.append(neg_log_likelihood)

        prev_end_loc = end_loc
        if end_loc == seq_len:
            break

    perplexity = torch.exp(torch.stack(nlls).sum() / end_loc)

    return {
        "perplexity": perplexity
    }

def tune(task, hyperparams, path, x_train, x_test, y_train, y_test):
    hyperparams = json.loads(hyperparams)
    model_name = hyperparams.pop("model_name")
    tokenizer = AutoTokenizer.from_pretrained(model_name)

    algorithm = {}

    if task == "text-classification":
        model = AutoModelForSequenceClassification.from_pretrained(model_name, num_labels=2)
        train = tokenize_text_classification(tokenizer, max_length, x_train, y_train)
        test = tokenize_text_classification(tokenizer, max_length, x_test, y_test)
        data_collator = DefaultDataCollator()
    elif task == "question-answering":
        max_length = hyperparams.pop("max_length", None)
        algorithm["stride"] = hyperparams.pop("stride", 128)
        algorithm["model"] = AutoModelForQuestionAnswering.from_pretrained(model_name)
        train = tokenize_question_answering(tokenizer, max_length, x_train, y_train)
        test = tokenize_question_answering(tokenizer, max_length, x_test, y_test)
        data_collator = DefaultDataCollator()
    elif task == "summarization":
        max_length = hyperparams.pop("max_length", 1024)
        model = AutoModelForSeq2SeqLM.from_pretrained(model_name)
        train = tokenize_summarization(tokenizer, max_length, x_train, y_train)
        test = tokenize_summarization(tokenizer, max_length, x_test, y_test)
        data_collator = DataCollatorForSeq2Seq(tokenizer=tokenizer, model=model)
    elif task == "translation":
        max_length = hyperparams.pop("max_length", None)
        model = AutoModelForSeq2SeqLM.from_pretrained(model_name)
        train = tokenize_translation(tokenizer, max_length, x_train, y_train)
        test = tokenize_translation(tokenizer, max_length, x_test, y_test)
        data_collator = DataCollatorForSeq2Seq(tokenizer, model=model, return_tensors="pt")
    elif task == "text-generation":
        max_length = hyperparams.pop("max_length", None)
        tokenizer.pad_token = tokenizer.eos_token
        model = AutoModelForCausalLM.from_pretrained(model_name)
        model.resize_token_embeddings(len(tokenizer))
        train = tokenize_text_generation(tokenizer, max_length, y_train)
        test = tokenize_text_generation(tokenizer, max_length, y_test)
        data_collator = DataCollatorForLanguageModeling(tokenizer, mlm=False, return_tensors="pt")
    else:
        raise PgMLException(f"unhandled task type: {task}")
    trainer = Trainer(
        model=model,
        args=TrainingArguments(output_dir=path, **hyperparams),
        train_dataset=train,
        eval_dataset=test,
        tokenizer=tokenizer,
        data_collator=data_collator,
    )
    start = time.perf_counter()
    trainer.train()
    fit_time = time.perf_counter() - start
    model.eval()
    if torch.cuda.is_available():
        torch.cuda.empty_cache()

    # Test
    start = time.perf_counter()
    if task == "summarization":
        metrics = compute_metrics_summarization(model, tokenizer, hyperparams, x_test, y_test)
    elif task == "text-classification":
        metrics = compute_metrics_text_classification(model, tokenizer, hyperparams, x_test, y_test)
    elif task == "question-answering":
        metrics = compute_metrics_question_answering(model, tokenizer, hyperparams, x_test, y_test)
    elif task == "translation":
        metrics = compute_metrics_translation(model, tokenizer, hyperparams, x_test, y_test)
    elif task == "text-generation":
        metrics = compute_metrics_text_generation(model, tokenizer, hyperparams, y_test)
    else:
        raise PgMLException(f"unhandled task type: {task}")
    metrics["score_time"] = time.perf_counter() - start
    metrics["fit_time"] = fit_time

    # Save the results
    if os.path.isdir(path):
        shutil.rmtree(path, ignore_errors=True)
    trainer.save_model()

    return metrics

class MissingModelError(Exception):
    pass

def get_transformer_by_model_id(model_id):
    global __cache_transformer_by_model_id
    if model_id in __cache_transformer_by_model_id:
        return __cache_transformer_by_model_id[model_id]
    else:
        raise MissingModelError

def load_model(model_id, task, dir):
    if task == "summarization":
        __cache_transformer_by_model_id[model_id] = {
            "tokenizer": AutoTokenizer.from_pretrained(dir),
            "model": AutoModelForSeq2SeqLM.from_pretrained(dir),
        }
    elif task == "text-classification":
        __cache_transformer_by_model_id[model_id] = {
            "tokenizer": AutoTokenizer.from_pretrained(dir),
            "model": AutoModelForSequenceClassification.from_pretrained(dir),
        }
    elif task == "translation":
        __cache_transformer_by_model_id[model_id] = {
            "tokenizer": AutoTokenizer.from_pretrained(dir),
            "model": AutoModelForSeq2SeqLM.from_pretrained(dir),
        }
    elif task == "question-answering":
        __cache_transformer_by_model_id[model_id] = {
            "tokenizer": AutoTokenizer.from_pretrained(dir),
            "model": AutoModelForQuestionAnswering.from_pretrained(dir),
        }
    elif task == "text-generation":
        __cache_transformer_by_model_id[model_id] = {
            "tokenizer": AutoTokenizer.from_pretrained(dir),
            "model": AutoModelForCausalLM.from_pretrained(dir),
        }

    else:
        raise Exception(f"unhandled task type: {task}")

def generate(model_id, data, config):
    result = get_transformer_by_model_id(model_id)
    tokenizer = result["tokenizer"]
    model = result["model"]
    config = json.loads(config)
    all_preds = []

    batch_size = 1 # TODO hyperparams
    batches = int(math.ceil(len(data) / batch_size))

    with torch.no_grad():
        for i in range(batches):
            start = i * batch_size
            end = min((i + 1) * batch_size, len(data))
            tokens = tokenizer.batch_encode_plus(
                data[start:end],
                padding=True,
                truncation=True,
                return_tensors="pt",
                return_token_type_ids=False,
            ).to(model.device)
            predictions = model.generate(**tokens, **config)
            decoded_preds = tokenizer.batch_decode(predictions, skip_special_tokens=True)
            all_preds.extend(decoded_preds)
    return all_preds
