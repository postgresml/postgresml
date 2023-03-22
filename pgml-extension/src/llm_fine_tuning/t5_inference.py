import torch
from transformers import T5Tokenizer, T5ForConditionalGeneration

tokenizer = T5Tokenizer.from_pretrained("google/flan-t5-xl")
model = T5ForConditionalGeneration.from_pretrained("google/flan-t5-xl", device_map="auto", torch_dtype=torch.float16)

input_text = "Continue the STORY so that it becomes action-packed: As the sweat dripped from his fevered brow to the keyboard"
input_ids = tokenizer(input_text, return_tensors="pt").input_ids.to("cuda")

outputs = model.generate(input_ids,min_length=200,max_length=250,repetition_penalty=2.0,temperature=1.2,num_return_sequences=1,top_k=50,top_p=0.95)
print(tokenizer.decode(outputs[0]))
