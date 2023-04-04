"""
Inputs:
- Model
- Min length
- Max length
- Temperature
- top_k
- top_p
- repetition penalty
- prompt

Output:
- CSV
- Postgres

Optional:
- Existing output data
"""

import yaml
import click
import torch
from transformers import AutoTokenizer, AutoModelForCausalLM, pipeline
import logging
from rich.logging import RichHandler
from rich.progress import track
from sklearn.model_selection import ParameterGrid
import pandas as pd
from pathlib import Path
from sqlalchemy import create_engine
import os
import warnings
import json

#logging.disable(logging.INFO)
warnings.filterwarnings('ignore')
#set_global_logging_level(logging.ERROR, ["transformers", "nlp", "torch", "tensorflow", "tensorboard", "wandb"])

FORMAT = "%(message)s"
logging.basicConfig(
    level="NOTSET", format=FORMAT, datefmt="[%X]", handlers=[RichHandler()]
)
log = logging.getLogger("rich")


def generate(model, params):
    log.info("Model in generate %s"%model)
    cuda_available = torch.cuda.is_available()
    if os.path.exists(model):
        model_name = model + "/model/"
        tokenizer_name = model + "/tokenizer/"
    else:
        model_name = model
        tokenizer_name = model
    

    tokenizer = AutoTokenizer.from_pretrained(tokenizer_name)

    if cuda_available:
        device = "cuda:0"
        model = AutoModelForCausalLM.from_pretrained(model_name,torch_dtype=torch.float16)
    else:
        device = "cpu"
        model = AutoModelForCausalLM.from_pretrained(model_name)

    generator = pipeline(
        "text-generation",
        model=model,
        tokenizer=tokenizer,
        device=device,
    )

    output_data = []
    for param in track(params, description= model_name):
        prompt = param["prompt"]

        if os.path.exists(prompt):
            log.info("Reading prompt from file")
            prompt = Path(prompt).read_text()
            param["prompt"] = prompt

        min_length = param["min_length"]
        max_length = param["max_length"]
        num_return_sequences = param["num_return_sequences"]
        temperature = param["temperature"]
        repetition_penalty = param["repetition_penalty"]
        top_k = param["top_k"]
        top_p = param["top_p"]

        data = param
        data["model"] = model_name
        try:
            log.info("Generating text..")

            min_length = min(min_length, max_length)

            outputs = generator(
                prompt,
                do_sample=True,
                min_length=min_length,
                max_length=max_length,
                num_return_sequences=num_return_sequences,
                temperature=temperature,
                repetition_penalty=repetition_penalty,
                top_k=top_k,
                top_p=top_p,
            )

            counter = 0
            for output in outputs:
                if isinstance(output, dict):
                    data["sequence_id"] = counter
                    data["generated_text"] = output["generated_text"]
                    counter += 1
                if isinstance(output, list):
                    for tt in output:
                        data["sequence_id"] = counter
                        data["generated_text"] = tt["generated_text"]
                        counter += 1
            data["error"] = "N/A"
        except Exception as e:
            data["error"] = str(e)

        output_data.append(data)
    return pd.DataFrame.from_dict(output_data)


@click.command()
@click.option(
    "--config_file",
    type=click.Path(),
    required=True,
    show_default=True,
    help="Configuration YAML file",
)
@click.option(
    "--dry_run",
    is_flag=True,
    default=False,
    help="Enabling this will not store results",
    show_default=True,
)
def bulk_generate(config_file, dry_run):
    if os.path.exists(config_file):
        try:
            with open(config_file, "r") as fp:
                config = yaml.safe_load(fp)
        except Exception as e:
            log.error(e)
            raise ValueError(e)

    else:
        msg = "File %s doesn't exist or not accessible" % config_file
        raise ValueError(msg)

    connection = config["postgres"]["connection"]
    table_name = config["postgres"]["table_name"]
    config.pop("postgres")

    models = config.pop("model")
    parameter_grid = list(ParameterGrid(config))
    log.info("Total runs = %d" % (len(parameter_grid)*len(models)))

    counter = 0
    for model in models:
        log.info(model)
        df = generate(model,parameter_grid)
        if not dry_run:
            engine = create_engine(connection, echo=False)
            df.to_sql(table_name, engine, if_exists="append", index=False)
        counter+=1

if __name__ == "__main__":
    bulk_generate()
