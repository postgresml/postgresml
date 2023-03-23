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
@click.option(
    "--repetition_penalty",
    default=1.0,
    help="Repetition penalty can be used to penalize words that were already generated or belong to the context.",
    show_default=True,
)
@click.option(
    "--top_k",
    default=50,
    help="Top K sampling",
    show_default=True,
)
@click.option(
    "--top_p",
    default=0.95,
    help="Top P cumulative probability sampling",
    show_default=True,
)
def generate(
    prompt, model_name, tokenizer_name, min_length, max_length, num_return_sequences, temperature, repetition_penalty,top_k,top_p
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
        repetition_penalty = repetition_penalty,
        top_k = top_k,
        top_p = top_p,
    )

    fp = open("output.txt","w")
    for _id, output in enumerate(outputs):
        fp.write("Generated %d: %s\n" % (_id, output["generated_text"]))
    fp.close()


if __name__ == "__main__":
    generate()
