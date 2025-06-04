import torch
from transformers import pipeline

# pipe = None
# def init():
#     huh = globals()
#     print(f"{torch.cuda.is_available()=}")
#     huh[pipe] = pipeline(
#         task="text-generation",
#         model="Qwen/Qwen2-1.5B-Instruct",
#         torch_dtype=torch.bfloat16,
#         device=0
#     )

messages = [
    {"role": "system", "content": "You are a helpful assistant."}
]

def reply():
    # Initialize the model here
    pipe = pipeline(
        task="text-generation",
        model="Qwen/Qwen2-1.5B-Instruct",
        torch_dtype=torch.bfloat16,
        device=0
    )
    # Read input from input.txt
    try:
        with open("input.txt", "r", encoding="utf-8") as f:
            user_input = f.read().strip()
    except FileNotFoundError:
        user_input = "No input provided."
    # Add user message
    messages.append({"role": "user", "content": user_input})
    # Generate response
    outputs = pipe(
        messages,
        max_new_tokens=256,
        do_sample=True,
        temperature=0.7,
        top_k=50,
        top_p=0.95,
    )
    # Get assistant's reply
    assistant_reply = outputs[0]["generated_text"][-1]["content"]
    # Write response to output.txt
    with open("output.txt", "w", encoding="utf-8") as f:
        f.write(assistant_reply)
    # Add assistant's message to chat memory
    messages.append({"role": "assistant", "content": assistant_reply})


def main():
    reply()

if __name__ == "__main__":
    main()

