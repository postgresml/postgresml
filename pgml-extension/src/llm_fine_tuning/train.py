# Reference: https://huggingface.co/course/chapter7/6
import click
import pandas as pd
import torch
from torch.utils.data import random_split

from datasets import load_dataset
from transformers import (
    AutoTokenizer,
    TrainingArguments,
    Trainer,
    AutoModelForCausalLM,
    DataCollatorForLanguageModeling,
)


import os
import subprocess
import shlex
from time import sleep
from utils import parse_gpu_utilization_file

# Logging
import logging
from rich.logging import RichHandler

FORMAT = "%(message)s"
logging.basicConfig(
    level="NOTSET", format=FORMAT, datefmt="[%X]", handlers=[RichHandler()]
)
log = logging.getLogger("rich")

# Torch
torch.manual_seed(42)


@click.command()
@click.argument("filename")
@click.argument("column_name")
@click.option(
    "--model_name",
    default="distilgpt2",
    help="Huggingface model name or path to model",
    show_default=True,
)
@click.option(
    "--tokenizer_name",
    default="distilgpt2",
    help="Hugging face tokenizer or path to tokenizer",
    show_default=True,
)
@click.option(
    "--batch_size", default=1, help="Batch size for training", show_default=True
)
@click.option(
    "--epochs", default=1, help="Number of epochs for training", show_default=True
)
@click.option(
    "--output_dir",
    default="./results",
    help="Output directory for model and tokenizers",
    show_default=True,
)
@click.option(
    "--get_gpu_utilization",
    default=False,
    help="Get GPU utiliziation during training",
    show_default=True,
)
@click.option(
    "--gpu_utilization_file",
    default="/tmp/gpu-stats.csv",
    help="File to write GPU stats if get_gpu_status flag is True",
    show_default=True,
)
@click.option(
    "--disable_save",
    default=False,
    help="Disable saving model and tokenizer after training - useful for benchmarking runtimes",
    show_default=True,
)
@click.option("--local_rank")
#@click.option("--deepspeed")
@click.option(
    "--fp16",
    default=False,
    help="Use FP16 instead of FP32 for model training",
    show_default=True,
)
def train(
    filename,
    column_name,
    model_name,
    tokenizer_name,
    batch_size,
    epochs,
    output_dir,
    get_gpu_utilization,
    gpu_utilization_file,
    disable_save,
    local_rank,
#    deepspeed,
    fp16,
):
    cuda_available = torch.cuda.is_available()

    if cuda_available:
        local_rank = int(os.environ["LOCAL_RANK"])
        torch.cuda.set_device(local_rank)

    nvidia_smi_query_started = False
    if cuda_available and get_gpu_utilization:
        gpu_utilization_file = gpu_utilization_file.strip()
        gpu_stats_command = (
            "nvidia-smi --query-gpu=gpu_name,gpu_bus_id,vbios_version,utilization.gpu,utilization.memory,memory.total,memory.free,memory.used\
                            --format=csv -l 5 -f "
            + gpu_utilization_file
        )
        subprocess_command = shlex.split(gpu_stats_command)
        process = subprocess.Popen(subprocess_command)
        log.info("Started gpu query process %d" % process.pid)
        nvidia_smi_query_started = True

    if not os.path.exists(output_dir):
        msg = "Folder %s doesn't exist .. creating one" % output_dir
        log.debug(msg)
        os.mkdir(output_dir)

    if os.path.exists(filename):
        log.info("Reading input CSV file: %s" % filename)
        dataset = load_dataset("csv", data_files=filename)
    else:
        msg = "File %s doesn't exist" % filename
        log.error(msg)
        raise ValueError(msg)

    log.info("Getting tokenizer from %s" % tokenizer_name)
    tokenizer = AutoTokenizer.from_pretrained(tokenizer_name)
    tokenizer.pad_token = tokenizer.eos_token

    if cuda_available:
        log.info("Creating AutoModelForCausalLM for %s on GPU memory" % model_name)
        model = AutoModelForCausalLM.from_pretrained(model_name).cuda()
    else:
        log.info("Creating AutoModelForCausalLM for %s on DRAM (CPU)" % model_name)
        model = AutoModelForCausalLM.from_pretrained(model_name)

    model.resize_token_embeddings(len(tokenizer))
    
    log.info(dataset)
    
    def tokenize_function(element):
        context_length = tokenizer.model_max_length
        outputs = tokenizer(
            element[column_name],
            truncation=True,
            max_length=context_length,
            return_overflowing_tokens=True,
            return_length=True,
            padding='max_length',
        )
        input_batch = []
        for length, input_ids in zip(outputs["length"], outputs["input_ids"]):
            if length == context_length:
                input_batch.append(input_ids)
        return {"input_ids": input_batch}

    tokenized_datasets = dataset.map(
        tokenize_function, batched=True, remove_columns=dataset["train"].column_names
    )
    log.info(tokenized_datasets)
    train_dataset, eval_dataset = random_split(tokenized_datasets["train"], [0.99, 0.01])

    training_args = TrainingArguments(
        output_dir=output_dir,
        num_train_epochs=epochs,
        per_device_train_batch_size=batch_size,
        per_device_eval_batch_size=batch_size,
        save_strategy="steps",
#        deepspeed="./ds_config.json",
        fp16=fp16,
    )

    data_collator = DataCollatorForLanguageModeling(tokenizer=tokenizer, mlm=False)

    trainer = Trainer(
        model=model,
        args=training_args,
        train_dataset=train_dataset,
        eval_dataset=eval_dataset,
        data_collator=data_collator,
    )

    log.info("Training initiated")
    trainer.train()

    if not disable_save:
        trainer.save_model(output_dir + "/model")
        tokenizer.save_pretrained(output_dir + "/tokenizer")

    if nvidia_smi_query_started:
        process.terminate()
        log.info("Stopped gpu nvidia-smi query process %d" % process.pid)
        sleep(
            1
        )  # Very important to wait for the process to termiante. Otherwise file won't be available for pandas

        if os.path.exists(gpu_utilization_file):
            log.info(
                "Displaying GPU utilization details from file %s", gpu_utilization_file
            )
            gpu_df = parse_gpu_utilization_file("/tmp/gpu-stats.csv")
            log.info("#####GPU Utilization####")
            log.info(gpu_df)
            log.info("#####GPU Utilization Summary####")
            log.info(gpu_df.describe())


if __name__ == "__main__":
    train()
