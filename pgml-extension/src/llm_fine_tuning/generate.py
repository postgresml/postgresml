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
torch.manual_seed(42)


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
)
@click.option(
    "--max_length",
    default=50,
    help="Max length of the response",
)
def generate(prompt, model_name, tokenizer_name, max_length):
    model = AutoModelForCausalLM.from_pretrained(model_name)
    tokenizer = AutoTokenizer.from_pretrained(tokenizer_name)
    generator = pipeline(
        "text-generation", model=model, tokenizer=tokenizer, max_length=max_length
    )
    log.info("Prompt: %s" % prompt)
    log.info("Generated: %s" % generator(prompt))


if __name__ == "__main__":
    generate()
