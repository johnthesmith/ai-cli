# AI CLI Combo Operations

## Introduction

AI CLI can combine operations: reading, writing, clipboard, commands — into a
single chain. This speeds up routine developer tasks without requiring agentic
behavior from the LLM or the utility.

## Remembering Information

```
ai --read=notes.md remember the key facts
```

Reads the file, extracts the important points, and saves them to long-term memory.

## Search in History

```
ai find the refactoring discussion in history
```

Reviews the dialogue history and finds the relevant discussions.

## Translating Documentation

```
ai --read=README.md --write=README_ru.md translate to Russian
```

Reads README.md, translates the content to Russian, and writes it to a new file.

## Code Refactoring

```
ai --read=src/main.rs --write=src/main_improved.rs refactor the code
```

Analyzes the code, improves the structure, and writes the fixed version.

## Splitting a File into Parts

```
ai --read=big.log --write=errors.log select only lines with ERROR
```

Reads the log file, filters out error lines, and writes them to a separate file.

## Data Transformation

```
ai --read=data.csv --write=data.json convert to JSON
```

Converts CSV to JSON and writes the result to a file.

## Translate and Copy to Clipboard

```
ai --read=docs/guide.md translate to English and put in the clipboard
```

Reads the document, translates it, and copies the result to the clipboard — without writing to a file.

## Generating Tests

```
ai --read=src/utils.js --write=tests/utils.test.js generate tests
```

Creates a test file based on the source code.

## Formatting Code

```
ai --read=src/helpers.ts --write=src/helpers.ts format the code
```

Reads the file, formats the code, and overwrites it.

## Analysis with a Report

```
ai --read=code.rs analyze the code
```

Analyzes the code and creates a report with recommendations.

## Comparing Files

```
ai --read=file1.txt --read=file2.txt find the differences and show them in history
```

Reads two files, compares them, and shows the differences.
