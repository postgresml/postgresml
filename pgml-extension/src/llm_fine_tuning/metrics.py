from transformers import AutoModelForCausalLM, AutoTokenizer
import torch
from tqdm import tqdm
import os
import click
import logging
from rich.logging import RichHandler
from datasets import load_dataset

FORMAT = "%(message)s"
logging.basicConfig(
    level="NOTSET", format=FORMAT, datefmt="[%X]", handlers=[RichHandler()]
)
log = logging.getLogger("rich")

torch.manual_seed(42)


@click.command()
@click.option(
    "--filename",
    default="netflix_titles_small.csv",
    help="CSV file name",
    show_default=True,
)
@click.option(
    "--column_name", default="description", help="CSV column name", show_default=True
)
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
    "--stride",
    default=512,
    help="Stride length for computing perplexity",
    show_default=True,
)
@click.option(
    "--max_length_key",
    default="n_positions",
    help="Key in model configuration that maps to max length of the embeddings",
    show_default=True,
)
def metrics(filename, column_name, model_name, tokenizer_name, stride, max_length_key):
    if os.path.exists(filename):
        test = load_dataset("csv", data_files=filename)
    else:
        msg = "File %s doesn't exist" % filename
        raise ValueError(msg)

    cuda_available = torch.cuda.is_available()

    device = "cpu"
    if cuda_available:
        device = "cuda:0"
    model = AutoModelForCausalLM.from_pretrained(model_name).to(device)

    tokenizer = AutoTokenizer.from_pretrained(tokenizer_name)

    full_text = ""
    for entry in test["train"][column_name]:
        if entry:
            full_text += "\n\n" + entry

    encodings = tokenizer(full_text, return_tensors="pt")

    config = model.config.to_dict()
    if max_length_key in config.keys():
        max_length = config[max_length_key]
    else:
        log.info("Configuration keys " + ",".join(config.keys()))
        raise ValueError("%s does not exist in model configuration"%max_length_key)

    stride = min(stride, max_length)
    seq_len = encodings.input_ids.size(1)

    nlls = []
    prev_end_loc = 0
    for begin_loc in tqdm(range(0, seq_len, stride)):
        end_loc = min(begin_loc + max_length, seq_len)
        trg_len = end_loc - prev_end_loc  # may be different from stride on last loop
        input_ids = encodings.input_ids[:, begin_loc:end_loc].to(device)
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

    ppl = torch.exp(torch.stack(nlls).sum() / end_loc)
    log.info("Number of parameters = %d, Perplexity = %0.3f (lower is better)" % (model.num_parameters(), ppl))


if __name__ == "__main__":
    metrics()
