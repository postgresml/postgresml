import pandas as pd


def parse_gpu_utilization_file(filename):
    gpu_df = pd.read_csv(filename)
    columns = [col.strip() for col in list(gpu_df.columns)]
    gpu_df.columns = columns
    memory_columns = [
        "memory.total [MiB]",
        "memory.free [MiB]",
        "memory.used [MiB]",
    ]
    for column in memory_columns:
        gpu_df[column] = gpu_df[column].map(lambda entry: int(entry.rstrip("MiB")))
    utilization_columns = ["utilization.gpu [%]", "utilization.memory [%]"]
    for column in utilization_columns:
        gpu_df[column] = gpu_df[column].map(lambda entry: int(entry.rstrip("%")))
    return gpu_df
