import pyembroidery
import argparse
import struct
import json


def thread_to_dict(thread):
    """Convert EmbThreadPec to a dictionary."""
    return {
        "hex_color": thread.color,
        "description": thread.description,
        "catalog_number": thread.catalog_number,
        "details": thread.details,
        "brand": thread.brand,
        "chart": thread.chart,
        "weight": thread.weight,
    }


def main(input, output):
    pattern = pyembroidery.read(input)

    stitches = pattern.stitches
    threads = pattern.threadlist
    thread_dicts = list(map(thread_to_dict, threads))
    threads = json.dumps(thread_dicts)
    threads_bytes = threads.encode("utf-8")
    threads_size = len(threads_bytes)

    with open(output, "wb") as f:
        # Write the number of rows (array size)
        f.write(struct.pack("I", len(stitches)))  # Unsigned int (4 bytes)

        # Write the array data (3 * i32 per row)
        for row in stitches:
            f.write(struct.pack("iii", *row))

        # Write the metadata size
        f.write(struct.pack("I", threads_size))  # Unsigned int (4 bytes)

        # Write the metadata JSON bytes
        f.write(threads_bytes)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Process two file paths.")
    parser.add_argument("input", type=str, help="Input embroidery")
    parser.add_argument("output", type=str, help="Output data")

    args = parser.parse_args()
    main(args.input, args.output)
