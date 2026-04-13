#!/usr/bin/env python3
"""
Batch update chat template for all GGUF files in a directory.
Creates new GGUF files with updated tokenizer.chat_template field using the GGUF reader/writer.
"""

import sys
import argparse
import logging
import os
from pathlib import Path
from typing import List

# Necessary to load the local gguf package
if "NO_LOCAL_GGUF" not in os.environ and (Path(__file__).parent.parent / 'gguf-py').exists():
    sys.path.insert(0, str(Path(__file__).parent))

from gguf import GGUFReader, GGUFWriter, GGUFValueType  # noqa: E402
import gguf  # noqa: E402

# =============================================================================
# CHAT TEMPLATE PLACEHOLDER - EDIT THIS VALUE
# =============================================================================
CHAT_TEMPLATE = r'''{%- if tools %} {{- '<|im_start|>system\n' }} {%- if messages[0].role == 'system' %} {{- messages[0].content + '\n\n' }} {%- endif %} {{- "# Tools\n\nYou may call one or more functions to assist with the user query.\n\nYou are provided with function signatures within <tools></tools> XML tags:\n<tools>" }} {%- for tool in tools %} {{- "\n" }} {{- tool | tojson }} {%- endfor %} {{- "\n</tools>\n\nFor each function call, return a json object with function name and arguments within <tool_call></tool_call> XML tags:\n<tool_call>\n{\"name\": <function-name>, \"arguments\": <args-json-object>}\n</tool_call><|im_end|>\n" }} {%- else %} {%- if messages[0].role == 'system' %} {{- '<|im_start|>system\n' + messages[0].content + '<|im_end|>\n' }} {%- endif %} {%- endif %} {%- set ns = namespace(multi_step_tool=true, last_query_index=messages|length - 1) %} {%- for forward_message in messages %} {%- set index = (messages|length - 1) - loop.index0 %} {%- set message = messages[index] %} {%- set current_content = message.content if message.content is defined and message.content is not none else '' %} {%- set tool_start = '<tool_response>' %} {%- set tool_start_length = tool_start|length %} {%- set start_of_message = current_content[:tool_start_length] %} {%- set tool_end = '</tool_response>' %} {%- set tool_end_length = tool_end|length %} {%- set start_pos = (current_content|length) - tool_end_length %} {%- if start_pos < 0 %} {%- set start_pos = 0 %} {%- endif %} {%- set end_of_message = current_content[start_pos:] %} {%- if ns.multi_step_tool and message.role == "user" and not(start_of_message == tool_start and end_of_message == tool_end) %} {%- set ns.multi_step_tool = false %} {%- set ns.last_query_index = index %} {%- endif %} {%- endfor %} {%- for message in messages %} {%- if (message.role == "user") or (message.role == "system" and not loop.first) %} {{- '<|im_start|>' + message.role + '\n' + message.content + '<|im_end|>' + '\n' }} {%- elif message.role == "assistant" %} {%- set content = message.content if message.content is defined and message.content is not none else '' %} {{- '<|im_start|>' + message.role + '\n' + content }} {%- if message.tool_calls %} {%- for tool_call in message.tool_calls %} {%- if (loop.first and content) or (not loop.first) %} {{- '\n' }} {%- endif %} {%- if tool_call.function %} {%- set tool_call = tool_call.function %} {%- endif %} {{- '<tool_call>\n{"name": "' }} {{- tool_call.name }} {{- '", "arguments": ' }} {%- if tool_call.arguments is string %} {{- tool_call.arguments }} {%- else %} {{- tool_call.arguments | tojson }} {%- endif %} {{- '}\n</tool_call>' }} {%- endfor %} {%- endif %} {{- '<|im_end|>\n' }} {%- elif message.role == "tool" %} {%- if loop.first or (messages[loop.index0 - 1].role != "tool") %} {{- '<|im_start|>user' }} {%- endif %} {{- '\n<tool_response>\n' }} {{- message.content }} {{- '\n</tool_response>' }} {%- if loop.last or (messages[loop.index0 + 1].role != "tool") %} {{- '<|im_end|>\n' }} {%- endif %} {%- endif %} {%- endfor %} {%- if add_generation_prompt %} {{- '<|im_start|>assistant\n<think>\n\n</think>\n\n' }} {%- endif %}'''

# =============================================================================

logger = logging.getLogger("batch-update-template")


def find_gguf_files(directory: Path, recursive: bool = True) -> List[Path]:
    """Find all .gguf files in the given directory."""
    pattern = "**/*.gguf" if recursive else "*.gguf"
    return list(directory.glob(pattern))


