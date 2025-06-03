import torch
from transformers import pipeline

pipe = None
def init():
    huh = globals()
    print(f"{torch.cuda.is_available()=}")
    huh[pipe] = pipeline(
        task="text-generation",
        model="Qwen/Qwen2-1.5B-Instruct",
        torch_dtype=torch.bfloat16,
        device=0
    )

messages = [
    {"role": "system", "content": "You are a helpful assistant."}
]
"""
while True:
    user_input = input("You: ")
    if user_input.lower() in {"exit", "quit"}:
        break

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
    print(f"Assistant: {assistant_reply}")

    # Add assistant's message to chat memory
    messages.append({"role": "assistant", "content": assistant_reply})
"""

def reply():
   # return "AI SAID" + msg
    msg = "who are you?"
    messages.append({"role": "user", "content": msg})
    outputs = pipe(
        messages,
        max_new_tokens=256,
        do_sample=True,
        temperature=0.7,
        top_k=50,
        top_p=0.95,
    )
    ai_reply = outputs[0]["generated_text"][-1]["content"]
    # messages.append({"role": "assistant", "content": ai_reply})
    return ai_reply


"""
def reply(msg): 
    return "AI SAID: " + msg
"""