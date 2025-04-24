TODO: adapt for rust

## Usage Instructions

This tool provides three primary functionalities: **extraction**, **modification**, and **verification**. Each functionality is invoked using a specific command-line argument.

### General Syntax
```bash
cSharpTool <command> [options]
```

### Commands

1. **extract**  
Use this command to perform the context extraction and generate an extracted.json file.  
The parameter projectPath is expected to point to a solution file (.sln).  
The parameter outputPath has to specify a target directory.  
The parameter targetFramework is used to specify in which target framework the solution should be opened (e.g. net6.0, net8.0).  
```bash
cSharpTool extract <projectPath> <outputPath> <targetFramework>
```
&nbsp;  
2. **modify**  
Use this command to either inject a generated method or a generated test suite into the project.
The parameter projectDir is expected to point to the project root file (top-level directory, not the .sln file).  
The parameter contextFilePath has to point to an context file that contains a single sample (called entry.json in CASCADE).  
The parameters codeKeyword has to be set to 'new_code' if the method from the context file should be injected. Otherwise set to 'code'.  
The parameters testKeyword has to be set to 'new_test' if the method from the context file should be injected. Otherwise set to 'test'.
```bash
cSharpTool modify <projectDir> <contextFilePath> <codeKeyword> <testKeyword>
```
&nbsp;  
3. **verify**  
Use this command to verify if an C\# file is syntactically correct. The program will return a verification result of 0 if valid, -1 otherwise.  
The parameter filepath should point to a C\# file.
```bash
cSharpTool verify <filepath>
```
