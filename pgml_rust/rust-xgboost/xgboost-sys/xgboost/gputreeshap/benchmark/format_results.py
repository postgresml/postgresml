import pandas as pd
import argparse


def print_format(df, format):
    if "csv" in format:
        print(df.to_csv(index=False))
    if "latex" in format:
        print(df.to_latex(index=False, float_format="%.2f"))
    if "text" in format:
        print(df)
    if "md" in format:
        from pytablewriter import MarkdownTableWriter
        writer = MarkdownTableWriter()
        writer.from_dataframe(df)
        writer.write_table()


parser = argparse.ArgumentParser(description='Format results from GPUTreeShap benchmark')
parser.add_argument('in_results', type=str,
                    help='csv results file')
parser.add_argument('in_models', type=str,
                    help='csv models file')
parser.add_argument("-format", default="csv", type=str,
                    help="Format of output tables. E.g. text,latex,csv,md")

args = parser.parse_args()
results = pd.read_csv(args.in_results)
models = pd.read_csv(args.in_models)
del models["num_rounds"]
del models["average_depth"]
del results["test_rows"]
models = models.rename(columns={"num_trees": "trees", "num_leaves": "leaves"})
results = results.rename(
    columns={"cpu_time(s)": "cpu(s)", "gpu_time(s)": "gpu(s)", "cpu_std": "std", "gpu_std": "std"})
print("Formatted models:")
print_format(models, args.format)
print("Formatted results:")
print_format(results, args.format)
