Dataset directory structure is organized as follows:

0. programming langauge
1. github user
2. github project
3. commit hash (which fixed the inconsistency)
4.1 numbered folders with samples
4.2 repository (optionally)


Remarks:

4.1: Folder contains:
	1. an analyzed.json file, which is required as input for CASCADE and contains the produced results
	2. a file called inconsistency.txt that either contains 'True' or 'False'

4.2: Contains the project in a .7z (7zip) file, checked out at the commit hash (from 3.) with some minor adjustments to build the project successfully. Inconsistency is manually reintroduced!
In some cases the zip file has the suffix 'Isolated'. This means the project was slimmed down to a minimal working example which is able to run the code containing the inconsistency.


Examples: 
- dataset\csharp\ardalis\SmartEnum\a5dcb209b3d2b4b1227a7419f2ddfe18cd1f56ba\repository\SmartEnumIsolated.7z
- dataset\csharp\ardalis\SmartEnum\a5dcb209b3d2b4b1227a7419f2ddfe18cd1f56ba\2\analyzed.json
