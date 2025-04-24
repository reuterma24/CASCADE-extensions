## Dataset 
Manually gathered C\# dataset for code documentation inconsistencies.

### Directory Structure
The dataset follows a specific directory structure. The folder levels are organized as follows:

0. Programming language
1. GitHub user
2. GitHub project
3. Commit hash (which fixed the inconsistency)  
4.1. Numbered folders that contain a sample  
4.2. Repository (optionally)  


> Remarks:  
> 4.1. Folder contains:
>	1. An analyzed.json file, which is required as input for CASCADE and also contains the produced results or our evaluation.
>	2. A file called inconsistency.txt that either contains 'True' or 'False' to label the sample.
>
> 4.2: Contains the project in a .7z (7zip) file, checked out at the commit hash (from 3.) with some minor adjustments to build the project successfully. Inconsistency is manually reintroduced!
> In some cases the zip file has the suffix 'Isolated'. This means the project was slimmed down to a minimal working example which is able to run the code containing the inconsistency.


Examples: 
- dataset\csharp\ardalis\SmartEnum\a5dcb209b3d2b4b1227a7419f2ddfe18cd1f56ba\repository\SmartEnumIsolated.7z
- dataset\csharp\ardalis\SmartEnum\a5dcb209b3d2b4b1227a7419f2ddfe18cd1f56ba\2\analyzed.json
