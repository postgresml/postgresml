import math
import os
import shutil
import time
import queue
import sys
import json
from datetime import datetime

import datasets
import numpy
import orjson
from rouge import Rouge
from sacrebleu.metrics import BLEU
from sentence_transformers import SentenceTransformer, CrossEncoder
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
    PegasusForConditionalGeneration,
    PegasusTokenizer,
    TrainingArguments,
    Trainer,
    GPTQConfig,
    PegasusForConditionalGeneration,
    PegasusTokenizer,
    TrainerCallback,
)

import threading
import logging
import evaluate
import torch.nn.functional as F
from trl import SFTTrainer, DataCollatorForCompletionOnlyLM
from trl.trainer import ConstantLengthDataset
from peft import LoraConfig, get_peft_model
from abc import abstractmethod

transformers.logging.set_verbosity_info()


__cache_transformer_by_model_id = {}
__cache_sentence_transformer_by_name = {}
__cache_transform_pipeline_by_task = {}

DTYPE_MAP = {
    "uint8": torch.uint8,
    "int8": torch.int8,
    "int16": torch.int16,
    "int32": torch.int32,
    "int64": torch.int64,
    "bfloat16": torch.bfloat16,
    "float16": torch.float16,
    "float32": torch.float32,
    "float64": torch.float64,
    "complex64": torch.complex64,
    "complex128": torch.complex128,
    "bool": torch.bool,
}


class WorkerThreads:
    def __init__(self):
        self.worker_threads = {}

    def delete_thread(self, id):
        del self.worker_threads[id]

    def update_thread(self, id, value):
        self.worker_threads[id] = value

    def get_thread(self, id):
        if id in self.worker_threads:
            return self.worker_threads[id]
        else:
            return None


worker_threads = WorkerThreads()


class PgMLException(Exception):
    pass


def orjson_default(obj):
    if isinstance(obj, numpy.float32):
        return float(obj)
    raise TypeError


def convert_dtype(kwargs):
    if "torch_dtype" in kwargs:
        kwargs["torch_dtype"] = DTYPE_MAP[kwargs["torch_dtype"]]


def convert_eos_token(tokenizer, args):
    if "eos_token" in args:
        args["eos_token_id"] = tokenizer.convert_tokens_to_ids(args.pop("eos_token"))


def ensure_device(kwargs):
    device = kwargs.get("device")
    device_map = kwargs.get("device_map")
    if device is None and device_map is None:
        if torch.cuda.is_available():
            kwargs["device"] = "cuda:" + str(os.getpid() % torch.cuda.device_count())
        else:
            kwargs["device"] = "cpu"


# Follows BaseStreamer template from transformers library
class TextIteratorStreamer:
    def __init__(self, tokenizer, skip_prompt=False, timeout=None, **decode_kwargs):
        self.tokenizer = tokenizer
        self.skip_prompt = skip_prompt
        self.timeout = timeout
        self.decode_kwargs = decode_kwargs
        self.next_tokens_are_prompt = True
        self.stop_signal = None
        self.text_queue = queue.Queue()
        self.token_cache = []
        self.text_index_cache = []

    def set_worker_thread_id(self, id):
        self.worker_thread_id = id

    def get_worker_thread_id(self):
        return self.worker_thread_id

    def put(self, values):
        if self.skip_prompt and self.next_tokens_are_prompt:
            self.next_tokens_are_prompt = False
            return
        output = []
        for i, v in enumerate(values):
            if len(self.token_cache) <= i:
                self.token_cache.append([])
                self.text_index_cache.append(0)
            token = v.tolist()  # Returns a list or number
            if type(token) == list:
                self.token_cache[i].extend(token)
            else:
                self.token_cache[i].append(token)
            text = self.tokenizer.decode(self.token_cache[i], **self.decode_kwargs)
            if text.endswith("\n"):
                output.append(text[self.text_index_cache[i] :])
                self.token_cache[i] = []
                self.text_index_cache[i] = 0
            else:
                printable_text = text[self.text_index_cache[i] : text.rfind(" ") + 1]
                self.text_index_cache[i] += len(printable_text)
                output.append(printable_text)
        if any(output):
            self.text_queue.put(output)

    def end(self):
        self.next_tokens_are_prompt = True
        output = []
        for i, tokens in enumerate(self.token_cache):
            text = self.tokenizer.decode(tokens, **self.decode_kwargs)
            output.append(text[self.text_index_cache[i] :])
        self.text_queue.put(output)
        self.text_queue.put(self.stop_signal)

    def __iter__(self):
        return self

    def __next__(self):
        value = self.text_queue.get(timeout=self.timeout)
        if value != self.stop_signal:
            return value


