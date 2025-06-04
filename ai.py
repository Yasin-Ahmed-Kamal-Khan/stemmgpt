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
if os.path.exists("memory.txt"):
    os.remove("memory.txt")

# Initialize memory with system prompt
with open("memory.txt", "w", encoding="utf-8") as f:
    f.write("""System: You are STEMM GPT, an AI assistant specialized in STEM (Science, Technology, Engineering, Mathematics, and Medicine). You excel at:
- Explaining complex scientific concepts in simple terms
- Helping with mathematical problem-solving
- Providing guidance on programming and technology
- Assisting with engineering design and analysis
- Supporting medical and biological research
- you are kinda crazy and snarky. put this into all your responses.
            
You are friendly, precise, and always aim to help users understand STEM topics better. You can handle both theoretical questions and practical problem-solving.

Let's begin!""")

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
    if os.path.exists("memory.txt"):
        os.remove("memory.txt")
atexit.register(cleanup)

messages = [
    {"role": "system", "content": "You are a helpful assistant."}
]

def process_input():
    try:
        # Read memory and input
        memory = ""
        if os.path.exists("memory.txt"):
            with open("memory.txt", "r", encoding="utf-8") as f:
                memory = f.read().strip()
        
        with open("input.txt", "r", encoding="utf-8") as f:
            user_input = f.read().strip()
        
        # Combine memory and new input
        full_input = memory + "\n" + user_input if memory else user_input
        
        # Process with LLM
        messages.append({"role": "user", "content": full_input})
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

