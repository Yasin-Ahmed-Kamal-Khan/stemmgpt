import torch
from transformers import pipeline
import time
import os
import atexit

# Clean up any existing files
if os.path.exists("input.txt"):
    os.remove("input.txt")
if os.path.exists("output.txt"):
    os.remove("output.txt")

# Load model once when server starts
print("Loading model...")
pipe = pipeline(
    task="text-generation",
    model="Qwen/Qwen2-1.5B-Instruct",
    torch_dtype=torch.bfloat16,
    device=0
)
print("Model loaded!")

# Create ready file to signal we're ready
with open("ready.txt", "w") as f:
    f.write("ready")

# Clean up ready file when shutting down
def cleanup():
    if os.path.exists("ready.txt"):
        os.remove("ready.txt")
    if os.path.exists("input.txt"):
        os.remove("input.txt")
    if os.path.exists("output.txt"):
        os.remove("output.txt")
atexit.register(cleanup)

messages = [
    {"role": "system", "content": "You are a helpful assistant."}
]

def process_input():
    try:
        # Read input from file
        with open("input.txt", "r", encoding="utf-8") as f:
            user_input = f.read().strip()
        
        # Process with LLM
        messages.append({"role": "user", "content": user_input})
        outputs = pipe(
            messages,
            max_new_tokens=256,
            do_sample=True,
            temperature=0.7,
            top_k=50,
            top_p=0.95,
        )
        assistant_reply = outputs[0]["generated_text"][-1]["content"]
        messages.append({"role": "assistant", "content": assistant_reply})
        
        # Write response to file
        with open("output.txt", "w", encoding="utf-8") as f:
            f.write(assistant_reply)
            
    except Exception as e:
        print(f"Error processing input: {e}")
        with open("output.txt", "w", encoding="utf-8") as f:
            f.write(f"Error: {str(e)}")

def main():
    print("Waiting for input...")
    while True:
        if os.path.exists("input.txt"):
            process_input()
            # Remove input file to signal we're done
            os.remove("input.txt")
        time.sleep(0.1)  # Small delay to prevent CPU spinning

if __name__ == "__main__":
    main()