def streaming_worker(worker_threads, model, **kwargs):
    thread_id = threading.get_native_id()
    try:
        worker_threads.update_thread(
            thread_id, json.dumps({"model": model.name_or_path})
        )
    except:
        worker_threads.update_thread(thread_id, "Error setting data")
    try:
        model.generate(**kwargs)
        worker_threads.delete_thread(thread_id)
    except BaseException as error:
        worker_threads.update_thread(thread_id, f"Error in streaming_worker: {error}")


class GGMLPipeline(object):
    def __init__(self, model_name, **task):
        import ctransformers

        task.pop("model", None)
        task.pop("task", None)
        task.pop("device", None)
        self.model = ctransformers.AutoModelForCausalLM.from_pretrained(
            model_name, **task
        )
        self.tokenizer = None
        self.task = "text-generation"

    def stream(self, inputs, **kwargs):
        output = self.model(inputs[0], stream=True, **kwargs)
        return ThreadedGeneratorIterator(output, inputs[0])

    def __call__(self, inputs, **kwargs):
        outputs = []
        for input in inputs:
            outputs.append(self.model(input, **kwargs))
        return outputs


class ThreadedGeneratorIterator:
    def __init__(self, output, starting_input):
        self.output = output
        self.done = False
        self.q = queue.Queue()
        self.q.put(starting_input)

        def do_work():
            for x in self.output:
                self.q.put(x)
            self.done = True

        thread = threading.Thread(target=do_work)
        thread.start()

    def __iter__(self):
        return self

    def __next__(self):
        if not self.done or not self.q.empty():
            v = self.q.get()
            self.q.task_done()
            return v