def update_chat_template(gguf_file: Path, chat_template: str, dry_run: bool = False, force: bool = False) -> bool:
    """Update the chat template for a single GGUF file by creating a new file."""
    logger.info(f"Processing: {gguf_file.name}")
    
    try:
        # Open file in read mode
        reader = GGUFReader(str(gguf_file), 'r')
        
        # Get the tokenizer.chat_template field
        field = reader.get_field('tokenizer.chat_template')
        if field is None:
            logger.error(f"Field 'tokenizer.chat_template' not found in {gguf_file.name}")
            return False
        
        # Check if it's a string field
        if field.types[0] != GGUFValueType.STRING:
            logger.error(f"Field 'tokenizer.chat_template' is not a string type in {gguf_file.name} (type: {field.types[0]})")
            return False
        
        # Get current value
        current_value = bytes(field.parts[-1]).decode('utf-8')
        
        logger.info(f"Current template length: {len(current_value)} characters")
        logger.info(f"New template length: {len(chat_template)} characters")
        
        if current_value == chat_template:
            logger.info(f"Template already matches target for {gguf_file.name}")
            return True
        
        if dry_run:
            logger.info(f"DRY RUN: Would update template in {gguf_file.name}")
            return True
        
        if not force:
            logger.warning("*** Warning *** Warning *** Warning ***")
            logger.warning("* Changing fields in a GGUF file can make it unusable. Proceed at your own risk.")
            logger.warning("* Enter exactly YES if you are positive you want to proceed:")
            response = input("YES, I am sure> ")
            if response != "YES":
                logger.info("You didn't enter YES. Skipping this file.")
                return False
        
        # Create a temporary file path
        temp_file = gguf_file.with_suffix('.gguf.tmp')
        
        try:
            # Get architecture and endianness from the original file
            arch = 'unknown'
            arch_field = reader.get_field(gguf.Keys.General.ARCHITECTURE)
            if arch_field:
                arch = arch_field.contents()
            
            # Create writer with same properties as original
            writer = GGUFWriter(str(temp_file), arch=arch, endianess=reader.endianess)
            
            # Get alignment if present
            alignment_field = reader.get_field(gguf.Keys.General.ALIGNMENT)
            if alignment_field:
                alignment = alignment_field.contents()
                if alignment is not None:
                    writer.data_alignment = alignment
            
            # Copy all metadata, applying our change to tokenizer.chat_template
            for field in reader.fields.values():
                # Skip virtual fields and fields written by GGUFWriter
                if field.name == gguf.Keys.General.ARCHITECTURE or field.name.startswith('GGUF.'):
                    continue
                
                # Apply our change to tokenizer.chat_template
                if field.name == 'tokenizer.chat_template':
                    writer.add_key_value(field.name, chat_template, GGUFValueType.STRING)
                else:
                    # Copy original value
                    value = field.contents()
                    value_type = field.types[0]
                    sub_type = None
                    if value_type == GGUFValueType.ARRAY and len(field.types) > 1:
                        sub_type = field.types[-1]
                    
                    if value is not None:
                        writer.add_key_value(field.name, value, value_type, sub_type=sub_type)
            
            # Copy all tensors
            logger.info(f"Copying {len(reader.tensors)} tensors...")
            for tensor in reader.tensors:
                writer.add_tensor(tensor.name, tensor.data, raw_shape=tensor.data.shape, raw_dtype=tensor.tensor_type)
            
            # Write the new file
            logger.info(f"Writing new file...")
            writer.open_output_file()
            writer.write_header_to_file()
            writer.write_kv_data_to_file()
            writer.write_tensors_to_file(progress=False)
            writer.close()
            
            # Replace original file with new file
            gguf_file.unlink()  # Remove original
            temp_file.rename(gguf_file)  # Rename temp to original
            
            logger.info(f"Successfully updated template in {gguf_file.name}")
            return True
            
        except Exception as e:
            # Clean up temp file if it exists
            if temp_file.exists():
                temp_file.unlink()
            raise e
        
    except Exception as e:
        logger.error(f"Failed to update {gguf_file.name}: {e}")
        return False


def main():
    parser = argparse.ArgumentParser(
        description="Batch update tokenizer.chat_template for all GGUF files in a directory"
    )
    parser.add_argument(
        "directory",
        type=str,
        help="Directory containing GGUF files to update"
    )
    parser.add_argument(
        "--recursive",
        action="store_true",
        default=False,
        help="Search for GGUF files recursively (default: False)"
    )
    parser.add_argument(
        "--no-recursive",
        dest="recursive",
        action="store_false",
        help="Don't search recursively, only check the specified directory"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be changed without actually modifying files"
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Update files without confirmation prompts"
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Enable verbose output"
    )
    parser.add_argument(
        "--template",
        type=str,
        help="Override the default chat template with a custom one"
    )

    args = parser.parse_args()

    # Set up logging
    logging.basicConfig(
        level=logging.DEBUG if args.verbose else logging.INFO,
        format="%(levelname)s: %(message)s"
    )

    # Validate directory
    directory = Path(args.directory)
    if not directory.exists():
        logger.error(f"Directory does not exist: {directory}")
        sys.exit(1)
    
    if not directory.is_dir():
        logger.error(f"Path is not a directory: {directory}")
        sys.exit(1)

    # Use custom template if provided, otherwise use the default
    chat_template = args.template if args.template else CHAT_TEMPLATE

    # Find GGUF files
    logger.info(f"Searching for GGUF files in: {directory}")
    gguf_files = find_gguf_files(directory, args.recursive)
    
    if not gguf_files:
        logger.info("No GGUF files found.")
        return

    logger.info(f"Found {len(gguf_files)} GGUF file(s)")

    # Show what will be updated
    if args.dry_run:
        logger.info("DRY RUN - No files will be modified")
    
    logger.info(f"Chat template to set:\n{chat_template}")
    
    # Get user confirmation if not forcing and not dry run
    user_confirmed = args.force
    if not args.force and not args.dry_run:
        print("\nFiles to be updated:")
        for f in gguf_files:
            print(f"  - {f}")
        
        response = input(f"\nUpdate {len(gguf_files)} file(s)? (y/N): ")
        if response.lower() not in ['y', 'yes']:
            logger.info("Operation cancelled.")
            return
        user_confirmed = True

    # Process files
    successful = 0
    failed = 0
    
    for gguf_file in gguf_files:
        # If user already confirmed at batch level, treat as force for individual files
        if update_chat_template(gguf_file, chat_template, args.dry_run, user_confirmed):
            successful += 1
        else:
            failed += 1

    # Summary
    logger.info(f"\nBatch update complete:")
    logger.info(f"  Successful: {successful}")
    logger.info(f"  Failed: {failed}")
    logger.info(f"  Total: {len(gguf_files)}")

    if failed > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
