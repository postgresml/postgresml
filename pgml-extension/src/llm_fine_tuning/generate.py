import click
import torch
from transformers import AutoTokenizer, AutoModelForCausalLM, pipeline
import logging
from rich.logging import RichHandler

FORMAT = "%(message)s"
logging.basicConfig(
    level="NOTSET", format=FORMAT, datefmt="[%X]", handlers=[RichHandler()]
)
log = logging.getLogger("rich")

# Torch
# torch.manual_seed(42)


@click.command()
@click.argument("prompt")
@click.option(
    "--model_name",
    default="distilgpt2",
    help="Hugging face model or path to the model",
    show_default=True,
)
@click.option(
    "--tokenizer_name",
    default="distilgpt2",
    help="Hugging face tokenizer or path to the tokenizer",
    show_default=True,
)
@click.option(
    "--max_length", default=50, help="Max length of the response", show_default=True
)
@click.option(
    "--num_return_sequences",
    default=1,
    help="Number of return sequences",
    show_default=True,
)
@click.option(
    "--temperature",
    default=1.0,
    help="Temperature for the models",
    show_default=True,
)
def generate(
    prompt, model_name, tokenizer_name, max_length, num_return_sequences, temperature
):
    model = AutoModelForCausalLM.from_pretrained(model_name)
    tokenizer = AutoTokenizer.from_pretrained(tokenizer_name)
    generator = pipeline(
        "text-generation", model=model, tokenizer=tokenizer, max_length=max_length
    )
    log.info("Prompt: %s" % prompt)
    log.info(
        "Generated: %s"
        % generator(
            prompt, num_return_sequences=num_return_sequences, temperature=temperature
        )
    )


if __name__ == "__main__":
    generate()
