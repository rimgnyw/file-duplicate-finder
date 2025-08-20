# File Duplicate Finder

A multi-threaded search program for finding files with identical names in
different directories written in rust.

## Usage

After extracting the binary the GUI can be launched by simply running the 
gui binary. 

Alternatively the standalone CLI program can be used. In this case,
the desired directory can be scanned by running:

```bash
./file-duplicate-finder <path/to/dir>...
```

If multiple directories are scanned and some are subdirectories of others
the program will filter them out to avoid any duplicate entries

The results will be written to `result.log`.

## The Datagenerator

This repo also contains `datagenerator.py`. This is a python script that can
generate a directory with text files as a way to test the program,
the script will output a `log.txt` with information about the generation
as well as a list of duplicate files and their respective paths.

The datagenerator can be run with the following command

```bash
python3 datagenerator.py <path/to/destination> <number of files to generate>
```

The generation can be further customised by editing the arguments sent to the main()
function in the script, such as the ratio of duplicates and the number of directories
each subdirectory contains.

### Example

Create a directory called "testdir" with 100 text files

```bash
python3 ./datagenerator.py ./testdir 100
```
