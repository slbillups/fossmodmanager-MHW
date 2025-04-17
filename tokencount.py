# Filename: count_tokens.py
import sys
import argparse
import tiktoken

# --- Model Info (for context) ---
# GPT-4o / GPT-4 / GPT-3.5-Turbo: Use 'cl100k_base' (Accurate)
# Claude 3.x: Uses own tokenizer ('cl100k_base' is an approximation)
# Gemini 1.5 Pro: Uses own tokenizer ('cl100k_base' is a rough approximation)
# Older OpenAI models (e.g., davinci): Use 'p50k_base' or 'r50k_base'

def count_file_tokens(filename, encoding_name="cl100k_base"):
    """Counts the tokens in a file using tiktoken.

    Args:
        filename (str): The path to the file.
        encoding_name (str): The name of the tiktoken encoding to use.

    Returns:
        int: The number of tokens, or None if an error occurs.
    """
    try:
        # Get the encoding object
        encoding = tiktoken.get_encoding(encoding_name)
        print(f"--- Using tiktoken encoding: {encoding_name} ---")
        if encoding_name == "cl100k_base":
            print("--- (Accurate for GPT-4o/4/3.5, approximation for Claude/Gemini) ---")
        else:
            print(f"--- (Check tiktoken docs for model compatibility: {encoding_name}) ---")

    except ValueError:
        print(f"Error: Encoding '{encoding_name}' not found.")
        print(f"Available tiktoken encodings: {tiktoken.list_encoding_names()}")
        return None

    try:
        # Read the file content
        with open(filename, 'r', encoding='utf-8') as f:
            content = f.read()
    except FileNotFoundError:
        print(f"Error: File not found: '{filename}'")
        return None
    except Exception as e:
        print(f"Error reading file '{filename}': {e}")
        return None

    # Encode the content and count tokens
    try:
        tokens = encoding.encode(content)
        num_tokens = len(tokens)
        return num_tokens
    except Exception as e:
        print(f"Error encoding content: {e}")
        return None

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Estimate token count for a file using tiktoken.")
    parser.add_argument("filename", help="Path to the input file.")
    parser.add_argument("-e", "--encoding", default="cl100k_base",
                        help="Tiktoken encoding name (default: cl100k_base). See script comments for model info.")

    args = parser.parse_args()

    token_count = count_file_tokens(args.filename, args.encoding)

    if token_count is not None:
        print(f"\nEstimated token count for '{args.filename}': {token_count}")
