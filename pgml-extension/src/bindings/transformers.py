import transformers
import json

def transform(task, args, inputs):
    task = json.loads(task)
    args = json.loads(args)
    inputs = json.loads(inputs)

    pipe = transformers.pipeline(**task)

    if pipe.task == "question-answering":
        inputs = [json.loads(input) for input in inputs]

    return json.dumps(pipe(inputs, **args))
