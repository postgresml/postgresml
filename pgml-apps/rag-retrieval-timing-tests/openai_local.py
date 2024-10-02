from openai import OpenAI
import time

# Create our OpenAI client
client = OpenAI()


# Get LLM response from OpenAI
def get_llm_response(query, context):
    print("\tGetting LLM response from OpenAI")
    tic = time.perf_counter()
    completion = client.chat.completions.create(
        model="gpt-3.5-turbo",
        messages=[
            {
                "role": "system",
                "content": f"You are a helpful assistant. Given the context, provide an answer to the user: \n{context}",
            },
            {"role": "user", "content": query},
        ],
    )
    toc = time.perf_counter()
    time_taken = toc - tic
    print(f"\tDone getting the LLM response: {time_taken:0.4f}")
    response = completion.choices[0].message.content
    return (response, time_taken)
