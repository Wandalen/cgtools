import pyembroidery
import argparse
import struct
import json


def dict_to_thread(thread_dict):
    """Convert a dictionary to EmbThreadPec."""
    thread = pyembroidery.EmbThread()
    thread.color = thread_dict.get("hex_color", None)
    thread.description = thread_dict.get("description", None)
    thread.catalog_number = thread_dict.get("catalog_number", None)
    thread.details = thread_dict.get("details", None)
    thread.brand = thread_dict.get("brand", None)
    thread.chart = thread_dict.get("chart", None)
    thread.weight = thread_dict.get("weight", None)
    return thread


def main(input, output):
    with open(input, "rb") as f:
        # Read the number of rows (array size)
        num_rows = struct.unpack("I", f.read(4))[0]

        # Read the array data (3 * i32 per row)
        stitches = []
        for _ in range(num_rows):
            row = struct.unpack("iii", f.read(12))
            stitches.append(row)

        # Read the metadata size
        threads_size = struct.unpack("I", f.read(4))[0]

        # Read the metadata JSON bytes
        threads_bytes = f.read(threads_size)
        str = threads_bytes.decode("utf-8")
        thread_dicts = json.loads(str)

    threads = list(map(dict_to_thread, thread_dicts))
    pattern = pyembroidery.EmbPattern()
    pattern.stitches = stitches
    pattern.threadlist = threads

    pyembroidery.write(pattern, output, settings={"version": 6.0})


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Process two file paths.")
    parser.add_argument("input", type=str, help="Input binary file")
    parser.add_argument("output", type=str, help="Output embroidery file")

    args = parser.parse_args()
    main(args.input, args.output)
