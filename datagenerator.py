import random
import shutil
from collections import defaultdict
from pathlib import Path


def create_nested_dirs(root, m, k, i):
    all_dirs = []

    def recurse(current_path, depth):
        if depth == i:
            return
        for _ in range(k):
            subdir = current_path / f"sub_{len(all_dirs)}"
            subdir.mkdir()
            all_dirs.append(subdir)
            recurse(subdir, depth + 1)

    for _ in range(m):
        top = root / f"top_{_}"
        top.mkdir()
        all_dirs.append(top)
        recurse(top, 1)

    return all_dirs

def distribute_files(n, p, dirs):
    num_dupes = int(n * p / 100)
    num_unique = n - num_dupes

    filenames = []
    # Unique files
    for i in range(1, num_unique + 1):
        filenames.append(f"u{i}.txt")

    # Duplicate base names
    for i in range(1, num_dupes + 1):
        filenames.append(f"d{i}.txt")

    random.shuffle(filenames)

    file_locations = defaultdict(list)
    all_paths = []

    # First, create unique files and one instance of each duplicate
    for filename in filenames:
        dir_path = random.choice(dirs)
        file_path = dir_path / filename
        file_path.touch()
        file_locations[filename].append(file_path)
        all_paths.append(file_path)

    # Now, ensure all duplicates have at least one additional copy elsewhere
    for filename in [f"d{i}.txt" for i in range(1, num_dupes + 1)]:
        existing_dirs = {p.parent for p in file_locations[filename]}
        other_dirs = [d for d in dirs if d not in existing_dirs]
        if other_dirs:
            dup_dir = random.choice(other_dirs)
            dup_path = dup_dir / filename
            dup_path.touch()
            file_locations[filename].append(dup_path)
            all_paths.append(dup_path)

    return all_paths, file_locations

def main(unique_file_names=100, dupe_ratio=20, num_top_dirs=2, subdirs_per_dir=2, subdir_depth=2):
    script_dir = Path(__file__).resolve().parent
    root = script_dir / "test_root"

    if root.exists():
        shutil.rmtree(root)
    root.mkdir(exist_ok=True)

    dirs = create_nested_dirs(root, num_top_dirs, subdirs_per_dir, subdir_depth)
    file_paths, file_map = distribute_files(unique_file_names, dupe_ratio, dirs)

    unique_files = sum(1 for k, v in file_map.items() if len(v) == 1 and k.startswith("u"))
    unique_duplicate_files = sum(1 for k, _ in file_map.items() if k.startswith("d"))
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
                log_lines.append(f"  {p}")

    log_text = "\n".join(log_lines)

    log_path = script_dir / "log.txt"
    with open(log_path, "w") as f:
        f.write(log_text)
    out_text = (
        f"Total files: {total_files}\n"
        f"Total directories: {total_dirs}\n"
        f"Unique files: {unique_files}\n"
        f"Total duplicate file names: {total_duplicate_files}\n"
        f"Unique duplicate file names: {unique_duplicate_files}\n")
    print(f"Generation complete.\n{out_text}")

if __name__ == "__main__":
    # You can change parameters here
    main(unique_file_names=500, dupe_ratio=30, num_top_dirs=2, subdirs_per_dir=2, subdir_depth=2)