class StandardPipeline(object):
    def __init__(self, model_name, **kwargs):
        # the default pipeline constructor doesn't pass all the kwargs (particularly load_in_4bit)
        # to the model constructor, so we construct the model/tokenizer manually if possible,
        # but that is only possible when the task is passed in, since if you pass the model
        # to the pipeline constructor, the task will no longer be inferred from the default...

        # See: https://huggingface.co/docs/hub/security-tokens
        # This renaming is for backwards compatability
        if "use_auth_token" in kwargs:
            kwargs["token"] = kwargs.pop("use_auth_token")

        self.model_name = model_name

        if (
            "task" in kwargs
            and model_name is not None
            and kwargs["task"]
            in [
                "text-classification",
                "question-answering",
                "summarization",
                "translation",
                "text-generation",
                "conversational",
            ]
        ):
            self.task = kwargs.pop("task")
            kwargs.pop("model", None)
            if self.task == "text-classification":
                self.model = AutoModelForSequenceClassification.from_pretrained(
                    model_name, **kwargs
                )
            elif self.task == "question-answering":
                self.model = AutoModelForQuestionAnswering.from_pretrained(
                    model_name, **kwargs
                )
            elif self.task == "summarization" or self.task == "translation":
                if model_name == "google/pegasus-xsum":
                    # HF auto model doesn't detect GPUs
                    self.model = PegasusForConditionalGeneration.from_pretrained(
                        model_name
                    )
                else:
                    self.model = AutoModelForSeq2SeqLM.from_pretrained(
                        model_name, **kwargs
                    )
            elif self.task == "text-generation" or self.task == "conversational":
                # See: https://huggingface.co/docs/transformers/main/quantization
                if "quantization_config" in kwargs:
                    quantization_config = kwargs.pop("quantization_config")
                    quantization_config = GPTQConfig(**quantization_config)
                    self.model = AutoModelForCausalLM.from_pretrained(
                        model_name, quantization_config=quantization_config, **kwargs
                    )
                else:
                    self.model = AutoModelForCausalLM.from_pretrained(
                        model_name, **kwargs
                    )
            else:
                raise PgMLException(f"Unhandled task: {self.task}")

            if model_name == "google/pegasus-xsum":
                kwargs.pop("token", None)

            if "token" in kwargs:
                self.tokenizer = AutoTokenizer.from_pretrained(
                    model_name, token=kwargs["token"]
                )
            else:
                if model_name == "google/pegasus-xsum":
                    self.tokenizer = PegasusTokenizer.from_pretrained(model_name)
                else:
                    self.tokenizer = AutoTokenizer.from_pretrained(model_name)

            pipe_kwargs = {
                "model": self.model,
                "tokenizer": self.tokenizer,
            }

            # https://huggingface.co/docs/transformers/en/model_doc/pegasus
            if model_name == "google/pegasus-xsum":
                pipe_kwargs["device"] = kwargs.get("device", "cpu")

            self.pipe = transformers.pipeline(
                self.task,
                **pipe_kwargs,
            )
        else:
            self.pipe = transformers.pipeline(**kwargs)
            self.tokenizer = self.pipe.tokenizer
            self.task = self.pipe.task
            self.model = self.pipe.model

        # Make sure we set the pad token if it does not exist
        if self.tokenizer.pad_token is None:
            self.tokenizer.pad_token = self.tokenizer.eos_token

    def stream(self, input, timeout=None, **kwargs):
        streamer = None
        generation_kwargs = None
        if self.task == "conversational":
            streamer = TextIteratorStreamer(
                self.tokenizer,
                timeout=timeout,
                skip_prompt=True,
                skip_special_tokens=True,
            )
            if "chat_template" in kwargs:
                input = self.tokenizer.apply_chat_template(
                    input,
                    add_generation_prompt=True,
                    tokenize=False,
                    chat_template=kwargs.pop("chat_template"),
                )
            else:
                input = self.tokenizer.apply_chat_template(
                    input, add_generation_prompt=True, tokenize=False
                )
            input = self.tokenizer(input, return_tensors="pt").to(self.model.device)
            generation_kwargs = dict(
                input,
                worker_threads=worker_threads,
                model=self.model,
                streamer=streamer,
                **kwargs,
            )
        else:
            streamer = TextIteratorStreamer(
                self.tokenizer, timeout=timeout, skip_special_tokens=True
            )
            input = self.tokenizer(input, return_tensors="pt", padding=True).to(
                self.model.device
            )
            generation_kwargs = dict(
                input,
                worker_threads=worker_threads,
                model=self.model,
                streamer=streamer,
                **kwargs,
            )
        # thread = Thread(target=self.model.generate, kwargs=generation_kwargs)
        thread = threading.Thread(target=streaming_worker, kwargs=generation_kwargs)
        thread.start()
        streamer.set_worker_thread_id(thread.native_id)
        return streamer

    def __call__(self, inputs, **kwargs):
        if self.task == "conversational":
            if "chat_template" in kwargs:
                inputs = self.tokenizer.apply_chat_template(
                    inputs,
                    add_generation_prompt=True,
                    tokenize=False,
                    chat_template=kwargs.pop("chat_template"),
                )
            else:
                inputs = self.tokenizer.apply_chat_template(
                    inputs, add_generation_prompt=True, tokenize=False
                )
            inputs = self.tokenizer(inputs, return_tensors="pt").to(self.model.device)
            args = dict(inputs, **kwargs)
            outputs = self.model.generate(**args)
            # We only want the new ouputs for conversational pipelines
            outputs = outputs[:, inputs["input_ids"].shape[1] :]
            outputs = self.tokenizer.batch_decode(outputs, skip_special_tokens=True)
            return outputs
        else:
            return self.pipe(inputs, **kwargs)


def get_model_from(task):
    task = orjson.loads(task)
    if "model" in task:
        return task["model"]

    if "task" in task:
        model = transformers.pipelines.SUPPORTED_TASKS[task["task"]]["default"]["model"]
        ty = "tf" if "tf" in model else "pt"
        return model[ty][0]


def create_pipeline(task):
    if isinstance(task, str):
        task = orjson.loads(task)
    ensure_device(task)
    convert_dtype(task)
    model_name = task.get("model", None)
    model_type = None
    if "model_type" in task:
        model_type = task["model_type"]
    if model_name:
        lower = model_name.lower()
    else:
        lower = None
    if lower and ("-ggml" in lower or "-gguf" in lower):
        pipe = GGMLPipeline(model_name, **task)
    else:
        try:
            pipe = StandardPipeline(model_name, **task)
        except TypeError as error:
            if "device" in task:
                # some models fail when given "device" kwargs, remove and try again
                task.pop("device")
                pipe = StandardPipeline(model_name, **task)
            else:
                raise error
    return pipe


def transform_using(pipeline, args, inputs, stream=False, timeout=None):
    args = orjson.loads(args)
    inputs = orjson.loads(inputs)

    if pipeline.task == "question-answering":
        inputs = [orjson.loads(input) for input in inputs]
    convert_eos_token(pipeline.tokenizer, args)

    if stream:
        return pipeline.stream(inputs, timeout=timeout, **args)
    return orjson.dumps(pipeline(inputs, **args), default=orjson_default).decode()


