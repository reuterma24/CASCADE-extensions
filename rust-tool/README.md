## Usage Instructions

This tool provides three primary functionalities: **extraction**, **modification**, and **verification**. Each functionality is invoked using a specific command-line argument.

### General Syntax
```bash
rust-tool <command> [options]
```

### Commands

1. **extract**  
Use this command to perform the context extraction and generate an extracted.json file.  
The parameter pathToProject is expected to point to a project root file (top-level folder).  
The parameter outputPath has to specify a target directory.
```bash
rust-tool extract <pathToProject> <outPath>
```
&nbsp;  
2. **modify**  
Use this command to either inject a generated method or a generated test suite into the project.
The parameter pathToProject is expected to point to the project root file (top-level directory, not the .sln file).  
The parameter pathToEntry has to point to an context file that contains a single sample (called entry.json in CASCADE).  
The parameters testKeyword has to be set to 'new_test' if the tests from the context file should be injected into the tests directory. Otherwise set to 'test'. If set to 'new_tests_inject' the test suite is directly injected into the source file of the method under test - allows testing private functions.
The parameters codeKeyword has to be set to 'new_code' if the method from the context file should be injected. Otherwise set to 'code'.  
```bash
rust-tool modify <pathToProject> <pathToEntry> <testKeyword> <codeKeyword>
```
&nbsp;  
3. **verify**  
Use this command to verify if a Rust file is syntactically correct. The program will return a verification result of 0 if valid, -1 otherwise.  
The parameter pathToFile should point to a Rust file.
```bash
rust-tool verify <pathToFile>
```
