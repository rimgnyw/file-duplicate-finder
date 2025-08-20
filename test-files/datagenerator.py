import random
import shutil
import argparse
from collections import defaultdict
from pathlib import Path
import string

def create_nested_dirs(root, m, k, i):
    all_dirs = []

    def recurse(current_path, depth):
        if depth == i:
            return
        for _ in range(random.randint(0, k)):
            subdir = current_path / f"sub_{len(all_dirs)}"
            subdir.mkdir(parents=True, exist_ok=True)
            all_dirs.append(subdir)
            recurse(subdir, depth + 1)

    for index in range(m):
        top = root / f"top_{index}"
        top.mkdir(parents=True, exist_ok=True)
        all_dirs.append(top)
        recurse(top, 1)

    return all_dirs


def random_content(size=100):
    return ''.join(random.choices(string.ascii_letters + string.digits + " \n", k=size))

def distribute_files(n, p, dirs):
    num_dupes = int(n * p / 100)
    num_unique = n - num_dupes

    file_locations = defaultdict(list)
    all_paths = []

    unique_filenames = [f"u{i}.txt" for i in range(1, num_unique + 1)]

    # Assign each unique file its own random content
    unique_contents = {fname: random_content() for fname in unique_filenames}

    max_dupe_names = min(num_dupes // 2, 20)
    dupe_filenames = [f"d{i}.txt" for i in range(1, max_dupe_names + 1)]

    # Assign each duplicate group a shared random content
    dupe_contents = {fname: random_content() for fname in dupe_filenames}

    dupe_file_list = []
    total_dupe_files = 0
    while total_dupe_files < num_dupes:
        fname = random.choice(dupe_filenames)
        remaining = num_dupes - total_dupe_files
        copies = random.randint(2, min(5, remaining))
        dupe_file_list.extend([fname] * copies)
        total_dupe_files += copies

    file_list = unique_filenames + dupe_file_list
    random.shuffle(file_list)

    for filename in file_list:
        dir_path = random.choice(dirs)
        file_path = dir_path / filename

        # Pick correct content based on whether it's unique or duplicate
        if filename.startswith("u"):
            content = unique_contents[filename]
        else:
            content = dupe_contents[filename]

        # Write file with content
        with open(file_path, "w") as f:
            f.write(content)

        abs_path = file_path.resolve()
        if abs_path not in file_locations[filename]:
            file_locations[filename].append(abs_path)
        all_paths.append(file_path.resolve())

    return all_paths, file_locations

def main(root_path, unique_file_names=1, dupe_ratio=30, num_top_dirs=10, subdirs_per_dir=10, subdir_depth=2):
    root = Path(root_path).resolve()

    if root.exists():
        shutil.rmtree(root)
    root.mkdir(parents=True, exist_ok=True)

    dirs = create_nested_dirs(root, num_top_dirs, subdirs_per_dir, subdir_depth)
    file_paths, file_map = distribute_files(unique_file_names, dupe_ratio, dirs)

    unique_files = sum(1 for k, v in file_map.items() if len(v) == 1 and k.startswith("u"))
    unique_duplicate_files = sum(1 for k in file_map if k.startswith("d"))
    total_duplicate_files = sum(len(v) for k, v in file_map.items() if k.startswith("d"))
    total_files = len(file_paths)
    total_dirs = len(dirs)

    log_lines = [
        f"Total files: {total_files}",
        f"Total directories: {total_dirs}",
        f"Unique files: {unique_files}",
        f"Total duplicate file names: {total_duplicate_files}",
        f"Unique duplicate file names: {unique_duplicate_files}",
        "",
        "Duplicate files and their locations:"
    ]

    for filename, paths in file_map.items():
        if filename.startswith("d"):
            log_lines.append(f"{filename}:")
            for p in paths:
                log_lines.append(f"  {p}")  # already absolute from .resolve()

    log_text = "\n".join(log_lines)

    log_path = root_path + "/log.txt"
    with open(log_path, "w") as f:
        f.write(log_text)

    out_text = (
        f"Generation complete.\n"
        f"Total files: {total_files}\n"
        f"Total directories: {total_dirs}\n"
        f"Unique files: {unique_files}\n"
        f"Total duplicate file names: {total_duplicate_files}\n"
        f"Unique duplicate file names: {unique_duplicate_files}\n"
        f"Log file written to: {log_path}"
    )
    print(out_text)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate file structure with duplicates.")
    parser.add_argument("root", type=str,
                        help="Root path where the structure should be created (e.g., ./testdata/100-files)")
    parser.add_argument("unique_file_names", type=int, nargs='?', default=500,
                        help="Number of unique file names (default: 500)")
    args = parser.parse_args()

    main(
        root_path=args.root,
        unique_file_names=args.unique_file_names
    )