def transform(task, args, inputs, stream=False):
    task = orjson.loads(task)
    args = orjson.loads(args)
    inputs = orjson.loads(inputs)

    key = ",".join([f"{key}:{val}" for (key, val) in sorted(task.items())])
    if key not in __cache_transform_pipeline_by_task:
        pipe = create_pipeline(task)
        __cache_transform_pipeline_by_task[key] = pipe

    pipe = __cache_transform_pipeline_by_task[key]

    if pipe.task == "question-answering":
        inputs = [orjson.loads(input) for input in inputs]
    convert_eos_token(pipe.tokenizer, args)

    if stream:
        return pipe.stream(inputs, **args)
    return orjson.dumps(pipe(inputs, **args), default=orjson_default).decode()


def create_cross_encoder(transformer):
    return CrossEncoder(transformer)


def rank_using(model, query, documents, kwargs):
    if isinstance(kwargs, str):
        kwargs = orjson.loads(kwargs)

    # The score is a numpy float32 before we convert it
    return [
        {"score": x.pop("score").item(), **x}
        for x in model.rank(query, documents, **kwargs)
    ]


def rank(transformer, query, documents, kwargs):
    kwargs = orjson.loads(kwargs)

    if transformer not in __cache_sentence_transformer_by_name:
        __cache_sentence_transformer_by_name[transformer] = create_cross_encoder(
            transformer
        )
    model = __cache_sentence_transformer_by_name[transformer]

    return rank_using(model, query, documents, kwargs)


def create_embedding(transformer):
    return SentenceTransformer(transformer)


def embed_using(model, transformer, inputs, kwargs):
    if isinstance(kwargs, str):
        kwargs = orjson.loads(kwargs)

    instructor = transformer.startswith("hkunlp/instructor")
    if instructor and "instruction" in kwargs:
        instruction = kwargs.pop("instruction")
        kwargs["prompt"] = instruction

    return model.encode(inputs, **kwargs)


def embed(transformer, inputs, kwargs):
    kwargs = orjson.loads(kwargs)

    ensure_device(kwargs)

    if transformer not in __cache_sentence_transformer_by_name:
        __cache_sentence_transformer_by_name[transformer] = create_embedding(
            transformer
        )
    model = __cache_sentence_transformer_by_name[transformer]

    return embed_using(model, transformer, inputs, kwargs)


def clear_gpu_cache(memory_usage: None):
    if not torch.cuda.is_available():
        raise PgMLException(f"No GPU available")
    mem_used = torch.cuda.memory_usage()
    if not memory_usage or mem_used >= int(memory_usage * 100.0):
        torch.cuda.empty_cache()
        return True
    return False


