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
    "--min_length", default=50, help="Min length of the response", show_default=True
)
@click.option(
    "--max_length", default=50, help="Min length of the response", show_default=True
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
    prompt, model_name, tokenizer_name, min_length, max_length, num_return_sequences, temperature
):
    cuda_available = torch.cuda.is_available()
    model = AutoModelForCausalLM.from_pretrained(model_name)
    tokenizer = AutoTokenizer.from_pretrained(tokenizer_name)
    if cuda_available:
        device = "cuda:0"
    else:
        device = "cpu"
    generator = pipeline(
        "text-generation", model=model, tokenizer=tokenizer, device=device, max_length = max_length
    )
    
    min_length = min(min_length,max_length)

    log.info("Prompt: %s" % prompt)
    outputs = generator(
        prompt,
        do_sample=True,
        min_length=min_length,
        num_return_sequences=num_return_sequences,
        temperature=temperature,
    )

    for _id, output in enumerate(outputs):
        log.info("Generated %d: %s" % (_id, output["generated_text"]))


if __name__ == "__main__":
    generate()
