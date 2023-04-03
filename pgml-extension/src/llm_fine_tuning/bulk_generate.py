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

#logging.disable(logging.INFO)
warnings.filterwarnings('ignore')
#set_global_logging_level(logging.ERROR, ["transformers", "nlp", "torch", "tensorflow", "tensorboard", "wandb"])

FORMAT = "%(message)s"
logging.basicConfig(
    level="NOTSET", format=FORMAT, datefmt="[%X]", handlers=[RichHandler()]
)
log = logging.getLogger("rich")


def generate(params):
    prompt = params["prompt"]
    model_name = params["model"]
    tokenizer_name = params["model"]
    min_length = params["min_length"]
    max_length = params["max_length"]
    num_return_sequences = params["num_return_sequences"]
    temperature = params["temperature"]
    repetition_penalty = params["repetition_penalty"]
    top_k = params["top_k"]
    top_p = params["top_p"]

    if os.path.exists(prompt):
        log.info("Reading prompt from file")
        prompt = Path(prompt).read_text()
        params["prompt"] = prompt

    cuda_available = torch.cuda.is_available()
    model = AutoModelForCausalLM.from_pretrained(model_name,torch_dtype=torch.float16)
    tokenizer = AutoTokenizer.from_pretrained(tokenizer_name)

    if cuda_available:
        device = "cuda:0"
    else:
        device = "cpu"

    output_data = []
    data = params
    try:
        log.info("Generating text..")
        generator = pipeline(
            "text-generation",
            model=model,
            tokenizer=tokenizer,
            device=device,
            max_length=max_length,
        )

        min_length = min(min_length, max_length)

        outputs = generator(
            prompt,
            do_sample=True,
            min_length=min_length,
            num_return_sequences=num_return_sequences,
            temperature=temperature,
            repetition_penalty=repetition_penalty,
            top_k=top_k,
            top_p=top_p,
        )

        counter = 0
        for output in outputs:
            key = "output_%d" % counter
            if isinstance(output, dict):
                data[key] = output["generated_text"]
                counter += 1
            if isinstance(output, list):
                for tt in output:
                    data[key] = tt["generated_text"]
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
    parameter_grid = list(ParameterGrid(config))
    log.info("Total runs = %d" % len(parameter_grid))

    engine = create_engine(connection, echo=False)

    counter = 0
    for param in track(parameter_grid):
        df = generate(param)
        if not dry_run:
            df.to_sql(table_name, engine, if_exists="append")
        counter+=1

if __name__ == "__main__":
    bulk_generate()