def load_dataset(name, subset, limit: None, kwargs: "{}"):
    kwargs = orjson.loads(kwargs)

    if limit:
        dataset = datasets.load_dataset(
            name, subset, split=f"train[:{limit}]", **kwargs
        )
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

    return orjson.dumps({"data": data, "types": types}).decode()


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
    encoding = tokenizer(
        y, max_length=max_length, truncation=True, padding="max_length"
    )
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
            decoded_preds = tokenizer.batch_decode(
                predictions, skip_special_tokens=True
            )
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
            slice = dataset.select(
                range(i * batch_size, min((i + 1) * batch_size, len(dataset)))
            )
            tokens = self.tokenizer(
                slice[feature], padding=True, truncation=True, return_tensors="pt"
            )
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
    if (
        y_prob.shape[1] == 2
    ):  # binary classification requires only the greater label by passed to roc_auc_score
        roc_auc_y_prob = y_prob[:, 1]
    metrics["roc_auc"] = roc_auc_score(
        y_test, roc_auc_y_prob, average="weighted", multi_class="ovo"
    )

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
            decoded_preds = tokenizer.batch_decode(
                predictions, skip_special_tokens=True
            )
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
            slice = dataset.select(
                range(i * batch_size, min((i + 1) * batch_size, len(dataset)))
            )
            tokens = self.algorithm["tokenizer"].encode_plus(
                slice["question"], slice["context"], return_tensors="pt"
            )
            tokens.to(self.algorithm["model"].device)
            outputs = self.algorithm["model"](**tokens)
            answer_start = torch.argmax(outputs[0])
            answer_end = torch.argmax(outputs[1]) + 1
            answer = self.algorithm["tokenizer"].convert_tokens_to_string(
                self.algorithm["tokenizer"].convert_ids_to_tokens(
                    tokens["input_ids"][0][answer_start:answer_end]
                )
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

    return {"perplexity": perplexity}


def tune(task, hyperparams, path, x_train, x_test, y_train, y_test):
    hyperparams = orjson.loads(hyperparams)
    model_name = hyperparams.pop("model_name")
    tokenizer = AutoTokenizer.from_pretrained(model_name)

    algorithm = {}

    if task == "text-classification":
        model = AutoModelForSequenceClassification.from_pretrained(
            model_name, num_labels=2
        )
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
        data_collator = DataCollatorForSeq2Seq(
            tokenizer, model=model, return_tensors="pt"
        )
    elif task == "text-generation":
        max_length = hyperparams.pop("max_length", None)
        tokenizer.pad_token = tokenizer.eos_token
        model = AutoModelForCausalLM.from_pretrained(model_name)
        model.resize_token_embeddings(len(tokenizer))
        train = tokenize_text_generation(tokenizer, max_length, y_train)
        test = tokenize_text_generation(tokenizer, max_length, y_test)
        data_collator = DataCollatorForLanguageModeling(
            tokenizer, mlm=False, return_tensors="pt"
        )
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
        metrics = compute_metrics_summarization(
            model, tokenizer, hyperparams, x_test, y_test
        )
    elif task == "text-classification":
        metrics = compute_metrics_text_classification(
            model, tokenizer, hyperparams, x_test, y_test
        )
    elif task == "question-answering":
        metrics = compute_metrics_question_answering(
            model, tokenizer, hyperparams, x_test, y_test
        )
    elif task == "translation":
        metrics = compute_metrics_translation(
            model, tokenizer, hyperparams, x_test, y_test
        )
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
    config = orjson.loads(config)
    all_preds = []

    batch_size = 1  # TODO hyperparams
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
            decoded_preds = tokenizer.batch_decode(
                predictions, skip_special_tokens=True
            )
            all_preds.extend(decoded_preds)
    return all_preds


#######################
# LLM Fine-Tuning
#######################


class PGMLCallback(TrainerCallback):
    "A callback that prints a message at the beginning of training"

    def __init__(self, project_id, model_id):
        self.project_id = project_id
        self.model_id = model_id

    def on_log(self, args, state, control, logs=None, **kwargs):
        if state.is_local_process_zero:
            logs["step"] = state.global_step
            logs["max_steps"] = state.max_steps
            logs["timestamp"] = str(datetime.now())
            r_log("info", json.dumps(logs, indent=4))
            r_insert_logs(self.project_id, self.model_id, json.dumps(logs))


class FineTuningBase:
    def __init__(
        self,
        project_id: int,
        model_id: int,
        train_dataset: datasets.Dataset,
        test_dataset: datasets.Dataset,
        path: str,
        hyperparameters: dict,
    ) -> None:
        # initialize class variables
        self.project_id = project_id
        self.model_id = model_id
        self.train_dataset = train_dataset
        self.test_dataset = test_dataset
        self.token = None
        self.load_in_8bit = False
        self.tokenizer_args = None

        # check if path is a directory
        if not os.path.isdir(path):
            os.makedirs(path, exist_ok=True)

        self.path = path

        # check if hyperparameters is a dictionary
        if "model_name" not in hyperparameters:
            raise ValueError("model_name is a required hyperparameter")
        else:
            self.model_name = hyperparameters.pop("model_name")

        if "token" in hyperparameters:
            self.token = hyperparameters.pop("token")

        if "training_args" in hyperparameters:
            self.training_args = hyperparameters.pop("training_args")
        else:
            self.training_args = None

        if "project_name" in hyperparameters:
            project_name = "_".join(hyperparameters.pop("project_name").split())
            self.training_args["hub_model_id"] = project_name

        if "load_in_8bit" in hyperparameters:
            self.load_in_8bit = hyperparameters.pop("load_in_8bit")

        if "tokenizer_args" in hyperparameters:
            self.tokenizer_args = hyperparameters.pop("tokenizer_args")

        self.tokenizer = AutoTokenizer.from_pretrained(
            self.model_name, token=self.token
        )

    def print_number_of_trainable_model_parameters(self, model):
        """Prints the number of trainable parameters in the model.

        This function traverses all the parameters of a given PyTorch model to
        count the total number of parameters as well as the number of trainable
        (i.e., requires gradient) parameters.

        Args:
            model: A PyTorch model whose parameters you want to count.
        """

        # Initialize counters for trainable and total parameters
        trainable_model_params = 0
        all_model_params = 0

        # Loop through all named parameters in the model
        for _, param in model.named_parameters():
            # Update the total number of parameters
            all_model_params += param.numel()

            # Check if the parameter requires gradient and update the trainable parameter counter
            if param.requires_grad:
                trainable_model_params += param.numel()

        # Calculate and print the number and percentage of trainable parameters
        r_log("info", f"Trainable model parameters: {trainable_model_params}")
        r_log("info", f"All model parameters: {all_model_params}")
        r_log(
            "info",
            f"Percentage of trainable model parameters: {100 * trainable_model_params / all_model_params:.2f}%",
        )

    def tokenize_function(self):
        pass

    def prepare_tokenized_datasets(self):
        pass

    def compute_metrics(self):
        pass

    def train(self):
        pass


class FineTuningTextClassification(FineTuningBase):
    def __init__(
        self,
        project_id: int,
        model_id: int,
        train_dataset: datasets.Dataset,
        test_dataset: datasets.Dataset,
        path: str,
        hyperparameters: dict,
    ) -> None:
        """
        Initializes a FineTuning object.

        Args:
            project_id (int): The ID of the project.
            model_id (int): The ID of the model.
            train_dataset (Dataset): The training dataset.
            test_dataset (Dataset): The test dataset.
            path (str): The path to save the model.
            hyperparameters (dict): The hyperparameters for fine-tuning.

        Returns:
            None
        """
        super().__init__(
            project_id, model_id, train_dataset, test_dataset, path, hyperparameters
        )

        self.classes = list(set(self.train_dataset["class"]))
        self.num_labels = len(self.classes)

        # create label2id and id2label dictionaries
        self.label2id = {}
        self.id2label = {}
        for _id, label in enumerate(self.classes):
            self.label2id[label] = _id
            self.id2label[_id] = label

        # add label column to train and test datasets
        def add_label_column(example):
            example["label"] = self.label2id[example["class"]]
            return example

        self.train_dataset = self.train_dataset.map(add_label_column)
        self.test_dataset = self.test_dataset.map(add_label_column)

        # load model
        self.model = AutoModelForSequenceClassification.from_pretrained(
            self.model_name,
            num_labels=self.num_labels,
            id2label=self.id2label,
            label2id=self.label2id,
        )

        self.model.config.id2label = self.id2label
        self.model.config.label2id = self.label2id

    def tokenize_function(self, example):
        """
        Tokenizes the input text using the tokenizer specified in the class.

        Args:
            example (dict): The input example containing the text to be tokenized.

        Returns:
            tokenized_example (dict): The tokenized example.

        """
        if self.tokenizer_args:
            tokenized_example = self.tokenizer(example["text"], **self.tokenizer_args)
        else:
            tokenized_example = self.tokenizer(
                example["text"], padding=True, truncation=True, return_tensors="pt"
            )
        return tokenized_example

    def prepare_tokenized_datasets(self):
        """
        Tokenizes the train and test datasets using the provided tokenize_function.

        Returns:
            None
        """
        self.train_dataset = self.train_dataset.map(
            self.tokenize_function, batched=True
        )
        self.test_dataset = self.test_dataset.map(self.tokenize_function, batched=True)

    def compute_metrics(self, eval_pred):
        """
        Compute the F1 score and accuracy metrics for evaluating model performance.

        Args:
            eval_pred (tuple): A tuple containing the logits and labels.

        Returns:
            dict: A dictionary containing the computed F1 score and accuracy.

        """
        f1_metric = evaluate.load("f1")
        accuracy_metric = evaluate.load("accuracy")

        logits, labels = eval_pred
        probabilities = F.softmax(torch.from_numpy(logits), dim=1)
        predictions = torch.argmax(probabilities, dim=1)

        f1 = f1_metric.compute(
            predictions=predictions, references=labels, average="macro"
        )["f1"]
        accuracy = accuracy_metric.compute(predictions=predictions, references=labels)[
            "accuracy"
        ]

        return {"f1": f1, "accuracy": accuracy}

    def train(self):
        """
        Trains the model using the specified training arguments, datasets, tokenizer, and data collator.
        Saves the trained model after training.
        """
        data_collator = DataCollatorWithPadding(tokenizer=self.tokenizer)

        args = TrainingArguments(
            output_dir=self.path, logging_dir=self.path, **self.training_args
        )

        self.trainer = Trainer(
            model=self.model,
            args=args,
            train_dataset=self.train_dataset,
            eval_dataset=self.test_dataset,
            tokenizer=self.tokenizer,
            data_collator=data_collator,
            compute_metrics=self.compute_metrics,
            callbacks=[PGMLCallback(self.project_id, self.model_id)],
        )

        self.trainer.train()

        self.trainer.save_model()

    def evaluate(self):
        """
        Evaluate the performance of the model on the evaluation dataset.

        Returns:
            metrics (dict): A dictionary containing the evaluation metrics.
        """
        metrics = self.trainer.evaluate()

        # Update the keys to match hardcoded metrics in Task definition
        if "eval_f1" in metrics.keys():
            metrics["f1"] = metrics.pop("eval_f1")

        if "eval_accuracy" in metrics.keys():
            metrics["accuracy"] = metrics.pop("eval_accuracy")

        # Drop all the keys that are not floats or ints to be compatible for pgml-extension metrics typechecks
        metrics = {
            key: value
            for key, value in metrics.items()
            if isinstance(value, (int, float))
        }

        return metrics


class FineTuningTextPairClassification(FineTuningTextClassification):
    def __init__(
        self,
        project_id: int,
        model_id: int,
        train_dataset: datasets.Dataset,
        test_dataset: datasets.Dataset,
        path: str,
        hyperparameters: dict,
    ) -> None:
        """
        Initializes a FineTuning object.

        Args:
            project_id (int): The ID of the project.
            model_id (int): The ID of the model.
            train_dataset (Dataset): The training dataset.
            test_dataset (Dataset): The test dataset.
            path (str): The path to save the model.
            hyperparameters (dict): The hyperparameters for fine-tuning.

        Returns:
            None
        """
        super().__init__(
            project_id, model_id, train_dataset, test_dataset, path, hyperparameters
        )

    def tokenize_function(self, example):
        """
        Tokenizes the input text using the tokenizer specified in the class.

        Args:
            example (dict): The input example containing the text to be tokenized.

        Returns:
            tokenized_example (dict): The tokenized example.

        """
        if self.tokenizer_args:
            tokenized_example = self.tokenizer(
                example["text1"], example["text2"], **self.tokenizer_args
            )
        else:
            tokenized_example = self.tokenizer(
                example["text1"],
                example["text2"],
                padding=True,
                truncation=True,
                return_tensors="pt",
            )
        return tokenized_example


class FineTuningConversation(FineTuningBase):
    def __init__(
        self,
        project_id: int,
        model_id: int,
        train_dataset: datasets.Dataset,
        test_dataset: datasets.Dataset,
        path: str,
        hyperparameters: dict,
    ) -> None:
        """
        Initializes a FineTuning object.

        Args:
            project_id (int): The ID of the project.
            model_id (int): The ID of the model.
            train_dataset (Dataset): The training dataset.
            test_dataset (Dataset): The test dataset.
            path (str): The path to save the model.
            hyperparameters (dict): The hyperparameters for fine-tuning.

        Returns:
            None
        """
        super().__init__(
            project_id, model_id, train_dataset, test_dataset, path, hyperparameters
        )

        # max sequence length
        self.max_seq_length = None

        # lora config parameters
        self.lora_config_params = None

        if "max_seq_length" in hyperparameters.keys():
            self.max_seq_length = hyperparameters.pop("max_seq_length")
        elif hasattr(self.tokenizer, "model_max_length"):
            self.max_seq_length = self.tokenizer.model_max_length
        else:
            self.max_seq_length = 1024

        if self.max_seq_length > 1e6:
            self.max_seq_length = 1024

        # train and test dataset
        self.train_dataset = train_dataset
        self.test_dataset = test_dataset

        if "lora_config" in hyperparameters:
            self.lora_config_params = hyperparameters.pop("lora_config")
        else:
            self.lora_config_params = {
                "r": 2,
                "lora_alpha": 4,
                "lora_dropout": 0.05,
                "bias": "none",
                "task_type": "CAUSAL_LM",
            }
            r_log(
                "info",
                "LoRA configuration are not set. Using default parameters"
                + json.dumps(self.lora_config_params),
            )

        self.prompt_template = None
        if "prompt_template" in hyperparameters.keys():
            self.prompt_template = hyperparameters.pop("prompt_template")

    def train(self):
        args = TrainingArguments(
            output_dir=self.path, logging_dir=self.path, **self.training_args
        )

        def formatting_prompts_func(example):
            system_content = example["system"]
            user_content = example["user"]
            assistant_content = example["assistant"]

            if self.prompt_template:
                text = self.prompt_template.format(
                    system=system_content,
                    user=user_content,
                    assistant=assistant_content,
                    eos_token=self.tokenizer.eos_token,
                )
            elif hasattr(self.tokenizer, "apply_chat_template"):
                messages = [
                    {"role": "system", "content": system_content},
                    {"role": "user", "content": user_content},
                    {"role": "assistant", "content": assistant_content},
                ]
                text = self.tokenizer.apply_chat_template(messages, tokenize=False)
            else:
                raise ValueError(
                    "Tokenizer doesn't have a chat template. Please pass a template in hyperparameters"
                )

            return text

        if self.load_in_8bit:
            model = AutoModelForCausalLM.from_pretrained(
                self.model_name,
                load_in_8bit=True,
                token=self.token,
            )
        else:
            model = AutoModelForCausalLM.from_pretrained(
                self.model_name,
                torch_dtype=torch.bfloat16,
                token=self.token,
            )

        # SFT Trainer
        self.trainer = SFTTrainer(
            model,
            args=args,
            train_dataset=self.train_dataset,
            eval_dataset=self.test_dataset,
            formatting_func=formatting_prompts_func,
            max_seq_length=self.max_seq_length,
            packing=True,
            peft_config=LoraConfig(**self.lora_config_params),
            callbacks=[PGMLCallback(self.project_id, self.model_id)],
        )
        r_log("info", "Creating Supervised Fine Tuning trainer done. Training ... ")

        # Train
        self.trainer.train()

        # Save the model
        self.trainer.save_model()

    def evaluate(self):
        metrics = self.trainer.evaluate()
        # Drop all the keys that are not floats or ints to be compatible for pgml-extension metrics typechecks
        metrics = {
            key: value
            for key, value in metrics.items()
            if isinstance(value, (int, float))
        }
        return metrics


def finetune_text_classification(
    task, hyperparams, path, x_train, x_test, y_train, y_test, project_id, model_id
):
    hyperparams = orjson.loads(hyperparams)
    # Prepare dataset
    train_dataset = datasets.Dataset.from_dict(
        {
            "text": x_train,
            "class": y_train,
        }
    )
    test_dataset = datasets.Dataset.from_dict(
        {
            "text": x_test,
            "class": y_test,
        }
    )

    finetuner = FineTuningTextClassification(
        project_id=project_id,
        model_id=model_id,
        train_dataset=train_dataset,
        test_dataset=test_dataset,
        path=path,
        hyperparameters=hyperparams,
    )

    finetuner.prepare_tokenized_datasets()

    finetuner.train()

    metrics = finetuner.evaluate()

    return metrics


def finetune_text_pair_classification(
    task,
    hyperparams,
    path,
    text1_train,
    text1_test,
    text2_train,
    text2_test,
    class_train,
    class_test,
    project_id,
    model_id,
):
    # Get model and tokenizer
    hyperparams = orjson.loads(hyperparams)

    # Prepare dataset
    train_dataset = datasets.Dataset.from_dict(
        {
            "text1": text1_train,
            "text2": text2_train,
            "class": class_train,
        }
    )
    test_dataset = datasets.Dataset.from_dict(
        {
            "text1": text1_test,
            "text2": text2_test,
            "class": class_test,
        }
    )

    finetuner = FineTuningTextPairClassification(
        project_id=project_id,
        model_id=model_id,
        train_dataset=train_dataset,
        test_dataset=test_dataset,
        path=path,
        hyperparameters=hyperparams,
    )

    finetuner.prepare_tokenized_datasets()

    finetuner.train()

    metrics = finetuner.evaluate()

    return metrics


## Conversation
def finetune_conversation(
    task,
    hyperparams,
    path,
    system_train,
    user_test,
    assistant_train,
    system_test,
    user_train,
    assistant_test,
    project_id,
    model_id,
):
    train_dataset = datasets.Dataset.from_dict(
        {
            "system": system_train,
            "user": user_train,
            "assistant": assistant_train,
        }
    )

    test_dataset = datasets.Dataset.from_dict(
        {
            "system": system_test,
            "user": user_test,
            "assistant": assistant_test,
        }
    )
    hyperparams = orjson.loads(hyperparams)

    finetuner = FineTuningConversation(
        project_id, model_id, train_dataset, test_dataset, path, hyperparams
    )

    finetuner.train()

    metrics = finetuner.evaluate()

    return metrics
